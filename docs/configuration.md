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
- `draw_coord_labels: bool`
- `center_radius: f32`

### Defaults

- `enabled = false`
- `draw_centers = true`
- `draw_cell_outlines = true`
- `draw_path_lines = true`
- `draw_coord_labels = false`
- `center_radius = 6.0`

### Valid ranges and guidance

- `center_radius > 0.0`
- use a radius smaller than the visual hex radius if you want the markers to stay inside the cell
- if you only want path previews, disable center and outline drawing to reduce gizmo clutter
- `draw_coord_labels` is reserved for future text rendering; the current gizmo-only debug runtime does not use it yet

### Example

```rust
app.add_plugins(
    saddle_world_hex_grid::HexGridPlugin::always_on(Update).with_debug_settings(
        saddle_world_hex_grid::HexGridDebugSettings {
            enabled: true,
            draw_centers: false,
            draw_cell_outlines: true,
            draw_path_lines: true,
            draw_coord_labels: false,
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
- `fov_cells: Vec<AxialHex>`
- `cell_color: Color`
- `highlight_color: Color`
- `path_color: Color`
- `fov_color: Color`

### Defaults

- empty cell, highlighted, path, and fov_cells vectors
- semi-transparent blue cell color
- amber highlight color
- bright green path color
- semi-transparent cyan fov color

### Guidance

- `cells` defines which hexes get center and outline rendering
- `highlighted` is best for rings, selected cells, or transient focus regions
- `path` is rendered as center-to-center line segments in order
- `fov_cells` renders FOV-visible hexes as colored outlines; populate from `range_fov` or `directional_fov`
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

## Field of view closure contracts

### `range_fov`

```rust
range_fov(origin, range, |hex| -> bool { ... })
```

Return `true` if the hex blocks line of sight. Blocking hexes are included in the result (you can see a wall, but not past it). The origin is always included.

### `directional_fov`

```rust
directional_fov(origin, range, direction, |hex| -> bool { ... })
```

Same blocking contract as `range_fov`, but restricted to a 120-degree cone facing the given `HexDirection`. The origin is always included.

## `HexBounds`

Circular bounds for hexagonal regions.

### Fields

- `center: AxialHex`
- `radius: u32`

### Methods

- `new(center, radius)` — construct bounds
- `contains(hex)` — test membership
- `hex_count()` — total cells in the region: `3R(R+1)+1`
- `iter()` — iterate over all contained hexes
- `intersects(other)` — test whether two bounds overlap
- `wrap(hex)` — wrap a hex coordinate to stay within the bounds (modular wrapping)

### Guidance

- Use `HexBounds` for quick region containment checks without allocating a set
- `wrap` is useful for toroidal or repeating maps
- Implements `IntoIterator` for ergonomic for-loops

## `HexagonalMap<T>`

Dense O(1)-indexed storage over a hexagonal region, backed by a flat `Vec<T>`.

### Constructors

- `HexagonalMap::new(center, radius, |hex| -> T)` — fill via closure
- `HexagonalMap::with_default(center, radius)` — fill with `T::default()`

### Methods

- `center()` — the center hex
- `radius()` — the radius of the storage region
- `len()` — total stored elements
- `is_empty()` — whether the map has no elements
- `contains(hex)` — test if a hex is within the storage bounds
- `get(hex) -> Option<&T>` — safe access
- `get_mut(hex) -> Option<&mut T>` — safe mutable access
- `iter()` — iterate over `(AxialHex, &T)` pairs
- `iter_mut()` — iterate over `(AxialHex, &mut T)` pairs

### Indexing

Supports `Index<AxialHex>` and `IndexMut<AxialHex>` for direct access. Panics on out-of-bounds access (use `get`/`get_mut` for fallible access).

### Guidance

- Prefer `HexagonalMap` over `HashMap<AxialHex, T>` when the region is known at construction time for better cache locality and O(1) access
- `iter()` and `iter_mut()` follow the internal row-major order without allocating temporary coordinate buffers
- The internal layout uses row-by-row flat indexing, so iteration order follows rows from bottom to top

## `GridEdge`

Canonical edge between two adjacent hexes.

### Fields

- `hex: AxialHex` — the canonical hex (lexicographically smaller)
- `direction: HexDirection` — the canonical direction

### Methods

- `new(hex, direction)` — creates edge in canonical form
- `hexes()` — the two hexes sharing this edge
- `vertices()` — the two vertices at the endpoints of this edge

### Guidance

- Two `GridEdge` values are equal regardless of which side you construct from: `GridEdge::new(a, East) == GridEdge::new(b, West)` when `b` is east of `a`
- Useful for strategy games with border features (rivers, walls, roads along hex edges)

## `GridVertex`

Canonical vertex shared by three adjacent hexes.

### Fields

- `hex: AxialHex` — the canonical hex (lexicographically smallest of the three)
- `direction: HexDiagonalDirection` — the canonical diagonal direction

### Methods

- `new(hex, direction)` — creates vertex in canonical form
- `hexes()` — the three hexes sharing this vertex
- `edges()` — the three edges meeting at this vertex

### Guidance

- Three `GridVertex` values are equal regardless of which hex you construct from
- Useful for strategy games with vertex features (cities, intersections, resource nodes)

## Distance methods on `AxialHex`

- `distance_to(other) -> u32` — Manhattan (hex) distance
- `distance_sq_to(other) -> f32` — squared Euclidean distance (no sqrt, useful for comparisons and sorting)
- `euclidean_distance_to(other) -> f32` — Euclidean distance between hex centers

## Direction enhancements on `HexDirection`

- `angle() -> f32` — direction angle in radians from positive x-axis
- `unit_vector() -> Vec2` — unit vector pointing in this direction
- `vertex_directions() -> [HexDiagonalDirection; 2]` — the two diagonal (vertex) directions adjacent to this face direction
- `from_angle(angle: f32) -> Self` — nearest face direction to an arbitrary angle
