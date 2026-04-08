#[cfg(feature = "e2e")]
mod e2e;
#[cfg(feature = "e2e")]
mod scenarios;

use saddle_world_hex_grid_example_support as support;

use bevy::prelude::*;
#[cfg(feature = "dev")]
use bevy::remote::RemotePlugin;
#[cfg(feature = "dev")]
use bevy_brp_extras::BrpExtrasPlugin;
use saddle_pane::prelude::*;
use saddle_world_hex_grid::{
    AxialHex, HexDebugOverlay, HexDirection, HexGridDebugSettings, HexGridPlugin, a_star,
    directional_fov, range_fov, reachable_within,
};
use std::collections::HashSet;
use support::{BoardKind, DemoBoard, DemoHexCell, OverlayText};

#[derive(Component)]
struct FlatOverlay;

#[derive(Component)]
struct PointyOverlay;

#[derive(Component)]
struct PathOverlay;

#[derive(Component)]
struct RangeOverlay;

#[derive(Component)]
struct FovOverlay;

#[derive(Component)]
struct SampleMarker {
    board: BoardKind,
}

#[derive(Resource, Reflect, Clone, Debug)]
#[reflect(Resource)]
pub struct LabControl {
    pub sample_local_point: Vec2,
    pub path_goal: AxialHex,
    pub reroute_barrier_enabled: bool,
    pub movement_budget: u32,
    pub range_radius: u32,
    pub show_attack_range: bool,
    pub fov_range: u32,
    pub fov_directional_mode: bool,
    pub fov_facing: HexDirection,
    pub fov_viewer: AxialHex,
}

#[cfg(feature = "e2e")]
#[derive(Resource, Clone, Debug, Default)]
pub(crate) struct E2EControlOverride(pub Option<LabControl>);

impl Default for LabControl {
    fn default() -> Self {
        Self {
            sample_local_point: Vec2::new(34.0, -8.0),
            path_goal: AxialHex::new(3, -2),
            reroute_barrier_enabled: false,
            movement_budget: 4,
            range_radius: 3,
            show_attack_range: false,
            fov_range: 4,
            fov_directional_mode: false,
            fov_facing: HexDirection::East,
            fov_viewer: AxialHex::ZERO,
        }
    }
}

#[derive(Resource, Pane)]
#[pane(title = "Hex Grid Lab")]
struct LabPane {
    #[pane(slider, min = 18.0, max = 40.0, step = 1.0)]
    hex_size: f32,
    #[pane(slider, min = -120.0, max = 120.0, step = 1.0)]
    sample_x: f32,
    #[pane(slider, min = -120.0, max = 120.0, step = 1.0)]
    sample_y: f32,
    #[pane(slider, min = -5.0, max = 5.0, step = 1.0)]
    goal_q: f32,
    #[pane(slider, min = -5.0, max = 5.0, step = 1.0)]
    goal_r: f32,
    #[pane(toggle)]
    reroute_barrier_enabled: bool,
    #[pane(slider, min = 1.0, max = 10.0, step = 1.0)]
    movement_budget: f32,
    #[pane(slider, min = 1.0, max = 6.0, step = 1.0)]
    range_radius: f32,
    #[pane(toggle)]
    show_attack_range: bool,
    #[pane(slider, min = 1.0, max = 6.0, step = 1.0)]
    fov_range: f32,
    #[pane(toggle)]
    fov_directional_mode: bool,
    #[pane(slider, min = 0.0, max = 5.0, step = 1.0)]
    fov_facing_index: f32,
}

impl Default for LabPane {
    fn default() -> Self {
        Self {
            hex_size: 25.0,
            sample_x: 34.0,
            sample_y: -8.0,
            goal_q: 3.0,
            goal_r: -2.0,
            reroute_barrier_enabled: false,
            movement_budget: 4.0,
            range_radius: 3.0,
            show_attack_range: false,
            fov_range: 4.0,
            fov_directional_mode: false,
            fov_facing_index: 0.0,
        }
    }
}

#[derive(Resource, Reflect, Clone, Debug, Default)]
#[reflect(Resource)]
pub struct LabDiagnostics {
    pub flat_hover_hex: AxialHex,
    pub pointy_hover_hex: AxialHex,
    pub path_exists: bool,
    pub path_len: usize,
    pub path_cost: u32,
    pub flat_neighbor_count: usize,
    pub reachable_count: usize,
    pub ring_count: usize,
    pub spiral_count: usize,
    pub attack_count: usize,
    pub fov_visible_count: usize,
    pub fov_hidden_behind_wall: bool,
}

