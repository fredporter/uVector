//! Output format converters
//!
//! Converts parsed SVG documents into various output formats.

use crate::parser::SvgDocument;

/// Convert SVG to CELX (grid-based sprite format)
pub fn to_celx(doc: &SvgDocument<'_>) -> anyhow::Result<String> {
    let mut output = String::new();

    output.push_str(&format!("CELX {} {}\n", doc.width as u32, doc.height as u32));
    output.push_str("LAYER 0\n");

    // Traverse SVG elements and convert to CELX grid commands
    for node in doc.doc.root_element().descendants() {
        if node.is_element() {
            let tag = node.tag_name().name();
            match tag {
                "rect" => {
                    let x = node.attribute("x").unwrap_or("0");
                    let y = node.attribute("y").unwrap_or("0");
                    let w = node.attribute("width").unwrap_or("0");
                    let h = node.attribute("height").unwrap_or("0");
                    let fill = node.attribute("fill").unwrap_or("black");
                    output.push_str(&format!("RECT {} {} {} {} {}\n", x, y, w, h, fill));
                }
                "circle" => {
                    let cx = node.attribute("cx").unwrap_or("0");
                    let cy = node.attribute("cy").unwrap_or("0");
                    let r = node.attribute("r").unwrap_or("0");
                    let fill = node.attribute("fill").unwrap_or("black");
                    output.push_str(&format!("CIRCLE {} {} {} {}\n", cx, cy, r, fill));
                }
                _ => {}
            }
        }
    }

    Ok(output)
}

/// Convert SVG to ASCII/Teletext art
pub fn to_ascii(doc: &SvgDocument<'_>) -> anyhow::Result<String> {
    let width = doc.width as usize;
    let height = doc.height as usize;

    // Create a simple ASCII grid
    let mut grid = vec![vec![' '; width.min(80)]; height.min(24)];

    // Fill based on element positions (simplified)
    for node in doc.doc.root_element().descendants() {
        if node.is_element() {
            let tag = node.tag_name().name();
            match tag {
                "rect" => {
                    let x: usize = node.attribute("x").unwrap_or("0").parse().unwrap_or(0);
                    let y: usize = node.attribute("y").unwrap_or("0").parse().unwrap_or(0);
                    let w: usize = node.attribute("width").unwrap_or("1").parse().unwrap_or(1);
                    let h: usize = node.attribute("height").unwrap_or("1").parse().unwrap_or(1);

                    for row in y..(y + h).min(grid.len()) {
                        for col in x..(x + w).min(grid[0].len()) {
                            grid[row][col] = '#';
                        }
                    }
                }
                _ => {}
            }
        }
    }

    let mut output = String::new();
    for row in &grid {
        let line: String = row.iter().collect();
        output.push_str(&line);
        output.push('\n');
    }

    Ok(output)
}

/// Generate a semantic description of the SVG content
pub fn describe(doc: &SvgDocument<'_>) -> anyhow::Result<String> {
    let mut elements = Vec::new();

    for node in doc.doc.root_element().descendants() {
        if node.is_element() {
            let tag = node.tag_name().name();
            match tag {
                "rect" => {
                    let x = node.attribute("x").unwrap_or("0");
                    let y = node.attribute("y").unwrap_or("0");
                    let w = node.attribute("width").unwrap_or("0");
                    let h = node.attribute("height").unwrap_or("0");
                    let fill = node.attribute("fill").unwrap_or("black");
                    elements.push(format!("  - Rectangle at ({}, {}) size {}×{} fill={}", x, y, w, h, fill));
                }
                "circle" => {
                    let cx = node.attribute("cx").unwrap_or("0");
                    let cy = node.attribute("cy").unwrap_or("0");
                    let r = node.attribute("r").unwrap_or("0");
                    let fill = node.attribute("fill").unwrap_or("black");
                    elements.push(format!("  - Circle at center ({}, {}) radius={} fill={}", cx, cy, r, fill));
                }
                "path" => {
                    let d = node.attribute("d").unwrap_or("");
                    let fill = node.attribute("fill").unwrap_or("black");
                    elements.push(format!("  - Path: \"{}\" fill={}", d, fill));
                }
                "text" => {
                    let content = node.text().unwrap_or("");
                    let x = node.attribute("x").unwrap_or("0");
                    let y = node.attribute("y").unwrap_or("0");
                    elements.push(format!("  - Text \"{}\" at ({}, {})", content, x, y));
                }
                _ => {}
            }
        }
    }

    let mut output = String::new();
    output.push_str(&format!("SVG Document: {}×{} units\n", doc.width, doc.height));
    output.push_str(&format!("Elements ({}):\n", elements.len()));
    for elem in elements {
        output.push_str(&elem);
        output.push('\n');
    }

    Ok(output)
}

/// Convert SVG to 40×25 Teletext G1 mosaic character grid
pub fn to_teletext(doc: &SvgDocument<'_>) -> anyhow::Result<String> {
    let pixmap = crate::render::to_pixmap(doc)?;
    let w = pixmap.width() as usize;
    let h = pixmap.height() as usize;
    let data = pixmap.data();

    // Convert RGBA to luminance grayscale (Rec. 601: 0.299 R + 0.587 G + 0.114 B)
    let mut grayscale = Vec::with_capacity(w * h);
    for chunk in data.chunks_exact(4) {
        let r = chunk[0] as f32;
        let g = chunk[1] as f32;
        let b = chunk[2] as f32;
        let a = chunk[3] as f32 / 255.0;
        let lum = ((0.299 * r + 0.587 * g + 0.114 * b) * a) as u8;
        grayscale.push(lum);
    }

    let screen = crate::char_map::quantize_luminance_to_teletext(&grayscale, w, h, 128);
    Ok(screen.to_plain_text())
}

/// Convert SVG to GridCore BOB JSON on the 4×4 dot lattice
pub fn to_bob(
    doc: &SvgDocument<'_>,
    palette_id: Option<&str>,
    width_dots: Option<u32>,
    height_dots: Option<u32>,
) -> anyhow::Result<String> {
    let bob = crate::bob::svg_to_bob(doc, "svg_bob", "SVG BOB", palette_id, width_dots, height_dots)?;
    Ok(serde_json::to_string_pretty(&bob)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_teletext() {
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="80" height="50"><rect x="0" y="0" width="80" height="50" fill="#ffffff"/></svg>"##;
        let doc = crate::parser::parse_svg(svg).unwrap();
        let teletext = to_teletext(&doc).unwrap();
        assert_eq!(teletext.lines().count(), 25);
        assert!(teletext.contains('█'));
    }
}
