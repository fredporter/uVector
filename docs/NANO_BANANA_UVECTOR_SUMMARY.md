# Nano-Banana-Pro-MCP and uVector — Updated Summary

This summary is scoped to the Nano-Banana-Pro-MCP vendor code and the uVector
spec/evolution path. Other platform topics are intentionally excluded.

## 1. Nano-Banana-Pro-MCP (Vendor)

Source: github.com/fredporter/nano-banana-pro-mcp

Role: MCP server wrapping Gemini image generation/edit APIs for agent workflows
(via stdio tool calls).

### Core Components

| File                    | Purpose                                                                                              |
| ----------------------- | ---------------------------------------------------------------------------------------------------- |
| `src/gemini.ts`         | Gemini client for generate/edit/describe image flows and base64 handling                             |
| `src/mono-rules.ts`     | Mono Core validation and enforcement rules                                                           |
| `src/styles.ts`         | Style presets (`mono_blueprint`, `mono_botanical`, `mono_chrome`, `mono_teletext`, `mono_editorial`) |
| `src/prompt-builder.ts` | Prompt assembly: user intent + style + uGrid + Mono Core constraints                                 |
| `src/tier.ts`           | Tier routing (Pro vs Flash), cost hints, batch window logic                                          |
| `src/vault-config.ts`   | Vault config/memory/cache integration hooks                                                          |
| `src/types.ts`          | API/tooling type contracts                                                                           |

### MCP Tools

| Tool             | Description                                 |
| ---------------- | ------------------------------------------- |
| `list_styles`    | Return style IDs                            |
| `generate_image` | Text-to-image with style/uGrid/tier options |
| `compose_images` | Multi-image blending                        |
| `edit_image`     | Iterative image edit/refine                 |
| `describe_image` | Image analysis/captioning                   |

### Vendor Characteristics

1. Mono Core linework discipline.
2. uGrid-aware prompt framing.
3. Vault-oriented cache/memory behavior.
4. Tiered cost/model routing.

## 2. uVector — Strategic Evolution

uVector evolves Nano-Banana-Pro-MCP into a broader uDOS image and vector
engine with split-repo-aware integration.

### Evolution Snapshot

| Aspect            | Nano-Banana-Pro-MCP           | uVector (current direction)                                     |
| ----------------- | ----------------------------- | --------------------------------------------------------------- |
| Scope             | Standalone Gemini MCP wrapper | Native image/vector subsystem for uDOS workflows                |
| Primary output    | PNG/base64 workflows          | PNG plus vector/SVG-oriented pipeline design                    |
| Mono rules        | Strong but tool-local         | Enforced as a platform contract                                 |
| Vault integration | Optional per use              | Default integration target                                      |
| Repo model        | Single vendor server          | Split-repo compatible (uCore host + external providers/plugins) |

### New Configuration Alignment (Current)

uVector planning should align with the active split-repo host configuration:

1. uCore is host shell and route assembly layer.
2. Extension ownership is manifest-driven.
3. External plugins are discovered via `UCORE_EXTENSION_MANIFEST_PATHS`.
4. CI now enforces:
   - extension manifest contract validation,
   - split-repo import/route smoke checks,
   - split-repo packaging layout validation.

This means uVector integration should be packaged/discoverable as an external
module rather than added as new in-core route ownership.

### uDOS Integration Targets

| Component        | uVector Role                                             |
| ---------------- | -------------------------------------------------------- |
| ProseUI          | Inline sprites, scene images, glyph production           |
| GridUI           | Cell-aware image quantization and teletext/pixel mapping |
| Character Editor | Bob/sprite sheet generation and frame extraction         |
| Layer Editor     | Layer composition support and render outputs             |
| Location Mapper  | Coordinate transforms and map asset generation           |

### Mono Core Contract (Platform)

1. Line/glyph layer: pure `#000000` on transparent.
2. No gray anti-aliased halo in line layer.
3. Color overlays are separate passes/layers.

### uGrid Mapping Contract

Canvas and composition should remain utility-grid-aware (`width_utiles x
height_utiles`) so outputs map to downstream editors without manual offsets.

### Asset Pipeline Direction

Prompt -> Generation -> Mono validation -> Vector/PNG outputs ->
Editor pipelines (character/grid/layer) -> Vault cache and world assets.

### Storage Direction

Vault paths remain the default target for generated assets and metadata,
including cache/history and world-scoped outputs.

## 3. Practical Next Steps

1. Add explicit SVG/vector output support path in uVector runtime interface.
2. Formalize line/color layer separation pipeline.
3. Bind uVector outputs to canvas/editor coordinate contracts.
4. Package uVector as a manifest-discoverable external capability compatible
   with current uCore split-repo gates.

## 4. Key Implementation References

| Area               | Reference               |
| ------------------ | ----------------------- |
| Mono rules         | `src/mono-rules.ts`     |
| Styles             | `src/styles.ts`         |
| Prompt composition | `src/prompt-builder.ts` |
| Tier/cost routing  | `src/tier.ts`           |
| Vault hooks        | `src/vault-config.ts`   |
| Gemini client      | `src/gemini.ts`         |

This brief is updated for the current split-repo and manifest-driven
configuration now active in uCore.
