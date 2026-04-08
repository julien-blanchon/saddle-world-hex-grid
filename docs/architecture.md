# `saddle-world-hex-grid` Architecture

## Design goals

`saddle-world-hex-grid` exists to provide durable hex-grid primitives that work in pure Rust first and Bevy second.

The design priorities are:

1. keep coordinate math independent from ECS and rendering
2. make world-space conversion explicit about orientation and size
3. keep traversal generic over caller-owned storage and terrain policy
4. offer an optional Bevy plugin for debug overlays without forcing a runtime model

## Coordinate model choices

### First-class coordinate systems

- `AxialHex`
- `CubeHex`

`AxialHex` is the ergonomic center of the API. It stores only `q` and `r`, derives `s` on demand, and keeps gameplay code readable.

`CubeHex` stays first-class because some invariants are clearer in cube space:

- exact `q + r + s == 0` validation
- interpolation and rounding
- rotation and reflection
- distance reasoning

The crate exposes both because forcing everything into axial coordinates tends to hide the cube invariant, while forcing everything into cube coordinates makes common gameplay code heavier than needed.

### Adapter coordinate systems

- `OffsetHex`
- `DoubledHex`

These are adapter types for storage and interoperability:

- `OffsetHex` maps cleanly to odd or even row and column layouts used by editors and tilemap tools.
- `DoubledHex` supports dense rectangular indexing patterns without making offset parity part of every algorithm.

Algorithms operate on `AxialHex` and `CubeHex`, then convert at the boundaries. This keeps the implementation smaller and avoids subtle parity bugs leaking into pathfinding or line drawing.

Adapter conversions remain explicit. In particular, `DoubledHex` does not implement a blanket `Into<AxialHex>` because the same doubled pair can only be interpreted correctly once the caller provides `DoubledHexMode`.

### Fractional coordinates

`FractionalHex` is kept explicit instead of hidden inside `HexLayout`. It exists for:

- robust world-to-hex rounding
- interpolation for line drawing
- debug and picking workflows where the intermediate fractional result matters

The rounding code follows the standard cube-rounding approach popularized by Red Blob Games: round each cube axis, then repair the axis with the largest rounding error to re-enforce the cube invariant.

## Layout architecture

`HexLayout` owns world-space conversion:

- orientation
- hex size
- origin

This is intentionally separate from coordinate storage. A grid coordinate should not imply how it is rendered.

The layout API exposes:

- center conversion: `hex_to_world`
- inverse conversion: `world_to_hex`
- intermediate conversion: `world_to_fractional`
- geometry helpers: `corners`, `edge_midpoints`, `rect_size`

Flat-top and pointy-top are both first-class because users often need both in the same workspace: gameplay, editors, debug tools, or imported content may disagree on orientation.

## Traversal architecture

Traversal APIs are graph-oriented instead of map-container-oriented.

### A*

`a_star(start, goal, edge_cost)` asks the caller for the cost of moving from one hex to another. The closure decides:

- which edges are blocked
- whether cost is symmetric or directed
- whether terrain is weighted
- whether world bounds or chunk streaming allow the move

The function returns `Option<HexPath>` and keeps its internal node representation private so the search implementation can change later without breaking consumers.

### Reachability

`reachable_within(start, budget, edge_cost)` is a budget-limited Dijkstra search. It returns `HexReachability`, which stores:

- per-cell cost
- predecessor links for path reconstruction back to the origin

This covers common tactical and builder workflows:

- movement previews
- service or influence ranges
- flood fill over weighted terrain
- previewing the cheapest route to any reachable target

## Shape and iterator architecture

Most geometry helpers are iterators rather than allocation-heavy constructors.

Current iterator families:

- lines
- filled ranges
- exact rings
- spirals
- parallelograms
- offset-projected rectangles
- doubled-projected rectangles
- triangles
- wedges (arc sectors between two directions)

The crate still offers convenience constructors such as `hexagon(...)`, but the iterator-first design keeps hot paths lighter and leaves allocation choices to the caller.

## Field of view architecture

The crate provides two FOV functions: `range_fov` (360-degree) and `directional_fov` (120-degree cone).

### Algorithm

Both use raycasting from the origin to the perimeter ring at the given range:

1. Enumerate every hex on the ring at distance `range` from the origin
2. For each perimeter hex, cast a line (using `line_to`) from origin to that hex
3. Walk the line; if a hex is blocking, include it but stop the ray

