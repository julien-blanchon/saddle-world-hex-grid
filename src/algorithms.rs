use crate::{AxialHex, DoubledHex, DoubledHexMode, FractionalHex, HexDirection, OffsetHexMode};
use std::ops::RangeInclusive;

/// Generates a triangle-shaped region.
///
/// For `side_length` = 3, generates a triangle with 3 rows:
/// - row 0: 1 hex
/// - row 1: 2 hexes
/// - row 2: 3 hexes
pub fn triangle(side_length: u32) -> TriangleIter {
    TriangleIter::new(side_length)
}

/// Generates hexes forming a wedge (partial ring arc) from `center`.
///
/// A wedge includes all hexes at exactly `radius` distance from `center`
/// that lie between `start_direction` and `end_direction` (inclusive),
/// walking clockwise.
pub fn wedge(
    center: AxialHex,
    radius: u32,
    start_direction: HexDirection,
    end_direction: HexDirection,
) -> WedgeIter {
    WedgeIter::new(center, radius, start_direction, end_direction)
}

pub fn parallelogram(
    origin: AxialHex,
    q_range: RangeInclusive<i32>,
    r_range: RangeInclusive<i32>,
) -> ParallelogramIter {
    ParallelogramIter::new(origin, q_range, r_range)
}

pub fn offset_rectangle(
    mode: OffsetHexMode,
    columns: RangeInclusive<i32>,
    rows: RangeInclusive<i32>,
) -> OffsetRectangleIter {
    OffsetRectangleIter::new(mode, columns, rows)
}

pub fn doubled_rectangle(
    mode: DoubledHexMode,
    columns: RangeInclusive<i32>,
    rows: RangeInclusive<i32>,
) -> DoubledRectangleIter {
    DoubledRectangleIter::new(mode, columns, rows)
}

#[derive(Clone, Debug)]
pub struct LineIter {
    start: FractionalHex,
    end: FractionalHex,
    steps: u32,
    index: u32,
}

impl LineIter {
    pub fn new(start: AxialHex, end: AxialHex) -> Self {
        Self {
            start: FractionalHex::from(start).nudged(),
            end: FractionalHex::from(end).nudged(),
            steps: start.distance_to(end),
            index: 0,
        }
    }
}

impl Iterator for LineIter {
    type Item = AxialHex;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index > self.steps {
            return None;
        }

        let hex = if self.steps == 0 {
            self.start.round()
        } else {
            let t = self.index as f32 / self.steps as f32;
            self.start.lerp(self.end, t).round()
        };

        self.index += 1;
        Some(hex)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining =
            self.steps.saturating_sub(self.index) as usize + usize::from(self.index <= self.steps);
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for LineIter {}

#[derive(Clone, Debug)]
pub struct RangeIter {
    center: AxialHex,
    radius: i32,
    q: i32,
    r: i32,
    current_r_end: i32,
    finished: bool,
}

impl RangeIter {
    pub fn new(center: AxialHex, radius: u32) -> Self {
        let radius = radius as i32;
        let q = -radius;
        let r = (-radius).max(-q - radius);
        let current_r_end = radius.min(-q + radius);
        Self {
            center,
            radius,
            q,
            r,
            current_r_end,
            finished: false,
        }
    }
}

impl Iterator for RangeIter {
    type Item = AxialHex;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        let hex = self.center + AxialHex::new(self.q, self.r);

        if self.r < self.current_r_end {
            self.r += 1;
        } else if self.q < self.radius {
            self.q += 1;
            self.r = (-self.radius).max(-self.q - self.radius);
            self.current_r_end = self.radius.min(-self.q + self.radius);
        } else {
            self.finished = true;
        }

        Some(hex)
    }
}

#[derive(Clone, Debug)]
pub struct RingIter {
    center: AxialHex,
    radius: u32,
    side: usize,
    step: u32,
    current: AxialHex,
    yielded_zero_radius: bool,
}

impl RingIter {
    pub fn new(center: AxialHex, radius: u32) -> Self {
        let current = center + HexDirection::SouthWest.vector() * radius as i32;
        Self {
            center,
            radius,
            side: 0,
            step: 0,
            current,
            yielded_zero_radius: false,
        }
    }
}

