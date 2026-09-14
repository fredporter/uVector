"""uVector Python bindings and Nano Banana bridge."""

from .uvector_banana import (
    STYLE_PRESETS,
    BananaStyle,
    NanoBananaClient,
    build_banana_prompt,
)
from .uvector_bob import (
    DOT_PX,
    MAX_BOB_DIM_DOTS,
    MAX_BOB_RAM_BYTES,
    MAX_BOB_GIF_BYTES,
    MOTIF_PRESETS,
    compile_svg_to_bob,
    compile_svgs_to_gif,
    generate_vector_motif,
)

__all__ = [
    "STYLE_PRESETS",
    "BananaStyle",
    "NanoBananaClient",
    "build_banana_prompt",
    "DOT_PX",
    "MAX_BOB_DIM_DOTS",
    "MAX_BOB_RAM_BYTES",
    "MAX_BOB_GIF_BYTES",
    "MOTIF_PRESETS",
    "compile_svg_to_bob",
    "compile_svgs_to_gif",
    "generate_vector_motif",
]
