use crate::{AxialHex, FractionalHex};
use bevy::{math::Vec2, prelude::Reflect};
use std::f32::consts::FRAC_PI_3;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Reflect)]
pub enum HexOrientation {
    #[default]
    FlatTop,
    PointyTop,
}

impl HexOrientation {
    const SQRT_3: f32 = 1.732_050_8;

    pub(crate) const fn forward_matrix(self) -> [f32; 4] {
        match self {
            Self::PointyTop => [Self::SQRT_3, Self::SQRT_3 * 0.5, 0.0, 1.5],
            Self::FlatTop => [1.5, 0.0, Self::SQRT_3 * 0.5, Self::SQRT_3],
        }
    }

    pub(crate) const fn inverse_matrix(self) -> [f32; 4] {
        match self {
            Self::PointyTop => [Self::SQRT_3 / 3.0, -1.0 / 3.0, 0.0, 2.0 / 3.0],
            Self::FlatTop => [2.0 / 3.0, 0.0, -1.0 / 3.0, Self::SQRT_3 / 3.0],
        }
    }

    pub(crate) const fn start_angle(self) -> f32 {
        match self {
            Self::FlatTop => 0.0,
            Self::PointyTop => 0.5,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
pub struct HexLayout {
    pub orientation: HexOrientation,
    pub origin: Vec2,
    pub hex_size: Vec2,
}

impl Default for HexLayout {
    fn default() -> Self {
        Self::flat().with_uniform_size(1.0)
    }
}

impl HexLayout {
    pub const fn new(orientation: HexOrientation) -> Self {
        Self {
            orientation,
            origin: Vec2::ZERO,
            hex_size: Vec2::ONE,
        }
    }

    pub const fn flat() -> Self {
        Self::new(HexOrientation::FlatTop)
    }

    pub const fn pointy() -> Self {
        Self::new(HexOrientation::PointyTop)
    }

    pub const fn with_origin(mut self, origin: Vec2) -> Self {
        self.origin = origin;
        self
    }

    pub const fn with_hex_size(mut self, hex_size: Vec2) -> Self {
        self.hex_size = hex_size;
        self
    }

    pub const fn with_uniform_size(mut self, size: f32) -> Self {
        self.hex_size = Vec2::splat(size);
        self
    }

    pub fn hex_to_world(self, hex: AxialHex) -> Vec2 {
        let [f0, f1, f2, f3] = self.orientation.forward_matrix();
        let local = Vec2::new(
            f0 * hex.q as f32 + f1 * hex.r as f32,
            f2 * hex.q as f32 + f3 * hex.r as f32,
        );
        self.origin + local * self.hex_size
    }

    pub fn fractional_to_world(self, hex: FractionalHex) -> Vec2 {
        let [f0, f1, f2, f3] = self.orientation.forward_matrix();
        let local = Vec2::new(f0 * hex.q + f1 * hex.r, f2 * hex.q + f3 * hex.r);
        self.origin + local * self.hex_size
    }

    pub fn world_to_fractional(self, world: Vec2) -> FractionalHex {
        let [b0, b1, b2, b3] = self.orientation.inverse_matrix();
        let point = (world - self.origin) / self.hex_size;
        let q = b0 * point.x + b1 * point.y;
        let r = b2 * point.x + b3 * point.y;
        FractionalHex::new(q, r, -q - r)
    }

    pub fn world_to_hex(self, world: Vec2) -> AxialHex {
        self.world_to_fractional(world).round()
    }

    pub fn corner_offsets(self) -> [Vec2; 6] {
        std::array::from_fn(|corner| {
            let angle = FRAC_PI_3 * (self.orientation.start_angle() + corner as f32);
            Vec2::new(angle.cos() * self.hex_size.x, angle.sin() * self.hex_size.y)
        })
    }

    pub fn corners(self, hex: AxialHex) -> [Vec2; 6] {
        let center = self.hex_to_world(hex);
        self.corner_offsets().map(|offset| center + offset)
    }

    pub fn edge_midpoints(self, hex: AxialHex) -> [Vec2; 6] {
        let corners = self.corners(hex);
        std::array::from_fn(|index| {
            let next = (index + 1) % 6;
            (corners[index] + corners[next]) * 0.5
        })
    }

    pub fn rect_size(self) -> Vec2 {
        match self.orientation {
            HexOrientation::FlatTop => self.hex_size * Vec2::new(2.0, HexOrientation::SQRT_3),
            HexOrientation::PointyTop => self.hex_size * Vec2::new(HexOrientation::SQRT_3, 2.0),
        }
    }
}

#[cfg(test)]
#[path = "layout_tests.rs"]
mod tests;
