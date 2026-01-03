use std::{
    io::Read,
    process::{Child, Command, Stdio},
};

pub struct VideoStream {
    _ytdlp: Child,
    ffmpeg: Child,
    frame_size: usize,
    pub width: usize,
    pub height: usize,
}

impl VideoStream {
    pub fn new(url: &str, width: u32, height: u32) -> anyhow::Result<Self> {
        // yt-dlp → stdout
        let mut ytdlp = Command::new("../tools/yt-dlp/yt-dlp.exe")
            .args(["-f", "bestvideo[ext=mp4]/best", "-o", "-", url])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;

        let ytdlp_stdout = ytdlp.stdout.take().unwrap();

        // 터미널 문자 비율 보정
        let char_aspect = 2.3;
        let real_height = (height as f32 * char_aspect) as u32;

        // 🔥 핵심: fps=30 강제
        let ffmpeg = Command::new("../tools/ffmpeg/ffmpeg.exe")
            .args([
                "-i", "pipe:0",
                "-an",
                "-vf",
                &format!(
                    "scale={}:{},fps=30,format=rgb24",
                    width, real_height
                ),
                "-f", "rawvideo",
                "pipe:1",
            ])
            .stdin(Stdio::from(ytdlp_stdout))
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;

        Ok(Self {
            _ytdlp: ytdlp,
            ffmpeg,
            frame_size: (width * real_height * 3) as usize,
            width: width as usize,
            height: real_height as usize,
        })
    }

    /// 그냥 "한 프레임"만 읽는다
    pub fn read_frame(&mut self, buf: &mut Vec<u8>) -> bool {
        buf.resize(self.frame_size, 0);
        self.ffmpeg
            .stdout
            .as_mut()
            .unwrap()
            .read_exact(buf)
            .is_ok()
    }
}
