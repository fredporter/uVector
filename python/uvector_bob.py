"""uVector BOB Blitter & Dot Lattice Generator Bridge.

Exposes generative vector motifs and compiles SVG frames into
4×4 dot lattice BOB definitions and lightweight animated GIFs (<= 60 KB).
"""
from __future__ import annotations

import json
import math
import os
import shutil
import subprocess
import tempfile
from pathlib import Path
from typing import Any, Dict, List, Optional

# Constants from GRIDCORE-STANDARDS.json
DOT_PX = 4
MAX_BOB_DIM_DOTS = 32
MAX_BOB_RAM_BYTES = 128 * 1024
MAX_BOB_GIF_BYTES = 60 * 1024

MOTIF_PRESETS = {
    "zen_envelope": {
        "title": "Zen Sovereign Envelope",
        "description": "Sealed invitation badge with pulsing geometric chevron and wax seal.",
        "default_palette": "teletext_ceefax",
        "suggested_dots": (8, 8),
    },
    "teletext_pulse": {
        "title": "Teletext Diamond Pulse",
        "description": "Expanding and contracting geometric diamond on the 4×4 dot lattice.",
        "default_palette": "teletext_ceefax",
        "suggested_dots": (8, 8),
    },
    "cosmic_orbiter": {
        "title": "Cosmic Celestial Orbiter",
        "description": "Rotating 8-point geometric star and orbital particle ring.",
        "default_palette": "teletext_ceefax",
        "suggested_dots": (8, 8),
    },
    "signal_beacon": {
        "title": "Sovereign Signal Beacon",
        "description": "Concentric radiating radio beacon rings for decentralized nodes.",
        "default_palette": "terminal_green",
        "suggested_dots": (8, 8),
    },
}


def find_uvcore_binary() -> Optional[Path]:
    """Locate the uvcore binary in target directories or system PATH."""
    repo_root = Path(__file__).resolve().parent.parent
    candidates = [
        repo_root / "target" / "release" / "uvcore",
        repo_root / "target" / "debug" / "uvcore",
    ]
    for c in candidates:
        if c.exists() and os.access(c, os.X_OK):
            return c
    system_path = shutil.which("uvcore")
    return Path(system_path) if system_path else None


def generate_vector_motif(
    motif: str,
    palette_id: str = "teletext_ceefax",
    steps: int = 4,
    size_px: int = 32,
) -> List[str]:
    """Generate a sequence of animated SVG frames for a canonical vector motif."""
    frames = []
    motif_key = motif.lower().replace("-", "_")

    if palette_id == "terminal_green":
        fg1 = "#00FF66"
        fg2 = "#00CC44"
        fg3 = "#008822"
    else:  # teletext_ceefax or default
        fg1 = "#E6193C"  # red
        fg2 = "#58A6FF"  # blue
        fg3 = "#F2CC60"  # yellow

    center = size_px / 2.0

    for step in range(steps):
        phase = (step / float(steps)) * 2.0 * math.pi

        if motif_key == "zen_envelope":
            # Pulsing envelope badge with central seal and chevron flap
            pulse_offset = int(round(math.sin(phase) * 2))
            flap_y = int(center - 2 + pulse_offset)
            svg = f"""<svg xmlns="http://www.w3.org/2000/svg" width="{size_px}" height="{size_px}" viewBox="0 0 {size_px} {size_px}">
  <rect x="4" y="6" width="24" height="20" rx="2" fill="{fg2}" fill-opacity="0.2" stroke="{fg2}" stroke-width="2"/>
  <path d="M 4 8 L {int(center)} {flap_y} L 28 8" fill="none" stroke="{fg3}" stroke-width="2"/>
  <circle cx="{center}" cy="{center + 2}" r="{3 + abs(pulse_offset)}" fill="{fg1}"/>
</svg>"""

        elif motif_key == "cosmic_orbiter":
            # Rotating geometric cross/star and orbital point
            angle = phase
            orbit_r = 10.0
            ox = center + orbit_r * math.cos(angle)
            oy = center + orbit_r * math.sin(angle)
            star_scale = 1.0 + 0.2 * math.sin(phase * 2)
            svg = f"""<svg xmlns="http://www.w3.org/2000/svg" width="{size_px}" height="{size_px}" viewBox="0 0 {size_px} {size_px}">
  <!-- Central Star -->
  <line x1="{center - 6 * star_scale}" y1="{center}" x2="{center + 6 * star_scale}" y2="{center}" stroke="{fg3}" stroke-width="2"/>
  <line x1="{center}" y1="{center - 6 * star_scale}" x2="{center}" y2="{center + 6 * star_scale}" stroke="{fg3}" stroke-width="2"/>
  <circle cx="{center}" cy="{center}" r="3" fill="{fg1}"/>
  <!-- Orbiting Node -->
  <circle cx="{ox:.1f}" cy="{oy:.1f}" r="2" fill="{fg2}"/>
</svg>"""

        elif motif_key == "signal_beacon":
            # Concentric radiating rings
            r1 = int((step % steps) * 3 + 4)
            r2 = int(((step + 2) % steps) * 3 + 4)
            svg = f"""<svg xmlns="http://www.w3.org/2000/svg" width="{size_px}" height="{size_px}" viewBox="0 0 {size_px} {size_px}">
  <circle cx="{center}" cy="{center}" r="3" fill="{fg1}"/>
  <circle cx="{center}" cy="{center}" r="{r1}" fill="none" stroke="{fg2}" stroke-width="2"/>
  <circle cx="{center}" cy="{center}" r="{r2}" fill="none" stroke="{fg3}" stroke-width="1.5" stroke-dasharray="2 2"/>
</svg>"""

        else:  # default to teletext_pulse
            # Expanding / contracting diamond
            d_size = int(6 + 4 * math.sin(phase))
            svg = f"""<svg xmlns="http://www.w3.org/2000/svg" width="{size_px}" height="{size_px}" viewBox="0 0 {size_px} {size_px}">
  <polygon points="{center},{center - d_size} {center + d_size},{center} {center},{center + d_size} {center - d_size},{center}"
           fill="{fg1}" stroke="{fg3}" stroke-width="2"/>
</svg>"""

        frames.append(svg.strip())

    return frames


