# `saddle-world-hex-grid` Configuration

## `HexOrientation`

Controls how hexes are projected into world space.

### Variants

- `FlatTop`
- `PointyTop`

### Implications

- `FlatTop` places horizontal top and bottom edges on each hex.
- `PointyTop` places a single corner at the top and bottom.
- Orientation changes both the center-position formula and the corner-angle basis. Never mix an orientation with a layout authored for the other one.

## `HexLayout`

World-space conversion settings.

### Fields

- `orientation: HexOrientation`
- `origin: Vec2`
- `hex_size: Vec2`

### Defaults

- `orientation = HexOrientation::FlatTop`
- `origin = Vec2::ZERO`
- `hex_size = Vec2::splat(1.0)`

### Constructors and builders

- `HexLayout::flat()`
- `HexLayout::pointy()`
- `with_origin(Vec2)`
- `with_hex_size(Vec2)`
- `with_uniform_size(f32)`

### Valid ranges and guidance

- `hex_size.x > 0.0`
- `hex_size.y > 0.0`
- use `with_uniform_size` unless you intentionally want stretched hexes
- keep `origin` in the same world space as the camera and gameplay systems that will use `world_to_hex`

### Main methods

- `hex_to_world(AxialHex) -> Vec2`
- `world_to_hex(Vec2) -> AxialHex`
- `world_to_fractional(Vec2) -> FractionalHex`
- `corners(AxialHex) -> [Vec2; 6]`
- `edge_midpoints(AxialHex) -> [Vec2; 6]`
- `rect_size() -> Vec2`

## `OffsetHexMode`

Selects the parity rule used by `OffsetHex`.

### Variants

- `OddColumns`
- `EvenColumns`
- `OddRows`
- `EvenRows`

### Use cases

- `OddColumns` / `EvenColumns`: column-based offset storage
- `OddRows` / `EvenRows`: row-based offset storage

Pick the mode that matches the external editor, tilemap, or file format you need to interoperate with.

## `DoubledHexMode`

Selects the doubled-coordinate projection used by `DoubledHex`.

### Variants

- `DoubleWidth`
- `DoubleHeight`

### Use cases

- `DoubleWidth`: doubled columns, useful when a wide rectangular projection is more convenient
- `DoubleHeight`: doubled rows, useful when height-major indexing fits better

## `HexGridDebugSettings`

Runtime debug resource used by the plugin.

### Fields

- `enabled: bool`
- `draw_centers: bool`
- `draw_cell_outlines: bool`
- `draw_path_lines: bool`
- `center_radius: f32`

### Defaults

- `enabled = false`
- `draw_centers = true`
- `draw_cell_outlines = true`
- `draw_path_lines = true`
- `center_radius = 6.0`

### Valid ranges and guidance

- `center_radius > 0.0`
- use a radius smaller than the visual hex radius if you want the markers to stay inside the cell
- if you only want path previews, disable center and outline drawing to reduce gizmo clutter

### Example

```rust
app.add_plugins(
    saddle_world_hex_grid::HexGridPlugin::always_on(Update).with_debug_settings(
        saddle_world_hex_grid::HexGridDebugSettings {
            enabled: true,
            draw_centers: false,
            draw_cell_outlines: true,
            draw_path_lines: true,
            center_radius: 4.0,
        },
    ),
);
```

## `HexDebugOverlay`

Per-overlay component consumed by the debug runtime.

### Fields

- `layout: HexLayout`
- `cells: Vec<AxialHex>`
- `highlighted: Vec<AxialHex>`
- `path: Vec<AxialHex>`
- `cell_color: Color`
- `highlight_color: Color`
- `path_color: Color`

### Defaults

- empty cell, highlighted, and path vectors
- semi-transparent blue cell color
- amber highlight color
- bright green path color

### Guidance

- `cells` defines which hexes get center and outline rendering
- `highlighted` is best for rings, selected cells, or transient focus regions
- `path` is rendered as center-to-center line segments in order
- use separate overlay entities when you want different color layers or update frequencies

## `HexGridPlugin`

Optional runtime integration surface for debug overlays.

### Fields

- `activate_schedule: Interned<dyn ScheduleLabel>`
- `deactivate_schedule: Interned<dyn ScheduleLabel>`
- `update_schedule: Interned<dyn ScheduleLabel>`
- `debug_settings: HexGridDebugSettings`

### Constructors

- `HexGridPlugin::new(activate, deactivate, update)`
- `HexGridPlugin::always_on(update)`
- `HexGridPlugin::default()` which is equivalent to `always_on(Update)`
- `with_debug_settings(HexGridDebugSettings)`

### Scheduling guidance

- Use `new(...)` when the overlay should only exist inside a state-driven screen or tool mode.
- Use `always_on(Update)` or `Default` for examples, labs, and always-available editor tooling.
- The plugin only draws when its runtime state is active.
- The plugin initializes `HexGridDebugGizmos` for you, so callers do not need a separate `init_gizmo_group::<HexGridDebugGizmos>()` step.

## Traversal closure contracts

The pathfinding functions are configured through closures rather than a config struct.

### `a_star`

```rust
a_star(start, goal, |from, to| -> Option<u32> { ... })
```

Return:

- `None` for blocked edges or out-of-bounds cells
- `Some(cost)` for a legal move with positive cost

### `reachable_within`

```rust
reachable_within(start, budget, |from, to| -> Option<u32> { ... })
```

The same closure contract applies. Positive costs are required. Zero-cost edges are not supported.

### Why closures are used

The closure-based design keeps storage and rule ownership outside the crate:

- `HashMap` worlds
- ECS queries
- chunked map streams
- tool-only procedural generators
- directional terrain rules

The crate only needs access to local traversal policy, not a fixed map container type.
