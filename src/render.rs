//! Raster rendering module
//!
//! Renders SVG documents to raster formats (PNG) using tiny-skia.
//! Now with full SVG path/shape rendering pipeline.

use crate::parser::SvgDocument;
use tiny_skia::*;

/// Render SVG to PNG bytes with full shape rendering
pub fn to_png(doc: &SvgDocument<'_>) -> anyhow::Result<Vec<u8>> {
    let width = (doc.width as u32).max(1);
    let height = (doc.height as u32).max(1);

    // Create a pixmap
    let mut pixmap = Pixmap::new(width, height)
        .ok_or_else(|| anyhow::anyhow!("Failed to create pixmap"))?;

    // Fill with white background
    pixmap.fill(Color::WHITE);

    // Render all SVG elements
    render_svg_elements(&doc, &mut pixmap)?;

    // Encode to PNG
    let png_data = pixmap.encode_png()?;
    Ok(png_data)
}

/// Render SVG elements onto a pixmap
fn render_svg_elements(doc: &SvgDocument<'_>, pixmap: &mut Pixmap) -> anyhow::Result<()> {
    let root = doc.doc.root_element();

    for node in root.descendants() {
        if !node.is_element() {
            continue;
        }

        let tag = node.tag_name().name();
        match tag {
            "rect" => render_rect(node, pixmap)?,
            "circle" => render_circle(node, pixmap)?,
            "ellipse" => render_ellipse(node, pixmap)?,
            "line" => render_line(node, pixmap)?,
            "polygon" => render_polygon(node, pixmap)?,
            "polyline" => render_polyline(node, pixmap)?,
            "path" => render_path(node, pixmap)?,
            "text" => render_text(node, pixmap)?,
            _ => {} // Skip unknown elements
        }
    }

    Ok(())
}

/// Parse a fill colour attribute into a tiny-skia Color
fn parse_fill_color(node: &roxmltree::Node) -> Option<Color> {
    let fill = node.attribute("fill").unwrap_or("black");
    parse_color(fill)
}

/// Parse a stroke colour attribute
fn parse_stroke_color(node: &roxmltree::Node) -> Option<Color> {
    let stroke = node.attribute("stroke");
    stroke.and_then(parse_color)
}

