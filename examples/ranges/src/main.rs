use saddle_world_hex_grid_example_support as support;

use bevy::prelude::*;
use saddle_world_hex_grid::{AxialHex, HexLayout, reachable_within};
use std::collections::HashSet;
use support::{BoardKind, DemoBoard, DemoHexCell, OverlayText};

#[derive(Resource)]
struct RangeDemo {
    board: DemoBoard,
    default_material: Handle<ColorMaterial>,
    range_material: Handle<ColorMaterial>,
    ring_material: Handle<ColorMaterial>,
    center_material: Handle<ColorMaterial>,
    weighted_cells: HashSet<AxialHex>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "hex_grid ranges".into(),
                resolution: (1260, 860).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, update_ranges)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    support::spawn_camera(&mut commands, "Ranges Camera");
    support::spawn_overlay(
        &mut commands,
        "Blue cells are reachable within budget 5.\nAmber cells form the exact radius-3 ring.\nThe green line shows the spiral traversal order.",
    );

    let board = support::spawn_demo_board(
        &mut commands,
        &mut meshes,
        &mut materials,
        BoardKind::Range,
        "Ranges And Spiral",
        HexLayout::flat()
            .with_uniform_size(32.0)
            .with_origin(Vec2::new(0.0, -20.0)),
        AxialHex::ZERO.hexagon(5),
        Color::srgb(0.16, 0.20, 0.24),
    );

    commands.insert_resource(RangeDemo {
        board,
        default_material: materials.add(Color::srgb(0.16, 0.20, 0.24)),
        range_material: materials.add(Color::srgb(0.25, 0.67, 0.93)),
        ring_material: materials.add(Color::srgb(0.94, 0.71, 0.24)),
        center_material: materials.add(Color::srgb(0.23, 0.90, 0.57)),
        weighted_cells: HashSet::from([
            AxialHex::new(1, 0),
            AxialHex::new(2, -1),
            AxialHex::new(-1, 1),
        ]),
    });
}

fn update_ranges(
    demo: Res<RangeDemo>,
    mut overlay: Single<&mut Text, With<OverlayText>>,
    mut cells: Query<(&DemoHexCell, &mut MeshMaterial2d<ColorMaterial>)>,
    mut gizmos: Gizmos,
) {
    let center = AxialHex::ZERO;
    let ring: HashSet<_> = center.ring(3).collect();
    let reachable = reachable_within(center, 5, |_, to| {
        if !demo.board.cells.contains_key(&to) {
            None
        } else if demo.weighted_cells.contains(&to) {
            Some(2)
        } else {
            Some(1)
        }
    });
    let reachable_cells: HashSet<_> = reachable.iter().map(|(hex, _)| hex).collect();
    let spiral: Vec<_> = center.spiral(3).collect();

    for (cell, mut material) in &mut cells {
        if cell.board != BoardKind::Range {
            continue;
        }

        let next = if cell.hex == center {
            demo.center_material.clone()
        } else if ring.contains(&cell.hex) {
            demo.ring_material.clone()
        } else if reachable_cells.contains(&cell.hex) {
            demo.range_material.clone()
        } else {
            demo.default_material.clone()
        };
        material.0 = next;
    }

    for pair in spiral.windows(2) {
        let a = demo.board.layout.hex_to_world(pair[0]);
        let b = demo.board.layout.hex_to_world(pair[1]);
        gizmos.line_2d(a, b, Color::srgb(0.28, 0.94, 0.64));
    }

    overlay.0 = format!(
        "Blue cells are reachable within budget 5.\nReachable: {}  Ring size: {}  Spiral length: {}\nWeighted cells: {:?}",
        reachable.len(),
        ring.len(),
        spiral.len(),
        demo.weighted_cells,
    );
}