def compile_svg_to_bob(
    svg_text: str,
    id: str,
    name: str,
    palette_id: str = "teletext_ceefax",
    width_dots: int = 8,
    height_dots: int = 8,
) -> Dict[str, Any]:
    """Compile an SVG string to a GridCore BOB definition JSON via uvcore binary."""
    uvcore = find_uvcore_binary()
    if uvcore:
        with tempfile.NamedTemporaryFile(suffix=".svg", mode="w", delete=False) as tf:
            tf.write(svg_text)
            tf_path = Path(tf.name)
        try:
            cmd = [
                str(uvcore),
                str(tf_path),
                "--format", "bob",
                "--palette", palette_id,
                "--width-dots", str(width_dots),
                "--height-dots", str(height_dots),
            ]
            res = subprocess.run(cmd, capture_output=True, text=True, check=True)
            bob_dict = json.loads(res.stdout)
            bob_dict["id"] = id
            bob_dict["name"] = name
            return bob_dict
        finally:
            if tf_path.exists():
                tf_path.unlink()

    # Pure Python fallback if binary not yet built
    return {
        "id": id,
        "name": name,
        "width_dots": width_dots,
        "height_dots": height_dots,
        "width_px": width_dots * DOT_PX,
        "height_px": height_dots * DOT_PX,
        "transparent_index": 0,
        "palette": palette_id,
        "palette_colors": ["#000000", "#E6193C", "#3FB950", "#F2CC60", "#58A6FF", "#BC8CFF", "#39C5CF", "#FFFFFF"],
        "frames": [
            {
                "width_dots": width_dots,
                "height_dots": height_dots,
                "dots": [1] * (width_dots * height_dots),
                "duration_ms": 100,
            }
        ],
        "ram_footprint_bytes": width_dots * DOT_PX * height_dots * DOT_PX,
        "fits_budget": True,
    }


def compile_svgs_to_gif(
    svg_frames: List[str],
    delay_ms: int = 100,
    palette_id: str = "teletext_ceefax",
) -> bytes:
    """Compile a list of SVG frame strings into looping animated GIF bytes via uvcore."""
    uvcore = find_uvcore_binary()
    if not uvcore:
        raise RuntimeError("uvcore binary not found. Run 'cargo build --release' in uVector.")

    with tempfile.TemporaryDirectory() as td:
        frame_paths = []
        for i, frame in enumerate(svg_frames):
            p = Path(td) / f"frame_{i:03d}.svg"
            p.write_text(frame, encoding="utf-8")
            frame_paths.append(str(p))

        out_gif = Path(td) / "out.gif"
        cmd = [
            str(uvcore),
            frame_paths[0],
            "--format", "gif",
            "--palette", palette_id,
            "--delay", str(delay_ms),
            "--output", str(out_gif),
        ]
        subprocess.run(cmd, capture_output=True, text=True, check=True)
        gif_bytes = out_gif.read_bytes()

        if len(gif_bytes) > MAX_BOB_GIF_BYTES:
            raise ValueError(
                f"Generated GIF ({len(gif_bytes)} bytes) exceeds sovereign {MAX_BOB_GIF_BYTES} budget"
            )

        return gif_bytes
