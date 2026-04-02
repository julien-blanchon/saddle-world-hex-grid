use super::*;
use crate::AxialHex;
use std::collections::HashSet;

#[test]
fn a_star_returns_none_when_goal_is_unreachable() {
    let blocked = HashSet::from([
        AxialHex::new(1, 0),
        AxialHex::new(1, -1),
        AxialHex::new(0, -1),
        AxialHex::new(-1, 0),
        AxialHex::new(-1, 1),
        AxialHex::new(0, 1),
    ]);

    let path = a_star(AxialHex::ZERO, AxialHex::new(2, -1), |_, to| {
        (!blocked.contains(&to)).then_some(1)
    });

    assert!(path.is_none());
}

#[test]
fn a_star_prefers_cheaper_weighted_route() {
    let expensive = HashSet::from([AxialHex::new(1, 0), AxialHex::new(2, 0)]);
    let path = a_star(AxialHex::ZERO, AxialHex::new(3, 0), |_, to| {
        Some(if expensive.contains(&to) { 5 } else { 1 })
    })
    .expect("expected path");

    assert_eq!(path.cells.first().copied(), Some(AxialHex::ZERO));
    assert_eq!(path.cells.last().copied(), Some(AxialHex::new(3, 0)));
    assert!(path.total_cost < 15);

    let mut reconstructed_cost = 0;
    for pair in path.cells.windows(2) {
        assert_eq!(pair[0].distance_to(pair[1]), 1);
        reconstructed_cost += if expensive.contains(&pair[1]) { 5 } else { 1 };
    }
    assert_eq!(path.total_cost, reconstructed_cost);
}

#[test]
fn a_star_handles_start_equals_goal() {
    let path = a_star(AxialHex::new(2, -3), AxialHex::new(2, -3), |_, _| Some(1))
        .expect("start equals goal should be reachable");
    assert_eq!(path.cells, vec![AxialHex::new(2, -3)]);
    assert_eq!(path.total_cost, 0);
}

#[test]
fn reachable_within_tracks_costs_and_paths() {
    let blocked = HashSet::from([AxialHex::new(1, 0)]);
    let reachable = reachable_within(AxialHex::ZERO, 3, |_, to| {
        (!blocked.contains(&to)).then_some(1)
    });

    assert!(reachable.contains(AxialHex::new(0, 1)));
    assert!(!reachable.contains(AxialHex::new(4, 0)));

    let path = reachable
        .path_to(AxialHex::new(0, 2))
        .expect("expected path within budget");
    assert_eq!(path.total_cost, 2);
    assert_eq!(path.cells.first().copied(), Some(AxialHex::ZERO));
}

#[test]
#[should_panic(expected = "saddle_world_hex_grid::a_star expects positive edge costs")]
fn a_star_rejects_zero_cost_edges() {
    let _ = a_star(AxialHex::ZERO, AxialHex::new(1, 0), |_, _| Some(0));
}

#[test]
#[should_panic(expected = "saddle_world_hex_grid::reachable_within expects positive edge costs")]
fn reachable_within_rejects_zero_cost_edges() {
    let _ = reachable_within(AxialHex::ZERO, 2, |_, _| Some(0));
}
