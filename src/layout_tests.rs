use super::*;

#[test]
fn flat_layout_world_round_trip_matches_original_hexes() {
    let layout = HexLayout::flat()
        .with_uniform_size(24.0)
        .with_origin(Vec2::new(120.0, -70.0));

    for q in -4..=4 {
        for r in -4..=4 {
            let hex = AxialHex::new(q, r);
            let world = layout.hex_to_world(hex);
            assert_eq!(layout.world_to_hex(world), hex);
        }
    }
}

#[test]
fn pointy_layout_world_round_trip_matches_original_hexes() {
    let layout = HexLayout::pointy()
        .with_hex_size(Vec2::new(18.0, 26.0))
        .with_origin(Vec2::new(-30.0, 80.0));

    for q in -4..=4 {
        for r in -4..=4 {
            let hex = AxialHex::new(q, r);
            let world = layout.hex_to_world(hex);
            assert_eq!(layout.world_to_hex(world), hex);
        }
    }
}

#[test]
fn corner_count_and_edge_midpoints_are_stable() {
    let layout = HexLayout::flat().with_uniform_size(10.0);
    let corners = layout.corners(AxialHex::new(2, -1));
    let edge_midpoints = layout.edge_midpoints(AxialHex::new(2, -1));

    assert_eq!(corners.len(), 6);
    assert_eq!(edge_midpoints.len(), 6);
}

#[test]
fn interior_points_round_back_to_their_source_hex() {
    let layouts = [
        HexLayout::flat()
            .with_uniform_size(18.0)
            .with_origin(Vec2::new(-20.0, 15.0)),
        HexLayout::pointy()
            .with_hex_size(Vec2::new(16.0, 24.0))
            .with_origin(Vec2::new(42.0, -30.0)),
    ];

    for layout in layouts {
        for hex in [AxialHex::ZERO, AxialHex::new(2, -1), AxialHex::new(-3, 2)] {
            let center = layout.hex_to_world(hex);
            for midpoint in layout.edge_midpoints(hex) {
                let interior = center + (midpoint - center) * 0.45;
                assert_eq!(layout.world_to_hex(interior), hex);
            }
        }
    }
}