impl Iterator for RingIter {
    type Item = AxialHex;

    fn next(&mut self) -> Option<Self::Item> {
        if self.radius == 0 {
            if self.yielded_zero_radius {
                return None;
            }
            self.yielded_zero_radius = true;
            return Some(self.center);
        }

        if self.side >= 6 {
            return None;
        }

        let hex = self.current;
        let direction = HexDirection::ALL[self.side];
        self.current += direction.vector();
        self.step += 1;

        if self.step >= self.radius {
            self.step = 0;
            self.side += 1;
        }

        Some(hex)
    }
}

#[derive(Clone, Debug)]
pub struct SpiralIter {
    center: AxialHex,
    max_radius: u32,
    current_radius: u32,
    yielded_center: bool,
    current_ring: Option<RingIter>,
}

impl SpiralIter {
    pub fn new(center: AxialHex, max_radius: u32) -> Self {
        Self {
            center,
            max_radius,
            current_radius: 1,
            yielded_center: false,
            current_ring: None,
        }
    }
}

impl Iterator for SpiralIter {
    type Item = AxialHex;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.yielded_center {
            self.yielded_center = true;
            return Some(self.center);
        }

        loop {
            if let Some(ring) = &mut self.current_ring {
                if let Some(hex) = ring.next() {
                    return Some(hex);
                }
                self.current_ring = None;
            }

            if self.current_radius > self.max_radius {
                return None;
            }

            self.current_ring = Some(RingIter::new(self.center, self.current_radius));
            self.current_radius += 1;
        }
    }
}

#[derive(Clone, Debug)]
pub struct ParallelogramIter {
    origin: AxialHex,
    q_end: i32,
    r_start: i32,
    r_end: i32,
    current_q: i32,
    current_r: i32,
    finished: bool,
}

impl ParallelogramIter {
    pub fn new(
        origin: AxialHex,
        q_range: RangeInclusive<i32>,
        r_range: RangeInclusive<i32>,
    ) -> Self {
        let q_start = *q_range.start();
        let q_end = *q_range.end();
        let r_start = *r_range.start();
        let r_end = *r_range.end();
        Self {
            origin,
            q_end,
            r_start,
            r_end,
            current_q: q_start,
            current_r: r_start,
            finished: false,
        }
    }
}

impl Iterator for ParallelogramIter {
    type Item = AxialHex;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        let hex = self.origin + AxialHex::new(self.current_q, self.current_r);
        if self.current_r < self.r_end {
            self.current_r += 1;
        } else if self.current_q < self.q_end {
            self.current_q += 1;
            self.current_r = self.r_start;
        } else {
            self.finished = true;
        }

        Some(hex)
    }
}

#[derive(Clone, Debug)]
pub struct OffsetRectangleIter {
    mode: OffsetHexMode,
    col_end: i32,
    row_start: i32,
    row_end: i32,
    current_col: i32,
    current_row: i32,
    finished: bool,
}

impl OffsetRectangleIter {
    pub fn new(
        mode: OffsetHexMode,
        columns: RangeInclusive<i32>,
        rows: RangeInclusive<i32>,
    ) -> Self {
        let col_start = *columns.start();
        let col_end = *columns.end();
        let row_start = *rows.start();
        let row_end = *rows.end();
        Self {
            mode,
            col_end,
            row_start,
            row_end,
            current_col: col_start,
            current_row: row_start,
            finished: false,
        }
    }
}

impl Iterator for OffsetRectangleIter {
    type Item = AxialHex;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        let hex = crate::OffsetHex::new(self.current_col, self.current_row).to_axial(self.mode);
        if self.current_row < self.row_end {
            self.current_row += 1;
        } else if self.current_col < self.col_end {
            self.current_col += 1;
            self.current_row = self.row_start;
        } else {
            self.finished = true;
        }

        Some(hex)
    }
}

#[derive(Clone, Debug)]
pub struct DoubledRectangleIter {
    mode: DoubledHexMode,
    col_end: i32,
    row_start: i32,
    row_end: i32,
    current_col: i32,
    current_row: i32,
}

