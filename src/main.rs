//! UVcore CLI — Universal Vector Core
//!
//! Converts SVG to various output formats.
//!
//! ## Usage
//!
//! ```bash
//! uvcore input.svg --format celx
//! uvcore input.svg --format ascii
//! uvcore input.svg --format describe
//! uvcore input.svg --format png --output output.png
//! ```

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "uvcore", version, about = "Universal Vector Core — SVG→everything")]
struct Cli {
    /// Input SVG file path
    #[arg(required_unless_present = "palettes")]
    input: Option<PathBuf>,

    /// Output format
    #[arg(short, long, default_value = "describe")]
    format: String,

    /// Output file path (optional)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Canonical palette ID (for gif, teletext, celx quantization)
    #[arg(long)]
    palette: Option<String>,

    /// Frame delay in milliseconds for GIF animations (default: 100)
    #[arg(long, default_value_t = 100)]
    delay: u32,

    /// Width in dots (1 dot = 4x4 px) for BOB blitter output
    #[arg(long)]
    width_dots: Option<u32>,

    /// Height in dots (1 dot = 4x4 px) for BOB blitter output
    #[arg(long)]
    height_dots: Option<u32>,

    /// Dump canonical palette registry as JSON
    #[arg(long)]
    palettes: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if cli.palettes {
        let reg = uvcore::palettes::PaletteRegistry::load();
        println!("{}", serde_json::to_string_pretty(&reg)?);
        return Ok(());
    }

    let input_path = cli.input.ok_or_else(|| anyhow::anyhow!("Input file required"))?;

    // Read SVG
    let svg_content = std::fs::read_to_string(&input_path)?;

    // Parse SVG
    let doc = uvcore::parser::parse_svg(&svg_content)?;

    match cli.format.as_str() {
        "celx" => {
            let celx = uvcore::formats::to_celx(&doc)?;
            print!("{}", celx);
        }
        "ascii" => {
            let ascii = uvcore::formats::to_ascii(&doc)?;
            print!("{}", ascii);
        }
        "describe" => {
            let desc = uvcore::formats::describe(&doc)?;
            println!("{}", desc);
        }
        "png" => {
            let png_data = uvcore::render::to_png(&doc)?;
            if let Some(path) = &cli.output {
                std::fs::write(path, png_data)?;
                println!("Wrote PNG to {}", path.display());
            } else {
                println!("PNG data ({} bytes)", png_data.len());
            }
        }
        "teletext" => {
            let teletext = uvcore::formats::to_teletext(&doc)?;
            print!("{}", teletext);
        }
        "bob" => {
            let bob_json = uvcore::formats::to_bob(
                &doc,
                cli.palette.as_deref(),
                cli.width_dots,
                cli.height_dots,
            )?;
            if let Some(path) = &cli.output {
                std::fs::write(path, &bob_json)?;
                println!("Wrote BOB definition to {}", path.display());
            } else {
                println!("{}", bob_json);
            }
        }
        "gif" => {
            let options = uvcore::animate::GifAnimationOptions {
                delay_ms: cli.delay,
                repeat_infinite: true,
                palette_id: cli.palette.clone(),
            };
            let gif_data = uvcore::animate::svgs_to_gif(&[&svg_content], &options)?;
            if let Some(path) = &cli.output {
                std::fs::write(path, &gif_data)?;
                println!("Wrote GIF to {}", path.display());
            } else {
                println!("GIF data ({} bytes)", gif_data.len());
            }
        }
        _ => {
            anyhow::bail!("Unknown format: {}. Use: celx, ascii, describe, png, teletext, bob, gif", cli.format);
        }
    }

    Ok(())
}
