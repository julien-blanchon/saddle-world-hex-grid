use crate::{
    HexDiagonalDirection, HexDirection, LineIter, OffsetRectangleIter, ParallelogramIter,
    RangeIter, RingIter, SpiralIter, doubled_rectangle, offset_rectangle, parallelogram,
};
use bevy::{math::Vec2, prelude::Reflect};
use std::{
    error::Error,
    fmt::{Display, Formatter},
    ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HexInvariantError {
    InvalidCubeSum { q: i32, r: i32, s: i32 },
    InvalidDoubledParity { col: i32, row: i32 },
}

impl Display for HexInvariantError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidCubeSum { q, r, s } => {
                write!(
                    f,
                    "invalid cube coordinates: q + r + s must equal 0, got {q} + {r} + {s}"
                )
            }
            Self::InvalidDoubledParity { col, row } => {
                write!(
                    f,
                    "invalid doubled coordinates: col + row must be even, got col={col}, row={row}"
                )
            }
        }
    }
}

impl Error for HexInvariantError {}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Reflect)]
pub struct AxialHex {
    pub q: i32,
    pub r: i32,
}

impl AxialHex {
    pub const ZERO: Self = Self::new(0, 0);

    pub const fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }

    pub const fn s(self) -> i32 {
        -self.q - self.r
    }

    pub const fn to_cube(self) -> CubeHex {
        CubeHex {
            q: self.q,
            r: self.r,
            s: self.s(),
        }
    }

    pub fn to_fractional(self) -> FractionalHex {
        FractionalHex::from(self)
    }

    pub fn to_offset(self, mode: OffsetHexMode) -> OffsetHex {
        match mode {
            OffsetHexMode::OddColumns => {
                let row = self.r + (self.q - (self.q & 1)) / 2;
                OffsetHex::new(self.q, row)
            }
            OffsetHexMode::EvenColumns => {
                let row = self.r + (self.q + (self.q & 1)) / 2;
                OffsetHex::new(self.q, row)
            }
            OffsetHexMode::OddRows => {
                let col = self.q + (self.r - (self.r & 1)) / 2;
                OffsetHex::new(col, self.r)
            }
            OffsetHexMode::EvenRows => {
                let col = self.q + (self.r + (self.r & 1)) / 2;
                OffsetHex::new(col, self.r)
            }
        }
    }

    pub fn to_doubled(self, mode: DoubledHexMode) -> DoubledHex {
        match mode {
            DoubledHexMode::DoubleWidth => DoubledHex {
                col: self.q * 2 + self.r,
                row: self.r,
            },
            DoubledHexMode::DoubleHeight => DoubledHex {
                col: self.q,
                row: self.r * 2 + self.q,
            },
        }
    }

    pub fn neighbors(self) -> [Self; 6] {
        HexDirection::ALL.map(|direction| self.neighbor(direction))
    }

    pub fn neighbor(self, direction: HexDirection) -> Self {
        self + direction.vector()
    }

    pub fn diagonal_neighbors(self) -> [Self; 6] {
        HexDiagonalDirection::ALL.map(|direction| self.diagonal_neighbor(direction))
    }

    pub fn diagonal_neighbor(self, direction: HexDiagonalDirection) -> Self {
        self + direction.vector()
    }

    pub fn length(self) -> u32 {
        self.distance_to(Self::ZERO)
    }

    /// Manhattan (hex) distance — the minimum number of steps between two hexes.
    pub fn distance_to(self, other: Self) -> u32 {
        let delta = self - other;
        delta
            .q
            .unsigned_abs()
            .max(delta.r.unsigned_abs())
            .max(delta.s().unsigned_abs())
    }

    /// Squared Euclidean distance between two hex centers.
    ///
    /// Cheaper than [`euclidean_distance_to`](Self::euclidean_distance_to) because
    /// it avoids a square root. Useful for distance comparisons.
    pub fn distance_sq_to(self, other: Self) -> f32 {
        let dq = (self.q - other.q) as f32;
        let dr = (self.r - other.r) as f32;
        // In axial coords, the squared Euclidean distance (unit hex size) is:
        // dq^2 + dq*dr + dr^2
        dq * dq + dq * dr + dr * dr
    }

    /// Euclidean distance between two hex centers (in hex-unit space).
    pub fn euclidean_distance_to(self, other: Self) -> f32 {
        self.distance_sq_to(other).sqrt()
    }

    pub fn rotate_cw(self, steps: i32) -> Self {
        Self::from(self.to_cube().rotate_cw(steps))
    }

    pub fn rotate_ccw(self, steps: i32) -> Self {
        Self::from(self.to_cube().rotate_ccw(steps))
    }

    pub fn rotate_cw_around(self, center: Self, steps: i32) -> Self {
        (self - center).rotate_cw(steps) + center
    }

    pub fn rotate_ccw_around(self, center: Self, steps: i32) -> Self {
        (self - center).rotate_ccw(steps) + center
    }

    pub fn reflect_q(self) -> Self {
        Self::from(self.to_cube().reflect_q())
    }

    pub fn reflect_r(self) -> Self {
        Self::from(self.to_cube().reflect_r())
    }

    pub fn reflect_s(self) -> Self {
        Self::from(self.to_cube().reflect_s())
    }

    /// Determines which diagonal direction `other` lies in relative to `self`.
    ///
    /// Returns a [`crate::DiagonalWay`] indicating either a single diagonal direction
    /// or a tie between two directions when the hex is on a sector boundary.
    pub fn diagonal_way_to(self, other: Self) -> crate::fov::DiagonalWay {
        crate::fov::diagonal_way(self, other)
    }

    pub fn line_to(self, target: Self) -> LineIter {
        LineIter::new(self, target)
    }

    pub fn range(self, radius: u32) -> RangeIter {
        RangeIter::new(self, radius)
    }

    pub fn ring(self, radius: u32) -> RingIter {
        RingIter::new(self, radius)
    }

    pub fn spiral(self, radius: u32) -> SpiralIter {
        SpiralIter::new(self, radius)
    }

    pub fn hexagon(self, radius: u32) -> RangeIter {
        self.range(radius)
    }

    pub fn parallelogram(
        self,
        q_range: std::ops::RangeInclusive<i32>,
        r_range: std::ops::RangeInclusive<i32>,
    ) -> ParallelogramIter {
        parallelogram(self, q_range, r_range)
    }

    pub fn offset_rectangle(
        mode: OffsetHexMode,
        columns: std::ops::RangeInclusive<i32>,
        rows: std::ops::RangeInclusive<i32>,
    ) -> OffsetRectangleIter {
        offset_rectangle(mode, columns, rows)
    }

    pub fn doubled_rectangle(
        mode: DoubledHexMode,
        columns: std::ops::RangeInclusive<i32>,
        rows: std::ops::RangeInclusive<i32>,
    ) -> crate::DoubledRectangleIter {
        doubled_rectangle(mode, columns, rows)
    }
}

