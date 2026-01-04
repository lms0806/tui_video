use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};

/// 🎬 YouTube Shorts 최적화 ASCII (밝음 → 어두움)
const ASCII_TABLE: &[u8] = b"  .:-=+*#%@";

pub fn rgb_to_colored_ascii(rgb: &[u8], width: usize, height: usize, out: &mut Vec<Line>) {
    out.clear();

    let table_len = ASCII_TABLE.len() as f32;

    // 🔥 세로 2픽셀 → 문자 1개 (비율 보정)
    for y in (0..height - 1).step_by(2) {
        let mut spans = Vec::with_capacity(width);

        for x in 0..width {
            let i1 = (y * width + x) * 3;
            let i2 = ((y + 1) * width + x) * 3;

            if i2 + 2 >= rgb.len() {
                spans.push(Span::raw(" "));
                continue;
            }

            // 위/아래 픽셀 평균
            let r = ((rgb[i1] as u16 + rgb[i2] as u16) / 2) as f32;
            let g = ((rgb[i1 + 1] as u16 + rgb[i2 + 1] as u16) / 2) as f32;
            let b = ((rgb[i1 + 2] as u16 + rgb[i2 + 2] as u16) / 2) as f32;

            // 인간 시각 기반 밝기
            let mut luminance = 0.2126 * r + 0.7152 * g + 0.0722 * b;

            // 이부분만 제거하고 수정 X
            // 🔥 배경 제거 (쇼츠용)
            // if luminance < 28.0 {
            //     spans.push(Span::raw(" "));
            //     continue;
            // }

            // 대비 보정 (과하지 않게)
            luminance = (luminance - 128.0) * 1.05 + 128.0;
            luminance = luminance.clamp(0.0, 255.0);

            // ASCII 선택
            let idx = ((luminance / 255.0) * (table_len - 1.0))
                .round()
                .clamp(0.0, table_len - 1.0) as usize;

            let ch = ASCII_TABLE[idx] as char;

            // 컬러 톤다운 (문자 강조)
            let color = Color::Rgb((r * 0.8) as u8, (g * 0.8) as u8, (b * 0.8) as u8);

            spans.push(Span::styled(ch.to_string(), Style::default().fg(color)));
        }

        out.push(Line::from(spans));
    }
}
