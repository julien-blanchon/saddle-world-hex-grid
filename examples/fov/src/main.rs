use saddle_world_hex_grid_example_support as support;

use bevy::{
    input::{ButtonState, mouse::MouseButtonInput},
    prelude::*,
};
use saddle_pane::prelude::*;
use saddle_world_hex_grid::{AxialHex, HexDirection, HexLayout, directional_fov, range_fov};
use std::collections::HashSet;
use support::{BoardKind, DemoBoard, DemoHexCell, OverlayText};

#[derive(Resource, Pane)]
#[pane(title = "FOV Controls")]
struct FovPane {
    #[pane(slider, min = 16.0, max = 44.0, step = 1.0)]
    hex_size: f32,
    #[pane(slider, min = 1.0, max = 8.0, step = 1.0)]
    fov_range: f32,
    #[pane(toggle)]
    directional_mode: bool,
}

impl Default for FovPane {
    fn default() -> Self {
        Self {
            hex_size: 28.0,
            fov_range: 5.0,
            directional_mode: false,
        }
    }
}

#[derive(Resource)]
struct FovDemo {
    board: DemoBoard,
    walls: HashSet<AxialHex>,
    viewer: AxialHex,
    facing: HexDirection,
    default_material: Handle<ColorMaterial>,
    wall_material: Handle<ColorMaterial>,
    visible_material: Handle<ColorMaterial>,
    viewer_material: Handle<ColorMaterial>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "hex_grid field of view".into(),
                resolution: (1260, 860).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(support::pane_plugins())
        .register_pane::<FovPane>()
        .add_systems(Startup, setup)
        .add_systems(Update, (sync_pane, handle_input, update_fov))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    support::spawn_camera(&mut commands, "FOV Camera");
    support::spawn_overlay(
        &mut commands,
        "Left click to toggle walls.\nRight click to move the viewer.\nQ/E to rotate facing direction.\nToggle directional mode in the pane.",
    );

    let layout = HexLayout::pointy()
        .with_uniform_size(28.0)
        .with_origin(Vec2::new(0.0, -10.0));
    let coords: Vec<_> = AxialHex::ZERO.hexagon(6).collect();

    let board = support::spawn_demo_board(
        &mut commands,
        &mut meshes,
        &mut materials,
        BoardKind::Primary,
        "Field of View",
        layout,
        coords,
        Color::srgb(0.12, 0.14, 0.18),
    );

    // Pre-place some walls
    let walls = HashSet::from([
        AxialHex::new(2, 0),
        AxialHex::new(2, -1),
        AxialHex::new(2, -2),
        AxialHex::new(-1, 3),
        AxialHex::new(0, 3),
        AxialHex::new(1, 2),
        AxialHex::new(-3, 1),
        AxialHex::new(-3, 2),
    ]);

    commands.insert_resource(FovDemo {
        board,
        walls,
        viewer: AxialHex::ZERO,
        facing: HexDirection::East,
        default_material: materials.add(Color::srgb(0.12, 0.14, 0.18)),
        wall_material: materials.add(Color::srgb(0.35, 0.15, 0.15)),
        visible_material: materials.add(Color::srgb(0.30, 0.68, 0.86)),
        viewer_material: materials.add(Color::srgb(0.92, 0.76, 0.24)),
    });
}

fn handle_input(
    mut clicks: MessageReader<MouseButtonInput>,
    keys: Res<ButtonInput<KeyCode>>,
    windows: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
    mut demo: ResMut<FovDemo>,
) {
    let (camera, camera_transform) = *camera;
    let hovered = support::cursor_world(windows.into_inner(), camera, camera_transform)
        .map(|world| demo.board.layout.world_to_hex(world))
        .filter(|hex| demo.board.cells.contains_key(hex));

    for click in clicks.read() {
        if click.state != ButtonState::Pressed {
            continue;
        }
        let Some(hex) = hovered else { continue };

        match click.button {
            MouseButton::Left => {
                if hex != demo.viewer && !demo.walls.insert(hex) {
                    demo.walls.remove(&hex);
                }
            }
            MouseButton::Right => {
                demo.walls.remove(&hex);
                demo.viewer = hex;
            }
            _ => {}
        }
    }

    if keys.just_pressed(KeyCode::KeyE) {
        demo.facing = demo.facing.rotate_cw(1);
    }
    if keys.just_pressed(KeyCode::KeyQ) {
        demo.facing = demo.facing.rotate_ccw(1);
    }
}

fn update_fov(
    pane: Res<FovPane>,
    demo: Res<FovDemo>,
    mut overlay: Single<&mut Text, With<OverlayText>>,
    mut cells: Query<(&DemoHexCell, &mut MeshMaterial2d<ColorMaterial>)>,
    mut gizmos: Gizmos,
) {
    let fov_range = pane.fov_range.round() as u32;
    let walls = &demo.walls;
    let is_blocking = |hex: AxialHex| walls.contains(&hex);

    let visible: HashSet<AxialHex> = if pane.directional_mode {
        directional_fov(demo.viewer, fov_range, demo.facing, is_blocking)
    } else {
        range_fov(demo.viewer, fov_range, is_blocking)
    };

    for (cell, mut material) in &mut cells {
        if cell.board != BoardKind::Primary {
            continue;
        }

        let next = if cell.hex == demo.viewer {
            demo.viewer_material.clone()
        } else if demo.walls.contains(&cell.hex) {
            if visible.contains(&cell.hex) {
                // Visible wall — use wall material but it's been "seen"
                demo.wall_material.clone()
            } else {
                demo.wall_material.clone()
            }
        } else if visible.contains(&cell.hex) {
            demo.visible_material.clone()
        } else {
            demo.default_material.clone()
        };
        material.0 = next;
    }

    // Draw facing direction as an arrow from viewer
    let viewer_world = demo.board.layout.hex_to_world(demo.viewer);
    let facing_target = demo.viewer.neighbor(demo.facing);
    let target_world = demo.board.layout.hex_to_world(facing_target);
    let dir = (target_world - viewer_world).normalize_or_zero();
    let arrow_end = viewer_world + dir * demo.board.layout.hex_size.x * 1.5;
    gizmos.line_2d(viewer_world, arrow_end, Color::srgb(1.0, 0.9, 0.3));

    let mode = if pane.directional_mode {
        format!("Directional ({:?})", demo.facing)
    } else {
        "360° Range".to_string()
    };

    overlay.0 = format!(
        "Left click to toggle walls. Right click to move viewer.\nQ/E to rotate facing direction.\nMode: {}  Range: {}  Visible: {}  Walls: {}",
        mode,
        fov_range,
        visible.len(),
        demo.walls.len(),
    );
}

fn sync_pane(
    pane: Res<FovPane>,
    mut demo: ResMut<FovDemo>,
    mut transforms: Query<&mut Transform>,
) {
    support::apply_hex_size(&mut demo.board, pane.hex_size, &mut transforms);
}
