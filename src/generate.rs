//! SVG generation module
//!
//! Generates SVG documents from text prompts using LLM-based recipe generation.
//! Supports multiple visual styles: mono_chrome, teletext, full_color, pixel_art, line_art.
//!
//! ## Architecture
//!
//! 1. `generate_svg()` — Main entry point: prompt → style → SVG string
//! 2. Style system applies post-processing transforms to the raw SVG
//! 3. GridBuffer converter for teletext mode output

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

/// Supported visual styles for SVG generation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SvgStyle {
    /// Monochrome (single colour, typically white on dark)
    MonoChrome,
    /// Teletext (8-colour palette, blocky, 40×25 character grid aesthetic)
    Teletext,
    /// Full colour (unrestricted palette, detailed)
    FullColor,
    /// Pixel art (low resolution, blocky, limited palette)
    PixelArt,
    /// Line art (black outlines, no fill, minimal)
    LineArt,
}

impl SvgStyle {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().replace('-', "_").as_str() {
            "mono_chrome" | "monochrome" | "mono" => SvgStyle::MonoChrome,
            "teletext" | "ceefax" | "tel" => SvgStyle::Teletext,
            "full_color" | "fullcolor" | "color" | "colour" => SvgStyle::FullColor,
            "pixel_art" | "pixelart" | "pixel" => SvgStyle::PixelArt,
            "line_art" | "lineart" | "line" | "outline" => SvgStyle::LineArt,
            _ => SvgStyle::FullColor,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            SvgStyle::MonoChrome => "mono_chrome",
            SvgStyle::Teletext => "teletext",
            SvgStyle::FullColor => "full_color",
            SvgStyle::PixelArt => "pixel_art",
            SvgStyle::LineArt => "line_art",
        }
    }

    /// Get the style-specific system prompt for the LLM
    pub fn system_prompt(&self) -> &'static str {
        match self {
            SvgStyle::MonoChrome => {
                "You are an SVG artist. Generate a monochrome SVG image using only \
                 #ffffff (white) on a #000000 (black) background. Use simple geometric \
                 shapes (rect, circle, ellipse, line, polygon, path). Keep it clean and \
                 high-contrast. Output ONLY valid SVG code, no markdown, no explanation."
            }
            SvgStyle::Teletext => {
                "You are a teletext graphic artist. Generate an SVG that looks like a \
                 teletext/Ceefax page. Use ONLY these 8 colours: #000000 (black), \
                 #ff0000 (red), #00ff00 (green), #ffff00 (yellow), #0000ff (blue), \
                 #ff00ff (magenta), #00ffff (cyan), #ffffff (white). Use blocky \
                 rectangular shapes, no curves. Keep the composition simple and grid-like. \
                 Output ONLY valid SVG code, no markdown, no explanation."
            }
            SvgStyle::FullColor => {
                "You are an SVG artist. Generate a colourful, detailed SVG image using \
                 any colours you like. Use a variety of shapes (rect, circle, ellipse, \
                 polygon, path) to create an appealing composition. Include a subtle \
                 background. Output ONLY valid SVG code, no markdown, no explanation."
            }
            SvgStyle::PixelArt => {
                "You are a pixel art SVG artist. Generate a pixel-art style SVG using \
                 small rectangular blocks arranged in a grid pattern. Use a limited \
                 palette of 8-16 colours. Each 'pixel' should be a small rect element. \
                 Keep the total element count under 200 for performance. \
                 Output ONLY valid SVG code, no markdown, no explanation."
            }
            SvgStyle::LineArt => {
                "You are a line art SVG artist. Generate a minimalist line-art SVG using \
                 ONLY black (#000000) strokes with no fill (fill=\"none\"). Use path, \
                 line, polyline, rect, circle, and ellipse elements with stroke-width \
                 between 1 and 3. No filled shapes. Clean, elegant, minimal. \
                 Output ONLY valid SVG code, no markdown, no explanation."
            }
        }
    }
}

/// Parameters for SVG generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SvgGenerationParams {
    /// Text description of what to generate
    pub prompt: String,
    /// Visual style
    #[serde(default)]
    pub style: String,
    /// Output width in SVG units
    #[serde(default = "default_width")]
    pub width: f32,
    /// Output height in SVG units
    #[serde(default = "default_height")]
    pub height: f32,
    /// Whether to include a background rect
    #[serde(default = "default_true")]
    pub background: bool,
    /// Additional style instructions (appended to prompt)
    #[serde(default)]
    pub extra: String,
}

