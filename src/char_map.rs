//! GridCore Character Mapping and Block Mosaic Engine
//!
//! Provides character block mapping, 40×25 Teletext G1 mosaic translation,
//! and 8×8 / 16×16 tile quantization for uCode GridCore.

use serde::{Deserialize, Serialize};

/// A single character cell on a GridCore / Teletext display.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GridCell {
    pub col: usize,
    pub row: usize,
    pub char_code: u8,
    pub glyph: char,
    pub fg_color: String,
    pub bg_color: String,
}

/// 40×25 GridCore Teletext Screen buffer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeletextScreen {
    pub columns: usize,
    pub rows: usize,
    pub cells: Vec<GridCell>,
}

impl TeletextScreen {
    pub fn new() -> Self {
        let columns = 40;
        let rows = 25;
        let mut cells = Vec::with_capacity(columns * rows);
        for row in 0..rows {
            for col in 0..columns {
                cells.push(GridCell {
                    col,
                    row,
                    char_code: 0x20,
                    glyph: ' ',
                    fg_color: "#FFFFFF".to_string(),
                    bg_color: "#000000".to_string(),
                });
            }
        }
        Self { columns, rows, cells }
    }

    pub fn get_cell(&self, col: usize, row: usize) -> Option<&GridCell> {
        if col < self.columns && row < self.rows {
            Some(&self.cells[row * self.columns + col])
        } else {
            None
        }
    }

    pub fn set_cell(&mut self, col: usize, row: usize, char_code: u8, glyph: char, fg: &str, bg: &str) {
        if col < self.columns && row < self.rows {
            let idx = row * self.columns + col;
            self.cells[idx] = GridCell {
                col,
                row,
                char_code,
                glyph,
                fg_color: fg.to_string(),
                bg_color: bg.to_string(),
            };
        }
    }

    /// Convert the screen into a plain text representation (40 chars × 25 lines).
    pub fn to_plain_text(&self) -> String {
        let mut out = String::with_capacity(self.columns * self.rows + self.rows);
        for row in 0..self.rows {
            for col in 0..self.columns {
                let cell = &self.cells[row * self.columns + col];
                out.push(cell.glyph);
            }
            out.push('\n');
        }
        out
    }

    /// Render to clean SVG with 40×25 character blocks.
    pub fn to_svg(&self) -> String {
        let char_w = 12;
        let char_h = 20;
        let width = self.columns * char_w;
        let height = self.rows * char_h;

        let mut svg = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}" width="{}" height="{}" style="background-color:#000000;font-family:monospace;font-size:16px;">"#,
            width, height, width, height
        );

        for cell in &self.cells {
            let x = cell.col * char_w;
            let y = cell.row * char_h;

            if cell.bg_color != "#000000" {
                svg.push_str(&format!(
                    r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}"/>"#,
                    x, y, char_w, char_h, cell.bg_color
                ));
            }

            if cell.glyph != ' ' {
                svg.push_str(&format!(
                    r#"<text x="{}" y="{}" fill="{}" dominant-baseline="hanging">{}</text>"#,
                    x + 1, y + 2, cell.fg_color, escape_xml(cell.glyph)
                ));
            }
        }

        svg.push_str("</svg>");
        svg
    }
}

fn escape_xml(c: char) -> String {
    match c {
        '<' => "&lt;".to_string(),
        '>' => "&gt;".to_string(),
        '&' => "&amp;".to_string(),
        '"' => "&quot;".to_string(),
        '\'' => "&apos;".to_string(),
        _ => c.to_string(),
    }
}

/// Map a 2×3 binary pixel sub-block matrix to a G1 mosaic character code and unicode block glyph.
/// Bits: 0=TL, 1=TR, 2=ML, 3=MR, 4=BL, 6=BR
pub fn sub_blocks_to_g1_mosaic(bits: [bool; 6]) -> (u8, char) {
    let mut bitmask: u8 = 0;
    if bits[0] { bitmask |= 1 << 0; }
    if bits[1] { bitmask |= 1 << 1; }
    if bits[2] { bitmask |= 1 << 2; }
    if bits[3] { bitmask |= 1 << 3; }
    if bits[4] { bitmask |= 1 << 4; }
    if bits[5] { bitmask |= 1 << 6; } // Teletext bit 6 is bottom-right

    let char_code = 0x20 + (bitmask & 0x5F);

    // Map common bit combinations to standard unicode block elements
    let glyph = match bitmask & 0x5F {
        0 => ' ',
        0x5F => '█', // full block
        0x15 => '▌', // left half
        0x4A => '▐', // right half
        0x03 => '▀', // upper half
        0x5C => '▄', // lower half
        0x01 | 0x04 | 0x10 => '░',
        0x02 | 0x08 | 0x40 => '▒',
        _ => '▓',
    };

    (char_code, glyph)
}

/// Quantize a luminance bitmap (grayscale values 0..255) into a 40×25 TeletextScreen.
pub fn quantize_luminance_to_teletext(
    grayscale: &[u8],
    img_width: usize,
    img_height: usize,
    threshold: u8,
) -> TeletextScreen {
    let mut screen = TeletextScreen::new();

    let cell_w = (img_width as f32) / 40.0;
    let cell_h = (img_height as f32) / 25.0;

    for row in 0..25 {
        for col in 0..40 {
            let start_x = (col as f32 * cell_w) as usize;
            let end_x = (((col + 1) as f32 * cell_w) as usize).min(img_width);
            let start_y = (row as f32 * cell_h) as usize;
            let end_y = (((row + 1) as f32 * cell_h) as usize).min(img_height);

            // Sample 2×3 sub-blocks inside this character cell
            let mut bits = [false; 6];
            let sub_w = ((end_x - start_x) as f32 / 2.0).max(1.0);
            let sub_h = ((end_y - start_y) as f32 / 3.0).max(1.0);

            for (sub_idx, (sx, sy)) in [(0,0), (1,0), (0,1), (1,1), (0,2), (1,2)].iter().enumerate() {
                let bx = (start_x as f32 + (*sx as f32) * sub_w) as usize;
                let by = (start_y as f32 + (*sy as f32) * sub_h) as usize;
                if bx < img_width && by < img_height {
                    let pixel_lum = grayscale[by * img_width + bx];
                    if pixel_lum >= threshold {
                        bits[sub_idx] = true;
                    }
                }
            }

            let (code, glyph) = sub_blocks_to_g1_mosaic(bits);
            screen.set_cell(col, row, code, glyph, "#FFFFFF", "#000000");
        }
    }

    screen
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_teletext_screen_geometry() {
        let screen = TeletextScreen::new();
        assert_eq!(screen.columns, 40);
        assert_eq!(screen.rows, 25);
        assert_eq!(screen.cells.len(), 1000);
        let text = screen.to_plain_text();
        assert_eq!(text.lines().count(), 25);
    }

    #[test]
    fn test_g1_mosaic_full_block() {
        let (code, glyph) = sub_blocks_to_g1_mosaic([true, true, true, true, true, true]);
        assert_eq!(glyph, '█');
        assert!(code > 0x20);
    }

    #[test]
    fn test_quantize_solid_white() {
        let white_img = vec![255u8; 80 * 50];
        let screen = quantize_luminance_to_teletext(&white_img, 80, 50, 128);
        assert_eq!(screen.cells[0].glyph, '█');
    }
}