#[derive(Resource)]
struct LabScene {
    flat: DemoBoard,
    pointy: DemoBoard,
    path: DemoBoard,
    range: DemoBoard,
    fov: DemoBoard,
    flat_overlay: Entity,
    pointy_overlay: Entity,
    path_overlay: Entity,
    range_overlay: Entity,
    fov_overlay: Entity,
    default_material: Handle<ColorMaterial>,
    selected_material: Handle<ColorMaterial>,
    blocked_material: Handle<ColorMaterial>,
    weighted_material: Handle<ColorMaterial>,
    path_material: Handle<ColorMaterial>,
    range_material: Handle<ColorMaterial>,
    ring_material: Handle<ColorMaterial>,
    attack_material: Handle<ColorMaterial>,
    wall_material: Handle<ColorMaterial>,
    fov_visible_material: Handle<ColorMaterial>,
    viewer_material: Handle<ColorMaterial>,
}

fn main() {
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb(0.06, 0.08, 0.10)));
    app.insert_resource(LabControl::default());
    app.insert_resource(LabPane::default());
    app.insert_resource(LabDiagnostics::default());
    #[cfg(feature = "e2e")]
    app.insert_resource(E2EControlOverride::default());
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "hex_grid crate-local lab".into(),
            resolution: (1540, 980).into(),
            ..default()
        }),
        ..default()
    }));
    app.add_plugins(support::pane_plugins());
    #[cfg(feature = "dev")]
    app.add_plugins(RemotePlugin::default());
    #[cfg(feature = "dev")]
    app.add_plugins(BrpExtrasPlugin::with_port(15702));
    #[cfg(feature = "e2e")]
    app.add_plugins(e2e::HexGridLabE2EPlugin);
    app.add_plugins(
        HexGridPlugin::default().with_debug_settings(HexGridDebugSettings {
            enabled: true,
            draw_centers: true,
            draw_cell_outlines: true,
            draw_path_lines: true,
            draw_coord_labels: false,
            center_radius: 4.0,
        }),
    );
    app.register_type::<LabControl>()
        .register_type::<LabDiagnostics>()
        .register_pane::<LabPane>()
        .add_systems(Startup, setup);
    #[cfg(feature = "e2e")]
    app.add_systems(
        Update,
        (sync_lab_pane, apply_e2e_control_override, update_lab).chain(),
    );
    #[cfg(not(feature = "e2e"))]
    app.add_systems(Update, (sync_lab_pane, update_lab).chain());
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    support::spawn_camera(&mut commands, "Lab Camera");
    support::spawn_overlay(
        &mut commands,
        "hex_grid lab\nTop row: flat-top and pointy-top world picking.\nRight: weighted A* preview.\nBottom left: ring, spiral, and movement range diagnostics.\nBottom right: FOV walls, viewer, and directional cone.",
    );

    let flat = support::spawn_demo_board(
        &mut commands,
        &mut meshes,
        &mut materials,
        BoardKind::Flat,
        "Flat Layout",
        saddle_world_hex_grid::HexLayout::flat()
            .with_uniform_size(25.0)
            .with_origin(Vec2::new(-480.0, 190.0)),
        AxialHex::ZERO.hexagon(3),
        Color::srgb(0.15, 0.19, 0.24),
    );
    let pointy = support::spawn_demo_board(
        &mut commands,
        &mut meshes,
        &mut materials,
        BoardKind::Pointy,
        "Pointy Layout",
        saddle_world_hex_grid::HexLayout::pointy()
            .with_uniform_size(25.0)
            .with_origin(Vec2::new(-180.0, 190.0)),
        AxialHex::ZERO.hexagon(3),
        Color::srgb(0.15, 0.19, 0.24),
    );
    let path = support::spawn_demo_board(
        &mut commands,
        &mut meshes,
        &mut materials,
        BoardKind::Path,
        "Weighted A*",
        saddle_world_hex_grid::HexLayout::pointy()
            .with_uniform_size(22.0)
            .with_origin(Vec2::new(260.0, 140.0)),
        AxialHex::ZERO.hexagon(5),
        Color::srgb(0.15, 0.19, 0.24),
    );
    let range = support::spawn_demo_board(
        &mut commands,
        &mut meshes,
        &mut materials,
        BoardKind::Range,
        "Ranges",
        saddle_world_hex_grid::HexLayout::flat()
            .with_uniform_size(24.0)
            .with_origin(Vec2::new(-30.0, -250.0)),
        AxialHex::ZERO.hexagon(4),
        Color::srgb(0.15, 0.19, 0.24),
    );
    let fov = support::spawn_demo_board(
        &mut commands,
        &mut meshes,
        &mut materials,
        BoardKind::Fov,
        "Field Of View",
        saddle_world_hex_grid::HexLayout::pointy()
            .with_uniform_size(22.0)
            .with_origin(Vec2::new(430.0, -230.0)),
        AxialHex::ZERO.hexagon(5),
        Color::srgb(0.12, 0.14, 0.18),
    );

    let marker_mesh = meshes.add(Circle::new(6.0));
    let marker_material = materials.add(Color::WHITE);
    commands.spawn((
        Name::new("Flat Sample Marker"),
        SampleMarker {
            board: BoardKind::Flat,
        },
        Mesh2d(marker_mesh.clone()),
        MeshMaterial2d(marker_material.clone()),
        Transform::default(),
    ));
    commands.spawn((
        Name::new("Pointy Sample Marker"),
        SampleMarker {
            board: BoardKind::Pointy,
        },
        Mesh2d(marker_mesh),
        MeshMaterial2d(marker_material),
        Transform::default(),
    ));

    let flat_overlay = commands
        .spawn((
            Name::new("Flat Debug Overlay"),
            FlatOverlay,
            HexDebugOverlay {
                layout: flat.layout,
                cells: flat.coords.clone(),
                highlighted: Vec::new(),
                ..default()
            },
        ))
        .id();
    let pointy_overlay = commands
        .spawn((
            Name::new("Pointy Debug Overlay"),
            PointyOverlay,
            HexDebugOverlay {
                layout: pointy.layout,
                cells: pointy.coords.clone(),
                highlighted: Vec::new(),
                ..default()
            },
        ))
        .id();
    let path_overlay = commands
        .spawn((
            Name::new("Path Debug Overlay"),
            PathOverlay,
            HexDebugOverlay {
                layout: path.layout,
                cells: path.coords.clone(),
                cell_color: Color::srgba(0.28, 0.48, 0.80, 0.55),
                highlight_color: Color::srgba(0.94, 0.74, 0.26, 0.95),
                path_color: Color::srgba(0.22, 0.92, 0.56, 0.95),
                ..default()
            },
        ))
        .id();
    let range_overlay = commands
        .spawn((
            Name::new("Range Debug Overlay"),
            RangeOverlay,
            HexDebugOverlay {
                layout: range.layout,
                cells: range.coords.clone(),
                cell_color: Color::srgba(0.26, 0.62, 0.96, 0.55),
                highlight_color: Color::srgba(0.95, 0.70, 0.28, 0.95),
                path_color: Color::srgba(0.26, 0.94, 0.63, 0.95),
                ..default()
            },
        ))
        .id();
    let fov_overlay = commands
        .spawn((
            Name::new("FOV Debug Overlay"),
            FovOverlay,
            HexDebugOverlay {
                layout: fov.layout,
                cells: fov.coords.clone(),
                cell_color: Color::srgba(0.26, 0.62, 0.96, 0.45),
                highlight_color: Color::srgba(0.95, 0.73, 0.28, 0.95),
                fov_color: Color::srgba(0.32, 0.84, 0.94, 0.75),
                ..default()
            },
        ))
        .id();

    commands.insert_resource(LabScene {
        flat,
        pointy,
        path,
        range,
        fov,
        flat_overlay,
        pointy_overlay,
        path_overlay,
        range_overlay,
        fov_overlay,
        default_material: materials.add(Color::srgb(0.15, 0.19, 0.24)),
        selected_material: materials.add(Color::srgb(0.92, 0.73, 0.26)),
        blocked_material: materials.add(Color::srgb(0.37, 0.14, 0.18)),
        weighted_material: materials.add(Color::srgb(0.61, 0.45, 0.19)),
        path_material: materials.add(Color::srgb(0.26, 0.86, 0.55)),
        range_material: materials.add(Color::srgb(0.26, 0.68, 0.96)),
        ring_material: materials.add(Color::srgb(0.96, 0.70, 0.28)),
        attack_material: materials.add(Color::srgba(0.90, 0.30, 0.25, 0.80)),
        wall_material: materials.add(Color::srgb(0.35, 0.15, 0.15)),
        fov_visible_material: materials.add(Color::srgb(0.30, 0.68, 0.86)),
        viewer_material: materials.add(Color::srgb(0.92, 0.76, 0.24)),
    });
}