/// Parse a CSS colour string into a tiny-skia Color
fn parse_color(s: &str) -> Option<Color> {
    let s = s.trim();
    match s {
        "black" | "#000" | "#000000" => Some(Color::BLACK),
        "white" | "#fff" | "#ffffff" => Some(Color::WHITE),
        "red" | "#f00" | "#ff0000" => Some(Color::from_rgba8(255, 0, 0, 255)),
        "green" | "#0f0" | "#00ff00" => Some(Color::from_rgba8(0, 128, 0, 255)),
        "lime" => Some(Color::from_rgba8(0, 255, 0, 255)),
        "blue" | "#00f" | "#0000ff" => Some(Color::from_rgba8(0, 0, 255, 255)),
        "yellow" | "#ff0" | "#ffff00" => Some(Color::from_rgba8(255, 255, 0, 255)),
        "cyan" | "#0ff" | "#00ffff" => Some(Color::from_rgba8(0, 255, 255, 255)),
        "magenta" | "#f0f" | "#ff00ff" => Some(Color::from_rgba8(255, 0, 255, 255)),
        "gray" | "grey" | "#808080" => Some(Color::from_rgba8(128, 128, 128, 255)),
        "orange" | "#ffa500" => Some(Color::from_rgba8(255, 165, 0, 255)),
        "purple" | "#800080" => Some(Color::from_rgba8(128, 0, 128, 255)),
        "transparent" | "none" => Some(Color::from_rgba8(0, 0, 0, 0)),
        // Try hex parsing
        _ if s.starts_with('#') => {
            let hex = &s[1..];
            if hex.len() == 3 {
                let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
                let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
                let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
                Some(Color::from_rgba8(r, g, b, 255))
            } else if hex.len() == 6 {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Color::from_rgba8(r, g, b, 255))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Parse a float attribute
fn parse_float(node: &roxmltree::Node, attr: &str) -> Option<f32> {
    node.attribute(attr)?.parse().ok()
}

/// Parse an opacity attribute
fn parse_opacity(node: &roxmltree::Node) -> f32 {
    node.attribute("opacity")
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(1.0)
}

/// Apply opacity to a Color (Color stores f32 channels)
fn apply_opacity(color: Color, opacity: f32) -> Color {
    Color::from_rgba(
        color.red(),
        color.green(),
        color.blue(),
        color.alpha() * opacity,
    ).unwrap_or(color)
}

/// Render a <rect> element
fn render_rect(node: roxmltree::Node, pixmap: &mut Pixmap) -> anyhow::Result<()> {
    let x = parse_float(&node, "x").unwrap_or(0.0);
    let y = parse_float(&node, "y").unwrap_or(0.0);
    let w = parse_float(&node, "width").unwrap_or(0.0);
    let h = parse_float(&node, "height").unwrap_or(0.0);
    let rx = parse_float(&node, "rx").unwrap_or(0.0);
    let ry = parse_float(&node, "ry").unwrap_or(0.0);

    if w <= 0.0 || h <= 0.0 {
        return Ok(());
    }

    let fill = parse_fill_color(&node);
    let stroke = parse_stroke_color(&node);
    let stroke_width = parse_float(&node, "stroke-width").unwrap_or(1.0);
    let opacity = parse_opacity(&node);

    let rect = Rect::from_xywh(x, y, w, h)
        .ok_or_else(|| anyhow::anyhow!("Invalid rect dimensions"))?;

    let mut paint = Paint::default();
    paint.anti_alias = true;

    // Fill
    if let Some(fill_color) = fill {
        if fill_color.is_opaque() || fill_color.alpha() > 0.0 {
            paint.set_color(apply_opacity(fill_color, opacity));

            if rx > 0.0 || ry > 0.0 {
                let mut pb = PathBuilder::new();
                pb.push_rect(rect);
                if let Some(path) = pb.finish() {
                    pixmap.fill_path(&path, &paint, FillRule::Winding, Transform::identity(), None);
                }
            } else {
                pixmap.fill_rect(rect, &paint, Transform::identity(), None);
            }
        }
    }

    // Stroke
    if let Some(stroke_color) = stroke {
        if (stroke_color.is_opaque() || stroke_color.alpha() > 0.0) && stroke_width > 0.0 {
            let mut stroke_paint = Paint::default();
            stroke_paint.anti_alias = true;
            stroke_paint.set_color(apply_opacity(stroke_color, opacity));

            let stroke = Stroke {
                width: stroke_width,
                ..Stroke::default()
            };

            let mut pb = PathBuilder::new();
            pb.push_rect(rect);
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, &stroke_paint, &stroke, Transform::identity(), None);
            }
        }
    }

    Ok(())
}

/// Render a <circle> element
fn render_circle(node: roxmltree::Node, pixmap: &mut Pixmap) -> anyhow::Result<()> {
    let cx = parse_float(&node, "cx").unwrap_or(0.0);
    let cy = parse_float(&node, "cy").unwrap_or(0.0);
    let r = parse_float(&node, "r").unwrap_or(0.0);

    if r <= 0.0 {
        return Ok(());
    }

    let fill = parse_fill_color(&node);
    let stroke = parse_stroke_color(&node);
    let stroke_width = parse_float(&node, "stroke-width").unwrap_or(1.0);
    let opacity = parse_opacity(&node);

    let rect = Rect::from_xywh(cx - r, cy - r, r * 2.0, r * 2.0)
        .ok_or_else(|| anyhow::anyhow!("Invalid circle dimensions"))?;

    let mut paint = Paint::default();
    paint.anti_alias = true;

    if let Some(fill_color) = fill {
        if fill_color.is_opaque() || fill_color.alpha() > 0.0 {
            paint.set_color(apply_opacity(fill_color, opacity));

            let mut pb = PathBuilder::new();
            pb.push_oval(rect);
            if let Some(path) = pb.finish() {
                pixmap.fill_path(&path, &paint, FillRule::Winding, Transform::identity(), None);
            }
        }
    }

    if let Some(stroke_color) = stroke {
        if (stroke_color.is_opaque() || stroke_color.alpha() > 0.0) && stroke_width > 0.0 {
            let mut stroke_paint = Paint::default();
            stroke_paint.anti_alias = true;
            stroke_paint.set_color(apply_opacity(stroke_color, opacity));

            let stroke = Stroke {
                width: stroke_width,
                ..Stroke::default()
            };

            let mut pb = PathBuilder::new();
            pb.push_oval(rect);
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, &stroke_paint, &stroke, Transform::identity(), None);
            }
        }
    }

    Ok(())
}