This is a simple, predictable approach that handles concave obstacles well. The trade-off is that perimeter size grows linearly with range, so very large ranges cast many rays. For typical game ranges (1–10), this is negligible.

### Directional FOV

`directional_fov` filters the perimeter ring to only include hexes within the 120-degree cone facing the given direction. It uses `diagonal_way` to classify each perimeter hex into a diagonal sector and only casts rays into the matching sector.

### Closure ownership

Like pathfinding, FOV is caller-owned: the `is_blocking` closure decides which hexes block line of sight. This keeps the FOV independent of any specific map storage.

## Bounds architecture

`HexBounds` is a lightweight circular bounds type (center + radius) for quick containment checks, iteration, and overlap tests without allocating a set. It also supports `wrap` for toroidal/repeating maps.

## Dense storage architecture

`HexagonalMap<T>` provides O(1) indexed storage over a hexagonal region. Internally it uses a flat `Vec<T>` with row-by-row indexing plus a precomputed row-start table, so any `(q, r)` lookup becomes:

1. translate into local coordinates around the map center
2. compute the row index
3. jump to the row start
4. offset within that row

This keeps random access constant-time while preserving a cache-friendly row-major iteration order.

## Grid topology architecture

`GridEdge` and `GridVertex` represent the edges and vertices of the hex grid in canonical form. Canonical form ensures that two constructions of the same geometric feature (from different hexes) always produce the same struct value, enabling use as `HashMap` keys or in sets.

- `GridEdge` picks the lexicographically smaller hex and adjusts the direction accordingly
- `GridVertex` picks the lexicographically smallest of the three hexes sharing the vertex

These types are useful for strategy and building games that need to reason about hex borders (rivers, walls, roads) and corners (cities, intersections).

## Plugin and debug architecture

The plugin is intentionally thin and optional.

### What the plugin owns

- runtime activation and deactivation through injectable schedules
- public `HexGridSystems`
- optional gizmo debug rendering through `HexDebugOverlay`
- reflective registration of the public runtime-facing types
- initialization of the crate's own gizmo config group

### What the plugin does not own

- map storage
- picking
- pathfinding state
- gameplay rules
- content-specific components

`HexDebugOverlay` is the main runtime integration point. The overlay component stores authored data:

- layout
- cells
- highlighted cells
- path cells
- FOV cells
- colors

The runtime system caches corner and path geometry into an internal component, then draws with `bevy_gizmos` if the user has initialized the gizmo group. This keeps the public overlay surface simple while avoiding recomputing corners every frame for unchanged overlays.

`draw_coord_labels` remains on `HexGridDebugSettings` as a forward-looking toggle, but the current runtime is intentionally gizmo-only. Label rendering would require an additional text-backed runtime surface, so the flag is currently a documented no-op rather than a half-implemented feature.

## Performance notes

### Complexity

- `distance_to`: `O(1)`
- neighbor lookup: `O(1)`
- line, ring, range, spiral: proportional to the number of returned cells
- `a_star`: `O(E log V)` in the explored frontier
- `reachable_within`: `O(E log V)` in the explored frontier

### Current tradeoffs

- `HashMap` and `BinaryHeap` are used from the standard library for the initial implementation.
- `HexagonalMap` iteration streams coordinates directly from row-major storage order instead of allocating temporary coordinate lists.
- Search internals are private so the crate can later switch to denser storage, reusable buffers, or bucketed queues without a public API break.
- Shape APIs favor iterators, but some runtime overlay paths still collect into `Vec`s because debug rendering wants owned geometry snapshots.

### Intentional omissions

The crate does not yet include:

- chunked search acceleration
- wraparound-aware heuristics
- specialized storage backends
- cached neighbor tables

These are valid future additions, but they are not necessary for a stable `0.1.0` shared-crate surface.

## Borrowed ideas and rejected scope

Borrowed ideas:

- Red Blob Games: coordinate systems, rounding, lines, rings, spirals, world-layout framing
- `hexx`: broad shape coverage, movement-field framing, and the reminder that storage should stay caller-owned
- Catlike Coding: practical weighted pathfinding concerns and large-map thinking

Rejected for now:

- wraparound maps (beyond the simple `HexBounds::wrap`)
- chunk coordinates
- mesh generation helpers beyond corners and edge midpoints
- a built-in ECS tile map or world representation

Those features are useful, but shipping them now would either bloat the surface area or force content assumptions into what should stay a general-purpose shared math and traversal crate.