impl DoubledRectangleIter {
    pub fn new(
        mode: DoubledHexMode,
        columns: RangeInclusive<i32>,
        rows: RangeInclusive<i32>,
    ) -> Self {
        let col_start = *columns.start();
        let col_end = *columns.end();
        let row_start = *rows.start();
        let row_end = *rows.end();
        Self {
            mode,
            col_end,
            row_start,
            row_end,
            current_col: col_start,
            current_row: row_start,
        }
    }

    fn advance(&mut self) -> bool {
        if self.current_row < self.row_end {
            self.current_row += 1;
            true
        } else if self.current_col < self.col_end {
            self.current_col += 1;
            self.current_row = self.row_start;
            true
        } else {
            false
        }
    }
}

impl Iterator for DoubledRectangleIter {
    type Item = AxialHex;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.current_col > self.col_end {
                return None;
            }

            let current = (self.current_col, self.current_row);
            if !self.advance() {
                self.current_col = self.col_end + 1;
            }

            let Ok(doubled) = DoubledHex::new(current.0, current.1) else {
                continue;
            };
            return Some(doubled.to_axial(self.mode));
        }
    }
}

#[derive(Clone, Debug)]
pub struct TriangleIter {
    side_length: i32,
    current_r: i32,
    current_q: i32,
}

impl TriangleIter {
    pub fn new(side_length: u32) -> Self {
        Self {
            side_length: side_length as i32,
            current_r: 0,
            current_q: 0,
        }
    }
}

impl Iterator for TriangleIter {
    type Item = AxialHex;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_r >= self.side_length {
            return None;
        }

        let hex = AxialHex::new(self.current_q, self.current_r);

        if self.current_q < self.current_r {
            self.current_q += 1;
        } else {
            self.current_r += 1;
            self.current_q = 0;
        }

        Some(hex)
    }
}

#[derive(Clone, Debug)]
pub struct WedgeIter {
    center: AxialHex,
    radius: u32,
    current: AxialHex,
    side: usize,
    step: u32,
    end_side: usize,
    end_step: u32,
    finished: bool,
    yielded_zero: bool,
}

impl WedgeIter {
    pub fn new(
        center: AxialHex,
        radius: u32,
        start_direction: HexDirection,
        end_direction: HexDirection,
    ) -> Self {
        if radius == 0 {
            return Self {
                center,
                radius: 0,
                current: center,
                side: 0,
                step: 0,
                end_side: 0,
                end_step: 0,
                finished: false,
                yielded_zero: false,
            };
        }

        let start_idx = start_direction.index();
        // Start position: move from center in the direction before `start_direction`
        // (the ring starts at the SouthWest corner and walks CW through the 6 sides)
        // We need to find which side/step on the ring corresponds to start_direction
        let start_side = (start_idx + 4) % 6; // Adjust for ring starting at SouthWest
        let end_idx = end_direction.index();
        let end_side = (end_idx + 4) % 6;

        // Starting position on the ring
        let mut pos = center + HexDirection::SouthWest.vector() * radius as i32;
        for s in 0..start_side {
            pos += HexDirection::ALL[s].vector() * radius as i32;
        }

        Self {
            center,
            radius,
            current: pos,
            side: start_side,
            step: 0,
            end_side,
            end_step: radius.saturating_sub(1),
            finished: false,
            yielded_zero: false,
        }
    }
}

impl Iterator for WedgeIter {
    type Item = AxialHex;

    fn next(&mut self) -> Option<Self::Item> {
        if self.radius == 0 {
            if self.yielded_zero {
                return None;
            }
            self.yielded_zero = true;
            return Some(self.center);
        }

        if self.finished {
            return None;
        }

        let hex = self.current;

        // Check if we've reached the end
        if self.side == self.end_side && self.step >= self.end_step {
            self.finished = true;
        }

        // Advance
        let direction = HexDirection::ALL[self.side % 6];
        self.current += direction.vector();
        self.step += 1;

        if self.step >= self.radius {
            self.step = 0;
            self.side += 1;
            if self.side >= 6 + self.end_side {
                self.finished = true;
            }
        }

        Some(hex)
    }
}

#[cfg(test)]
#[path = "algorithms_tests.rs"]
mod tests;
