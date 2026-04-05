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

        for r in -radius_i..=radius_i {
            let q_min = (-radius_i).max(-r - radius_i);
            let q_max = radius_i.min(-r + radius_i);
            for q in q_min..=q_max {
                data.push(init(center + AxialHex::new(q, r)));
            }
        }

        Self {
            center,
            radius: radius_i,
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

        // Calculate flat index: sum of row lengths for rows before `r`,
        // plus the offset within row `r`.
        let mut idx = 0usize;
        for row in -self.radius..r {
            let q_min = (-self.radius).max(-row - self.radius);
            let q_max = self.radius.min(-row + self.radius);
            idx += (q_max - q_min + 1) as usize;
        }
        let q_min = (-self.radius).max(-r - self.radius);
        idx += (q - q_min) as usize;

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
        let center = self.center;
        let radius = self.radius;
        let mut idx = 0;
        let mut pairs = Vec::with_capacity(self.data.len());
        for r in -radius..=radius {
            let q_min = (-radius).max(-r - radius);
            let q_max = radius.min(-r + radius);
            for q in q_min..=q_max {
                pairs.push((center + AxialHex::new(q, r), &self.data[idx]));
                idx += 1;
            }
        }
        pairs.into_iter()
    }

    /// Iterates over all `(hex, &mut value)` pairs.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (AxialHex, &mut T)> {
        let center = self.center;
        let radius = self.radius;
        let mut idx = 0;
        let mut row_starts = Vec::new();
        for r in -radius..=radius {
            let q_min = (-radius).max(-r - radius);
            let q_max = radius.min(-r + radius);
            row_starts.push((r, q_min, q_max, idx));
            idx += (q_max - q_min + 1) as usize;
        }

        let data_ptr = self.data.as_mut_ptr();
        let mut pairs = Vec::with_capacity(self.data.len());
        for (r, q_min, q_max, start) in row_starts {
            for (offset, q) in (q_min..=q_max).enumerate() {
                // SAFETY: each index is accessed exactly once (no aliasing)
                let value = unsafe { &mut *data_ptr.add(start + offset) };
                pairs.push((center + AxialHex::new(q, r), value));
            }
        }
        pairs.into_iter()
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

#[cfg(test)]
#[path = "storage_tests.rs"]
mod tests;
