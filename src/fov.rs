use crate::{AxialHex, HexDiagonalDirection, HexDirection};
use std::collections::HashSet;

/// Computes a 360-degree field of view from `origin` out to `range`.
///
/// Casts rays from `origin` to every hex on the perimeter ring at `range`.
/// Each ray stops when it reaches a hex for which `is_blocking` returns `true`.
/// The blocking hex itself is **included** in the result (you can see a wall,
/// but not past it).
///
/// The caller owns the map and decides what counts as opaque.
pub fn range_fov(
    origin: AxialHex,
    range: u32,
    is_blocking: impl Fn(AxialHex) -> bool,
) -> HashSet<AxialHex> {
    if range == 0 {
        return HashSet::from([origin]);
    }

    let mut visible = HashSet::new();
    visible.insert(origin);

    for target in origin.ring(range) {
        for hex in origin.line_to(target) {
            visible.insert(hex);
            if is_blocking(hex) {
                break;
            }
        }
    }

    visible
}

/// Computes a directional (120-degree cone) field of view from `origin`.
///
/// Only hexes whose diagonal direction from the origin matches the two
/// vertex directions flanking `direction` are considered. This produces
/// a wedge-shaped visible area facing the given edge direction.
///
/// Like [`range_fov`], blocking hexes are included in the result.
pub fn directional_fov(
    origin: AxialHex,
    range: u32,
    direction: HexDirection,
    is_blocking: impl Fn(AxialHex) -> bool,
) -> HashSet<AxialHex> {
    if range == 0 {
        return HashSet::from([origin]);
    }

    let [a, b] = direction.vertex_directions();

    let mut visible = HashSet::new();
    visible.insert(origin);

    for target in origin.ring(range) {
        let way = origin.diagonal_way_to(target);
        let in_cone = match way {
            DiagonalWay::Single(d) => d == a || d == b,
            DiagonalWay::Tie(d1, d2) => d1 == a || d1 == b || d2 == a || d2 == b,
        };
        if !in_cone {
            continue;
        }

        for hex in origin.line_to(target) {
            visible.insert(hex);
            if is_blocking(hex) {
                break;
            }
        }
    }

    visible
}

/// Result of computing which diagonal direction a hex lies in relative to another.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiagonalWay {
    /// The hex lies clearly in one diagonal direction.
    Single(HexDiagonalDirection),
    /// The hex lies exactly on the boundary between two diagonal directions.
    Tie(HexDiagonalDirection, HexDiagonalDirection),
}

/// Determines which diagonal direction `to` lies in relative to `from`.
///
/// If the hex is exactly on the boundary between two diagonal sectors,
/// returns [`DiagonalWay::Tie`].
pub(crate) fn diagonal_way(from: AxialHex, to: AxialHex) -> DiagonalWay {
    let delta = to - from;
    if delta == AxialHex::ZERO {
        return DiagonalWay::Single(HexDiagonalDirection::EastNorthEast);
    }

    let q = delta.q as f64;
    let r = delta.r as f64;

    // Find the closest diagonal direction by dot product with each diagonal vector
    let diag_vectors: [(i32, i32); 6] = [
        (2, -1),  // ENE
        (1, -2),  // N
        (-1, -1), // WNW
        (-2, 1),  // WSW
        (-1, 2),  // S
        (1, 1),   // ESE
    ];

    // Compute dot product with each diagonal direction to find the best match
    let mut best_idx = 0;
    let mut best_dot = f64::NEG_INFINITY;
    let mut second_best_idx = 0;
    let mut second_best_dot = f64::NEG_INFINITY;

    let len = (q * q + r * r).sqrt();
    let nq = q / len;
    let nr = r / len;

    for (i, (dq, dr)) in diag_vectors.iter().enumerate() {
        let dlen = ((*dq as f64).powi(2) + (*dr as f64).powi(2)).sqrt();
        let dot = nq * (*dq as f64 / dlen) + nr * (*dr as f64 / dlen);
        if dot > best_dot {
            second_best_dot = best_dot;
            second_best_idx = best_idx;
            best_dot = dot;
            best_idx = i;
        } else if dot > second_best_dot {
            second_best_dot = dot;
            second_best_idx = i;
        }
    }

    let dirs = HexDiagonalDirection::ALL;
    // If the two best are very close (within epsilon), it's a tie
    if (best_dot - second_best_dot).abs() < 1e-10 {
        DiagonalWay::Tie(dirs[best_idx], dirs[second_best_idx])
    } else {
        DiagonalWay::Single(dirs[best_idx])
    }
}

#[cfg(test)]
#[path = "fov_tests.rs"]
mod tests;
