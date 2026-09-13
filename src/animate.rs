//! Animated BOB & GIF Generator
//!
//! Rasterizes SVG frames, color-quantizes against canonical USX/uVector palettes,
//! and encodes optimized, looping GIF animations (portable BOBs).

use anyhow::{anyhow, Result};
use image::codecs::gif::{GifEncoder, Repeat};
use image::{Delay, Frame, Rgba, RgbaImage};
use std::io::Cursor;
use crate::palettes::{parse_hex, quantize_color, Palette, PaletteRegistry};
use crate::parser::parse_svg;
use crate::render::to_pixmap;

/// Options for GIF animation generation
#[derive(Debug, Clone)]
pub struct GifAnimationOptions {
    /// Delay between frames in milliseconds (e.g., 100 for 10fps, 125 for 8fps)
    pub delay_ms: u32,
    /// Whether the animation loops indefinitely
    pub repeat_infinite: bool,
    /// Optional canonical palette ID to quantize against
    pub palette_id: Option<String>,
}

impl Default for GifAnimationOptions {
    fn default() -> Self {
        Self {
            delay_ms: 100,
            repeat_infinite: true,
            palette_id: None,
        }
    }
}

/// Quantize an RgbaImage in-place against a canonical palette.
/// Pixels with alpha < 128 are mapped to fully transparent (0, 0, 0, 0).
pub fn quantize_frame(img: &mut RgbaImage, palette: &Palette) {
    for pixel in img.pixels_mut() {
        if pixel[3] < 128 {
            *pixel = Rgba([0, 0, 0, 0]);
            continue;
        }

        let hex = format!("#{:02X}{:02X}{:02X}", pixel[0], pixel[1], pixel[2]);
        if let Some(matched) = quantize_color(&hex, palette) {
            if let Some((r, g, b)) = parse_hex(&matched.hex) {
                *pixel = Rgba([r, g, b, 255]);
            }
        }
    }
}

/// Render a series of SVG keyframe strings to an animated GIF byte vector.
pub fn svgs_to_gif<S: AsRef<str>>(
    svg_frames: &[S],
    options: &GifAnimationOptions,
) -> Result<Vec<u8>> {
    if svg_frames.is_empty() {
        return Err(anyhow!("Cannot generate GIF from empty frame list"));
    }

    let registry = PaletteRegistry::load();
    let target_palette = options
        .palette_id
        .as_deref()
        .and_then(|id| registry.get(id));

    let delay = Delay::from_numer_denom_ms(options.delay_ms.max(20), 1);
    let mut frames = Vec::new();

    for svg_str in svg_frames {
        let doc = parse_svg(svg_str.as_ref())
            .map_err(|e| anyhow!("Failed to parse SVG frame: {:?}", e))?;
        let pixmap = to_pixmap(&doc)?;
        let width = pixmap.width();
        let height = pixmap.height();

        let mut rgba_img = RgbaImage::from_raw(width, height, pixmap.data().to_vec())
            .ok_or_else(|| anyhow!("Failed to construct RgbaImage from pixmap"))?;

        if let Some(pal) = target_palette {
            quantize_frame(&mut rgba_img, pal);
        }

        let frame = Frame::from_parts(rgba_img, 0, 0, delay);
        frames.push(frame);
    }

    let mut output = Vec::new();
    {
        let mut encoder = GifEncoder::new(Cursor::new(&mut output));
        if options.repeat_infinite {
            encoder.set_repeat(Repeat::Infinite)?;
        }
        encoder.encode_frames(frames.into_iter())?;
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_svgs_to_gif_basic() {
        let frame1 = r##"<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32"><rect width="32" height="32" fill="#E6193C"/></svg>"##;
        let frame2 = r##"<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32"><rect width="32" height="32" fill="#3FB950"/></svg>"##;

        let options = GifAnimationOptions {
            delay_ms: 100,
            repeat_infinite: true,
            palette_id: Some("teletext_ceefax".to_string()),
        };

        let gif_bytes = svgs_to_gif(&[frame1, frame2], &options).expect("GIF encoding failed");
        assert!(!gif_bytes.is_empty());
        assert_eq!(&gif_bytes[0..3], b"GIF");
    }

    #[test]
    fn test_empty_frames_error() {
        let options = GifAnimationOptions::default();
        let result = svgs_to_gif::<&str>(&[], &options);
        assert!(result.is_err());
    }
}
