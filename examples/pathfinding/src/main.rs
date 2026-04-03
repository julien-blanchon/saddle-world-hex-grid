use saddle_world_hex_grid_example_support as support;

use bevy::{
    input::{ButtonState, mouse::MouseButtonInput},
    prelude::*,
};
use saddle_pane::prelude::*;
use saddle_world_hex_grid::{AxialHex, HexLayout, HexPath, a_star};
use std::collections::HashSet;
use support::{BoardKind, DemoBoard, DemoHexCell, OverlayText};

#[derive(Resource)]
struct PathfindingDemo {
    board: DemoBoard,
    start: AxialHex,
    hovered: Option<AxialHex>,
    blocked: HashSet<AxialHex>,
    weighted: HashSet<AxialHex>,
    default_material: Handle<ColorMaterial>,
    blocked_material: Handle<ColorMaterial>,
    weighted_material: Handle<ColorMaterial>,
    path_material: Handle<ColorMaterial>,
    start_material: Handle<ColorMaterial>,
    hover_material: Handle<ColorMaterial>,
}

fn main() {
    App::new()
        .insert_resource(support::HexExamplePane { hex_size: 28.0 })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "hex_grid pathfinding".into(),
                resolution: (1260, 860).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(support::pane_plugins())
        .register_pane::<support::HexExamplePane>()
        .add_systems(Startup, setup)
        .add_systems(Update, (sync_pane, update_hover, handle_clicks, repaint_path))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    support::spawn_camera(&mut commands, "Pathfinding Camera");
    support::spawn_overlay(
        &mut commands,
        "Move the cursor to preview a path.\nLeft click toggles blockers. Right click toggles weighted terrain (cost 3).",
    );

    let board = support::spawn_demo_board(
        &mut commands,
        &mut meshes,
        &mut materials,
        BoardKind::Path,
        "A* Pathfinding",
        HexLayout::pointy()
            .with_uniform_size(28.0)
            .with_origin(Vec2::new(0.0, -10.0)),
        AxialHex::ZERO.hexagon(5),
        Color::srgb(0.16, 0.20, 0.24),
    );

    commands.insert_resource(PathfindingDemo {
        board,
        start: AxialHex::ZERO,
        hovered: None,
        blocked: HashSet::from([
            AxialHex::new(1, 0),
            AxialHex::new(1, -1),
            AxialHex::new(2, -1),
        ]),
        weighted: HashSet::from([
            AxialHex::new(-1, 1),
            AxialHex::new(-2, 2),
            AxialHex::new(0, 2),
        ]),
        default_material: materials.add(Color::srgb(0.16, 0.20, 0.24)),
        blocked_material: materials.add(Color::srgb(0.20, 0.07, 0.09)),
        weighted_material: materials.add(Color::srgb(0.63, 0.44, 0.18)),
        path_material: materials.add(Color::srgb(0.26, 0.86, 0.55)),
        start_material: materials.add(Color::srgb(0.24, 0.74, 0.94)),
        hover_material: materials.add(Color::srgb(0.94, 0.74, 0.26)),
    });
}

fn update_hover(
    windows: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
    mut demo: ResMut<PathfindingDemo>,
) {
    let (camera, camera_transform) = *camera;
    demo.hovered = support::cursor_world(windows.into_inner(), camera, camera_transform)
        .map(|world| demo.board.layout.world_to_hex(world))
        .filter(|hex| demo.board.cells.contains_key(hex));
}

fn handle_clicks(mut clicks: MessageReader<MouseButtonInput>, mut demo: ResMut<PathfindingDemo>) {
    let hovered = demo.hovered;
    for click in clicks.read() {
        if click.state != ButtonState::Pressed {
            continue;
        }

        let Some(hex) = hovered else {
            continue;
        };
        if hex == demo.start {
            continue;
        }

        match click.button {
            MouseButton::Left => {
                if !demo.blocked.insert(hex) {
                    demo.blocked.remove(&hex);
                }
                demo.weighted.remove(&hex);
            }
            MouseButton::Right => {
                if !demo.weighted.insert(hex) {
                    demo.weighted.remove(&hex);
                }
                demo.blocked.remove(&hex);
            }
            _ => {}
        }
    }
}

fn repaint_path(
    demo: Res<PathfindingDemo>,
    mut overlay: Single<&mut Text, With<OverlayText>>,
    mut cells: Query<(&DemoHexCell, &mut MeshMaterial2d<ColorMaterial>)>,
) {
    let path = demo.hovered.and_then(|goal| current_path(&demo, goal));
    let path_cells: HashSet<_> = path
        .as_ref()
        .map(|path| path.cells.iter().copied().collect())
        .unwrap_or_default();

    for (cell, mut material) in &mut cells {
        if cell.board != BoardKind::Path {
            continue;
        }

        let next = if cell.hex == demo.start {
            demo.start_material.clone()
        } else if Some(cell.hex) == demo.hovered {
            demo.hover_material.clone()
        } else if demo.blocked.contains(&cell.hex) {
            demo.blocked_material.clone()
        } else if path_cells.contains(&cell.hex) {
            demo.path_material.clone()
        } else if demo.weighted.contains(&cell.hex) {
            demo.weighted_material.clone()
        } else {
            demo.default_material.clone()
        };
        material.0 = next;
    }

    overlay.0 = if let Some(goal) = demo.hovered {
        if let Some(path) = path {
            format!(
                "Move the cursor to preview a path.\nLeft click toggles blockers. Right click toggles weighted terrain (cost 3).\nGoal: ({}, {})  Path length: {}  Total cost: {}",
                goal.q,
                goal.r,
                path.cells.len(),
                path.total_cost,
            )
        } else {
            format!(
                "Move the cursor to preview a path.\nGoal: ({}, {}) is unreachable with the current blockers.",
                goal.q, goal.r,
            )
        }
    } else {
        "Move the cursor to preview a path.\nLeft click toggles blockers. Right click toggles weighted terrain (cost 3)."
            .to_string()
    };
}

fn current_path(demo: &PathfindingDemo, goal: AxialHex) -> Option<HexPath> {
    a_star(demo.start, goal, |_, to| {
        if !demo.board.cells.contains_key(&to) || demo.blocked.contains(&to) {
            None
        } else if demo.weighted.contains(&to) {
            Some(3)
        } else {
            Some(1)
        }
    })
}

fn sync_pane(
    pane: Res<support::HexExamplePane>,
    mut demo: ResMut<PathfindingDemo>,
    mut transforms: Query<&mut Transform>,
) {
    support::apply_hex_size(&mut demo.board, pane.hex_size, &mut transforms);
}
