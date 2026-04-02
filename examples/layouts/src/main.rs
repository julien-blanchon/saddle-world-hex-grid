use saddle_world_hex_grid_example_support as support;

use bevy::prelude::*;
use saddle_world_hex_grid::{AxialHex, HexLayout};
use support::{BoardKind, DemoBoard, DemoHexCell, OverlayText};

#[derive(Component)]
struct LayoutMarker {
    board: BoardKind,
}

#[derive(Resource)]
struct LayoutsDemo {
    flat: DemoBoard,
    pointy: DemoBoard,
    sample_local: Vec2,
    default_material: Handle<ColorMaterial>,
    highlight_material: Handle<ColorMaterial>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "hex_grid layouts".into(),
                resolution: (1260, 860).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, animate_layouts)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    support::spawn_camera(&mut commands, "Layouts Camera");
    support::spawn_overlay(
        &mut commands,
        "Two boards use the same local sample offset.\nThe highlighted cell changes because flat-top and pointy-top layouts project world space differently.",
    );

    let coords: Vec<_> = AxialHex::ZERO.hexagon(3).collect();
    let flat = support::spawn_demo_board(
        &mut commands,
        &mut meshes,
        &mut materials,
        BoardKind::Flat,
        "Flat Top",
        HexLayout::flat()
            .with_uniform_size(32.0)
            .with_origin(Vec2::new(-280.0, -40.0)),
        coords.iter().copied(),
        Color::srgb(0.17, 0.21, 0.27),
    );
    let pointy = support::spawn_demo_board(
        &mut commands,
        &mut meshes,
        &mut materials,
        BoardKind::Pointy,
        "Pointy Top",
        HexLayout::pointy()
            .with_uniform_size(32.0)
            .with_origin(Vec2::new(280.0, -40.0)),
        coords,
        Color::srgb(0.17, 0.21, 0.27),
    );

    let marker_mesh = meshes.add(Circle::new(6.0));
    let marker_material = materials.add(Color::WHITE);
    for board in [BoardKind::Flat, BoardKind::Pointy] {
        commands.spawn((
            Name::new(match board {
                BoardKind::Flat => "Flat Marker",
                BoardKind::Pointy => "Pointy Marker",
                _ => "Marker",
            }),
            LayoutMarker { board },
            Mesh2d(marker_mesh.clone()),
            MeshMaterial2d(marker_material.clone()),
            Transform::default(),
        ));
    }

    commands.insert_resource(LayoutsDemo {
        flat,
        pointy,
        sample_local: Vec2::new(18.0, 10.0),
        default_material: materials.add(Color::srgb(0.17, 0.21, 0.27)),
        highlight_material: materials.add(Color::srgb(0.91, 0.47, 0.31)),
    });
}

fn animate_layouts(
    time: Res<Time>,
    demo: Res<LayoutsDemo>,
    mut markers: Query<(&LayoutMarker, &mut Transform)>,
    mut cells: Query<(&DemoHexCell, &mut MeshMaterial2d<ColorMaterial>)>,
    mut overlay: Single<&mut Text, With<OverlayText>>,
) {
    let local = Vec2::new(
        ops::sin(time.elapsed_secs() * 0.8) * 58.0,
        ops::cos(time.elapsed_secs() * 1.1) * 44.0,
    ) + demo.sample_local;

    let flat_world = demo.flat.layout.origin + local;
    let pointy_world = demo.pointy.layout.origin + local;
    let flat_hex = demo.flat.layout.world_to_hex(flat_world);
    let pointy_hex = demo.pointy.layout.world_to_hex(pointy_world);

    for (marker, mut transform) in &mut markers {
        transform.translation = match marker.board {
            BoardKind::Flat => flat_world.extend(1.0),
            BoardKind::Pointy => pointy_world.extend(1.0),
            _ => Vec3::ZERO,
        };
    }

    for (cell, mut material) in &mut cells {
        let next = match cell.board {
            BoardKind::Flat if cell.hex == flat_hex => demo.highlight_material.clone(),
            BoardKind::Pointy if cell.hex == pointy_hex => demo.highlight_material.clone(),
            BoardKind::Flat | BoardKind::Pointy => demo.default_material.clone(),
            _ => continue,
        };
        material.0 = next;
    }

    overlay.0 = format!(
        "Two boards use the same local sample offset.\nLocal sample: ({:.1}, {:.1})\nFlat-top -> ({}, {})\nPointy-top -> ({}, {})",
        local.x, local.y, flat_hex.q, flat_hex.r, pointy_hex.q, pointy_hex.r,
    );
}
