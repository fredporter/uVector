//! Canonical Palette Registry and Color Quantization Module
//!
//! uVector is the source of truth for bundled color palettes and customization definitions.
//! uCore and uCode consume base presets only.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single color definition within a palette.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PaletteColor {
    pub name: String,
    pub hex: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ansi: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

/// A color palette definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Palette {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub base_preset: bool,
    pub colors: Vec<PaletteColor>,
}

/// Palette registry containing all canonical palettes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PaletteRegistry {
    pub title: String,
    pub version: String,
    pub source_of_truth: String,
    pub description: String,
    pub palettes: HashMap<String, Palette>,
}

impl PaletteRegistry {
    /// Load canonical palettes from the bundled JSON, falling back to embedded baseline.
    pub fn load() -> Self {
        const EMBEDDED_JSON: &str = include_str!("../palettes/canonical_palettes.json");
        serde_json::from_str(EMBEDDED_JSON).unwrap_or_else(|_| Self::default_fallback())
    }

    /// Get a palette by ID.
    pub fn get(&self, id: &str) -> Option<&Palette> {
        self.palettes.get(id)
    }

    /// Return list of base presets only (for uCore and uCode consumption).
    pub fn get_base_presets(&self) -> Vec<&Palette> {
        self.palettes
            .values()
            .filter(|p| p.base_preset)
            .collect()
    }

    fn default_fallback() -> Self {
        let mut palettes = HashMap::new();
        palettes.insert(
            "teletext_ceefax".to_string(),
            Palette {
                id: "teletext_ceefax".to_string(),
                name: "Teletext Ceefax".to_string(),
                description: "8-color broadcast teletext standard palette".to_string(),
                base_preset: true,
                colors: vec![
                    PaletteColor { name: "black".to_string(), hex: "#000000".to_string(), ansi: Some(30), role: None },
                    PaletteColor { name: "red".to_string(), hex: "#E6193C".to_string(), ansi: Some(31), role: None },
                    PaletteColor { name: "green".to_string(), hex: "#3FB950".to_string(), ansi: Some(32), role: None },
                    PaletteColor { name: "yellow".to_string(), hex: "#F2CC60".to_string(), ansi: Some(33), role: None },
                    PaletteColor { name: "blue".to_string(), hex: "#58A6FF".to_string(), ansi: Some(34), role: None },
                    PaletteColor { name: "magenta".to_string(), hex: "#BC8CFF".to_string(), ansi: Some(35), role: None },
                    PaletteColor { name: "cyan".to_string(), hex: "#39C5CF".to_string(), ansi: Some(36), role: None },
                    PaletteColor { name: "white".to_string(), hex: "#FFFFFF".to_string(), ansi: Some(37), role: None },
                ],
            },
        );
        Self {
            title: "uVector Canonical Palette Registry".to_string(),
            version: "1.0.0".to_string(),
            source_of_truth: "uVector".to_string(),
            description: "Fallback palette registry".to_string(),
            palettes,
        }
    }
}

/// Parse a hex color string ("#RRGGBB" or "RRGGBB") into (r, g, b) tuple.
pub fn parse_hex(hex: &str) -> Option<(u8, u8, u8)> {
    let clean = hex.trim_start_matches('#');
    if clean.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&clean[0..2], 16).ok()?;
    let g = u8::from_str_radix(&clean[2..4], 16).ok()?;
    let b = u8::from_str_radix(&clean[4..6], 16).ok()?;
    Some((r, g, b))
}

/// Find the closest matching color in a palette using Euclidean RGB distance.
pub fn quantize_color<'a>(hex: &str, palette: &'a Palette) -> Option<&'a PaletteColor> {
    let (r1, g1, b1) = parse_hex(hex)?;
    let mut best_dist = u64::MAX;
    let mut best_match = None;

    for color in &palette.colors {
        if let Some((r2, g2, b2)) = parse_hex(&color.hex) {
            let dr = (r1 as i64) - (r2 as i64);
            let dg = (g1 as i64) - (g2 as i64);
            let db = (b1 as i64) - (b2 as i64);
            let dist = (dr * dr + dg * dg + db * db) as u64;
            if dist < best_dist {
                best_dist = dist;
                best_match = Some(color);
            }
        }
    }

    best_match
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_canonical_palettes() {
        let reg = PaletteRegistry::load();
        assert!(reg.palettes.len() >= 5);
        assert!(reg.get("teletext_ceefax").is_some());
        assert!(reg.get("architectural_blueprint").is_some());
        assert!(reg.get("editorial_linocut").is_some());
        // Verify Amber CRT is NOT in palettes
        assert!(reg.get("mono_amber").is_none());
        assert!(reg.get("amber_crt").is_none());
    }

    #[test]
    fn test_quantize_color() {
        let reg = PaletteRegistry::load();
        let tel = reg.get("teletext_ceefax").unwrap();

        // Bright green should match green
        let nearest = quantize_color("#00FF10", tel).unwrap();
        assert_eq!(nearest.name, "green");

        // Blue should match blue
        let nearest = quantize_color("#4090FF", tel).unwrap();
        assert_eq!(nearest.name, "blue");
    }
}
