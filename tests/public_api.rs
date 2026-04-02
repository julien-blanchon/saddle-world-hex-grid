use saddle_world_hex_grid::{
    AxialHex, DoubledHexMode, HexLayout, HexOrientation, HexPath, OffsetHexMode, a_star,
    reachable_within,
};

#[test]
fn public_api_supports_pure_math_and_traversal() {
    let layout = HexLayout::new(HexOrientation::PointyTop)
        .with_uniform_size(20.0)
        .with_origin(bevy::math::Vec2::new(30.0, -10.0));
    let hex = AxialHex::new(2, -1);

    let world = layout.hex_to_world(hex);
    assert_eq!(layout.world_to_hex(world), hex);
    assert_eq!(
        hex.to_offset(OffsetHexMode::OddColumns)
            .to_axial(OffsetHexMode::OddColumns),
        hex
    );
    assert_eq!(
        hex.to_doubled(DoubledHexMode::DoubleWidth)
            .to_axial(DoubledHexMode::DoubleWidth),
        hex
    );

    let path: HexPath =
        a_star(AxialHex::ZERO, AxialHex::new(2, -1), |_, _| Some(1)).expect("expected path");
    assert_eq!(path.cells.first().copied(), Some(AxialHex::ZERO));

    let reachable = reachable_within(AxialHex::ZERO, 2, |_, _| Some(1));
    assert!(reachable.contains(AxialHex::new(0, 1)));
}