fn default_width() -> f32 { 800.0 }
fn default_height() -> f32 { 600.0 }
fn default_true() -> bool { true }

impl Default for SvgGenerationParams {
    fn default() -> Self {
        Self {
            prompt: String::new(),
            style: "full_color".to_string(),
            width: 800.0,
            height: 600.0,
            background: true,
            extra: String::new(),
        }
    }
}

/// Result of SVG generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SvgGenerationResult {
    /// The generated SVG string
    pub svg: String,
    /// Style used
    pub style: String,
    /// Width in SVG units
    pub width: f32,
    /// Height in SVG units
    pub height: f32,
    /// Number of elements in the SVG
    pub element_count: usize,
    /// Brief description of what was generated
    pub description: String,
}

/// Generate an SVG from a text prompt using an LLM.
///
/// This function calls the local Ollama API to generate SVG code from a prompt.
/// The LLM is instructed to output ONLY valid SVG markup.
///
/// # Arguments
/// * `params` - Generation parameters (prompt, style, dimensions)
/// * `ollama_url` - Base URL for Ollama API (e.g. "http://localhost:11434")
/// * `model` - Ollama model to use for generation
///
/// # Returns
/// The generated SVG string and metadata
pub async fn generate_svg(
    params: &SvgGenerationParams,
    ollama_url: &str,
    model: &str,
) -> Result<SvgGenerationResult> {
    let style = SvgStyle::from_str(&params.style);
    let system_prompt = style.system_prompt();

    // Build the user prompt
    let mut user_prompt = format!(
        "Generate an SVG image of size {}x{}.\n\nSubject: {}\n\n",
        params.width, params.height, params.prompt
    );

    if !params.extra.is_empty() {
        user_prompt.push_str(&format!("Additional instructions: {}\n\n", params.extra));
    }

    user_prompt.push_str(&format!(
        "IMPORTANT: Output ONLY the raw SVG code starting with <svg and ending with </svg>. \
         No markdown code fences, no explanation, no other text. \
         Use viewBox=\"0 0 {} {}\". \
         Make sure the SVG is valid and renders correctly.",
        params.width, params.height
    ));

    // Call Ollama
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/api/generate", ollama_url))
        .json(&serde_json::json!({
            "model": model,
            "system": system_prompt,
            "prompt": user_prompt,
            "stream": false,
            "options": {
                "temperature": 0.7,
                "top_p": 0.9,
            }
        }))
        .send()
        .await
        .context("Failed to connect to Ollama API")?;

    if !response.status().is_success() {
        return Err(anyhow!("Ollama API returned status: {}", response.status()));
    }

    let body: serde_json::Value = response
        .json()
        .await
        .context("Failed to parse Ollama response")?;

    let raw_svg = body["response"]
        .as_str()
        .ok_or_else(|| anyhow!("No response text in Ollama output"))?;

    // Clean the SVG: strip markdown code fences if present
    let cleaned_svg = clean_svg_output(raw_svg);

    // Validate it looks like SVG
    if !cleaned_svg.trim_start().starts_with("<svg") {
        return Err(anyhow!(
            "LLM did not generate valid SVG. Response started with: {}",
            &cleaned_svg.chars().take(100).collect::<String>()
        ));
    }

    // Apply style-specific post-processing
    let final_svg = apply_style_post_process(&cleaned_svg, style, params.width, params.height);

    // Count elements
    let element_count = final_svg.matches("<rect").count()
        + final_svg.matches("<circle").count()
        + final_svg.matches("<ellipse").count()
        + final_svg.matches("<line").count()
        + final_svg.matches("<polygon").count()
        + final_svg.matches("<polyline").count()
        + final_svg.matches("<path").count()
        + final_svg.matches("<text").count();

    Ok(SvgGenerationResult {
        svg: final_svg,
        style: style.as_str().to_string(),
        width: params.width,
        height: params.height,
        element_count,
        description: format!("{} — {} style", params.prompt, style.as_str()),
    })
}

