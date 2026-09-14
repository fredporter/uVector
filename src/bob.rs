//! GridCore BOB (Blitter Object) & Dot Lattice Quantizer
//!
//! Implements canonical specifications from:
//! - global-knowledge/standards/GRIDCORE-STANDARDS.json
//! - global-knowledge/standards/DEVICE-DISPLAY-STANDARDS.md
//!
//! Invariants:
//! - 1 dot = 4×4 physical device pixels.
//! - Terminal square cell = 2×2 dots (8×8 px).
//! - Teletext tall cell = 3×5 dots (12×20 px).
//! - Super-cell = 4×4 dots (16×16 px).
//! - Max BOB dimension: 32 dots (128 px).
//! - Max RAM budget: 128 KB.
//! - Max GIF budget: 60 KB.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::palettes::{quantize_color, Palette, PaletteRegistry};
use crate::parser::{parse_svg, SvgDocument};
use crate::render::to_pixmap_transparent;

pub const DOT_PX: u32 = 4;
pub const MAX_BOB_DIM_PX: u32 = 128;
pub const MAX_BOB_DIM_DOTS: u32 = MAX_BOB_DIM_PX / DOT_PX; // 32 dots
pub const MAX_BOB_RAM_BYTES: usize = 128 * 1024; // 128 KB
pub const MAX_BOB_GIF_BYTES: usize = 60 * 1024; // 60 KB

