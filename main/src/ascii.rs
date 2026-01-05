use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};

const ASCII_TABLE: &[u8] = b"  .:-=+*#%@";

pub fn rgb_to_colored_ascii(rgb: &[u8], width: usize, height: usize, out: &mut Vec<Line>) {
    out.clear();

    let table_len = ASCII_TABLE.len() as u8;

    for y in (0..height - 1).step_by(2) {
        let mut spans = Vec::with_capacity(width);

        let mut x = 0;

        // 🔥 SIMD 4픽셀 처리
        while x + 4 <= width {
            unsafe {
                let mut lum = [0u8; 4];
                let mut colors = [(0u8, 0u8, 0u8); 4];

                for i in 0..4 {
                    let idx1 = ((y * width + (x + i)) * 3) as isize;
                    let idx2 = (((y + 1) * width + (x + i)) * 3) as isize;

                    let r = (rgb[idx1 as usize] as u16 + rgb[idx2 as usize] as u16) >> 1;
                    let g = (rgb[idx1 as usize + 1] as u16 + rgb[idx2 as usize + 1] as u16) >> 1;
                    let b = (rgb[idx1 as usize + 2] as u16 + rgb[idx2 as usize + 2] as u16) >> 1;

                    // 🔥 정수 밝기 근사
                    let l = ((54 * r + 183 * g + 19 * b) >> 8) as u8;

                    lum[i] = l;
                    colors[i] = ((r as u8), (g as u8), (b as u8));
                }

                for i in 0..4 {
                    let idx = (lum[i] as u16 * (table_len as u16 - 1) / 255) as usize;
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
            }

            x += 4;
        }

        // 🔹 나머지 픽셀 (fallback)
        while x < width {
            let i1 = (y * width + x) * 3;
            let i2 = ((y + 1) * width + x) * 3;

            let r = ((rgb[i1] as u16 + rgb[i2] as u16) >> 1) as u8;
            let g = ((rgb[i1 + 1] as u16 + rgb[i2 + 1] as u16) >> 1) as u8;
            let b = ((rgb[i1 + 2] as u16 + rgb[i2 + 2] as u16) >> 1) as u8;

            let l = ((54 * r as u16 + 183 * g as u16 + 19 * b as u16) >> 8) as u8;
            let idx = (l as usize * (ASCII_TABLE.len() - 1)) / 255;
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