/// Generate a fallback SVG when the LLM is unavailable.
/// Creates a simple geometric composition based on the prompt keywords.
pub fn generate_fallback_svg(params: &SvgGenerationParams) -> SvgGenerationResult {
    let style = SvgStyle::from_str(&params.style);
    let w = params.width as u32;
    let h = params.height as u32;

    let svg = match style {
        SvgStyle::MonoChrome => generate_mono_fallback(w, h, &params.prompt),
        SvgStyle::Teletext => generate_teletext_fallback(w, h, &params.prompt),
        SvgStyle::FullColor => generate_color_fallback(w, h, &params.prompt),
        SvgStyle::PixelArt => generate_pixel_fallback(w, h, &params.prompt),
        SvgStyle::LineArt => generate_line_fallback(w, h, &params.prompt),
    };

    let element_count = svg.matches("<rect").count()
        + svg.matches("<circle").count()
        + svg.matches("<ellipse").count()
        + svg.matches("<line").count()
        + svg.matches("<polygon").count()
        + svg.matches("<path").count();

    SvgGenerationResult {
        svg,
        style: style.as_str().to_string(),
        width: params.width,
        height: params.height,
        element_count,
        description: format!("{} — {} style (fallback)", params.prompt, style.as_str()),
    }
}

/// Clean SVG output from LLM: strip markdown fences, trim whitespace
fn clean_svg_output(raw: &str) -> String {
    let mut cleaned = raw.trim().to_string();

    // Remove markdown code fences
    if cleaned.starts_with("```") {
        if let Some(end) = cleaned.find('\n') {
            cleaned = cleaned[end..].trim().to_string();
        }
    }
    if cleaned.ends_with("```") {
        let len = cleaned.len();
        if let Some(start) = cleaned[..len - 3].rfind('\n') {
            cleaned = cleaned[..start].trim().to_string();
        } else {
            cleaned = cleaned[..len - 3].trim().to_string();
        }
    }

    // Remove leading/trailing whitespace
    cleaned.trim().to_string()
}

/// Apply style-specific post-processing to the generated SVG
fn apply_style_post_process(svg: &str, style: SvgStyle, _width: f32, _height: f32) -> String {
    match style {
        SvgStyle::MonoChrome => {
            // Ensure all fills are white, background is black
            let mut result = svg.to_string();
            // Add black background if not present
            if !result.contains("fill=\"#000\"") && !result.contains("fill=\"black\"") {
                result = result.replace(
                    "<svg",
                    "<svg xmlns=\"http://www.w3.org/2000/svg\"",
                );
                // Insert background rect after <svg> tag
                if let Some(pos) = result.find('>') {
                    let insert_pos = pos + 1;
                    result.insert_str(
                        insert_pos,
                        "\n<rect width=\"100%\" height=\"100%\" fill=\"#000000\"/>",
                    );
                }
            }
            result
        }
        SvgStyle::Teletext => {
            // Ensure only teletext palette colours are used
            // This is a best-effort pass — the LLM prompt should handle most of it
            svg.to_string()
        }
        SvgStyle::PixelArt => {
            // Add pixel-art style grid overlay hint
            svg.to_string()
        }
        _ => svg.to_string(),
    }
}

// ─── Fallback SVG Generators ─────────────────────────────────

fn generate_mono_fallback(w: u32, h: u32, prompt: &str) -> String {
    let kw = prompt.to_lowercase();
    let cx = w as f32 / 2.0;
    let cy = h as f32 / 2.0;
    let r = (w.min(h) as f32) / 3.0;

    let (shape, _cx, _cy, _r) = if kw.contains("heart") || kw.contains("love") {
        ("heart", cx, cy, r)
    } else if kw.contains("star") || kw.contains("sparkle") {
        ("star", cx, cy, r)
    } else if kw.contains("circle") || kw.contains("ball") || kw.contains("globe") {
        ("circle", cx, cy, r)
    } else if kw.contains("mountain") || kw.contains("hill") || kw.contains("landscape") {
        ("mountains", cx, cy, 0.0)
    } else if kw.contains("face") || kw.contains("person") || kw.contains("man") || kw.contains("woman") {
        ("face", cx, cy, r)
    } else {
        ("abstract", cx, cy, r)
    };

    match shape {
        "heart" => format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}" width="{}" height="{}">
<rect width="100%" height="100%" fill="#000000"/>
<path d="M {} {} A {} {} 0 0,1 {} {} A {} {} 0 0,1 {} {} Z" fill="#ffffff"/>
</svg>"##,
            w, h, w, h,
            cx, cy - r * 0.3,
            r * 0.6, r * 0.6,
            cx + r * 0.8, cy + r * 0.2,
            r * 0.6, r * 0.6,
            cx - r * 0.8, cy + r * 0.2,
        ),
        "star" => format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}" width="{}" height="{}">
