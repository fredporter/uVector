//! Raster rendering module
//!
//! Renders SVG documents to raster formats (PNG) using tiny-skia.

use crate::parser::SvgDocument;

/// Render SVG to PNG bytes
pub fn to_png(doc: &SvgDocument) -> anyhow::Result<Vec<u8>> {
    let width = doc.width as u32;
    let height = doc.height as u32;

    // Create a pixmap
    let mut pixmap = tiny_skia::Pixmap::new(width.max(1), height.max(1))
        .ok_or_else(|| anyhow::anyhow!("Failed to create pixmap"))?;

    // Fill with white background
    pixmap.fill(tiny_skia::Color::WHITE);

    // TODO: Full SVG rendering pipeline
    // For now, this is a placeholder that returns an empty white PNG

    // Encode to PNG
    let png_data = pixmap.encode_png()?;
    Ok(png_data)
}
