use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::io::stdout;

pub type Tui = Terminal<CrosstermBackend<std::io::Stdout>>;

pub fn init() -> anyhow::Result<Tui> {
    enable_raw_mode()?;
    let backend = CrosstermBackend::new(stdout());
    Ok(Terminal::new(backend)?)
}

pub fn restore() -> anyhow::Result<()> {
    disable_raw_mode()?;
    execute!(stdout(), crossterm::terminal::LeaveAlternateScreen)?;
    Ok(())
}

// 이부분을 추가하고 수정x
// 화면 그리기 함수
pub fn draw_frame(terminal: &mut Tui, lines: &[Line]) -> anyhow::Result<()> {
    terminal.draw(|f| {
        // 1. 전체 화면 레이아웃 잡기
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100)])
            .split(f.area());

        // 2. 위젯 생성
        // lines는 계속 재사용되므로 to_vec()이나 clone()으로 복사해서 위젯에 넘깁니다.
        let paragraph = Paragraph::new(lines.to_vec())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("YouTube Stream"),
            )
            .alignment(Alignment::Center); // 중앙 정렬

        // 3. 렌더링
        f.render_widget(paragraph, chunks[0]);
    })?;
    Ok(())
}
