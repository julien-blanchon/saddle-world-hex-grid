use crate::AxialHex;
use bevy::prelude::Reflect;

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
