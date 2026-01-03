use std::thread;

use anyhow::Result;
use bytes::Bytes;

use ffmpeg_next as ffmpeg;
use ffmpeg_next::Packet;
use ffmpeg_next::codec::Id;
use ffmpeg_next::decoder;
use ffmpeg_next::format::Pixel;
use ffmpeg_next::software::scaling::{context::Context as Scaler, flag::Flags};
use ffmpeg_next::util::frame::Video as VideoFrame;

use futures::StreamExt;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio_stream::wrappers::ReceiverStream;
use tokio_util::io::{StreamReader, SyncIoBridge};

// Pure Rust WebM Parser
use webm_iterable::WebmIterator;
use webm_iterable::matroska_spec::MatroskaSpec;

pub fn spawn_decoding_thread(processor: VideoProcessor, tx_frames: Sender<Vec<u8>>) {
    // std::thread::spawn을 사용하여 Tokio 런타임과 완전히 분리된 스레드를 만듭니다.
    // 이렇게 하면 SyncIoBridge가 block_on을 써도 안전합니다.
    thread::spawn(move || {
        if let Err(e) = run_decoder_loop(processor, tx_frames) {
            eprintln!("Decoder thread error: {:?}", e);
        }
    });
}

fn run_decoder_loop(processor: VideoProcessor, tx_frames: Sender<Vec<u8>>) -> anyhow::Result<()> {
    let mut processor = processor;
    let mut scaler: Option<Scaler> = None;

    while let Ok(Some(decoded_frame)) = processor.next_decoded_frame() {
        let width = decoded_frame.width();
        let height = decoded_frame.height();

        // 2. Scaler 초기화 (필요시)
        if scaler.is_none()
            || scaler.as_ref().unwrap().input().width != width
            || scaler.as_ref().unwrap().input().height != height
        {
            let new_scaler = Scaler::get(
                decoded_frame.format(),
                width,
                height,
                Pixel::RGB24, // TUI용 포맷
                // width,
                // height, // TUI 목표 크기
                100,
                50,
                Flags::BILINEAR,
            )?;
            scaler = Some(new_scaler);
        }

        // 3. 변환 및 Vec<u8> 추출
        if let Some(ctx) = &mut scaler {
            let mut rgb_frame = VideoFrame::empty();
            ctx.run(&decoded_frame, &mut rgb_frame)?;

            let width = rgb_frame.width() as usize;
            let height = rgb_frame.height() as usize;
            let stride = rgb_frame.stride(0); // 실제 메모리 한 줄의 길이
            let data = rgb_frame.data(0);

            // 🔥 [수정] Stride(여백)을 제거하고 순수 데이터만 복사
            let mut clean_rgb_data = Vec::with_capacity(width * height * 3);

            for y in 0..height {
                let start = y * stride;
                let end = start + width * 3;
                // 실제 데이터 구간만 잘라서 추가
                clean_rgb_data.extend_from_slice(&data[start..end]);
            }

            // 4. 깨끗한 데이터 전송
            if tx_frames.blocking_send(clean_rgb_data).is_err() {
                break;
            }
        }
    }

    Ok(())
}

pub struct VideoProcessor {
    // WebM 파서 (Iterator 형태)
    // 제네릭 복잡도를 피하기 위해 Box<dyn Iterator> 사용 가능하지만,
    // 여기서는 로직 설명을 위해 풀어씁니다.
    // 실제로는 스트림을 계속 읽어야 하므로 Iterator를 멤버로 가집니다.
    pub parser: Box<dyn Iterator<Item = Result<MatroskaSpec, anyhow::Error>> + Send>,

    pub decoder: ffmpeg::decoder::Video,
    // pub scaler: Option<Scaler>,
}

