use crate::AxialHex;
use std::collections::{BinaryHeap, HashMap};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HexPath {
    pub cells: Vec<AxialHex>,
    pub total_cost: u32,
}

impl HexPath {
    pub fn len(&self) -> usize {
        self.cells.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HexReachability {
    origin: AxialHex,
    costs: HashMap<AxialHex, u32>,
    came_from: HashMap<AxialHex, AxialHex>,
}

impl HexReachability {
    pub fn origin(&self) -> AxialHex {
        self.origin
    }

    pub fn contains(&self, hex: AxialHex) -> bool {
        self.costs.contains_key(&hex)
    }

    pub fn cost(&self, hex: AxialHex) -> Option<u32> {
        self.costs.get(&hex).copied()
    }

    pub fn len(&self) -> usize {
        self.costs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.costs.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (AxialHex, u32)> + '_ {
        self.costs.iter().map(|(hex, cost)| (*hex, *cost))
    }

    pub fn path_to(&self, goal: AxialHex) -> Option<HexPath> {
        let total_cost = self.cost(goal)?;
        let mut cells = vec![goal];
        let mut current = goal;
        while current != self.origin {
            current = *self.came_from.get(&current)?;
            cells.push(current);
        }
        cells.reverse();
        Some(HexPath { cells, total_cost })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SearchNode {
    hex: AxialHex,
    score: u32,
}

impl Ord for SearchNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .score
            .cmp(&self.score)
            .then_with(|| other.hex.q.cmp(&self.hex.q))
            .then_with(|| other.hex.r.cmp(&self.hex.r))
    }
}

impl PartialOrd for SearchNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub fn a_star(
    start: AxialHex,
    goal: AxialHex,
    mut edge_cost: impl FnMut(AxialHex, AxialHex) -> Option<u32>,
) -> Option<HexPath> {
    if start == goal {
        return Some(HexPath {
            cells: vec![start],
            total_cost: 0,
        });
    }

    let mut open = BinaryHeap::new();
    let mut costs = HashMap::from([(start, 0_u32)]);
    let mut came_from = HashMap::new();

    open.push(SearchNode {
        hex: start,
        score: start.distance_to(goal),
    });

    while let Some(SearchNode { hex, .. }) = open.pop() {
        if hex == goal {
            let total_cost = costs[&goal];
            let mut cells = vec![goal];
            let mut current = goal;
            while let Some(previous) = came_from.get(&current).copied() {
                cells.push(previous);
                current = previous;
            }
            cells.reverse();
            return Some(HexPath { cells, total_cost });
        }

        let current_cost = costs[&hex];
        for neighbor in hex.neighbors() {
            let Some(step_cost) = edge_cost(hex, neighbor) else {
                continue;
            };
            assert!(
                step_cost > 0,
                "saddle_world_hex_grid::a_star expects positive edge costs"
            );

            let new_cost = current_cost.saturating_add(step_cost);
            let is_better = costs.get(&neighbor).is_none_or(|best| new_cost < *best);
            if is_better {
                costs.insert(neighbor, new_cost);
                came_from.insert(neighbor, hex);
                open.push(SearchNode {
                    hex: neighbor,
                    score: new_cost.saturating_add(neighbor.distance_to(goal)),
                });
            }
        }
    }

    None
}

pub fn reachable_within(
    start: AxialHex,
    budget: u32,
    mut edge_cost: impl FnMut(AxialHex, AxialHex) -> Option<u32>,
) -> HexReachability {
    let mut open = BinaryHeap::new();
    let mut costs = HashMap::from([(start, 0_u32)]);
    let mut came_from = HashMap::new();

    open.push(SearchNode {
        hex: start,
        score: 0,
    });

    while let Some(SearchNode { hex, score }) = open.pop() {
        if score > budget {
            continue;
        }

        let current_cost = costs[&hex];
        for neighbor in hex.neighbors() {
            let Some(step_cost) = edge_cost(hex, neighbor) else {
                continue;
            };
            assert!(
                step_cost > 0,
                "saddle_world_hex_grid::reachable_within expects positive edge costs"
            );

            let new_cost = current_cost.saturating_add(step_cost);
            if new_cost > budget {
                continue;
            }

            let is_better = costs.get(&neighbor).is_none_or(|best| new_cost < *best);
            if is_better {
                costs.insert(neighbor, new_cost);
                came_from.insert(neighbor, hex);
                open.push(SearchNode {
                    hex: neighbor,
                    score: new_cost,
                });
            }
        }
    }

    HexReachability {
        origin: start,
        costs,
        came_from,
    }
}

#[cfg(test)]
#[path = "pathfinding_tests.rs"]
mod tests;
