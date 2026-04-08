use crate::AxialHex;

/// Dense hexagonal map storage backed by a `Vec`.
///
/// Stores values for all hexes within a given radius of a center hex.
/// Index math converts axial coordinates to a flat array index with O(1) access.
///
/// # Layout
///
/// Internally the hexes are stored row by row (by `r` coordinate),
/// each row containing the valid `q` range for that `r` within the radius.
#[derive(Clone, Debug)]
pub struct HexagonalMap<T> {
    center: AxialHex,
    radius: i32,
    row_starts: Vec<usize>,
    data: Vec<T>,
}

impl<T> HexagonalMap<T> {
    /// Creates a new hexagonal map filled with values produced by a closure.
    ///
    /// The closure receives the axial coordinate of each hex.
    pub fn new(center: AxialHex, radius: u32, mut init: impl FnMut(AxialHex) -> T) -> Self {
        let radius_i = radius as i32;
        let count = 3 * (radius as usize) * (radius as usize + 1) + 1;
        let mut data = Vec::with_capacity(count);
        let mut row_starts = Vec::with_capacity((radius_i * 2 + 1) as usize);

        for r in -radius_i..=radius_i {
            row_starts.push(data.len());
            let (q_min, q_max) = row_q_bounds(radius_i, r);
            for q in q_min..=q_max {
                data.push(init(center + AxialHex::new(q, r)));
            }
        }

        Self {
            center,
            radius: radius_i,
            row_starts,
            data,
        }
    }

    /// Returns the center hex of this map.
    pub fn center(&self) -> AxialHex {
        self.center
    }

    /// Returns the radius of this map.
    pub fn radius(&self) -> u32 {
        self.radius as u32
    }

    /// Returns the number of hexes stored.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns `true` if the map contains no hexes (radius 0 still has 1 hex).
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns `true` if the given hex is within this map's bounds.
    pub fn contains(&self, hex: AxialHex) -> bool {
        self.index_of(hex).is_some()
    }

    fn index_of(&self, hex: AxialHex) -> Option<usize> {
        let delta = hex - self.center;
        let q = delta.q;
        let r = delta.r;

        if q.abs() > self.radius || r.abs() > self.radius || (q + r).abs() > self.radius {
            return None;
        }

        let row_index = (r + self.radius) as usize;
        let (q_min, _) = row_q_bounds(self.radius, r);
        let idx = self.row_starts[row_index] + (q - q_min) as usize;

        Some(idx)
    }

    /// Gets a reference to the value at the given hex, if it exists.
    pub fn get(&self, hex: AxialHex) -> Option<&T> {
        self.index_of(hex).map(|i| &self.data[i])
    }

    /// Gets a mutable reference to the value at the given hex, if it exists.
    pub fn get_mut(&mut self, hex: AxialHex) -> Option<&mut T> {
        self.index_of(hex).map(|i| &mut self.data[i])
    }

    /// Iterates over all `(hex, value)` pairs.
    pub fn iter(&self) -> impl Iterator<Item = (AxialHex, &T)> {
        HexagonalMapCoordsIter::new(self.center, self.radius).zip(self.data.iter())
    }

    /// Iterates over all `(hex, &mut value)` pairs.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (AxialHex, &mut T)> {
        HexagonalMapCoordsIter::new(self.center, self.radius).zip(self.data.iter_mut())
    }
}

impl<T> std::ops::Index<AxialHex> for HexagonalMap<T> {
    type Output = T;

    fn index(&self, hex: AxialHex) -> &Self::Output {
        self.get(hex)
            .expect("hex coordinate out of HexagonalMap bounds")
    }
}

impl<T> std::ops::IndexMut<AxialHex> for HexagonalMap<T> {
    fn index_mut(&mut self, hex: AxialHex) -> &mut Self::Output {
        self.get_mut(hex)
            .expect("hex coordinate out of HexagonalMap bounds")
    }
}

impl<T: Default> HexagonalMap<T> {
    /// Creates a new hexagonal map filled with default values.
    pub fn with_default(center: AxialHex, radius: u32) -> Self {
        Self::new(center, radius, |_| T::default())
    }
}

#[derive(Clone, Debug)]
struct HexagonalMapCoordsIter {
    center: AxialHex,
    radius: i32,
    q: i32,
    r: i32,
    current_q_max: i32,
    finished: bool,
}

impl HexagonalMapCoordsIter {
    fn new(center: AxialHex, radius: i32) -> Self {
        let r = -radius;
        let (q_min, q_max) = row_q_bounds(radius, r);
        Self {
            center,
            radius,
            q: q_min,
            r,
            current_q_max: q_max,
            finished: false,
        }
    }
}

impl Iterator for HexagonalMapCoordsIter {
    type Item = AxialHex;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        let hex = self.center + AxialHex::new(self.q, self.r);

        if self.q < self.current_q_max {
            self.q += 1;
        } else if self.r < self.radius {
            self.r += 1;
            let (q_min, q_max) = row_q_bounds(self.radius, self.r);
            self.q = q_min;
            self.current_q_max = q_max;
        } else {
            self.finished = true;
        }

        Some(hex)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.finished {
            return (0, Some(0));
        }

        let remaining_rows = (self.radius - self.r).max(0) as usize;
        let current_row_remaining = (self.current_q_max - self.q + 1).max(0) as usize;
        let trailing = (1..=remaining_rows)
            .map(|offset| {
                let row = self.r + offset as i32;
                let (q_min, q_max) = row_q_bounds(self.radius, row);
                (q_max - q_min + 1) as usize
            })
            .sum::<usize>();
        let remaining = current_row_remaining + trailing;
        (remaining, Some(remaining))
    }
}

fn row_q_bounds(radius: i32, r: i32) -> (i32, i32) {
    ((-radius).max(-r - radius), radius.min(-r + radius))
}

#[cfg(test)]
#[path = "storage_tests.rs"]
mod tests;