/// Render an <ellipse> element
fn render_ellipse(node: roxmltree::Node, pixmap: &mut Pixmap) -> anyhow::Result<()> {
    let cx = parse_float(&node, "cx").unwrap_or(0.0);
    let cy = parse_float(&node, "cy").unwrap_or(0.0);
    let rx = parse_float(&node, "rx").unwrap_or(0.0);
    let ry = parse_float(&node, "ry").unwrap_or(0.0);

    if rx <= 0.0 || ry <= 0.0 {
        return Ok(());
    }

    let fill = parse_fill_color(&node);
    let stroke = parse_stroke_color(&node);
    let stroke_width = parse_float(&node, "stroke-width").unwrap_or(1.0);
    let opacity = parse_opacity(&node);

    let rect = Rect::from_xywh(cx - rx, cy - ry, rx * 2.0, ry * 2.0)
        .ok_or_else(|| anyhow::anyhow!("Invalid ellipse dimensions"))?;

    let mut paint = Paint::default();
    paint.anti_alias = true;

    if let Some(fill_color) = fill {
        if fill_color.is_opaque() || fill_color.alpha() > 0.0 {
            paint.set_color(apply_opacity(fill_color, opacity));

            let mut pb = PathBuilder::new();
            pb.push_oval(rect);
            if let Some(path) = pb.finish() {
                pixmap.fill_path(&path, &paint, FillRule::Winding, Transform::identity(), None);
            }
        }
    }

    if let Some(stroke_color) = stroke {
        if (stroke_color.is_opaque() || stroke_color.alpha() > 0.0) && stroke_width > 0.0 {
            let mut stroke_paint = Paint::default();
            stroke_paint.anti_alias = true;
            stroke_paint.set_color(apply_opacity(stroke_color, opacity));

            let stroke = Stroke {
                width: stroke_width,
                ..Stroke::default()
            };

            let mut pb = PathBuilder::new();
            pb.push_oval(rect);
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, &stroke_paint, &stroke, Transform::identity(), None);
            }
        }
    }

    Ok(())
}

/// Render a <line> element
fn render_line(node: roxmltree::Node, pixmap: &mut Pixmap) -> anyhow::Result<()> {
    let x1 = parse_float(&node, "x1").unwrap_or(0.0);
    let y1 = parse_float(&node, "y1").unwrap_or(0.0);
    let x2 = parse_float(&node, "x2").unwrap_or(0.0);
    let y2 = parse_float(&node, "y2").unwrap_or(0.0);

    let stroke = parse_stroke_color(&node).unwrap_or(Color::BLACK);
    let stroke_width = parse_float(&node, "stroke-width").unwrap_or(1.0);
    let opacity = parse_opacity(&node);

    if stroke_width <= 0.0 {
        return Ok(());
    }

    let mut paint = Paint::default();
    paint.anti_alias = true;
    paint.set_color(apply_opacity(stroke, opacity));

    let stroke_style = Stroke {
        width: stroke_width,
        ..Stroke::default()
    };

    let mut path_builder = PathBuilder::new();
    path_builder.move_to(x1, y1);
    path_builder.line_to(x2, y2);
    if let Some(path) = path_builder.finish() {
        pixmap.stroke_path(&path, &paint, &stroke_style, Transform::identity(), None);
    }

    Ok(())
}

/// Render a <polygon> element
fn render_polygon(node: roxmltree::Node, pixmap: &mut Pixmap) -> anyhow::Result<()> {
    let points_str = node.attribute("points").unwrap_or("");
    let points = parse_points(points_str);

    if points.len() < 6 {
        return Ok(()); // Need at least 3 points (6 coords)
    }

    let fill = parse_fill_color(&node);
    let stroke = parse_stroke_color(&node);
    let stroke_width = parse_float(&node, "stroke-width").unwrap_or(1.0);
    let opacity = parse_opacity(&node);

    let mut path_builder = PathBuilder::new();
    path_builder.move_to(points[0], points[1]);
    for i in (2..points.len()).step_by(2) {
        path_builder.line_to(points[i], points[i + 1]);
    }
    path_builder.close();

    if let Some(path) = path_builder.finish() {
        let mut paint = Paint::default();
        paint.anti_alias = true;

        if let Some(fill_color) = fill {
            if fill_color.is_opaque() || fill_color.alpha() > 0.0 {
                paint.set_color(apply_opacity(fill_color, opacity));
                pixmap.fill_path(&path, &paint, FillRule::Winding, Transform::identity(), None);
            }
        }

        if let Some(stroke_color) = stroke {
            if (stroke_color.is_opaque() || stroke_color.alpha() > 0.0) && stroke_width > 0.0 {
                paint.set_color(apply_opacity(stroke_color, opacity));
                let stroke_style = Stroke {
                    width: stroke_width,
                    ..Stroke::default()
                };
                pixmap.stroke_path(&path, &paint, &stroke_style, Transform::identity(), None);
            }
        }
    }

    Ok(())
}

