use super::*;

#[test]
fn hexagonal_map_new_has_correct_count() {
    let map = HexagonalMap::<i32>::new(AxialHex::ZERO, 3, |hex| hex.q + hex.r);
    assert_eq!(map.len(), 37); // 3*3*(3+1)+1 = 37
}

#[test]
fn hexagonal_map_get_and_index_agree() {
    let map = HexagonalMap::new(AxialHex::ZERO, 2, |hex| hex.q * 10 + hex.r);
    for hex in AxialHex::ZERO.range(2) {
        assert_eq!(map.get(hex), Some(&map[hex]));
    }
}

#[test]
fn hexagonal_map_out_of_bounds_returns_none() {
    let map = HexagonalMap::<i32>::with_default(AxialHex::ZERO, 2);
    assert!(map.get(AxialHex::new(3, 0)).is_none());
    assert!(map.get(AxialHex::new(0, 3)).is_none());
}

#[test]
fn hexagonal_map_mutation_works() {
    let mut map = HexagonalMap::with_default(AxialHex::ZERO, 1);
    map[AxialHex::new(1, 0)] = 42;
    assert_eq!(map[AxialHex::new(1, 0)], 42);
    assert_eq!(map[AxialHex::ZERO], 0);
}

#[test]
fn hexagonal_map_iter_visits_all_hexes() {
    let map = HexagonalMap::new(AxialHex::new(1, -1), 2, |hex| hex.q);
    let collected: Vec<_> = map.iter().collect();
    assert_eq!(collected.len(), map.len());
    // Verify the center hex
    assert!(
        collected
            .iter()
            .any(|(hex, val)| *hex == AxialHex::new(1, -1) && **val == 1)
    );
}

#[test]
fn hexagonal_map_contains_works() {
    let map = HexagonalMap::<()>::new(AxialHex::ZERO, 2, |_| ());
    assert!(map.contains(AxialHex::ZERO));
    assert!(map.contains(AxialHex::new(2, 0)));
    assert!(!map.contains(AxialHex::new(3, 0)));
}

#[test]
fn hexagonal_map_off_center_works() {
    let center = AxialHex::new(5, -3);
    let map = HexagonalMap::new(center, 1, |hex| hex.q);
    assert_eq!(map[center], 5);
    assert_eq!(map[AxialHex::new(6, -3)], 6);
    assert!(map.get(AxialHex::new(0, 0)).is_none());
}
