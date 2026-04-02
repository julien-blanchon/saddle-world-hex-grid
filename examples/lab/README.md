# `saddle-world-hex-grid-lab`

Crate-local showcase and verification app for `saddle-world-hex-grid`.

It keeps the richer runtime surface inside the shared crate:

- flat-top and pointy-top layout comparison with a shared sample point
- weighted A* path preview with deterministic blockers
- ring, spiral, and movement-range overlays
- BRP enabled in `dev`
- E2E scenarios behind the `e2e` feature

## Run

```bash
cargo run -p saddle-world-hex-grid-lab
```

## E2E

```bash
cargo run -p saddle-world-hex-grid-lab --features e2e -- smoke_launch
cargo run -p saddle-world-hex-grid-lab --features e2e -- hex_grid_smoke
cargo run -p saddle-world-hex-grid-lab --features e2e -- hex_grid_layouts
cargo run -p saddle-world-hex-grid-lab --features e2e -- hex_grid_pathfinding
cargo run -p saddle-world-hex-grid-lab --features e2e -- hex_grid_ranges
```

## BRP

```bash
uv run --active --project .codex/skills/bevy-brp/script brp app launch saddle-world-hex-grid-lab
uv run --active --project .codex/skills/bevy-brp/script brp resource list
uv run --active --project .codex/skills/bevy-brp/script brp resource get saddle_world_hex_grid_lab::LabDiagnostics
uv run --active --project .codex/skills/bevy-brp/script brp world query bevy_ecs::name::Name
uv run --active --project .codex/skills/bevy-brp/script brp extras screenshot /tmp/saddle_world_hex_grid_lab.png
uv run --active --project .codex/skills/bevy-brp/script brp extras shutdown
```

If the helper launcher cannot keep the lab alive in the background, use the direct foreground fallback:

```bash
BRP_EXTRAS_PORT=15731 cargo run -p saddle-world-hex-grid-lab
BRP_PORT=15731 uv run --active --project .codex/skills/bevy-brp/script brp resource get saddle_world_hex_grid_lab::LabDiagnostics
```
