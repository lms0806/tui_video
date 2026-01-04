use std::{sync::Arc, time::Duration};

use rusty_ytdl::*;
use tokio::{
    join,
    sync::mpsc::{self, Receiver, Sender, error::SendError},
    time::{Instant, sleep},
};

use std::io::stdin;

use crate::{ascii::rgb_to_colored_ascii, ffmpeg_fn::VideoProcessor};

mod ascii;
mod ffmpeg_fn;
mod tui;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // url을 입력받음
    let mut url = String::new();
    println!("input youtube url");
    stdin().read_line(&mut url).unwrap();
    let (sender, receiver) = mpsc::channel(32);

    // 받은 url에서 받을 수 있는 유튜브 영상중, WebM을 찾음
    let info = Video::new(&url)?.get_info().await?;
    let format = info
        .formats
        .iter()
        .filter(|f| f.has_video && f.mime_type.container == "webm")
        .min_by_key(|f| f.bitrate)
        .expect("No WebM format found");

    // 그 태그를 저장
    let target_itag = format.itag;

    // 낮은 퀄리티의 비디오(오디오 제외) 영상중 WebM 포멧 영상을 찾음
    let video_option = VideoOptions {
        quality: rusty_ytdl::VideoQuality::LowestVideo,
        filter: VideoSearchOptions::Custom(Arc::new(move |f| f.itag == target_itag)),
        ..Default::default()
    };
    let video = Video::new_with_options(url, video_option).unwrap();

    // 영상 다운로드 테스트
    // let res = video.download(std::path::Path::new(r"test.mp4")).await;

    // if let Err(e) = res {
    //     println!("{:?}", e);
    // }

    // get_video는 영상을 청크단위로 받고, 보냄(Send)
    // print_tui는 받은 청크를 디코딩해서 출력(Receive)
    let (_, _) = join!(get_video(video, sender), print_tui(receiver));

    println!("END");
    Ok(())
}

async fn get_video(
    video: Video<'_>,
    sen: Sender<bytes::Bytes>,
) -> Result<(), SendError<bytes::Bytes>> {
    let stream = video.stream().await.unwrap();

    // 청크가 받아질 때 마다 send해서 print_tui로 보냄
    while let Some(chunk) = stream.chunk().await.unwrap() {
        sen.send(chunk).await?;
    }

    Ok(())
}

async fn print_tui(rec: Receiver<bytes::Bytes>) -> anyhow::Result<()> {
    let mut terminal = tui::init()?;

    // 1. 프레임 데이터(Vec<u8>)를 받을 채널
    let (frame_tx, mut frame_rx) = mpsc::channel(10); // 버퍼를 작게 잡아 지연을 줄임

    // 2. 디코더 스레드 실행 (VideoProcessor가 저쪽에서 돕니다)
    // new안에 비동기 Receiver를 블로킹하여 동기 io로 변환하기 때문에, 여기서 생성해야함
    let processor = VideoProcessor::new(rec)?;

    // 스레드 기능을 이용하여 해당 스레드에서 동기적으로 프레임을 받음
    ffmpeg_fn::spawn_decoding_thread(processor, frame_tx);

    // rgb_to_colored_ascii에서 생성한 Vec을 저장하는 버퍼
    let mut tui_frame = Vec::new();

    // FPS 설정 (30 FPS 기준)
    let target_frame_duration = Duration::from_millis(33);

    // 3. 메인 루프: 받아서 -> 변환하고 -> 그리고 -> 쉰다
    while let Some(frame_data) = frame_rx.recv().await {
        // 화면을 그리는 시간 기록
        let start_time = Instant::now();

        // (1) ASCII 변환
        // frame_data는 이미 100x50 RGB로 리사이징되어 옴 (VideoProcessor에서 처리함)
        tui_frame.clear();
        rgb_to_colored_ascii(&frame_data, 100, 50, &mut tui_frame);

        // (2) 그리기
        tui::draw_frame(&mut terminal, &tui_frame)?;

        // (3) FPS 시간 맞추기
        // 그리는 시간과 해당 변수가 생성되는 시간이 프레임보다 빠를경우 대기
        let elapsed = start_time.elapsed();
        if elapsed < target_frame_duration {
            sleep(target_frame_duration - elapsed).await;
        }
    }

    // 영상이 종료되면 터미널 화면을 원상복구함
    tui::restore()?;
    Ok(())
}
