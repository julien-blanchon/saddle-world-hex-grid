use saddle_world_hex_grid_example_support as support;

use bevy::{
    input::{ButtonState, mouse::MouseButtonInput},
    prelude::*,
};
use saddle_pane::prelude::*;
use saddle_world_hex_grid::{AxialHex, HexLayout, HexPath, HexReachability, reachable_within};
use std::collections::{HashMap, HashSet};
use support::{BoardKind, DemoBoard, DemoHexCell, OverlayText};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Terrain {
    Plains,
    Forest,
    Mountain,
    Water,
}

impl Terrain {
    fn move_cost(self) -> Option<u32> {
        match self {
            Terrain::Plains => Some(1),
            Terrain::Forest => Some(2),
            Terrain::Mountain => Some(3),
            Terrain::Water => None,
        }
    }

    fn color(self) -> Color {
        match self {
            Terrain::Plains => Color::srgb(0.30, 0.55, 0.28),
            Terrain::Forest => Color::srgb(0.15, 0.35, 0.15),
            Terrain::Mountain => Color::srgb(0.45, 0.40, 0.35),
            Terrain::Water => Color::srgb(0.15, 0.30, 0.60),
        }
    }
}

#[derive(Resource, Pane)]
#[pane(title = "Strategy Controls")]
struct StrategyPane {
    #[pane(slider, min = 18.0, max = 40.0, step = 1.0)]
    hex_size: f32,
    #[pane(slider, min = 1.0, max = 10.0, step = 1.0)]
    move_budget: f32,
}

impl Default for StrategyPane {
    fn default() -> Self {
        Self {
            hex_size: 26.0,
            move_budget: 4.0,
        }
    }
}

#[derive(Resource)]
struct StrategyDemo {
    board: DemoBoard,
    terrain: HashMap<AxialHex, Terrain>,
    unit_pos: AxialHex,
    hovered: Option<AxialHex>,
    reachable: Option<HexReachability>,
    preview_path: Option<HexPath>,
    terrain_materials: HashMap<Terrain, Handle<ColorMaterial>>,
    unit_material: Handle<ColorMaterial>,
    reachable_material: Handle<ColorMaterial>,
    path_material: Handle<ColorMaterial>,
    attack_material: Handle<ColorMaterial>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "hex_grid strategy board".into(),
                resolution: (1300, 900).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(support::pane_plugins())
        .register_pane::<StrategyPane>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (sync_pane, update_hover, handle_clicks, update_visuals),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    support::spawn_camera(&mut commands, "Strategy Camera");
    support::spawn_overlay(
        &mut commands,
        "Strategy board with terrain types and movement ranges.\nHover to preview path. Left click to move unit.\nRight click to cycle terrain type.\nBlue = reachable, green = path, red ring = attack range.",
    );

    let layout = HexLayout::pointy()
        .with_uniform_size(26.0)
        .with_origin(Vec2::new(0.0, -20.0));
    let coords: Vec<_> = AxialHex::ZERO.hexagon(5).collect();

    // Assign terrain
    let mut terrain = HashMap::new();
    for &hex in &coords {
        let t = match (hex.q.abs() + hex.r.abs()) % 7 {
            0..=2 => Terrain::Plains,
            3 | 4 => Terrain::Forest,
            5 => Terrain::Mountain,
            _ => Terrain::Water,
        };
        terrain.insert(hex, t);
    }
    // Ensure origin is always plains
    terrain.insert(AxialHex::ZERO, Terrain::Plains);

    let board = support::spawn_demo_board(
        &mut commands,
        &mut meshes,
        &mut materials,
        BoardKind::Primary,
        "Strategy Board",
        layout,
        coords,
        Terrain::Plains.color(),
    );

    let mut terrain_materials = HashMap::new();
    for t in [
        Terrain::Plains,
        Terrain::Forest,
        Terrain::Mountain,
        Terrain::Water,
    ] {
        terrain_materials.insert(t, materials.add(t.color()));
    }

    commands.insert_resource(StrategyDemo {
        board,
        terrain,
        unit_pos: AxialHex::ZERO,
        hovered: None,
        reachable: None,
        preview_path: None,
        terrain_materials,
        unit_material: materials.add(Color::srgb(0.92, 0.78, 0.24)),
        reachable_material: materials.add(Color::srgba(0.25, 0.55, 0.90, 0.80)),
        path_material: materials.add(Color::srgb(0.30, 0.90, 0.50)),
        attack_material: materials.add(Color::srgba(0.90, 0.30, 0.25, 0.60)),
    });
}

fn update_hover(
    windows: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
    mut demo: ResMut<StrategyDemo>,
) {
    let (camera, camera_transform) = *camera;
    demo.hovered = support::cursor_world(windows.into_inner(), camera, camera_transform)
        .map(|world| demo.board.layout.world_to_hex(world))
        .filter(|hex| demo.board.cells.contains_key(hex));
}