impl From<CubeHex> for AxialHex {
    fn from(value: CubeHex) -> Self {
        Self::new(value.q, value.r)
    }
}

impl Add for AxialHex {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.q + rhs.q, self.r + rhs.r)
    }
}

impl AddAssign for AxialHex {
    fn add_assign(&mut self, rhs: Self) {
        self.q += rhs.q;
        self.r += rhs.r;
    }
}

impl Sub for AxialHex {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.q - rhs.q, self.r - rhs.r)
    }
}

impl SubAssign for AxialHex {
    fn sub_assign(&mut self, rhs: Self) {
        self.q -= rhs.q;
        self.r -= rhs.r;
    }
}

impl Neg for AxialHex {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.q, -self.r)
    }
}

impl Mul<i32> for AxialHex {
    type Output = Self;

    fn mul(self, rhs: i32) -> Self::Output {
        Self::new(self.q * rhs, self.r * rhs)
    }
}

impl Div<i32> for AxialHex {
    type Output = Self;

    fn div(self, rhs: i32) -> Self::Output {
        Self::new(self.q / rhs, self.r / rhs)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect)]
pub struct CubeHex {
    pub q: i32,
    pub r: i32,
    pub s: i32,
}

impl CubeHex {
    pub const ZERO: Self = Self { q: 0, r: 0, s: 0 };

    pub fn new(q: i32, r: i32, s: i32) -> Result<Self, HexInvariantError> {
        if q + r + s == 0 {
            Ok(Self { q, r, s })
        } else {
            Err(HexInvariantError::InvalidCubeSum { q, r, s })
        }
    }

    pub const fn from_axial(hex: AxialHex) -> Self {
        Self {
            q: hex.q,
            r: hex.r,
            s: hex.s(),
        }
    }

    pub const fn to_axial(self) -> AxialHex {
        AxialHex::new(self.q, self.r)
    }

    pub fn rotate_cw(self, steps: i32) -> Self {
        let mut result = self;
        for _ in 0..steps.rem_euclid(6) {
            result = Self {
                q: -result.s,
                r: -result.q,
                s: -result.r,
            };
        }
        result
    }

    pub fn rotate_ccw(self, steps: i32) -> Self {
        let mut result = self;
        for _ in 0..steps.rem_euclid(6) {
            result = Self {
                q: -result.r,
                r: -result.s,
                s: -result.q,
            };
        }
        result
    }

    pub const fn reflect_q(self) -> Self {
        Self {
            q: self.q,
            r: self.s,
            s: self.r,
        }
    }

    pub const fn reflect_r(self) -> Self {
        Self {
            q: self.s,
            r: self.r,
            s: self.q,
        }
    }