/// Render a <polyline> element
fn render_polyline(node: roxmltree::Node, pixmap: &mut Pixmap) -> anyhow::Result<()> {
    let points_str = node.attribute("points").unwrap_or("");
    let points = parse_points(points_str);

    if points.len() < 4 {
        return Ok(()); // Need at least 2 points (4 coords)
    }

    let stroke = parse_stroke_color(&node).unwrap_or(Color::BLACK);
    let stroke_width = parse_float(&node, "stroke-width").unwrap_or(1.0);
    let opacity = parse_opacity(&node);

    if stroke_width <= 0.0 {
        return Ok(());
    }

    let mut path_builder = PathBuilder::new();
    path_builder.move_to(points[0], points[1]);
    for i in (2..points.len()).step_by(2) {
        path_builder.line_to(points[i], points[i + 1]);
    }

    if let Some(path) = path_builder.finish() {
        let mut paint = Paint::default();
        paint.anti_alias = true;
        paint.set_color(apply_opacity(stroke, opacity));

        let stroke_style = Stroke {
            width: stroke_width,
            ..Stroke::default()
        };
        pixmap.stroke_path(&path, &paint, &stroke_style, Transform::identity(), None);
    }

    Ok(())
}

/// Render a <path> element
fn render_path(node: roxmltree::Node, pixmap: &mut Pixmap) -> anyhow::Result<()> {
    let d = node.attribute("d").unwrap_or("");
    if d.is_empty() {
        return Ok(());
    }

    let fill = parse_fill_color(&node);
    let stroke = parse_stroke_color(&node);
    let stroke_width = parse_float(&node, "stroke-width").unwrap_or(1.0);
    let opacity = parse_opacity(&node);

    // Parse SVG path data into tiny-skia path
    if let Some(path) = parse_svg_path(d) {
        let mut paint = Paint::default();
        paint.anti_alias = true;

        if let Some(fill_color) = fill {
            if fill_color.is_opaque() || fill_color.alpha() > 0.0 {
                paint.set_color(apply_opacity(fill_color, opacity));
                pixmap.fill_path(&path, &paint, FillRule::Winding, Transform::identity(), None);
            }
        }

        if let Some(stroke_color) = stroke {
            if (stroke_color.is_opaque() || stroke_color.alpha() > 0.0) && stroke_width > 0.0 {
                paint.set_color(apply_opacity(stroke_color, opacity));
                let stroke_style = Stroke {
                    width: stroke_width,
                    ..Stroke::default()
                };
                pixmap.stroke_path(&path, &paint, &stroke_style, Transform::identity(), None);
            }
        }
    }

    Ok(())
}

/// Render a <text> element (simplified — renders as filled rects for now)
fn render_text(node: roxmltree::Node, pixmap: &mut Pixmap) -> anyhow::Result<()> {
    let content = node.text().unwrap_or("");
    if content.is_empty() {
        return Ok(());
    }

    let x = parse_float(&node, "x").unwrap_or(0.0);
    let y = parse_float(&node, "y").unwrap_or(0.0);
    let fill = parse_fill_color(&node).unwrap_or(Color::BLACK);
    let font_size = parse_float(&node, "font-size").unwrap_or(16.0);
    let opacity = parse_opacity(&node);

    // Simplified: render text as a small filled rectangle placeholder
    // Full text rendering would require a font rasterizer
    let text_width = content.len() as f32 * font_size * 0.6;
    let text_height = font_size;

    if let Some(rect) = Rect::from_xywh(x, y - text_height, text_width, text_height) {
        let mut paint = Paint::default();
        paint.anti_alias = true;
        paint.set_color(apply_opacity(fill, opacity * 0.3));
        pixmap.fill_rect(rect, &paint, Transform::identity(), None);
    }

    Ok(())
}

