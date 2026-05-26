//! SVG parser module
//!
//! Parses SVG documents into an intermediate representation
//! that can be consumed by format converters.

use roxmltree::Document;

/// A parsed SVG document with extracted metadata
#[derive(Debug, Clone)]
pub struct SvgDocument {
    /// Width in user units (from viewBox or width attr)
    pub width: f64,
    /// Height in user units
    pub height: f64,
    /// Raw XML document for detailed traversal
    pub doc: Document,
}

/// Parse an SVG string into an SvgDocument
pub fn parse_svg(svg_content: &str) -> anyhow::Result<SvgDocument> {
    let doc = Document::parse(svg_content)?;

    let root = doc.root_element();

    // Extract dimensions
    let width = root
        .attribute("width")
        .and_then(parse_length)
        .unwrap_or(100.0);

    let height = root
        .attribute("height")
        .and_then(parse_length)
        .unwrap_or(100.0);

    Ok(SvgDocument { width, height, doc })
}

/// Parse a length attribute (e.g., "100", "100px")
fn parse_length(s: &str) -> Option<f64> {
    let s = s.trim();
    if let Some(px) = s.strip_suffix("px") {
        px.trim().parse().ok()
    } else {
        s.parse().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_svg() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="100">
            <rect x="10" y="10" width="50" height="50" fill="red"/>
        </svg>"#;

        let result = parse_svg(svg).unwrap();
        assert_eq!(result.width, 200.0);
        assert_eq!(result.height, 100.0);
    }
}