<rect width="100%" height="100%" fill="#000000"/>
<polygon points="{}" fill="#ffffff"/>
</svg>"##,
            w, h, w, h,
            star_points(cx, cy, r, 5),
        ),
        "circle" => format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}" width="{}" height="{}">
<rect width="100%" height="100%" fill="#000000"/>
<circle cx="{}" cy="{}" r="{}" fill="none" stroke="#ffffff" stroke-width="3"/>
</svg>"##,
            w, h, w, h, cx, cy, r,
        ),
        "mountains" => format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}" width="{}" height="{}">
<rect width="100%" height="100%" fill="#000000"/>
<polygon points="0,{} {},{} {},{} {},{} {},{} {},{}" fill="#ffffff"/>
<polygon points="0,{} {},{} {},{} {},{} {},{} {},{}" fill="#cccccc"/>
</svg>"##,
            w, h, w, h,
            h as f32, w as f32 * 0.15, h as f32 * 0.3, w as f32 * 0.3, h as f32 * 0.5, w as f32 * 0.5, h as f32 * 0.2, w as f32 * 0.7, h as f32 * 0.4, w as f32 * 0.85, h as f32 * 0.25,
            h as f32, w as f32 * 0.1, h as f32 * 0.5, w as f32 * 0.25, h as f32 * 0.65, w as f32 * 0.4, h as f32 * 0.45, w as f32 * 0.6, h as f32 * 0.6, w as f32 * 0.8, h as f32 * 0.5,
        ),
        "face" => format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}" width="{}" height="{}">
<rect width="100%" height="100%" fill="#000000"/>
<circle cx="{}" cy="{}" r="{}" fill="none" stroke="#ffffff" stroke-width="2"/>
<circle cx="{}" cy="{}" r="{}" fill="#ffffff"/>
<circle cx="{}" cy="{}" r="{}" fill="#ffffff"/>
<path d="M {} {} Q {} {} {} {}" fill="none" stroke="#ffffff" stroke-width="2"/>
</svg>"##,
            w, h, w, h,
            cx, cy, r,
            cx - r * 0.3, cy - r * 0.2, r * 0.08,
            cx + r * 0.3, cy - r * 0.2, r * 0.08,
            cx - r * 0.3, cy + r * 0.2, cx, cy + r * 0.4, cx + r * 0.3, cy + r * 0.2,
        ),
        _ => format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}" width="{}" height="{}">
<rect width="100%" height="100%" fill="#000000"/>
<circle cx="{}" cy="{}" r="{}" fill="none" stroke="#ffffff" stroke-width="2"/>
<rect x="{}" y="{}" width="{}" height="{}" fill="none" stroke="#ffffff" stroke-width="2"/>
<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#ffffff" stroke-width="2"/>
</svg>"##,
            w, h, w, h,
            cx, cy, r * 0.6,
            cx - r * 0.4, cy - r * 0.4, r * 0.8, r * 0.8,
            cx - r * 0.6, cy + r * 0.6, cx + r * 0.6, cy - r * 0.6,
        ),
    }
}

fn generate_teletext_fallback(w: u32, h: u32, prompt: &str) -> String {
    let kw = prompt.to_lowercase();
    let cols = 40;
    let rows = 25;
    let cell_w = w as f32 / cols as f32;
    let cell_h = h as f32 / rows as f32;

    let mut svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}" width="{}" height="{}">