impl VideoProcessor {
    pub fn new(rx: Receiver<Bytes>) -> Result<Self> {
        ffmpeg::init()?;

        // 1. Async Receiver -> Sync Reader 변환
        // (webm_iterable은 std::io::Read를 요구합니다)
        let stream = ReceiverStream::new(rx).map(|b| Ok(b) as std::io::Result<Bytes>);
        let async_reader = StreamReader::new(stream);
        let sync_reader = SyncIoBridge::new(async_reader);

        // 2. WebM 파서 생성 (Pure Rust Demuxer!)
        // 들어오는 바이트를 해석해서 EBML 태그 단위로 쪼개줍니다.
        let iterator = WebmIterator::new(sync_reader, &[]);

        let mapped_iterator =
            iterator.map(|res| res.map_err(|e| anyhow::anyhow!("WebM parse error: {:?}", e)));

        // 3. 디코더 생성
        // WebM은 주로 VP9 코덱을 씁니다. (YouTube 기본)
        let codec = decoder::find(Id::VP9).ok_or_else(|| anyhow::anyhow!("VP9 codec not found"))?;

        let context = ffmpeg::codec::context::Context::new();
        let decoder = context.decoder().open_as(codec)?.video()?;

        // 필수: VP9은 해상도 변경이 잦으므로 open을 미리 해두거나 패킷을 통해 자동 감지하게 둡니다.
        // decoder.open()은 파라미터가 없으면 에러가 날 수 있으나,
        // VP9 스트림은 첫 패킷에 정보가 있어 send_packet으로 초기화가 가능합니다.
        // 일단 열어둡니다.

        // if decoder.open().is_err() {
        //     // 파라미터 없이 열기 실패시 무시 (첫 패킷 처리때 열림)
        // }

        Ok(Self {
            parser: Box::new(mapped_iterator),
            decoder,
        })
    }

    pub fn next_decoded_frame(&mut self) -> Result<Option<VideoFrame>> {
        while let Some(tag_result) = self.parser.next() {
            // 파싱 에러가 나도 스트림을 끊지 말고 로그 찍고 계속 갑니다 (중요!)
            let tag = match tag_result {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("Parser warning: {:?}", e);
                    continue;
                }
            };

            match tag {
                MatroskaSpec::SimpleBlock(data) => {
                    // [핵심] WebM SimpleBlock 헤더 벗기기
                    // 구조: [TrackNum(VINT)] + [Timecode(2byte)] + [Flags(1byte)] + [Real Data]

                    let header_len = get_simple_block_header_len(&data);

                    if data.len() <= header_len {
                        continue; // 데이터가 없으면 패스
                    }

                    // 껍질(헤더)을 제외한 알맹이만 추출
                    let payload = &data[header_len..];

                    // FFmpeg Packet 생성 (Payload만 복사)
                    let packet = Packet::copy(payload);

                    // 디코더에 전송 (에러 나도 죽지 않게 처리)
                    if let Err(e) = self.decoder.send_packet(&packet) {
                        // VP9은 간혹 첫 패킷 동기화 실패할 수 있음. 무시하고 진행.
                        continue;
                    }

                    let mut decoded_frame = VideoFrame::empty();
                    if self.decoder.receive_frame(&mut decoded_frame).is_ok() {
                        return Ok(Some(decoded_frame));
                    }
                }

                // BlockGroup 등을 쓰는 경우도 대응 (보통 YouTube는 SimpleBlock 씀)
                MatroskaSpec::Block(data) => {
                    // Block 구조도 SimpleBlock과 유사하지만 Flags가 다를 수 있음
                    // 일단 SimpleBlock 로직과 동일하게 처리 시도
                    let header_len = get_simple_block_header_len(&data);
                    if data.len() > header_len {
                        let payload = &data[header_len..];
                        let packet = Packet::copy(payload);
                        self.decoder.send_packet(&packet).ok();

                        let mut decoded_frame = VideoFrame::empty();
                        if self.decoder.receive_frame(&mut decoded_frame).is_ok() {
                            return Ok(Some(decoded_frame));
                        }
                    }
                }
                _ => continue,
            }
        }
        Err(anyhow::anyhow!("Stream ended"))
    }

    pub fn get_fps(&self) -> f64 {
        // WebM 헤더 파싱해서 정확히 얻을 수 있지만,
        // YouTube 스트리밍은 보통 30/60이므로 기본값 반환 후
        // 실제 속도에 맞춰 Sleep하는 로직 사용
        30.0
    }
}

// [도우미 함수] EBML VINT(가변 길이 정수) 파싱하여 헤더 길이 계산
fn get_simple_block_header_len(data: &[u8]) -> usize {
    if data.is_empty() {
        return 0;
    }

    // 1. Track Number (VINT) 길이 계산
    let first_byte = data[0];
    let vint_len = if first_byte & 0x80 != 0 {
        1
    }
    // 1xxx xxxx
    else if first_byte & 0x40 != 0 {
        2
    }
    // 01xx xxxx
    else if first_byte & 0x20 != 0 {
        3
    }
    // 001x xxxx
    else {
        4
    }; // 0001 xxxx (보통 4바이트 안 넘음)

    // 2. 전체 헤더 길이 = TrackNum(VINT) + Timecode(2) + Flags(1)
    vint_len + 2 + 1
}
