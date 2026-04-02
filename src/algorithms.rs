use crate::{AxialHex, DoubledHex, DoubledHexMode, FractionalHex, HexDirection, OffsetHexMode};
use std::ops::RangeInclusive;

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

#[cfg(test)]
#[path = "algorithms_tests.rs"]
mod tests;
