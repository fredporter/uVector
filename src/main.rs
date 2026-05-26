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
    input: PathBuf,

    /// Output format
    #[arg(short, long, default_value = "describe")]
    format: String,

    /// Output file path (optional)
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Read SVG
    let svg_content = std::fs::read_to_string(&cli.input)?;

    // Parse SVG
    let doc = uvcore::parser::parse_svg(&svg_content)?;

    match cli.format.as_str() {
        "celx" => {
            let celx = uvcore::formats::to_celx(&doc)?;
            println!("{}", celx);
        }
        "ascii" => {
            let ascii = uvcore::formats::to_ascii(&doc)?;
            println!("{}", ascii);
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
        _ => {
            anyhow::bail!("Unknown format: {}. Use: celx, ascii, describe, png", cli.format);
        }
    }

    Ok(())
}
