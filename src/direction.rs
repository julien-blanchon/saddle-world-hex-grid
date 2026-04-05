use crate::AxialHex;
use bevy::{math::Vec2, prelude::Reflect};
use std::f32::consts::FRAC_PI_3;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect)]
pub enum HexDirection {
    East,
    NorthEast,
    NorthWest,
    West,
    SouthWest,
    SouthEast,
}

impl HexDirection {
    pub const ALL: [Self; 6] = [
        Self::East,
        Self::NorthEast,
        Self::NorthWest,
        Self::West,
        Self::SouthWest,
        Self::SouthEast,
    ];

    pub const fn index(self) -> usize {
        match self {
            Self::East => 0,
            Self::NorthEast => 1,
            Self::NorthWest => 2,
            Self::West => 3,
            Self::SouthWest => 4,
            Self::SouthEast => 5,
        }
    }

    pub const fn vector(self) -> AxialHex {
        match self {
            Self::East => AxialHex::new(1, 0),
            Self::NorthEast => AxialHex::new(1, -1),
            Self::NorthWest => AxialHex::new(0, -1),
            Self::West => AxialHex::new(-1, 0),
            Self::SouthWest => AxialHex::new(-1, 1),
            Self::SouthEast => AxialHex::new(0, 1),
        }
    }

    pub const fn opposite(self) -> Self {
        Self::ALL[(self.index() + 3) % 6]
    }

    pub const fn rotate_cw(self, steps: i32) -> Self {
        let index = self.index() as i32;
        Self::ALL[(index + steps).rem_euclid(6) as usize]
    }

    pub const fn rotate_ccw(self, steps: i32) -> Self {
        self.rotate_cw(-steps)
    }

    /// Returns the angle of this direction in radians, measured counter-clockwise
    /// from the positive x-axis. Values range from -PI to PI.
    pub fn angle(self) -> f32 {
        // Each direction is 60 degrees apart, starting from East (0 radians)
        // East=0, NE=60, NW=120, W=180, SW=-120, SE=-60
        match self {
            Self::East => 0.0,
            Self::NorthEast => FRAC_PI_3,
            Self::NorthWest => FRAC_PI_3 * 2.0,
            Self::West => std::f32::consts::PI,
            Self::SouthWest => -FRAC_PI_3 * 2.0,
            Self::SouthEast => -FRAC_PI_3,
        }
    }

    /// Returns a unit-length 2D vector pointing in this direction.
    pub fn unit_vector(self) -> Vec2 {
        let angle = self.angle();
        Vec2::new(angle.cos(), angle.sin())
    }

    /// Returns the two diagonal (vertex) directions that flank this edge direction.
    ///
    /// The first element is CCW of this direction, the second is CW.
    /// Useful for directional FOV cone calculations.
    pub const fn vertex_directions(self) -> [crate::HexDiagonalDirection; 2] {
        use crate::HexDiagonalDirection as D;
        match self {
            Self::East => [D::EastNorthEast, D::EastSouthEast],
            Self::NorthEast => [D::North, D::EastNorthEast],
            Self::NorthWest => [D::WestNorthWest, D::North],
            Self::West => [D::WestSouthWest, D::WestNorthWest],
            Self::SouthWest => [D::South, D::WestSouthWest],
            Self::SouthEast => [D::EastSouthEast, D::South],
        }
    }

    /// Returns the direction closest to the given angle (in radians).
    pub fn from_angle(angle: f32) -> Self {
        let normalized = angle.rem_euclid(std::f32::consts::TAU);
        let index = (normalized / FRAC_PI_3 + 0.5).floor() as usize % 6;
        Self::ALL[index]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect)]
pub enum HexDiagonalDirection {
    EastNorthEast,
    North,
    WestNorthWest,
    WestSouthWest,
    South,
    EastSouthEast,
}

impl HexDiagonalDirection {
    pub const ALL: [Self; 6] = [
        Self::EastNorthEast,
        Self::North,
        Self::WestNorthWest,
        Self::WestSouthWest,
        Self::South,
        Self::EastSouthEast,
    ];

    pub const fn vector(self) -> AxialHex {
        match self {
            Self::EastNorthEast => AxialHex::new(2, -1),
            Self::North => AxialHex::new(1, -2),
            Self::WestNorthWest => AxialHex::new(-1, -1),
            Self::WestSouthWest => AxialHex::new(-2, 1),
            Self::South => AxialHex::new(-1, 2),
            Self::EastSouthEast => AxialHex::new(1, 1),
        }
    }
}
