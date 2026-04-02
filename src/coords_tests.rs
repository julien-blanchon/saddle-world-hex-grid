use super::*;

#[test]
fn cube_constructor_rejects_invalid_triplets() {
    assert!(CubeHex::new(1, 2, -3).is_ok());
    assert!(CubeHex::new(1, 2, -2).is_err());
}

#[test]
fn axial_cube_round_trip_preserves_coordinates() {
    for q in -6..=6 {
        for r in -6..=6 {
            let axial = AxialHex::new(q, r);
            let cube = axial.to_cube();
            assert_eq!(cube.q + cube.r + cube.s, 0);
            assert_eq!(AxialHex::from(cube), axial);
        }
    }
}

#[test]
fn offset_round_trips_for_all_modes() {
    let modes = [
        OffsetHexMode::OddColumns,
        OffsetHexMode::EvenColumns,
        OffsetHexMode::OddRows,
        OffsetHexMode::EvenRows,
    ];

    for mode in modes {
        for q in -6..=6 {
            for r in -6..=6 {
                let axial = AxialHex::new(q, r);
                let offset = axial.to_offset(mode);
                assert_eq!(offset.to_axial(mode), axial);
            }
        }
    }
}

#[test]
fn doubled_round_trips_for_both_modes() {
    let modes = [DoubledHexMode::DoubleWidth, DoubledHexMode::DoubleHeight];
    for mode in modes {
        for q in -6..=6 {
            for r in -6..=6 {
                let axial = AxialHex::new(q, r);
                let doubled = axial.to_doubled(mode);
                assert_eq!(doubled.to_axial(mode), axial);
                assert_eq!((doubled.col + doubled.row) % 2, 0);
            }
        }
    }
}

#[test]
fn doubled_constructor_rejects_invalid_parity() {
    assert!(DoubledHex::new(2, 0).is_ok());
    assert!(DoubledHex::new(1, 0).is_err());
}

#[test]
fn rounded_fractional_hex_preserves_invariant() {
    let rounded = FractionalHex::new(2.2, -1.7, -0.5).round();
    let cube = rounded.to_cube();
    assert_eq!(cube.q + cube.r + cube.s, 0);
}

#[test]
fn rotations_and_reflections_preserve_hex_invariants() {
    let center = AxialHex::new(3, -2);
    let sample = AxialHex::new(2, -1);

    for steps in -8..=8 {
        let rotated = sample.rotate_cw_around(center, steps);
        assert_eq!(center.distance_to(rotated), center.distance_to(sample));
        let cube = rotated.to_cube();
        assert_eq!(cube.q + cube.r + cube.s, 0);
    }

    for reflected in [sample.reflect_q(), sample.reflect_r(), sample.reflect_s()] {
        let cube = reflected.to_cube();
        assert_eq!(cube.q + cube.r + cube.s, 0);
        assert_eq!(reflected.length(), sample.length());
    }
}
