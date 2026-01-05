use std::{
    io::Read,
    process::{Child, Command, Stdio},
};
use rusty_ytdl::{Video, VideoOptions, VideoQuality, VideoSearchOptions};

pub struct VideoStream {
    ffmpeg: Child,
    frame_size: usize,
    pub width: usize,
    pub height: usize,
}

// 오디오 프로세스를 별도로 관리하기 위한 구조체
pub struct AudioStream {
    pub ffmpeg: Child,
}

impl VideoStream {
    // 반환 타입을 변경하여 오디오 스트림도 함께 반환 (Video URL, Audio URL 각각 추출)
    pub async fn new(url: &str, width: u32, height: u32) -> anyhow::Result<(Self, AudioStream)> {
        // rusty_ytdl을 사용하여 비디오 정보 가져오기
        let video = Video::new(url)?;
        let info = video.get_info().await?;
    
        // 1. 비디오 포맷 선택
        let video_format = rusty_ytdl::choose_format(&info.formats, &VideoOptions {
            quality: VideoQuality::HighestVideo,
            filter: VideoSearchOptions::Video,
            ..Default::default()
        })?;

        // 2. 오디오 포맷 선택 (추가됨)
        let audio_format = rusty_ytdl::choose_format(&info.formats, &VideoOptions {
            quality: VideoQuality::HighestAudio,
            filter: VideoSearchOptions::Audio,
            ..Default::default()
        })?;

        let video_url = video_format.url.as_str();
        let audio_url = audio_format.url.as_str();

        // 터미널 문자 비율 보정
        let char_aspect = 2.3;
        let real_height = (height as f32 * char_aspect) as u32;

        // 비디오용 ffmpeg 실행
        let ffmpeg_video = Command::new("../tools/ffmpeg/ffmpeg.exe")
            .args([
                "-i",
                video_url,
                "-an",
                "-vf",
                &format!("scale={}:{},fps=30,format=rgb24", width, real_height),
                "-f",
                "rawvideo",
                "pipe:1",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;

        // 오디오용 ffmpeg 실행 (추가됨)
        // PCM s16le 포맷, 44.1kHz, 2채널로 디코딩하여 파이프로 전송
        let ffmpeg_audio = Command::new("../tools/ffmpeg/ffmpeg.exe")
            .args([
                "-i",
                audio_url,
                "-vn",             // 비디오 제외
                "-f", "s16le",     // Signed 16-bit Little Endian PCM
                "-ac", "2",        // 2채널 (Stereo)
                "-ar", "44100",    // 샘플 레이트
                "-acodec", "pcm_s16le",
                "pipe:1",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;

        Ok((
            Self {
                ffmpeg: ffmpeg_video,
                frame_size: (width * real_height * 3) as usize,
                width: width as usize,
                height: real_height as usize,
            },
            AudioStream {
                ffmpeg: ffmpeg_audio,
            }
        ))
    }

    /// 그냥 "한 프레임"만 읽는다
    pub fn read_frame(&mut self, buf: &mut Vec<u8>) -> bool {
        buf.resize(self.frame_size, 0);
        self.ffmpeg.stdout.as_mut().unwrap().read_exact(buf).is_ok()
    }
}
