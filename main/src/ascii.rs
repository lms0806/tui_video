use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use rayon::prelude::*; // Rayon 병렬 반복자 사용

const ASCII_TABLE: &[u8] = b"  .:-=+*#%@";

pub fn rgb_to_colored_ascii(rgb: &[u8], width: usize, height: usize, out: &mut Vec<Line>) {
    // 기존 벡터 재사용 대신 병렬 처리 결과를 수집
    // par_chunks_exact 등을 사용하여 y 단위로 병렬 처리 가능하지만,
    // Rayon은 collect()가 순서를 보장하므로 아래와 같이 작성 가능합니다.

    let lines: Vec<Line> = (0..height - 1)
        .into_par_iter() // 병렬 이터레이터로 변경
        .step_by(2)
        .map(|y| {
            let mut spans = Vec::with_capacity(width);
            let mut x = 0;
            let table_len = ASCII_TABLE.len();

            // ============================
            // SIMD path (4 pixels) - 로직 동일
            // ============================
            while x + 4 <= width {
                // ... (기존 로직과 동일) ...
                let mut lum = [0u8; 4];
                let mut colors = [(0u8, 0u8, 0u8); 4];

                for i in 0..4 {
                    let idx1 = (y * width + (x + i)) * 3;
                    let idx2 = ((y + 1) * width + (x + i)) * 3;
                    
                    // 범위 체크 없이 접근 (unsafe)를 사용하면 더 빠르지만 안전을 위해 유지
                    let r = (rgb[idx1] as u16 + rgb[idx2] as u16) >> 1;
                    let g = (rgb[idx1 + 1] as u16 + rgb[idx2 + 1] as u16) >> 1;
                    let b = (rgb[idx1 + 2] as u16 + rgb[idx2 + 2] as u16) >> 1;

                    colors[i] = (r as u8, g as u8, b as u8);
                    lum[i] = ((54 * r + 183 * g + 19 * b) >> 8) as u8;
                }
                
                for i in 0..4 {
                    let idx = lum[i] as usize * (table_len - 1) / 255;
                    let ch = ASCII_TABLE[idx] as char;
                    let (r, g, b) = colors[i];
                    
                    // Style::default() 호출 비용을 줄이려면 미리 상수로 정의하거나 재사용 고려
                    spans.push(Span::styled(
                         ch.to_string(),
                         Style::new().fg(Color::Rgb(
                            (r as f32 * 0.8) as u8,
                            (g as f32 * 0.8) as u8,
                            (b as f32 * 0.8) as u8,
                        )),
                    ));
                }
                x += 4;
            }

            // ============================
            // Scalar fallback - 로직 동일
            // ============================
            while x < width {
                 // ... (기존 로직과 동일) ...
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
                    Style::new().fg(Color::Rgb(
                        (r as f32 * 0.8) as u8,
                        (g as f32 * 0.8) as u8,
                        (b as f32 * 0.8) as u8,
                    )),
                ));

                x += 1;
            }
            Line::from(spans)
        })
        .collect();

    *out = lines;
}
