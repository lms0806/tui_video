mod app;
mod ascii;
mod tui;
mod video;

use std::{
    io::{self, stdout},
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
use tokio::time::sleep;
use video::VideoStream;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("YouTube URL:");
    let mut url = String::new();
    io::stdin().read_line(&mut url)?;
    let url = url.trim();

    let width = 120;
    let height = 50;
    let frame_time = Duration::from_millis(33);

    let mut app = App::new();
    let mut terminal = tui::init()?;
    execute!(stdout(), EnterAlternateScreen)?;

    let mut video = VideoStream::new(url, width, height)?;

    let mut rgb_buf = Vec::new();
    let mut ascii_lines = Vec::new();

    while !app.should_quit {
        let start = Instant::now();

        if event::poll(Duration::from_millis(1))? {
            if let Event::Key(k) = event::read()? {
                match k.code {
                    KeyCode::Char('q') => app.should_quit = true,
                    KeyCode::Char(' ') => app.playing = !app.playing,
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
                f.render_widget(p, f.size());
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
