//! UVcore — Universal Vector Core
//!
//! SVG→everything conversion engine for uCode3.
//!
//! ## Output Formats
//!
//! - **Cell/CELX** — Grid-based sprite format for uCode2 display
//! - **ASCII/Teletext** — Character-based rendering for uCode1 terminals
//! - **Semantic Description** — AI-readable vector descriptions
//! - **Raster** — PNG/BMP output via tiny-skia
//!
//! ## Status
//!
//! Pre-release. API is unstable and subject to change.

pub mod parser;
pub mod render;
pub mod formats;
pub mod generate;

/// The current UVcore version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