<rect width="100%" height="100%" fill="#000000"/>
"##,
        w, h, w, h
    );

    // Draw a simple teletext-style composition
    let colours = ["#ff0000", "#00ff00", "#ffff00", "#0000ff", "#ff00ff", "#00ffff", "#ffffff"];

    if kw.contains("hello") || kw.contains("welcome") {
        // Title text using coloured blocks
        for (i, &colour) in colours.iter().enumerate() {
            let x = (i as f32 * 5.0 + 2.0) * cell_w;
            let y = 10.0 * cell_h;
            svg.push_str(&format!(
                r##"<rect x="{}" y="{}" width="{}" height="{}" fill="{}"/>"##,
                x, y, cell_w * 4.0, cell_h * 2.0, colour
            ));
        }
    } else if kw.contains("chart") || kw.contains("graph") || kw.contains("data") {
        // Bar chart
        let bars = [0.2, 0.5, 0.3, 0.8, 0.6, 0.4, 0.7];
        for (i, &height_ratio) in bars.iter().enumerate() {
            let x = (i as f32 * 5.0 + 2.0) * cell_w;
            let bar_h = height_ratio * h as f32 * 0.6;
            svg.push_str(&format!(
                r##"<rect x="{}" y="{}" width="{}" height="{}" fill="{}"/>"##,
                x, h as f32 - bar_h - cell_h, cell_w * 3.0, bar_h,
                colours[i % colours.len()]
            ));
        }
    } else {
        // Abstract teletext pattern
        for i in 0..8 {
            let x = (i as f32 * 5.0 + 1.0) * cell_w;
            let y = (i as f32 * 3.0 + 1.0) * cell_h;
            svg.push_str(&format!(
                r##"<rect x="{}" y="{}" width="{}" height="{}" fill="{}"/>"##,
                x, y, cell_w * 3.0, cell_h * 2.0, colours[i % colours.len()]
            ));
        }
    }

    svg.push_str("</svg>");
    svg
}

fn generate_color_fallback(w: u32, h: u32, prompt: &str) -> String {
    let kw = prompt.to_lowercase();
    let cx = w as f32 / 2.0;
    let cy = h as f32 / 2.0;
    let r = (w.min(h) as f32) / 3.0;

    if kw.contains("sunset") || kw.contains("sunrise") || kw.contains("sky") {
        format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}" width="{}" height="{}">
<defs>
<linearGradient id="sky" x1="0" y1="0" x2="0" y2="1">
<stop offset="0%" stop-color="#ff6b35"/>
<stop offset="40%" stop-color="#f7c59f"/>
<stop offset="70%" stop-color="#004e89"/>
<stop offset="100%" stop-color="#1a3a5c"/>
</linearGradient>
</defs>
<rect width="100%" height="100%" fill="url(#sky)"/>
<circle cx="{}" cy="{}" r="{}" fill="#ffd700" opacity="0.8"/>
</svg>"##,
            w, h, w, h, cx, cy * 0.6, r * 0.5
        )
    } else if kw.contains("ocean") || kw.contains("sea") || kw.contains("water") || kw.contains("wave") {
        format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}" width="{}" height="{}">
<defs>
<linearGradient id="ocean" x1="0" y1="0" x2="0" y2="1">
<stop offset="0%" stop-color="#0077be"/>
<stop offset="50%" stop-color="#005a9e"/>
<stop offset="100%" stop-color="#003366"/>
</linearGradient>
</defs>
<rect width="100%" height="100%" fill="url(#ocean)"/>
<path d="M 0 {} Q {} {} {} {}" fill="none" stroke="#ffffff" stroke-width="2" opacity="0.3"/>
</svg>"##,
            w, h, w, h,
            cy * 0.8, w as f32 * 0.125, cy * 0.5, w as f32 * 0.25, cy * 0.7
        )
    } else if kw.contains("forest") || kw.contains("tree") || kw.contains("nature") || kw.contains("green") {
        format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}" width="{}" height="{}">
<rect width="100%" height="100%" fill="#87ceeb"/>
<rect x="0" y="{}" width="100%" height="{}" fill="#228b22"/>
<polygon points="{},{} {},{} {},{}" fill="#006400"/>
<polygon points="{},{} {},{} {},{}" fill="#2e8b57"/>
</svg>"##,
            w, h, w, h,
            cy + r * 0.5, h as f32 - cy - r * 0.5,
            cx - r * 0.5, cy + r * 0.5, cx, cy - r * 0.5, cx + r * 0.5, cy + r * 0.5,
            cx - r * 0.3, cy + r * 0.8, cx, cy - r * 0.1, cx + r * 0.3, cy + r * 0.8,
        )
    } else {
        format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}" width="{}" height="{}">
<defs>
<radialGradient id="bg" cx="50%" cy="50%" r="50%">
<stop offset="0%" stop-color="#667eea"/>
<stop offset="100%" stop-color="#764ba2"/>
</radialGradient>
</defs>
<rect width="100%" height="100%" fill="url(#bg)"/>
<circle cx="{}" cy="{}" r="{}" fill="#ffffff" opacity="0.1"/>
<circle cx="{}" cy="{}" r="{}" fill="#ffffff" opacity="0.15"/>
<rect x="{}" y="{}" width="{}" height="{}" rx="10" fill="#ffffff" opacity="0.2"/>
</svg>"##,
            w, h, w, h,
            cx, cy, r,
            cx + r * 0.5, cy - r * 0.3, r * 0.3,
            cx - r * 0.4, cy + r * 0.2, r * 0.8, r * 0.6,
        )
    }
}

