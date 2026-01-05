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

impl VideoStream {
    pub async fn new(url: &str, width: u32, height: u32) -> anyhow::Result<Self> {
        // rusty_ytdl을 사용하여 비디오 정보 가져오기
        let video = Video::new(url)?;
        let info = video.get_info().await?;
        
        // 가장 좋은 화질의 비디오 포맷 선택 (오디오 제외, Video only)
        // 수정: &info -> &info.formats
        let format = rusty_ytdl::choose_format(&info.formats, &VideoOptions {
            quality: VideoQuality::HighestVideo,
            filter: VideoSearchOptions::Video,
            ..Default::default()
        })?;

        let stream_url = format.url.as_str();

        // 터미널 문자 비율 보정
        let char_aspect = 2.3;
        let real_height = (height as f32 * char_aspect) as u32;

        // 🔥 핵심: fps=30 강제
        // yt-dlp 파이프 대신 직접 추출한 URL을 ffmpeg 입력으로 사용
        let ffmpeg = Command::new("../tools/ffmpeg/ffmpeg.exe")
            .args([
                "-i",
                stream_url, // URL 직접 전달
                "-an",
                "-vf",
                &format!("scale={}:{},fps=30,format=rgb24", width, real_height),
                "-f",
                "rawvideo",
                "pipe:1",
            ])
            .stdin(Stdio::null()) // 입력 파이프 제거
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;

        Ok(Self {
            ffmpeg,
            frame_size: (width * real_height * 3) as usize,
            width: width as usize,
            height: real_height as usize,
        })
    }

    /// 그냥 "한 프레임"만 읽는다
    pub fn read_frame(&mut self, buf: &mut Vec<u8>) -> bool {
        buf.resize(self.frame_size, 0);
        self.ffmpeg.stdout.as_mut().unwrap().read_exact(buf).is_ok()
    }
}
