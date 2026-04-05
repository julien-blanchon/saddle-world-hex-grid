use crate::{AxialHex, HexDiagonalDirection, HexDirection};
use bevy::prelude::Reflect;

/// An edge shared between two adjacent hexes.
///
/// An edge is identified by the hex it belongs to and the direction of the edge.
/// Two adjacent hexes share the same physical edge — `GridEdge::new(a, dir)` and
/// `GridEdge::new(a.neighbor(dir), dir.opposite())` refer to the same edge.
///
/// The canonical form always uses the hex with the smaller `(q, r)` pair
/// (lexicographic order) so that edge equality works correctly.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect)]
pub struct GridEdge {
    /// The canonical hex (smallest `(q, r)` of the two adjacent hexes).
    pub hex: AxialHex,
    /// The direction from `hex` toward the other adjacent hex.
    pub direction: HexDirection,
}

impl GridEdge {
    /// Creates a new grid edge in canonical form.
    pub fn new(hex: AxialHex, direction: HexDirection) -> Self {
        let neighbor = hex.neighbor(direction);
        let opposite = direction.opposite();

        // Canonical: pick the hex with smaller (q, r)
        if (hex.q, hex.r) <= (neighbor.q, neighbor.r) {
            Self { hex, direction }
        } else {
            Self {
                hex: neighbor,
                direction: opposite,
            }
        }
    }

    /// Returns the two hexes that share this edge.
    pub fn hexes(&self) -> [AxialHex; 2] {
        [self.hex, self.hex.neighbor(self.direction)]
    }

    /// Returns the two vertices at the endpoints of this edge.
    pub fn vertices(&self) -> [GridVertex; 2] {
        let dir_idx = self.direction.index();

        // The two diagonal directions adjacent to this edge direction
        let diag_cw = HexDiagonalDirection::ALL[dir_idx];
        let diag_ccw = HexDiagonalDirection::ALL[(dir_idx + 5) % 6]; // previous

        [
            GridVertex::new(self.hex, diag_ccw),
            GridVertex::new(self.hex, diag_cw),
        ]
    }
}

/// A vertex shared between three adjacent hexes.
///
/// A vertex is identified by a hex and a diagonal direction.
/// Three hexes share each vertex. The canonical form normalizes to the
/// smallest `(q, r)` of the three hexes to ensure vertex equality.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect)]
pub struct GridVertex {
    /// The canonical hex (smallest `(q, r)` of the three sharing this vertex).
    pub hex: AxialHex,
    /// The diagonal direction from `hex` toward the vertex.
    pub direction: HexDiagonalDirection,
}

impl GridVertex {
    /// Creates a new grid vertex in canonical form.
    pub fn new(hex: AxialHex, direction: HexDiagonalDirection) -> Self {
        // A vertex is shared by 3 hexes. Find all three and pick the canonical one.
        let candidates = vertex_hexes_and_dirs(hex, direction);
        let mut best = candidates[0];
        for candidate in &candidates[1..] {
            if (candidate.0.q, candidate.0.r) < (best.0.q, best.0.r) {
                best = *candidate;
            }
        }
        Self {
            hex: best.0,
            direction: best.1,
        }
    }

    /// Returns the three hexes that share this vertex.
    pub fn hexes(&self) -> [AxialHex; 3] {
        let candidates = vertex_hexes_and_dirs(self.hex, self.direction);
        [candidates[0].0, candidates[1].0, candidates[2].0]
    }

    /// Returns the three edges that meet at this vertex.
    pub fn edges(&self) -> [GridEdge; 3] {
        let hexes = self.hexes();
        [
            GridEdge::new(hexes[0], adjacent_edge_direction(hexes[0], hexes[1])),
            GridEdge::new(hexes[1], adjacent_edge_direction(hexes[1], hexes[2])),
            GridEdge::new(hexes[2], adjacent_edge_direction(hexes[2], hexes[0])),
        ]
    }
}

/// Returns the 3 (hex, diagonal_direction) tuples that represent the same vertex.
///
/// Diagonal direction at index `i` sits between edge directions `i` and `(i+1)%6`.
/// The three hexes sharing this vertex are:
/// - `hex` with diagonal direction `i`
/// - `hex.neighbor(edge[i])` with diagonal direction `(i+2)%6`
/// - `hex.neighbor(edge[(i+1)%6])` with diagonal direction `(i+4)%6`
fn vertex_hexes_and_dirs(
    hex: AxialHex,
    direction: HexDiagonalDirection,
) -> [(AxialHex, HexDiagonalDirection); 3] {
    let i = direction.index();

    let edge_a = HexDirection::ALL[i];
    let edge_b = HexDirection::ALL[(i + 1) % 6];

    let neighbor_a = hex.neighbor(edge_a);
    let neighbor_b = hex.neighbor(edge_b);

    let dir_from_a = HexDiagonalDirection::ALL[(i + 2) % 6];
    let dir_from_b = HexDiagonalDirection::ALL[(i + 4) % 6];

    [
        (hex, direction),
        (neighbor_a, dir_from_a),
        (neighbor_b, dir_from_b),
    ]
}

/// Find the edge direction from hex `a` to adjacent hex `b`.
fn adjacent_edge_direction(a: AxialHex, b: AxialHex) -> HexDirection {
    let delta = b - a;
    for dir in HexDirection::ALL {
        if dir.vector() == delta {
            return dir;
        }
    }
    // Fallback (should not happen for truly adjacent hexes)
    HexDirection::East
}

impl HexDiagonalDirection {
    /// Returns the index of this direction (0..6).
    pub const fn index(self) -> usize {
        match self {
            Self::EastNorthEast => 0,
            Self::North => 1,
            Self::WestNorthWest => 2,
            Self::WestSouthWest => 3,
            Self::South => 4,
            Self::EastSouthEast => 5,
        }
    }

    /// Returns the opposite diagonal direction.
    pub const fn opposite(self) -> Self {
        Self::ALL[(self.index() + 3) % 6]
    }

    /// Rotates this direction clockwise by the given number of 60-degree steps.
    pub const fn rotate_cw(self, steps: i32) -> Self {
        let index = self.index() as i32;
        Self::ALL[(index + steps).rem_euclid(6) as usize]
    }

    /// Rotates this direction counter-clockwise by the given number of 60-degree steps.
    pub const fn rotate_ccw(self, steps: i32) -> Self {
        self.rotate_cw(-steps)
    }
}

#[cfg(test)]
#[path = "topology_tests.rs"]
mod tests;