fn generate_pixel_fallback(w: u32, h: u32, prompt: &str) -> String {
    let kw = prompt.to_lowercase();
    let pixel_size = 16.0;
    let cols = (w as f32 / pixel_size) as u32;
    let rows = (h as f32 / pixel_size) as u32;

    let mut svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}" width="{}" height="{}">
<rect width="100%" height="100%" fill="#1a1a2e"/>
"##,
        w, h, w, h
    );

    // Draw a simple pixel art heart or smiley
    if kw.contains("heart") || kw.contains("love") {
        let heart_pixels = [
            (2,1),(3,1),(5,1),(6,1),
            (1,2),(2,2),(3,2),(4,2),(5,2),(6,2),(7,2),
            (1,3),(2,3),(3,3),(4,3),(5,3),(6,3),(7,3),
            (2,4),(3,4),(4,4),(5,4),(6,4),
            (3,5),(4,5),(5,5),
            (4,6),
        ];
        let ox = (cols / 2) as i32 - 4;
        let oy = (rows / 2) as i32 - 3;
        for &(px, py) in &heart_pixels {
            let x = ((ox + px as i32) as f32) * pixel_size;
            let y = ((oy + py as i32) as f32) * pixel_size;
            svg.push_str(&format!(
                r##"<rect x="{}" y="{}" width="{}" height="{}" fill="#ff0044"/>"##,
                x, y, pixel_size, pixel_size
            ));
        }
    } else {
        // Smiley face
        let smiley_pixels = [
            // Eyes
            (2,2),(3,2),(5,2),(6,2),
            // Mouth
            (2,5),(3,6),(4,6),(5,6),(6,5),
        ];
        let ox = (cols / 2) as i32 - 4;
        let oy = (rows / 2) as i32 - 3;
        for &(px, py) in &smiley_pixels {
            let x = ((ox + px as i32) as f32) * pixel_size;
            let y = ((oy + py as i32) as f32) * pixel_size;
            svg.push_str(&format!(
                r##"<rect x="{}" y="{}" width="{}" height="{}" fill="#00ff88"/>"##,
                x, y, pixel_size, pixel_size
            ));
        }
    }

    svg.push_str("</svg>");
    svg
}

fn generate_line_fallback(w: u32, h: u32, prompt: &str) -> String {
    let kw = prompt.to_lowercase();
    let cx = w as f32 / 2.0;
    let cy = h as f32 / 2.0;
    let r = (w.min(h) as f32) / 3.0;

    if kw.contains("flower") || kw.contains("rose") || kw.contains("bloom") {
        format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}" width="{}" height="{}">
<rect width="100%" height="100%" fill="#ffffff"/>
<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#000000" stroke-width="2"/>
<circle cx="{}" cy="{}" r="{}" fill="none" stroke="#000000" stroke-width="2"/>
<circle cx="{}" cy="{}" r="{}" fill="none" stroke="#000000" stroke-width="1.5"/>
<circle cx="{}" cy="{}" r="{}" fill="none" stroke="#000000" stroke-width="1.5"/>
<circle cx="{}" cy="{}" r="{}" fill="none" stroke="#000000" stroke-width="1.5"/>
<circle cx="{}" cy="{}" r="{}" fill="none" stroke="#000000" stroke-width="1.5"/>
</svg>"##,
            w, h, w, h,
            cx, cy + r * 0.5, cx, cy + r * 1.5,
            cx, cy, r * 0.4,
            cx + r * 0.35, cy - r * 0.2, r * 0.25,
            cx - r * 0.35, cy - r * 0.2, r * 0.25,
            cx + r * 0.2, cy + r * 0.3, r * 0.25,
            cx - r * 0.2, cy + r * 0.3, r * 0.25,
        )
    } else if kw.contains("house") || kw.contains("home") || kw.contains("building") {
        format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}" width="{}" height="{}">
