use super::*;

#[test]
fn range_fov_zero_radius_returns_origin_only() {
    let visible = range_fov(AxialHex::ZERO, 0, |_| false);
    assert_eq!(visible.len(), 1);
    assert!(visible.contains(&AxialHex::ZERO));
}

#[test]
fn range_fov_no_blockers_returns_full_range() {
    let origin = AxialHex::ZERO;
    let visible = range_fov(origin, 3, |_| false);
    let expected: HashSet<_> = origin.range(3).collect();
    assert_eq!(visible, expected);
}

#[test]
fn range_fov_wall_blocks_cells_behind_it() {
    let origin = AxialHex::ZERO;
    let wall = AxialHex::new(1, 0);
    let behind_wall = AxialHex::new(2, 0);

    let visible = range_fov(origin, 3, |hex| hex == wall);

    // The wall itself should be visible
    assert!(visible.contains(&wall));
    // The cell directly behind the wall should NOT be visible
    assert!(!visible.contains(&behind_wall));
}

#[test]
fn range_fov_origin_is_always_visible() {
    let origin = AxialHex::new(2, -1);
    let visible = range_fov(origin, 2, |hex| hex != origin);
    assert!(visible.contains(&origin));
}

#[test]
fn directional_fov_returns_subset_of_range_fov() {
    let origin = AxialHex::ZERO;
    let full = range_fov(origin, 4, |_| false);
    let cone = directional_fov(origin, 4, HexDirection::East, |_| false);

    // Cone should be a subset of the full FOV
    assert!(cone.is_subset(&full));
    // Cone should be smaller than the full FOV (roughly 1/3)
    assert!(cone.len() < full.len());
    // Origin always included
    assert!(cone.contains(&origin));
}

#[test]
fn directional_fov_faces_correct_direction() {
    let origin = AxialHex::ZERO;
    let cone = directional_fov(origin, 3, HexDirection::East, |_| false);

    // Hex at (3, 0) should be visible (directly east)
    assert!(cone.contains(&AxialHex::new(3, 0)));
    // Hex at (-3, 0) should NOT be visible (directly west, opposite direction)
    assert!(!cone.contains(&AxialHex::new(-3, 0)));
}

#[test]
fn diagonal_way_returns_single_for_diagonal_direction() {
    let from = AxialHex::ZERO;
    // (2, -1) is exactly ENE
    let to = AxialHex::new(2, -1);
    let way = diagonal_way(from, to);
    assert_eq!(
        way,
        DiagonalWay::Single(HexDiagonalDirection::EastNorthEast)
    );
}
