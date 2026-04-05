use crate::{AxialHex, RangeIter};
use bevy::prelude::Reflect;

/// Circular hex bounds centered on a specific hex with a given radius.
///
/// Useful for defining map boundaries, checking containment, and iterating
/// over all hexes within the bounds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect)]
pub struct HexBounds {
    pub center: AxialHex,
    pub radius: u32,
}

impl HexBounds {
    pub const fn new(center: AxialHex, radius: u32) -> Self {
        Self { center, radius }
    }

    /// Returns `true` if the given hex lies within (or on) the boundary.
    pub fn contains(&self, hex: AxialHex) -> bool {
        self.center.distance_to(hex) <= self.radius
    }

    /// The number of hexes contained in these bounds.
    pub fn hex_count(&self) -> usize {
        let r = self.radius as usize;
        3 * r * (r + 1) + 1
    }

    /// Iterate over all hexes within these bounds.
    pub fn iter(&self) -> RangeIter {
        self.center.range(self.radius)
    }

    /// Returns `true` if the two bounds overlap (share at least one hex).
    pub fn intersects(&self, other: &HexBounds) -> bool {
        let dist = self.center.distance_to(other.center);
        dist <= self.radius + other.radius
    }

    /// Wraps a hex coordinate into these bounds using modular arithmetic
    /// on cube coordinates. Useful for toroidal/wraparound maps.
    pub fn wrap(&self, hex: AxialHex) -> AxialHex {
        if self.contains(hex) {
            return hex;
        }
        // Simple approach: find the closest hex within bounds
        // by walking from center toward the hex and stopping at the boundary
        let delta = hex - self.center;
        let dist = self.center.distance_to(hex);
        if dist == 0 {
            return hex;
        }
        // Scale the delta down to fit within radius
        let frac = self.radius as f32 / dist as f32;
        let fq = delta.q as f32 * frac;
        let fr = delta.r as f32 * frac;
        let wrapped = crate::FractionalHex::new(
            self.center.q as f32 + fq,
            self.center.r as f32 + fr,
            -(self.center.q as f32 + fq) - (self.center.r as f32 + fr),
        );
        wrapped.round()
    }
}

impl IntoIterator for HexBounds {
    type Item = AxialHex;
    type IntoIter = RangeIter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
#[path = "bounds_tests.rs"]
mod tests;