fn update_lab(
    control: Res<LabControl>,
    mut diagnostics: ResMut<LabDiagnostics>,
    scene: Res<LabScene>,
    mut cells: Query<(&DemoHexCell, &mut MeshMaterial2d<ColorMaterial>)>,
    mut overlays: Query<&mut HexDebugOverlay>,
    mut markers: Query<(&SampleMarker, &mut Transform)>,
    mut overlay_text: Single<&mut Text, With<OverlayText>>,
) {
    let path_blocked = path_blocked_cells(control.reroute_barrier_enabled);
    let weighted_cells = weighted_path_cells();
    let fov_walls = fov_wall_cells();
    let flat_world = scene.flat.layout.origin + control.sample_local_point;
    let pointy_world = scene.pointy.layout.origin + control.sample_local_point;
    let flat_hover = scene.flat.layout.world_to_hex(flat_world);
    let pointy_hover = scene.pointy.layout.world_to_hex(pointy_world);
    let path = a_star(AxialHex::ZERO, control.path_goal, |_, to| {
        if !scene.path.cells.contains_key(&to) || path_blocked.contains(&to) {
            None
        } else if weighted_cells.contains(&to) {
            Some(3)
        } else {
            Some(1)
        }
    });
    let range_reachable = reachable_within(AxialHex::ZERO, control.movement_budget, |_, to| {
        if !scene.range.cells.contains_key(&to) {
            None
        } else if weighted_cells.contains(&to) {
            Some(2)
        } else {
            Some(1)
        }
    });
    let ring: Vec<_> = AxialHex::ZERO.ring(control.range_radius).collect();
    let spiral: Vec<_> = AxialHex::ZERO.spiral(control.range_radius).collect();
    let reachable_set: HashSet<_> = range_reachable.iter().map(|(hex, _)| hex).collect();
    let path_cells: HashSet<_> = path
        .as_ref()
        .map(|path| path.cells.iter().copied().collect())
        .unwrap_or_default();
    let flat_neighbors: HashSet<_> = flat_hover
        .neighbors()
        .into_iter()
        .filter(|hex| scene.flat.cells.contains_key(hex))
        .collect();
    let attack_range: HashSet<_> = reachable_set
        .iter()
        .flat_map(|hex| hex.neighbors())
        .filter(|hex| !reachable_set.contains(hex) && scene.range.cells.contains_key(hex))
        .collect();
    let fov_visible = if control.fov_directional_mode {
        directional_fov(
            control.fov_viewer,
            control.fov_range,
            control.fov_facing,
            |hex| fov_walls.contains(&hex),
        )
    } else {
        range_fov(control.fov_viewer, control.fov_range, |hex| {
            fov_walls.contains(&hex)
        })
    };
    let fov_visible_on_board: HashSet<_> = fov_visible
        .into_iter()
        .filter(|hex| scene.fov.cells.contains_key(hex))
        .collect();

    diagnostics.flat_hover_hex = flat_hover;
    diagnostics.pointy_hover_hex = pointy_hover;
    diagnostics.path_exists = path.is_some();
    diagnostics.path_len = path.as_ref().map_or(0, |path| path.cells.len());
    diagnostics.path_cost = path.as_ref().map_or(0, |path| path.total_cost);
    diagnostics.flat_neighbor_count = flat_neighbors.len();
    diagnostics.reachable_count = range_reachable.len();
    diagnostics.ring_count = ring.len();
    diagnostics.spiral_count = spiral.len();
    diagnostics.attack_count = attack_range.len();
    diagnostics.fov_visible_count = fov_visible_on_board.len();
    diagnostics.fov_hidden_behind_wall = !fov_visible_on_board.contains(&AxialHex::new(3, 0));

    for (marker, mut transform) in &mut markers {
        transform.translation = match marker.board {
            BoardKind::Flat => flat_world.extend(4.0),
            BoardKind::Pointy => pointy_world.extend(4.0),
            _ => transform.translation,
        };
    }

    for (cell, mut material) in &mut cells {
        let next = match cell.board {
            BoardKind::Flat if cell.hex == flat_hover => scene.selected_material.clone(),
            BoardKind::Flat if flat_neighbors.contains(&cell.hex) => scene.range_material.clone(),
            BoardKind::Flat => scene.default_material.clone(),
            BoardKind::Pointy if cell.hex == pointy_hover => scene.selected_material.clone(),
            BoardKind::Pointy => scene.default_material.clone(),
            BoardKind::Path if cell.hex == AxialHex::ZERO || cell.hex == control.path_goal => {
                scene.selected_material.clone()
            }
            BoardKind::Path if path_blocked.contains(&cell.hex) => scene.blocked_material.clone(),
            BoardKind::Path if path_cells.contains(&cell.hex) => scene.path_material.clone(),
            BoardKind::Path if weighted_cells.contains(&cell.hex) => {
                scene.weighted_material.clone()
            }
            BoardKind::Path => scene.default_material.clone(),
            BoardKind::Range if cell.hex == AxialHex::ZERO => scene.selected_material.clone(),
            BoardKind::Range if control.show_attack_range && attack_range.contains(&cell.hex) => {
                scene.attack_material.clone()
            }
            BoardKind::Range if ring.contains(&cell.hex) => scene.ring_material.clone(),
            BoardKind::Range if reachable_set.contains(&cell.hex) => scene.range_material.clone(),
            BoardKind::Range => scene.default_material.clone(),
            BoardKind::Fov if cell.hex == control.fov_viewer => scene.viewer_material.clone(),
            BoardKind::Fov if fov_walls.contains(&cell.hex) => scene.wall_material.clone(),
            BoardKind::Fov if fov_visible_on_board.contains(&cell.hex) => {
                scene.fov_visible_material.clone()
            }
            BoardKind::Fov => scene.default_material.clone(),
            BoardKind::Primary => scene.default_material.clone(),
        };
        material.0 = next;
    }

    if let Ok(mut flat_overlay) = overlays.get_mut(scene.flat_overlay) {
        flat_overlay.highlighted = std::iter::once(flat_hover).chain(flat_neighbors).collect();
        flat_overlay.path.clear();
    }
    if let Ok(mut pointy_overlay) = overlays.get_mut(scene.pointy_overlay) {
        pointy_overlay.highlighted = vec![pointy_hover];
        pointy_overlay.path.clear();
    }
    if let Ok(mut path_overlay) = overlays.get_mut(scene.path_overlay) {
        path_overlay.highlighted = vec![AxialHex::ZERO, control.path_goal];
        path_overlay.path = path
            .as_ref()
            .map(|path| path.cells.clone())
            .unwrap_or_default();
    }
    if let Ok(mut range_overlay) = overlays.get_mut(scene.range_overlay) {
        range_overlay.cells = reachable_set.iter().copied().collect();
        range_overlay.highlighted = if control.show_attack_range {
            attack_range.iter().copied().collect()
        } else {
            ring.clone()
        };
        range_overlay.path = if control.show_attack_range {
            Vec::new()
        } else {
            spiral.clone()
        };
    }
    if let Ok(mut fov_overlay) = overlays.get_mut(scene.fov_overlay) {
        fov_overlay.cells = scene.fov.coords.clone();
        fov_overlay.highlighted = vec![control.fov_viewer];
        fov_overlay.path.clear();
        fov_overlay.fov_cells = fov_visible_on_board.iter().copied().collect();
    }

    overlay_text.0 = format!(
        "hex_grid lab\nsample local ({:.1}, {:.1}) -> flat ({}, {}) / pointy ({}, {})  flat neighbors {}\npath goal ({}, {})  exists {}  len {}  cost {}\nmovement budget {}  reachable {}  ring {}  spiral {}  attack {}\nfov mode {}  facing {:?}  range {}  visible {}",
        control.sample_local_point.x,
        control.sample_local_point.y,
        diagnostics.flat_hover_hex.q,
        diagnostics.flat_hover_hex.r,
        diagnostics.pointy_hover_hex.q,
        diagnostics.pointy_hover_hex.r,
        diagnostics.flat_neighbor_count,
        control.path_goal.q,
        control.path_goal.r,
        diagnostics.path_exists,
        diagnostics.path_len,
        diagnostics.path_cost,
        control.movement_budget,
        diagnostics.reachable_count,
        diagnostics.ring_count,
        diagnostics.spiral_count,
        diagnostics.attack_count,
        if control.fov_directional_mode {
            "directional"
        } else {
            "360"
        },
        control.fov_facing,
        control.fov_range,
        diagnostics.fov_visible_count,
    );
}

