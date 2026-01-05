mod app;
mod ascii;
mod tui;
mod video;

use std::{
    io::{self, stdout, Read},
    time::{Duration, Instant},
};

use app::App;
use ascii::rgb_to_colored_ascii;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::EnterAlternateScreen,
};
use ratatui::{
    text::Text,
    widgets::{Paragraph, Wrap},
};
// rodio 임포트 추가
use rodio::{OutputStream, Sink, Source};
use tokio::time::sleep;
use video::VideoStream;

// --- 추가된 구조체: 오디오 스트림 소스 정의 ---
// rodio가 이터레이터의 오디오 속성(채널, 샘플레이트)을 알 수 있도록 직접 Source를 구현합니다.
struct AudioStreamSource {
    rx: std::sync::mpsc::IntoIter<i16>,
}

impl Iterator for AudioStreamSource {
    type Item = i16;
    fn next(&mut self) -> Option<Self::Item> {
        self.rx.next()
    }
}

impl Source for AudioStreamSource {
    fn current_frame_len(&self) -> Option<usize> {
        None // 길이를 알 수 없음 (스트리밍)
    }
    fn channels(&self) -> u16 {
        2 // 스테레오
    }
    fn sample_rate(&self) -> u32 {
        44100 // 44.1kHz
    }
    fn total_duration(&self) -> Option<Duration> {
        None // 전체 길이를 알 수 없음
    }
}
// ------------------------------------------

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut url = String::new();
    println!("Input YouTube URL:");
    io::stdin().read_line(&mut url)?;
    let url = url.trim();

    let width = 120;
    let height = 50;
    let frame_time = Duration::from_millis(30);

    let mut app = App::new();
    
    // 오디오 시스템 초기화
    let (_stream, stream_handle) = OutputStream::try_default()?;
    let sink = Sink::try_new(&stream_handle)?;

    let mut terminal = tui::init()?;
    execute!(stdout(), EnterAlternateScreen)?;

    let (mut video, mut audio) = VideoStream::new(url, width, height).await?;

    // --- 오디오 연결 로직 수정됨 ---
    // mpsc 채널 생성
    let (tx, rx) = std::sync::mpsc::channel::<i16>();

    if let Some(mut stdout) = audio.ffmpeg.stdout.take() {
        // 별도 스레드에서 ffmpeg 출력을 읽어 채널로 전송
        std::thread::spawn(move || {
            let mut buf = [0u8; 2];
            while stdout.read_exact(&mut buf).is_ok() {
                let sample = i16::from_le_bytes(buf);
                if tx.send(sample).is_err() { break; }
            }
        });
    }

    // 커스텀 소스 사용
    let source = AudioStreamSource { rx: rx.into_iter() };
    
    sink.append(source);
    sink.play();
    // ----------------------------

    let mut rgb_buf = Vec::new();
    let mut ascii_lines = Vec::new();

    while !app.should_quit {
        let start = Instant::now();

        if event::poll(Duration::from_millis(1))? {
            if let Event::Key(k) = event::read()? {
                match k.code {
                    KeyCode::Char('q') => app.should_quit = true,
                    KeyCode::Char(' ') => {
                        app.playing = !app.playing;
                        // 오디오 일시정지/재생 연동
                        if app.playing {
                            sink.play();
                        } else {
                            sink.pause();
                        }
                    },
                    _ => {}
                }
            }
        }

        if app.playing {
            if !video.read_frame(&mut rgb_buf) {
                break;
            }

            // 🔥 문자 비율 보정은 ASCII 단계에서만
            rgb_to_colored_ascii(&rgb_buf, video.width, video.height, &mut ascii_lines);

            terminal.draw(|f| {
                let p = Paragraph::new(Text::from(ascii_lines.clone())).wrap(Wrap { trim: false });
                f.render_widget(p, f.area());
            })?;
        }

        let elapsed = start.elapsed();
        if elapsed < frame_time {
            sleep(frame_time - elapsed).await;
        }
    }

    tui::restore()?;
    Ok(())
}