    pub const fn reflect_s(self) -> Self {
        Self {
            q: self.r,
            r: self.q,
            s: self.s,
        }
    }
}

impl Add for CubeHex {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            q: self.q + rhs.q,
            r: self.r + rhs.r,
            s: self.s + rhs.s,
        }
    }
}

impl Sub for CubeHex {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            q: self.q - rhs.q,
            r: self.r - rhs.r,
            s: self.s - rhs.s,
        }
    }
}

impl Neg for CubeHex {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            q: -self.q,
            r: -self.r,
            s: -self.s,
        }
    }
}

impl Mul<i32> for CubeHex {
    type Output = Self;

    fn mul(self, rhs: i32) -> Self::Output {
        Self {
            q: self.q * rhs,
            r: self.r * rhs,
            s: self.s * rhs,
        }
    }
}

impl From<AxialHex> for CubeHex {
    fn from(value: AxialHex) -> Self {
        Self::from_axial(value)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
pub struct FractionalHex {
    pub q: f32,
    pub r: f32,
    pub s: f32,
}

impl FractionalHex {
    pub const fn new(q: f32, r: f32, s: f32) -> Self {
        Self { q, r, s }
    }

    pub fn from_axial(hex: AxialHex) -> Self {
        Self::new(hex.q as f32, hex.r as f32, hex.s() as f32)
    }

    pub fn round(self) -> AxialHex {
        let mut q = self.q.round();
        let mut r = self.r.round();
        let s = self.s.round();

        let q_diff = (q - self.q).abs();
        let r_diff = (r - self.r).abs();
        let s_diff = (s - self.s).abs();

        if q_diff > r_diff && q_diff > s_diff {
            q = -r - s;
        } else if r_diff > s_diff {
            r = -q - s;
        }

        AxialHex::new(q as i32, r as i32)
    }

    pub fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            q: self.q + (other.q - self.q) * t,
            r: self.r + (other.r - self.r) * t,
            s: self.s + (other.s - self.s) * t,
        }
    }

    pub fn nudged(self) -> Self {
        Self {
            q: self.q + 1.0e-6,
            r: self.r + 1.0e-6,
            s: self.s - 2.0e-6,
        }
    }

    pub fn to_vec2(self) -> Vec2 {
        Vec2::new(self.q, self.r)
    }
}

impl From<AxialHex> for FractionalHex {
    fn from(value: AxialHex) -> Self {
        Self::from_axial(value)
    }
}

impl From<CubeHex> for FractionalHex {
    fn from(value: CubeHex) -> Self {
        Self::new(value.q as f32, value.r as f32, value.s as f32)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect)]
pub enum OffsetHexMode {
    OddColumns,
    EvenColumns,
    OddRows,
    EvenRows,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Reflect)]
pub struct OffsetHex {
    pub col: i32,
    pub row: i32,
}

impl OffsetHex {
    pub const fn new(col: i32, row: i32) -> Self {
        Self { col, row }
    }

    pub fn to_axial(self, mode: OffsetHexMode) -> AxialHex {
        match mode {
            OffsetHexMode::OddColumns => {
                let q = self.col;
                let r = self.row - (self.col - (self.col & 1)) / 2;
                AxialHex::new(q, r)
            }
            OffsetHexMode::EvenColumns => {
                let q = self.col;
                let r = self.row - (self.col + (self.col & 1)) / 2;
                AxialHex::new(q, r)
            }
            OffsetHexMode::OddRows => {
                let q = self.col - (self.row - (self.row & 1)) / 2;
                AxialHex::new(q, self.row)
            }
            OffsetHexMode::EvenRows => {
                let q = self.col - (self.row + (self.row & 1)) / 2;
                AxialHex::new(q, self.row)
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect)]
pub enum DoubledHexMode {
    DoubleWidth,
    DoubleHeight,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Reflect)]
pub struct DoubledHex {
    pub col: i32,
    pub row: i32,
}

impl DoubledHex {
    pub fn new(col: i32, row: i32) -> Result<Self, HexInvariantError> {
        if (col + row) % 2 == 0 {
            Ok(Self { col, row })
        } else {
            Err(HexInvariantError::InvalidDoubledParity { col, row })
        }
    }

    pub const fn new_unchecked(col: i32, row: i32) -> Self {
        Self { col, row }
    }

    pub fn to_axial(self, mode: DoubledHexMode) -> AxialHex {
        match mode {
            DoubledHexMode::DoubleWidth => AxialHex::new((self.col - self.row) / 2, self.row),
            DoubledHexMode::DoubleHeight => AxialHex::new(self.col, (self.row - self.col) / 2),
        }
    }
}

#[cfg(test)]
#[path = "coords_tests.rs"]
mod tests;