fn sync_lab_pane(
    pane: Res<LabPane>,
    mut control: ResMut<LabControl>,
    mut scene: ResMut<LabScene>,
    mut cell_transforms: Query<&mut Transform, With<DemoHexCell>>,
    mut overlays: Query<&mut HexDebugOverlay>,
) {
    if !pane.is_changed() {
        return;
    }

    control.sample_local_point = Vec2::new(pane.sample_x, pane.sample_y);
    control.path_goal = AxialHex::new(pane.goal_q.round() as i32, pane.goal_r.round() as i32);
    control.reroute_barrier_enabled = pane.reroute_barrier_enabled;
    control.movement_budget = pane.movement_budget.round().max(1.0) as u32;
    control.range_radius = pane.range_radius.round().max(1.0) as u32;
    control.show_attack_range = pane.show_attack_range;
    control.fov_range = pane.fov_range.round().max(1.0) as u32;
    control.fov_directional_mode = pane.fov_directional_mode;
    control.fov_facing = HexDirection::ALL[pane.fov_facing_index.round() as usize % 6];

    support::apply_hex_size(&mut scene.flat, pane.hex_size, &mut cell_transforms);
    support::apply_hex_size(&mut scene.pointy, pane.hex_size, &mut cell_transforms);
    support::apply_hex_size(&mut scene.path, pane.hex_size, &mut cell_transforms);
    support::apply_hex_size(&mut scene.range, pane.hex_size, &mut cell_transforms);
    support::apply_hex_size(&mut scene.fov, pane.hex_size, &mut cell_transforms);

    if let Ok(mut overlay) = overlays.get_mut(scene.flat_overlay) {
        overlay.layout = scene.flat.layout;
    }
    if let Ok(mut overlay) = overlays.get_mut(scene.pointy_overlay) {
        overlay.layout = scene.pointy.layout;
    }
    if let Ok(mut overlay) = overlays.get_mut(scene.path_overlay) {
        overlay.layout = scene.path.layout;
    }
    if let Ok(mut overlay) = overlays.get_mut(scene.range_overlay) {
        overlay.layout = scene.range.layout;
    }
    if let Ok(mut overlay) = overlays.get_mut(scene.fov_overlay) {
        overlay.layout = scene.fov.layout;
    }
}