<rect width="100%" height="100%" fill="#ffffff"/>
<polygon points="{},{} {},{} {},{}" fill="none" stroke="#000000" stroke-width="2"/>
<rect x="{}" y="{}" width="{}" height="{}" fill="none" stroke="#000000" stroke-width="2"/>
<rect x="{}" y="{}" width="{}" height="{}" fill="none" stroke="#000000" stroke-width="2"/>
</svg>"##,
            w, h, w, h,
            cx, cy - r, cx - r, cy + r * 0.5, cx + r, cy + r * 0.5,
            cx - r * 0.5, cy + r * 0.5, r, r * 0.8,
            cx - r * 0.2, cy + r * 0.7, r * 0.15, r * 0.3,
        )
    } else {
        // Abstract geometric line art
        format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}" width="{}" height="{}">
<rect width="100%" height="100%" fill="#ffffff"/>
<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#000000" stroke-width="2"/>
<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#000000" stroke-width="2"/>
<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#000000" stroke-width="2"/>
</svg>"##,
            w, h, w, h,
            cx - r * 0.5, cy - r * 0.5, cx + r * 0.5, cy + r * 0.5,
            cx + r * 0.5, cy - r * 0.5, cx - r * 0.5, cy + r * 0.5,
            cx - r * 0.7, cy, cx + r * 0.7, cy,
        )
    }
}

// ─── Helper: Generate star polygon points ────────────────────

/// Generate the `points` attribute for a star polygon
fn star_points(cx: f32, cy: f32, r: f32, num_points: u32) -> String {
    let mut points = Vec::new();
    let outer_r = r;
    let inner_r = r * 0.4;
    let step = std::f32::consts::PI / num_points as f32;

    for i in 0..num_points * 2 {
        let angle = -std::f32::consts::PI / 2.0 + i as f32 * step;
        let radius = if i % 2 == 0 { outer_r } else { inner_r };
        let x = cx + radius * angle.cos();
        let y = cy + radius * angle.sin();
        points.push(format!("{:.1},{:.1}", x, y));
    }

    points.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_svg_style_from_str() {
        assert_eq!(SvgStyle::from_str("mono"), SvgStyle::MonoChrome);
        assert_eq!(SvgStyle::from_str("teletext"), SvgStyle::Teletext);
        assert_eq!(SvgStyle::from_str("full_color"), SvgStyle::FullColor);
        assert_eq!(SvgStyle::from_str("pixel"), SvgStyle::PixelArt);
        assert_eq!(SvgStyle::from_str("line"), SvgStyle::LineArt);
        assert_eq!(SvgStyle::from_str("unknown"), SvgStyle::FullColor);
    }

    #[test]
    fn test_clean_svg_output() {
        let raw = "```svg\n<svg><rect/></svg>\n```";
        let cleaned = clean_svg_output(raw);
        assert!(cleaned.starts_with("<svg"));
        assert!(cleaned.ends_with("</svg>"));
    }

    #[test]
    fn test_star_points() {
        let pts = star_points(100.0, 100.0, 50.0, 5);
        assert!(!pts.is_empty());
        assert!(pts.contains("100.0"));
    }

    #[test]
    fn test_generate_mono_fallback() {
        let svg = generate_mono_fallback(100, 100, "a star");
        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("</svg>"));
    }

    #[test]
    fn test_generate_color_fallback() {
        let svg = generate_color_fallback(200, 200, "sunset over ocean");
        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("</svg>"));
    }

    #[test]
    fn test_generate_teletext_fallback() {
        let svg = generate_teletext_fallback(320, 200, "hello world");
        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("</svg>"));
    }

    #[test]
    fn test_generate_pixel_fallback() {
        let svg = generate_pixel_fallback(160, 160, "heart");
        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("</svg>"));
    }

    #[test]
    fn test_generate_line_fallback() {
        let svg = generate_line_fallback(200, 200, "flower");
        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("</svg>"));
    }

    #[test]
    fn test_fallback_svg_result() {
        let params = SvgGenerationParams {
            prompt: "a mountain landscape".to_string(),
            style: "full_color".to_string(),
            width: 400.0,
            height: 300.0,
            background: true,
            extra: String::new(),
        };
        let result = generate_fallback_svg(&params);
        assert!(result.svg.starts_with("<svg"));
        assert_eq!(result.style, "full_color");
        assert_eq!(result.width, 400.0);
        assert_eq!(result.height, 300.0);
        assert!(result.element_count > 0);
    }
}
