use super::*;
use std::collections::HashSet;

#[test]
fn neighbor_and_diagonal_counts_match_hex_expectations() {
    let origin = AxialHex::ZERO;
    let neighbors: HashSet<_> = origin.neighbors().into_iter().collect();
    let diagonals: HashSet<_> = origin.diagonal_neighbors().into_iter().collect();

    assert_eq!(neighbors.len(), 6);
    assert_eq!(diagonals.len(), 6);
    assert!(neighbors.is_disjoint(&diagonals));
}

#[test]
fn distance_is_symmetric_and_respects_triangle_inequality_on_sample_grid() {
    let samples = [
        AxialHex::new(0, 0),
        AxialHex::new(3, -2),
        AxialHex::new(-4, 1),
        AxialHex::new(2, 5),
    ];

    for a in samples {
        for b in samples {
            assert_eq!(a.distance_to(b), b.distance_to(a));
            for c in samples {
                assert!(a.distance_to(c) <= a.distance_to(b) + b.distance_to(c));
            }
        }
    }
}

#[test]
fn line_includes_endpoints_and_contiguous_neighbors() {
    let start = AxialHex::new(-2, 3);
    let end = AxialHex::new(4, -1);
    let line: Vec<_> = start.line_to(end).collect();

    assert_eq!(line.first().copied(), Some(start));
    assert_eq!(line.last().copied(), Some(end));
    assert_eq!(line.len(), start.distance_to(end) as usize + 1);
    for pair in line.windows(2) {
        assert_eq!(pair[0].distance_to(pair[1]), 1);
    }
}

#[test]
fn ring_has_expected_cardinality_and_distance() {
    let center = AxialHex::new(1, -2);
    assert_eq!(center.ring(0).collect::<Vec<_>>(), vec![center]);

    for radius in 1..=5 {
        let ring: Vec<_> = center.ring(radius).collect();
        assert_eq!(ring.len(), 6 * radius as usize);
        for hex in ring {
            assert_eq!(center.distance_to(hex), radius);
        }
    }
}

#[test]
fn spiral_contains_every_hex_up_to_radius_once() {
    let center = AxialHex::new(-2, 1);
    let spiral: Vec<_> = center.spiral(3).collect();
    let unique: HashSet<_> = spiral.iter().copied().collect();
    let expected: HashSet<_> = center.range(3).collect();

    assert_eq!(spiral.len(), 37);
    assert_eq!(unique.len(), spiral.len());
    assert_eq!(unique, expected);
}

#[test]
fn rectangle_and_parallelogram_adapters_produce_expected_counts() {
    let offset: Vec<_> = offset_rectangle(OffsetHexMode::OddColumns, 0..=3, 0..=2).collect();
    let doubled: Vec<_> = doubled_rectangle(DoubledHexMode::DoubleWidth, 0..=6, 0..=4).collect();
    let parallelogram: Vec<_> = parallelogram(AxialHex::new(2, -1), -1..=2, 0..=1).collect();

    assert_eq!(offset.len(), 12);
    assert_eq!(doubled.len(), 18);
    assert_eq!(parallelogram.len(), 8);
}
