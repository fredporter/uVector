# uVector (uvcore)

**Universal Vector Core** — high-performance Rust conversion engine and Python bridge for SVG, vector geometry, rasterization, and GridCore render targets.

## Overview

`uVector` provides the deterministic geometry and rasterization bridge for the uDos ecosystem:
- **Rust Engine (`uvcore`):** Pure Rust SVG parser (`roxmltree`), software rasterizer (`tiny-skia`), and CLI.
- **Conversion Pipeline:** Converts SVG vector graphics into raster images, Teletext glyph mosaics, and GridCore cell layers.
- **Python Extension:** Python bindings providing image intelligence and vector translation for uCore and standalone pipelines.

## Capabilities

- **SVG to Raster:** High-fidelity PNG rendering via `tiny-skia` without external C dependencies.
- **SVG to Teletext / GridCore:** Quantizes vector paths into 40x25 character cells, Bedstead glyphs, and Teletext graphic blocks.
- **Local-First & Offline:** Zero cloud dependency; fast local execution for design systems and retro displays.

## Building & Testing

### Rust Engine

```bash
cargo build --release
cargo test
```

### Python Bindings & Tests

```bash
cd python
pip install -e .
pytest ../tests
```

## Extension Manifest

Exposed as a uCore tool extension via `ucore-extension.json`:
- `id`: `uvector`
- `kind`: `tool`
- `api_prefix`: `/api/vector`

## License

MIT / Apache 2.0.