pub const SQUARE_CELL_DOTS_W: u32 = 2;
pub const SQUARE_CELL_DOTS_H: u32 = 2;
pub const TALL_CELL_DOTS_W: u32 = 3;
pub const TALL_CELL_DOTS_H: u32 = 5;
pub const SUPER_CELL_DOTS_W: u32 = 4;
pub const SUPER_CELL_DOTS_H: u32 = 4;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BobFrameJson {
    pub width_dots: u32,
    pub height_dots: u32,
    pub dots: Vec<u8>,
    pub duration_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BobDefinitionJson {
    pub id: String,
    pub name: String,
    pub width_dots: u32,
    pub height_dots: u32,
    pub width_px: u32,
    pub height_px: u32,
    pub transparent_index: u8,
    pub palette: String,
    pub palette_colors: Vec<String>,
    pub frames: Vec<BobFrameJson>,
    pub ram_footprint_bytes: usize,
    pub fits_budget: bool,
}

/// Quantize an RGB color to a 1-based index in the given palette.
/// Index 0 is reserved for transparent.
pub fn quantize_color_index(r: u8, g: u8, b: u8, palette: &Palette) -> u8 {
    let hex = format!("#{:02X}{:02X}{:02X}", r, g, b);
    if let Some(matched) = quantize_color(&hex, palette) {
        if let Some(idx) = palette.colors.iter().position(|c| c.name == matched.name) {
            // 1-based palette index (0 is transparent)
            return (idx as u8) + 1;
        }
    }
    1
}

/// Downsample an SVG pixmap down to the 4×4 dot lattice.
pub fn rasterize_svg_to_dots(
    doc: &SvgDocument<'_>,
    width_dots: u32,
    height_dots: u32,
    palette: &Palette,
) -> Result<Vec<u8>> {
    let pixmap = to_pixmap_transparent(doc)?;
    let p_w = pixmap.width();
    let p_h = pixmap.height();
    let data = pixmap.data();

    let mut dots = Vec::with_capacity((width_dots * height_dots) as usize);

    for dy in 0..height_dots {
        for dx in 0..width_dots {
            // Sample the 4x4 physical pixel block corresponding to this dot
            let start_x = dx * DOT_PX;
            let start_y = dy * DOT_PX;

            let mut sum_r: u32 = 0;
            let mut sum_g: u32 = 0;
            let mut sum_b: u32 = 0;
            let mut opaque_count: u32 = 0;

            for py in 0..DOT_PX {
                let y = start_y + py;
                if y >= p_h {
                    continue;
                }
                for px in 0..DOT_PX {
                    let x = start_x + px;
                    if x >= p_w {
                        continue;
                    }

                    let offset = ((y * p_w + x) * 4) as usize;
                    if offset + 3 < data.len() {
                        let a = data[offset + 3];
                        if a >= 128 {
                            sum_r += data[offset] as u32;
                            sum_g += data[offset + 1] as u32;
                            sum_b += data[offset + 2] as u32;
                            opaque_count += 1;
                        }
                    }
                }
            }

            // If mostly transparent, map to transparent index 0
            if opaque_count < 4 {
                dots.push(0);
            } else {
                let avg_r = (sum_r / opaque_count) as u8;
                let avg_g = (sum_g / opaque_count) as u8;
                let avg_b = (sum_b / opaque_count) as u8;
                let color_idx = quantize_color_index(avg_r, avg_g, avg_b, palette);
                dots.push(color_idx);
            }
        }
    }

    Ok(dots)
}

/// Convert a single SVG document to a BobDefinitionJson.
pub fn svg_to_bob(
    doc: &SvgDocument<'_>,
    id: &str,
    name: &str,
    palette_id: Option<&str>,
    width_dots: Option<u32>,
    height_dots: Option<u32>,
) -> Result<BobDefinitionJson> {
    let registry = PaletteRegistry::load();
    let target_pal_id = palette_id.unwrap_or("teletext_ceefax");
    let palette = registry
        .get(target_pal_id)
        .ok_or_else(|| anyhow!("Unknown palette '{}'", target_pal_id))?;

    let w_dots = width_dots.unwrap_or_else(|| ((doc.width as u32 + DOT_PX - 1) / DOT_PX).max(1));
    let h_dots = height_dots.unwrap_or_else(|| ((doc.height as u32 + DOT_PX - 1) / DOT_PX).max(1));

    if w_dots > MAX_BOB_DIM_DOTS || h_dots > MAX_BOB_DIM_DOTS {
        return Err(anyhow!(
            "BOB dimensions ({}×{} dots = {}×{} px) exceed maximum {}×{} px limit",
            w_dots,
            h_dots,
            w_dots * DOT_PX,
            h_dots * DOT_PX,
            MAX_BOB_DIM_PX,
            MAX_BOB_DIM_PX
        ));
    }

    let dots = rasterize_svg_to_dots(doc, w_dots, h_dots, palette)?;
    let frame = BobFrameJson {
        width_dots: w_dots,
        height_dots: h_dots,
        dots,
        duration_ms: 100,
    };

    let w_px = w_dots * DOT_PX;
    let h_px = h_dots * DOT_PX;
    let ram_footprint_bytes = (w_px * h_px * 1) as usize;
    let fits_budget = ram_footprint_bytes <= MAX_BOB_RAM_BYTES;

    let palette_colors = palette.colors.iter().map(|c| c.hex.clone()).collect();

    Ok(BobDefinitionJson {
        id: id.to_string(),
        name: name.to_string(),
        width_dots: w_dots,
        height_dots: h_dots,
        width_px: w_px,
        height_px: h_px,
        transparent_index: 0,
        palette: target_pal_id.to_string(),
        palette_colors,
        frames: vec![frame],
        ram_footprint_bytes,
        fits_budget,
    })
}

/// Convert multiple SVG frames into a multi-frame BobDefinitionJson.
pub fn svgs_to_bob(
    svg_frames: &[&str],
    id: &str,
    name: &str,
    palette_id: Option<&str>,
    width_dots: Option<u32>,
    height_dots: Option<u32>,
    delay_ms: u32,
) -> Result<BobDefinitionJson> {
    if svg_frames.is_empty() {
        return Err(anyhow!("Cannot generate BOB from empty frame list"));
    }

    let registry = PaletteRegistry::load();
    let target_pal_id = palette_id.unwrap_or("teletext_ceefax");
    let palette = registry
        .get(target_pal_id)
        .ok_or_else(|| anyhow!("Unknown palette '{}'", target_pal_id))?;

    let first_doc = parse_svg(svg_frames[0])
        .map_err(|e| anyhow!("Failed to parse SVG frame 0: {:?}", e))?;

    let w_dots = width_dots.unwrap_or_else(|| ((first_doc.width as u32 + DOT_PX - 1) / DOT_PX).max(1));
    let h_dots = height_dots.unwrap_or_else(|| ((first_doc.height as u32 + DOT_PX - 1) / DOT_PX).max(1));

    if w_dots > MAX_BOB_DIM_DOTS || h_dots > MAX_BOB_DIM_DOTS {
        return Err(anyhow!(
            "BOB dimensions ({}×{} dots) exceed maximum {}×{} px",
            w_dots,
            h_dots,
            MAX_BOB_DIM_PX,
            MAX_BOB_DIM_PX
        ));
    }

    let mut frames = Vec::with_capacity(svg_frames.len());
    for (i, svg_str) in svg_frames.iter().enumerate() {
        let doc = parse_svg(svg_str)
            .map_err(|e| anyhow!("Failed to parse SVG frame {}: {:?}", i, e))?;
        let dots = rasterize_svg_to_dots(&doc, w_dots, h_dots, palette)?;
        frames.push(BobFrameJson {
            width_dots: w_dots,
            height_dots: h_dots,
            dots,
            duration_ms: delay_ms.max(20),
        });
    }

    let w_px = w_dots * DOT_PX;
    let h_px = h_dots * DOT_PX;
    let ram_footprint_bytes = (w_px * h_px * frames.len() as u32) as usize;
    let fits_budget = ram_footprint_bytes <= MAX_BOB_RAM_BYTES;

    let palette_colors = palette.colors.iter().map(|c| c.hex.clone()).collect();

    Ok(BobDefinitionJson {
        id: id.to_string(),
        name: name.to_string(),
        width_dots: w_dots,
        height_dots: h_dots,
        width_px: w_px,
        height_px: h_px,
        transparent_index: 0,
        palette: target_pal_id.to_string(),
        palette_colors,
        frames,
        ram_footprint_bytes,
        fits_budget,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dot_lattice_invariants() {
        assert_eq!(DOT_PX, 4);
        assert_eq!(MAX_BOB_DIM_DOTS, 32);
        assert_eq!(SQUARE_CELL_DOTS_W, 2);
        assert_eq!(SQUARE_CELL_DOTS_H, 2);
        assert_eq!(TALL_CELL_DOTS_W, 3);
        assert_eq!(TALL_CELL_DOTS_H, 5);
        assert_eq!(SUPER_CELL_DOTS_W, 4);
        assert_eq!(SUPER_CELL_DOTS_H, 4);
    }

    #[test]
    fn test_svg_to_bob_basic() {
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32"><rect width="32" height="32" fill="#E6193C"/></svg>"##;
        let doc = parse_svg(svg).unwrap();
        let bob = svg_to_bob(&doc, "test_bob", "Test BOB", Some("teletext_ceefax"), Some(8), Some(8))
            .expect("Failed to convert SVG to BOB");

        assert_eq!(bob.width_dots, 8);
        assert_eq!(bob.height_dots, 8);
        assert_eq!(bob.width_px, 32);
        assert_eq!(bob.height_px, 32);
        assert_eq!(bob.frames.len(), 1);
        assert_eq!(bob.frames[0].dots.len(), 64);
        // Red maps to non-zero palette index in teletext_ceefax
        assert_ne!(bob.frames[0].dots[0], 0);
        assert!(bob.fits_budget);
    }

    #[test]
    fn test_multiframe_bob() {
        let frame1 = r##"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16"><rect width="16" height="16" fill="#3FB950"/></svg>"##;
        let frame2 = r##"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16"><rect width="16" height="16" fill="#58A6FF"/></svg>"##;

        let bob = svgs_to_bob(&[frame1, frame2], "walker", "Walker BOB", None, Some(4), Some(4), 80)
            .expect("Failed to build multi-frame BOB");

        assert_eq!(bob.frames.len(), 2);
        assert_eq!(bob.width_dots, 4);
        assert_eq!(bob.height_dots, 4);
        assert_eq!(bob.ram_footprint_bytes, 16 * 16 * 2);
        assert!(bob.fits_budget);
    }

    #[test]
    fn test_oversized_bob_rejected() {
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="200"><rect width="200" height="200" fill="#FFFFFF"/></svg>"##;
        let doc = parse_svg(svg).unwrap();
        let result = svg_to_bob(&doc, "giant", "Giant", None, Some(33), Some(10));
        assert!(result.is_err());
    }
}
