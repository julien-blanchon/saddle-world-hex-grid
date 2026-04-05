use super::*;

#[test]
fn contains_checks_distance_correctly() {
    let bounds = HexBounds::new(AxialHex::ZERO, 2);
    assert!(bounds.contains(AxialHex::ZERO));
    assert!(bounds.contains(AxialHex::new(1, 0)));
    assert!(bounds.contains(AxialHex::new(2, 0)));
    assert!(!bounds.contains(AxialHex::new(3, 0)));
}

#[test]
fn hex_count_matches_iteration() {
    for radius in 0..=5 {
        let bounds = HexBounds::new(AxialHex::new(1, -1), radius);
        let count = bounds.iter().count();
        assert_eq!(count, bounds.hex_count());
    }
}

#[test]
fn intersects_detects_overlapping_bounds() {
    let a = HexBounds::new(AxialHex::ZERO, 2);
    let b = HexBounds::new(AxialHex::new(3, 0), 2);
    assert!(a.intersects(&b));

    let c = HexBounds::new(AxialHex::new(10, 0), 2);
    assert!(!a.intersects(&c));
}

#[test]
fn into_iterator_yields_all_hexes() {
    let bounds = HexBounds::new(AxialHex::ZERO, 2);
    let hexes: Vec<_> = bounds.into_iter().collect();
    assert_eq!(hexes.len(), bounds.hex_count());
}

#[test]
fn wrap_returns_hex_inside_bounds() {
    let bounds = HexBounds::new(AxialHex::ZERO, 3);
    let outside = AxialHex::new(10, 0);
    let wrapped = bounds.wrap(outside);
    assert!(bounds.contains(wrapped));
}
