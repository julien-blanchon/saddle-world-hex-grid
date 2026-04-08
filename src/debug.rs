use crate::{AxialHex, HexLayout};
use bevy::{
    ecs::system::SystemState,
    gizmos::{config::GizmoConfigGroup, gizmos::GizmoStorage, prelude::Gizmos},
    prelude::*,
};

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct HexGridDebugGizmos;

#[derive(Clone, Debug, Reflect, Resource)]
pub struct HexGridDebugSettings {
    pub enabled: bool,
    pub draw_centers: bool,
    pub draw_cell_outlines: bool,
    pub draw_path_lines: bool,
    /// Reserved for future text label rendering.
    ///
    /// The current debug runtime only draws gizmos, so this flag has no effect yet.
    pub draw_coord_labels: bool,
    pub center_radius: f32,
}

impl Default for HexGridDebugSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            draw_centers: true,
            draw_cell_outlines: true,
            draw_path_lines: true,
            draw_coord_labels: false,
            center_radius: 6.0,
        }
    }
}

#[derive(Clone, Debug, Reflect, Component)]
pub struct HexDebugOverlay {
    pub layout: HexLayout,
    pub cells: Vec<AxialHex>,
    pub highlighted: Vec<AxialHex>,
    pub path: Vec<AxialHex>,
    /// Cells visible via FOV — drawn as filled outlines in `fov_color`.
    pub fov_cells: Vec<AxialHex>,
    pub cell_color: Color,
    pub highlight_color: Color,
    pub path_color: Color,
    pub fov_color: Color,
}

impl Default for HexDebugOverlay {
    fn default() -> Self {
        Self {
            layout: HexLayout::default(),
            cells: Vec::new(),
            highlighted: Vec::new(),
            path: Vec::new(),
            fov_cells: Vec::new(),
            cell_color: Color::srgba(0.55, 0.72, 0.94, 0.85),
            highlight_color: Color::srgba(0.98, 0.76, 0.25, 0.95),
            path_color: Color::srgba(0.20, 0.93, 0.55, 0.95),
            fov_color: Color::srgba(0.95, 0.85, 0.30, 0.65),
        }
    }
}

#[derive(Clone, Debug, Default, Component)]
pub(crate) struct HexDebugOverlayState {
    cells: Vec<[Vec2; 6]>,
    highlighted: Vec<[Vec2; 6]>,
    path_centers: Vec<Vec2>,
    fov: Vec<[Vec2; 6]>,
}

pub(crate) fn sync_debug_overlays(
    mut commands: Commands,
    overlays: Query<
        (Entity, &HexDebugOverlay),
        Or<(Added<HexDebugOverlay>, Changed<HexDebugOverlay>)>,
    >,
) {
    for (entity, overlay) in &overlays {
        let state = HexDebugOverlayState {
            cells: overlay
                .cells
                .iter()
                .copied()
                .map(|hex| overlay.layout.corners(hex))
                .collect(),
            highlighted: overlay
                .highlighted
                .iter()
                .copied()
                .map(|hex| overlay.layout.corners(hex))
                .collect(),
            path_centers: overlay
                .path
                .iter()
                .copied()
                .map(|hex| overlay.layout.hex_to_world(hex))
                .collect(),
            fov: overlay
                .fov_cells
                .iter()
                .copied()
                .map(|hex| overlay.layout.corners(hex))
                .collect(),
        };
        commands.entity(entity).insert(state);
    }
}

type DebugDrawState<'w, 's> = SystemState<(
    Res<'w, HexGridDebugSettings>,
    Gizmos<'w, 's, HexGridDebugGizmos>,
    Query<'w, 's, (&'static HexDebugOverlay, &'static HexDebugOverlayState)>,
)>;

pub(crate) fn draw_debug(world: &mut World) {
    if !world.contains_resource::<GizmoStorage<HexGridDebugGizmos, ()>>() {
        return;
    }

    let mut state: DebugDrawState<'_, '_> = SystemState::new(world);
    let (settings, mut gizmos, overlays) = state.get_mut(world);
    if !settings.enabled {
        return;
    }

    for (overlay, cached) in &overlays {
        if settings.draw_cell_outlines {
            for corners in &cached.cells {
                draw_outline(&mut gizmos, corners, overlay.cell_color);
            }
            for corners in &cached.highlighted {
                draw_outline(&mut gizmos, corners, overlay.highlight_color);
            }
        }

        if settings.draw_centers {
            for hex in &overlay.cells {
                gizmos.circle_2d(
                    overlay.layout.hex_to_world(*hex),
                    settings.center_radius,
                    overlay.cell_color,
                );
            }
            for hex in &overlay.highlighted {
                gizmos.circle_2d(
                    overlay.layout.hex_to_world(*hex),
                    settings.center_radius * 1.2,
                    overlay.highlight_color,
                );
            }
        }

        if settings.draw_cell_outlines {
            for corners in &cached.fov {
                draw_outline(&mut gizmos, corners, overlay.fov_color);
            }
        }

        if settings.draw_path_lines {
            for pair in cached.path_centers.windows(2) {
                gizmos.line_2d(pair[0], pair[1], overlay.path_color);
            }
        }
    }
}

fn draw_outline(
    gizmos: &mut Gizmos<'_, '_, HexGridDebugGizmos>,
    corners: &[Vec2; 6],
    color: Color,
) {
    for index in 0..6 {
        let next = (index + 1) % 6;
        gizmos.line_2d(corners[index], corners[next], color);
    }
}
