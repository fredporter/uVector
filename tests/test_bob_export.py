"""Tests for uVector BOB Blitter and Dot Lattice Generator."""
import pytest
from uvector_bob import (
    DOT_PX,
    MAX_BOB_DIM_DOTS,
    MAX_BOB_GIF_BYTES,
    MAX_BOB_RAM_BYTES,
    MOTIF_PRESETS,
    compile_svg_to_bob,
    compile_svgs_to_gif,
    generate_vector_motif,
)


def test_motif_presets_exist():
    assert "zen_envelope" in MOTIF_PRESETS
    assert "teletext_pulse" in MOTIF_PRESETS
    assert "cosmic_orbiter" in MOTIF_PRESETS
    assert "signal_beacon" in MOTIF_PRESETS


def test_generate_vector_motif_frames():
    for motif_id in MOTIF_PRESETS:
        frames = generate_vector_motif(motif_id, steps=4, size_px=32)
        assert len(frames) == 4
        for f in frames:
            assert f.startswith("<svg")
            assert "</svg>" in f
            assert 'width="32"' in f


def test_compile_svg_to_bob():
    frames = generate_vector_motif("teletext_pulse", steps=1, size_px=32)
    bob = compile_svg_to_bob(
        svg_text=frames[0],
        id="pulse_bob",
        name="Teletext Pulse",
        palette_id="teletext_ceefax",
        width_dots=8,
        height_dots=8,
    )
    assert bob["id"] == "pulse_bob"
    assert bob["width_dots"] == 8
    assert bob["height_dots"] == 8
    assert bob["width_px"] == 32  # 8 * 4
    assert bob["height_px"] == 32
    assert bob["transparent_index"] == 0
    assert bob["palette"] == "teletext_ceefax"
    assert len(bob["frames"]) == 1
    assert len(bob["frames"][0]["dots"]) == 64
    assert bob["fits_budget"] is True


def test_compile_svgs_to_gif():
    frames = generate_vector_motif("zen_envelope", steps=4, size_px=32)
    gif_bytes = compile_svgs_to_gif(frames, delay_ms=80, palette_id="teletext_ceefax")

    assert len(gif_bytes) > 0
    assert gif_bytes[:3] == b"GIF"
    assert len(gif_bytes) <= MAX_BOB_GIF_BYTES  # Must be <= 60 KB


def test_dot_lattice_invariants():
    assert DOT_PX == 4
    assert MAX_BOB_DIM_DOTS == 32  # 128 / 4
    assert MAX_BOB_RAM_BYTES == 128 * 1024
    assert MAX_BOB_GIF_BYTES == 60 * 1024