#[cfg(feature = "e2e")]
fn apply_e2e_control_override(
    override_control: Res<E2EControlOverride>,
    mut control: ResMut<LabControl>,
) {
    if let Some(snapshot) = &override_control.0 {
        *control = snapshot.clone();
    }
}

fn path_blocked_cells(reroute_barrier_enabled: bool) -> HashSet<AxialHex> {
    let mut blocked = HashSet::from([
        AxialHex::new(1, 0),
        AxialHex::new(1, -1),
        AxialHex::new(2, -1),
        AxialHex::new(2, -2),
    ]);
    if reroute_barrier_enabled {
        blocked.extend([
            AxialHex::new(-1, 1),
            AxialHex::new(0, 1),
            AxialHex::new(1, 1),
            AxialHex::new(2, 0),
        ]);
    }
    blocked
}

fn weighted_path_cells() -> HashSet<AxialHex> {
    HashSet::from([
        AxialHex::new(-2, 2),
        AxialHex::new(-1, 2),
        AxialHex::new(0, 2),
        AxialHex::new(1, -2),
    ])
}

pub(crate) fn fov_wall_cells() -> HashSet<AxialHex> {
    HashSet::from([
        AxialHex::new(2, 0),
        AxialHex::new(2, -1),
        AxialHex::new(2, -2),
        AxialHex::new(-1, 3),
        AxialHex::new(0, 3),
        AxialHex::new(1, 2),
        AxialHex::new(-3, 1),
        AxialHex::new(-3, 2),
    ])
}
