//! Output format converters
//!
//! Converts parsed SVG documents into various output formats.

use crate::parser::SvgDocument;

/// Convert SVG to CELX (grid-based sprite format)
pub fn to_celx(doc: &SvgDocument) -> anyhow::Result<String> {
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
pub fn to_ascii(doc: &SvgDocument) -> anyhow::Result<String> {
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
pub fn describe(doc: &SvgDocument) -> anyhow::Result<String> {
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