fn handle_clicks(mut clicks: MessageReader<MouseButtonInput>, mut demo: ResMut<StrategyDemo>) {
    let hovered = demo.hovered;
    for click in clicks.read() {
        if click.state != ButtonState::Pressed {
            continue;
        }
        let Some(hex) = hovered else { continue };

        match click.button {
            MouseButton::Left => {
                // Move unit if reachable
                if let Some(ref reachable) = demo.reachable
                    && reachable.contains(hex)
                {
                    demo.unit_pos = hex;
                    demo.reachable = None;
                }
            }
            MouseButton::Right => {
                // Cycle terrain
                if let Some(t) = demo.terrain.get_mut(&hex) {
                    *t = match *t {
                        Terrain::Plains => Terrain::Forest,
                        Terrain::Forest => Terrain::Mountain,
                        Terrain::Mountain => Terrain::Water,
                        Terrain::Water => Terrain::Plains,
                    };
                }
                demo.reachable = None; // Invalidate
            }
            _ => {}
        }
    }
}

fn update_visuals(
    pane: Res<StrategyPane>,
    mut demo: ResMut<StrategyDemo>,
    mut overlay: Single<&mut Text, With<OverlayText>>,
    mut cells: Query<(&DemoHexCell, &mut MeshMaterial2d<ColorMaterial>)>,
    mut gizmos: Gizmos,
) {
    let budget = pane.move_budget.round() as u32;
    let terrain = &demo.terrain;

    // Compute reachable if not cached
    if demo.reachable.is_none() {
        let unit_pos = demo.unit_pos;
        demo.reachable = Some(reachable_within(unit_pos, budget, |_, to| {
            terrain.get(&to).and_then(|t| t.move_cost())
        }));
    }
    let reachable = demo.reachable.as_ref().unwrap();

    // Compute path preview to hovered cell
    let preview_path = demo.hovered.and_then(|goal| {
        if !reachable.contains(goal) {
            return None;
        }
        reachable.path_to(goal)
    });
    let path_cells: HashSet<_> = preview_path
        .as_ref()
        .map(|p| p.cells.iter().copied().collect())
        .unwrap_or_default();

    // Compute attack range (reachable + 1 ring of neighbors)
    let reachable_set: HashSet<_> = reachable.iter().map(|(hex, _)| hex).collect();
    let attack_range: HashSet<_> = reachable_set
        .iter()
        .flat_map(|hex| hex.neighbors())
        .filter(|hex| !reachable_set.contains(hex) && demo.board.cells.contains_key(hex))
        .collect();

    for (cell, mut material) in &mut cells {
        if cell.board != BoardKind::Primary {
            continue;
        }

        let next = if cell.hex == demo.unit_pos {
            demo.unit_material.clone()
        } else if path_cells.contains(&cell.hex) {
            demo.path_material.clone()
        } else if reachable_set.contains(&cell.hex) {
            demo.reachable_material.clone()
        } else if attack_range.contains(&cell.hex) {
            demo.attack_material.clone()
        } else if let Some(t) = demo.terrain.get(&cell.hex) {
            demo.terrain_materials[t].clone()
        } else {
            demo.terrain_materials[&Terrain::Plains].clone()
        };
        material.0 = next;
    }

    // Draw path preview as line
    if let Some(ref path) = preview_path {
        for pair in path.cells.windows(2) {
            let a = demo.board.layout.hex_to_world(pair[0]);
            let b = demo.board.layout.hex_to_world(pair[1]);
            gizmos.line_2d(a, b, Color::srgb(0.30, 0.95, 0.55));
        }
    }

    let hovered_info = if let Some(hex) = demo.hovered {
        let t = demo.terrain.get(&hex).copied().unwrap_or(Terrain::Plains);
        let cost_str = t
            .move_cost()
            .map_or("impassable".to_string(), |c| c.to_string());
        let path_str = preview_path
            .as_ref()
            .map(|p| format!("  Path cost: {}", p.total_cost))
            .unwrap_or_default();
        format!(
            "\nHovered: ({}, {})  Terrain: {:?}  Move cost: {}{}",
            hex.q, hex.r, t, cost_str, path_str
        )
    } else {
        String::new()
    };

    overlay.0 = format!(
        "Strategy board with terrain types and movement ranges.\nHover to preview path. Left click to move. Right click to cycle terrain.\nBudget: {}  Reachable: {}  Attack targets: {}{}",
        budget,
        reachable.len(),
        attack_range.len(),
        hovered_info,
    );

    demo.preview_path = preview_path;
}

fn sync_pane(
    pane: Res<StrategyPane>,
    mut demo: ResMut<StrategyDemo>,
    mut transforms: Query<&mut Transform>,
) {
    if pane.is_changed() {
        demo.reachable = None; // Invalidate on budget change
    }
    support::apply_hex_size(&mut demo.board, pane.hex_size, &mut transforms);
}
