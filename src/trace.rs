//! Image Tracing and Vectorization Module
//!
//! Traces raster bitmaps (PNG, JPG, BMP) into scalable vector (SVG) paths,
//! styled with Mono Core design presets for Prose UI and GridCore.

use crate::char_map::quantize_luminance_to_teletext;
use serde::{Deserialize, Serialize};

/// Result of tracing a raster image.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracedVectorResult {
    pub width: usize,
    pub height: usize,
    pub style: String,
    pub element_count: usize,
    pub svg: String,
    pub teletext: String,
}

/// Trace a luminance or binary bitmap into an SVG document and Teletext screen.
pub fn trace_bitmap(
    grayscale: &[u8],
    width: usize,
    height: usize,
    threshold: u8,
    style_preset: &str,
) -> TracedVectorResult {
    // Determine colors based on style preset
    let (bg_color, _stroke_color, fill_color) = match style_preset {
        "mono_blueprint" => ("#0A2342", "#00E5FF", "#00E5FF"),
        "mono_paper" => ("#FAF8F5", "#111111", "#111111"),
        "mono_teletext" => ("#000000", "#FFFFFF", "#39C5CF"),
        "pixel_art" => ("#141013", "#FFFFFF", "#56C639"),
        _ => ("#FFFFFF", "#000000", "#000000"), // minimal_line / default
    };

    // Scan horizontal runs of foreground pixels (run-length path generation)
    let mut paths = Vec::new();
    let mut element_count = 0;

    for y in 0..height {
        let mut in_run = false;
        let mut run_start = 0;

        for x in 0..width {
            let lum = grayscale[y * width + x];
            let is_fg = lum >= threshold;

            if is_fg && !in_run {
                in_run = true;
                run_start = x;
            } else if !is_fg && in_run {
                in_run = false;
                paths.push(format!(
                    r#"<rect x="{}" y="{}" width="{}" height="1" fill="{}"/>"#,
                    run_start,
                    y,
                    x - run_start,
                    fill_color
                ));
                element_count += 1;
            }
        }

        if in_run {
            paths.push(format!(
                r#"<rect x="{}" y="{}" width="{}" height="1" fill="{}"/>"#,
                run_start,
                y,
                width - run_start,
                fill_color
            ));
            element_count += 1;
        }
    }

    // Build SVG wrapper
    let mut svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}" width="{}" height="{}" style="background:{};">"#,
        width, height, width, height, bg_color
    );
    svg.push_str("\n<!-- uVector Traced Geometry -->\n");
    for p in paths {
        svg.push_str(&p);
        svg.push('\n');
    }
    svg.push_str("</svg>");

    // Also generate GridCore Teletext 40x25 character grid
    let teletext_screen = quantize_luminance_to_teletext(grayscale, width, height, threshold);
    let teletext = teletext_screen.to_plain_text();

    TracedVectorResult {
        width,
        height,
        style: style_preset.to_string(),
        element_count,
        svg,
        teletext,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_simple_box() {
        let w = 20;
        let h = 20;
        let mut pixels = vec![0u8; w * h];
        // Draw 10x10 white box in center
        for y in 5..15 {
            for x in 5..15 {
                pixels[y * w + x] = 255;
            }
        }

        let result = trace_bitmap(&pixels, w, h, 128, "mono_blueprint");
        assert!(result.element_count >= 10);
        assert!(result.svg.contains("<svg"));
        assert!(result.svg.contains("#0A2342"));
        assert!(result.svg.contains("#00E5FF"));
        assert_eq!(result.teletext.lines().count(), 25);
    }
}
