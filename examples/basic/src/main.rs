use saddle_world_hex_grid_example_support as support;

use bevy::prelude::*;
use saddle_world_hex_grid::{AxialHex, HexLayout};
use support::{BoardKind, DemoBoard, DemoHexCell, OverlayText};

#[derive(Resource)]
struct BasicDemo {
    board: DemoBoard,
    default_material: Handle<ColorMaterial>,
    hovered_material: Handle<ColorMaterial>,
    neighbor_material: Handle<ColorMaterial>,
    hovered_hex: Option<AxialHex>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "hex_grid basic".into(),
                resolution: (1200, 860).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, update_hover)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    support::spawn_camera(&mut commands, "Basic Camera");
    support::spawn_overlay(
        &mut commands,
        "Move the cursor across the board.\nThe hovered cell turns gold and its six neighbors turn mint.",
    );

    let layout = HexLayout::flat()
        .with_uniform_size(32.0)
        .with_origin(Vec2::new(0.0, -10.0));
    let coords: Vec<_> = AxialHex::ZERO.hexagon(4).collect();

    let default_material = materials.add(Color::srgb(0.16, 0.20, 0.25));
    let hovered_material = materials.add(Color::srgb(0.92, 0.76, 0.24));
    let neighbor_material = materials.add(Color::srgb(0.25, 0.74, 0.63));
    let board = support::spawn_demo_board(
        &mut commands,
        &mut meshes,
        &mut materials,
        BoardKind::Primary,
        "Basic Hover",
        layout,
        coords,
        Color::srgb(0.16, 0.20, 0.25),
    );

    commands.insert_resource(BasicDemo {
        board,
        default_material,
        hovered_material,
        neighbor_material,
        hovered_hex: None,
    });
}

fn update_hover(
    windows: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
    mut demo: ResMut<BasicDemo>,
    mut text: Single<&mut Text, With<OverlayText>>,
    mut cells: Query<(&DemoHexCell, &mut MeshMaterial2d<ColorMaterial>)>,
) {
    let (camera, camera_transform) = *camera;
    let cursor_world = support::cursor_world(windows.into_inner(), camera, camera_transform);
    let hovered = cursor_world
        .map(|world| (world, demo.board.layout.world_to_hex(world)))
        .filter(|(_, hex)| demo.board.cells.contains_key(hex));

    for (cell, mut material) in &mut cells {
        if cell.board != BoardKind::Primary {
            continue;
        }

        let next = if let Some((_, hovered_hex)) = hovered {
            if cell.hex == hovered_hex {
                demo.hovered_material.clone()
            } else if hovered_hex.neighbors().contains(&cell.hex) {
                demo.neighbor_material.clone()
            } else {
                demo.default_material.clone()
            }
        } else {
            demo.default_material.clone()
        };
        material.0 = next;
    }

    if let Some((world, hex)) = hovered {
        text.0 = format!(
            "Move the cursor across the board.\nHovered hex: ({}, {})\nWorld position: ({:.1}, {:.1})\nHex center: ({:.1}, {:.1})",
            hex.q,
            hex.r,
            world.x,
            world.y,
            demo.board.layout.hex_to_world(hex).x,
            demo.board.layout.hex_to_world(hex).y,
        );
        demo.hovered_hex = Some(hex);
    } else {
        text.0 = "Move the cursor across the board.\nHovered hex: outside the board".to_string();
        demo.hovered_hex = None;
    }
}
