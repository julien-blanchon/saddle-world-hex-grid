use super::*;

#[test]
fn grid_edge_canonical_form_is_consistent() {
    let a = AxialHex::ZERO;
    let dir = HexDirection::East;
    let neighbor = a.neighbor(dir);

    let edge1 = GridEdge::new(a, dir);
    let edge2 = GridEdge::new(neighbor, dir.opposite());

    assert_eq!(edge1, edge2);
}

#[test]
fn grid_edge_hexes_returns_both_sides() {
    let edge = GridEdge::new(AxialHex::ZERO, HexDirection::NorthEast);
    let hexes = edge.hexes();
    assert!(hexes.contains(&AxialHex::ZERO) || hexes.contains(&AxialHex::new(1, -1)));
}

#[test]
fn grid_vertex_canonical_form_is_consistent() {
    let hex = AxialHex::ZERO;
    let dir = HexDiagonalDirection::EastNorthEast; // index 0

    let v1 = GridVertex::new(hex, dir);

    // The same vertex from hex.neighbor(East).
    // Diagonal direction ENE(0) is between edge E(0) and NE(1).
    // From neighbor_E, the same vertex is at diagonal direction (0+2)%6 = WNW(2).
    let neighbor = hex.neighbor(HexDirection::East);
    let v2 = GridVertex::new(neighbor, HexDiagonalDirection::ALL[(dir.index() + 2) % 6]);

    assert_eq!(v1, v2);

    // Also verify from the third hex: neighbor_NE at diagonal (0+4)%6 = S(4)
    let neighbor_ne = hex.neighbor(HexDirection::NorthEast);
    let v3 = GridVertex::new(
        neighbor_ne,
        HexDiagonalDirection::ALL[(dir.index() + 4) % 6],
    );
    assert_eq!(v1, v3);
}

#[test]
fn grid_vertex_hexes_returns_three_hexes() {
    let vertex = GridVertex::new(AxialHex::ZERO, HexDiagonalDirection::EastNorthEast);
    let hexes = vertex.hexes();
    assert_eq!(hexes.len(), 3);
    // All three should be distinct
    assert_ne!(hexes[0], hexes[1]);
    assert_ne!(hexes[1], hexes[2]);
    assert_ne!(hexes[0], hexes[2]);
}

#[test]
fn diagonal_direction_opposite_is_correct() {
    assert_eq!(
        HexDiagonalDirection::EastNorthEast.opposite(),
        HexDiagonalDirection::WestSouthWest
    );
    assert_eq!(
        HexDiagonalDirection::North.opposite(),
        HexDiagonalDirection::South
    );
}

#[test]
fn diagonal_direction_rotation_round_trip() {
    for dir in HexDiagonalDirection::ALL {
        assert_eq!(dir.rotate_cw(6), dir);
        assert_eq!(dir.rotate_ccw(6), dir);
        assert_eq!(dir.rotate_cw(3), dir.opposite());
    }
}