/// Parse SVG path data string into a tiny-skia Path
fn parse_svg_path(d: &str) -> Option<Path> {
    let mut builder = PathBuilder::new();
    let mut tokens = d.split_whitespace().peekable();
    let mut current_x = 0.0f32;
    let mut current_y = 0.0f32;
    let mut start_x = 0.0f32;
    let mut start_y = 0.0f32;
    let mut first_command = true;

    while let Some(token) = tokens.next() {
        match token {
            "M" | "m" => {
                let x = tokens.next()?.parse().ok()?;
                let y = tokens.next()?.parse().ok()?;
                let is_relative = token == "m";
                let (px, py) = if is_relative {
                    (current_x + x, current_y + y)
                } else {
                    (x, y)
                };
                if first_command {
                    builder.move_to(px, py);
                    first_command = false;
                } else {
                    builder.line_to(px, py);
                }
                current_x = px;
                current_y = py;
                start_x = px;
                start_y = py;
            }
            "L" | "l" => {
                let x = tokens.next()?.parse().ok()?;
                let y = tokens.next()?.parse().ok()?;
                let is_relative = token == "l";
                let (px, py) = if is_relative {
                    (current_x + x, current_y + y)
                } else {
                    (x, y)
                };
                builder.line_to(px, py);
                current_x = px;
                current_y = py;
            }
            "H" | "h" => {
                let x = tokens.next()?.parse().ok()?;
                let is_relative = token == "h";
                let px = if is_relative { current_x + x } else { x };
                builder.line_to(px, current_y);
                current_x = px;
            }
            "V" | "v" => {
                let y = tokens.next()?.parse().ok()?;
                let is_relative = token == "v";
                let py = if is_relative { current_y + y } else { y };
                builder.line_to(current_x, py);
                current_y = py;
            }
            "C" | "c" => {
                let x1 = tokens.next()?.parse().ok()?;
                let y1 = tokens.next()?.parse().ok()?;
                let x2 = tokens.next()?.parse().ok()?;
                let y2 = tokens.next()?.parse().ok()?;
                let x = tokens.next()?.parse().ok()?;
                let y = tokens.next()?.parse().ok()?;
                let is_relative = token == "c";
                let (cp1x, cp1y) = if is_relative {
                    (current_x + x1, current_y + y1)
                } else {
                    (x1, y1)
                };
                let (cp2x, cp2y) = if is_relative {
                    (current_x + x2, current_y + y2)
                } else {
                    (x2, y2)
                };
                let (px, py) = if is_relative {
                    (current_x + x, current_y + y)
                } else {
                    (x, y)
                };
                builder.cubic_to(cp1x, cp1y, cp2x, cp2y, px, py);
                current_x = px;
                current_y = py;
            }
            "Q" | "q" => {
                let x1 = tokens.next()?.parse().ok()?;
                let y1 = tokens.next()?.parse().ok()?;
                let x = tokens.next()?.parse().ok()?;
                let y = tokens.next()?.parse().ok()?;
                let is_relative = token == "q";
                let (cpx, cpy) = if is_relative {
                    (current_x + x1, current_y + y1)
                } else {
                    (x1, y1)
                };
                let (px, py) = if is_relative {
                    (current_x + x, current_y + y)
                } else {
                    (x, y)
                };
                builder.quad_to(cpx, cpy, px, py);
                current_x = px;
                current_y = py;
            }
            "Z" | "z" => {
                builder.close();
                current_x = start_x;
                current_y = start_y;
            }
            _ => {
                // Try to parse as numbers (implicit L/l commands)
                if let Ok(x) = token.parse::<f32>() {
                    if let Some(y_str) = tokens.next() {
                        if let Ok(y) = y_str.parse::<f32>() {
                            builder.line_to(x, y);
                            current_x = x;
                            current_y = y;
                        }
                    }
                }
            }
        }
    }

    builder.finish()
}

/// Parse SVG points attribute string into a flat vec of f32 coordinates
fn parse_points(s: &str) -> Vec<f32> {
    s.split(|c: char| c.is_whitespace() || c == ',')
        .filter_map(|p| p.parse::<f32>().ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_color_hex() {
        let c = parse_color("#ff0000").unwrap();
        assert!((c.red() - 1.0).abs() < 0.01);
        assert!((c.green() - 0.0).abs() < 0.01);
        assert!((c.blue() - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_color_named() {
        let c = parse_color("blue").unwrap();
        assert!((c.red() - 0.0).abs() < 0.01);
        assert!((c.green() - 0.0).abs() < 0.01);
        assert!((c.blue() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_points() {
        let pts = parse_points("10,20 30,40 50,60");
        assert_eq!(pts, vec![10.0, 20.0, 30.0, 40.0, 50.0, 60.0]);
    }
}
