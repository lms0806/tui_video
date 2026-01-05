use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};

const ASCII_TABLE: &[u8] = b"  .:-=+*#%@";

pub fn rgb_to_colored_ascii(rgb: &[u8], width: usize, height: usize, out: &mut Vec<Line>) {
    out.clear();

    let table_len = ASCII_TABLE.len();

    for y in (0..height - 1).step_by(2) {
        let mut spans = Vec::with_capacity(width);
        let mut x = 0;

        // ============================
        // SIMD path (4 pixels)
        // ============================
        while x + 4 <= width {
            let mut lum = [0u8; 4];
            let mut colors = [(0u8, 0u8, 0u8); 4];

            // RGB 평균 → u16
            for i in 0..4 {
                let idx1 = (y * width + (x + i)) * 3;
                let idx2 = ((y + 1) * width + (x + i)) * 3;

                let r = (rgb[idx1] as u16 + rgb[idx2] as u16) >> 1;
                let g = (rgb[idx1 + 1] as u16 + rgb[idx2 + 1] as u16) >> 1;
                let b = (rgb[idx1 + 2] as u16 + rgb[idx2 + 2] as u16) >> 1;

                colors[i] = (r as u8, g as u8, b as u8);
                lum[i] = ((54 * r + 183 * g + 19 * b) >> 8) as u8;
            }

            // ASCII + 출력
            for i in 0..4 {
                let idx = lum[i] as usize * (table_len - 1) / 255;
                let ch = ASCII_TABLE[idx] as char;
                let (r, g, b) = colors[i];

                spans.push(Span::styled(
                    ch.to_string(),
                    Style::default().fg(Color::Rgb(
                        (r as f32 * 0.8) as u8,
                        (g as f32 * 0.8) as u8,
                        (b as f32 * 0.8) as u8,
                    )),
                ));
            }
            x += 4;
        }

        // ============================
        // Scalar fallback
        // ============================
        while x < width {
            let i1 = (y * width + x) * 3;
            let i2 = ((y + 1) * width + x) * 3;

            let r = ((rgb[i1] as u16 + rgb[i2] as u16) >> 1) as u8;
            let g = ((rgb[i1 + 1] as u16 + rgb[i2 + 1] as u16) >> 1) as u8;
            let b = ((rgb[i1 + 2] as u16 + rgb[i2 + 2] as u16) >> 1) as u8;

            let l = ((54 * r as u16 + 183 * g as u16 + 19 * b as u16) >> 8) as u8;
            let idx = l as usize * (table_len - 1) / 255;
            let ch = ASCII_TABLE[idx] as char;

            spans.push(Span::styled(
                ch.to_string(),
                Style::default().fg(Color::Rgb(
                    (r as f32 * 0.8) as u8,
                    (g as f32 * 0.8) as u8,
                    (b as f32 * 0.8) as u8,
                )),
            ));

            x += 1;
        }

        out.push(Line::from(spans));
    }
}
