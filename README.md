# Saddle World Hex Grid

Hexagonal grid math, layout conversion, iterators, and traversal helpers for Bevy.

The crate stays math-first: coordinate conversion, neighbors, distance, rings, spirals, pathfinding, and movement budgets work in ordinary Rust code without starting a Bevy app. The plugin is optional and only owns the runtime-facing debug overlay surface.

## Why this crate exists

`saddle-world-hex-grid` is meant to be a reusable shared module for:

- tactics and roguelike boards
- 4X or strategy overlays
- procgen and map editors
- district or influence radius tools
- hybrid 2D or 3D games that need world-space picking over a hex lattice

It borrows concepts from Red Blob Games' hex grid guides, `hexx`, and Catlike Coding's hex map tutorials, but reimplements the coordinate and traversal code directly to keep the API small, project-agnostic, and Bevy-friendly.

## Quick start

```toml
[dependencies]
saddle-world-hex-grid = { git = "https://github.com/julien-blanchon/saddle-world-hex-grid" }
```

```rust
use bevy::prelude::*;
use saddle_world_hex_grid::{AxialHex, HexDebugOverlay, HexGridPlugin, HexLayout, a_star};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(HexGridPlugin::always_on(Update))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    let layout = HexLayout::pointy().with_uniform_size(32.0);
    let start = AxialHex::ZERO;
    let goal = AxialHex::new(4, -2);

    let path = a_star(start, goal, |_, to| {
        let in_bounds = to.distance_to(AxialHex::ZERO) <= 5;
        in_bounds.then_some(1)
    });

    commands.spawn((
        Name::new("Hex Debug Overlay"),
        HexDebugOverlay {
            layout,
            cells: AxialHex::ZERO.hexagon(3).collect(),
            path: path.map(|path| path.cells).unwrap_or_default(),
            ..default()
        },
    ));
}
```

## Public API

### Coordinate types

- `AxialHex`: ergonomic default coordinate type for gameplay and storage.
- `CubeHex`: fully explicit cube coordinates with `q + r + s == 0` invariant checks.
- `OffsetHex`: odd-r / even-r / odd-q / even-q adapter for rectangular storage or tilemap interop.
- `DoubledHex`: double-width / double-height adapter for dense rectangular projections and editor-style indexing.
- `FractionalHex`: world-picking and interpolation helper with robust rounding back to `AxialHex`.

Mode-dependent adapters stay explicit on purpose:

- `axial.to_offset(mode)` and `offset.to_axial(mode)`
- `axial.to_doubled(mode)` and `doubled.to_axial(mode)`

There is no blanket `From<DoubledHex>` conversion because doubled coordinates are ambiguous without their projection mode.

### Layout and orientation

- `HexOrientation::{FlatTop, PointyTop}`
- `HexLayout`
  - `hex_to_world`
  - `world_to_hex`
  - `world_to_fractional`
  - `corners`
  - `edge_midpoints`
  - `rect_size`

Orientation is always explicit in world-space conversion. The same `AxialHex` resolves to different world positions depending on the selected layout.

### Directions and local steps

- `HexDirection`
- `HexDiagonalDirection`

### Iterators and shape helpers

- `line_to`
- `range`
- `ring`
- `spiral`
- `hexagon`
- `parallelogram`
- `offset_rectangle`
- `doubled_rectangle`

The iterator-based API keeps most shape queries allocation-light. Collect into a `Vec` only when you need ownership.

### Pathfinding and movement range

- `a_star(start, goal, edge_cost)`
- `reachable_within(start, budget, edge_cost)`
- `HexPath`
- `HexReachability`

Traversal is storage-agnostic. The caller owns map storage and terrain rules. The crate only asks for an edge-cost closure:

- return `None` to block a move
- return `Some(cost)` for a legal move
- use larger costs for terrain, rivers, roads, or one-way rules

This separation keeps the core reusable for `HashMap`, arrays, ECS-backed worlds, editor tools, or generated maps.

## Plugin and debug surface

The plugin is intentionally thin:

- `HexGridPlugin`
- `HexGridSystems::{SyncDebug, DrawDebug}`
- `HexGridDebugSettings`
- `HexDebugOverlay`
- `HexGridDebugGizmos`

Use the plugin when you want runtime gizmo overlays for centers, outlines, highlighted cells, or path previews. Skip it entirely if you only need math and traversal.

The plugin initializes its own `HexGridDebugGizmos` group, so `HexDebugOverlay` works out of the box once the plugin is added and debug drawing is enabled.

The plugin uses injectable schedules:

```rust
app.add_plugins(saddle_world_hex_grid::HexGridPlugin::new(
    OnEnter(MyState::Active),
    OnExit(MyState::Active),
    Update,
));
```

## Coordinate systems

First-class:

- `AxialHex`
- `CubeHex`

Adapters:

- `OffsetHex`
- `DoubledHex`

`AxialHex` is the main public workhorse because it is compact, readable, and cheap to convert. `CubeHex` stays first-class because distance, interpolation, and invariant-sensitive algorithms are easier to reason about in cube space. Offset and doubled forms exist for interoperability and storage layout, not as the core algorithm surface.

## Deferred features

Deliberately not included in `0.1.0`:

- wraparound or toroidal maps
- chunk or region coordinates
- field-of-view or visibility solvers
- canonical global ordering for `Ord`
- built-in map storage containers

The architecture leaves room for these later, but the initial crate keeps the durable primitives small.

## Examples

| Example | Focus |
| --- | --- |
| `basic` | cursor hover, neighbor lookup, and flat-top world picking |
| `layouts` | flat-top vs pointy-top layout conversion using the same local sample point |
| `ranges` | reachable-within, exact rings, and spiral traversal overlays |
| `pathfinding` | weighted A* over caller-owned blocked and weighted cells |
| `saddle-world-hex-grid-lab` | richer crate-local showcase with BRP and E2E verification |

Run them with:

```bash
cargo run -p saddle-world-hex-grid-example-basic
cargo run -p saddle-world-hex-grid-example-layouts
cargo run -p saddle-world-hex-grid-example-ranges
cargo run -p saddle-world-hex-grid-example-pathfinding
cargo run -p saddle-world-hex-grid-lab
```

## Dependency philosophy

Runtime dependencies stay minimal:

- `bevy = "0.18"` only

No project-specific crates, no `game_core`, and no wrapper dependency on `hexx`. The crate owns its math, traversal, and debug code directly.
