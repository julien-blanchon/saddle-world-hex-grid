use bevy::prelude::*;
use saddle_world_hex_grid::{AxialHex, HexLayout, HexOrientation};
use std::{collections::HashMap, f32::consts::FRAC_PI_6};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BoardKind {
    Primary,
    Flat,
    Pointy,
    Path,
    Range,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct DemoHexCell {
    pub board: BoardKind,
    pub hex: AxialHex,
}

#[derive(Component)]
pub struct OverlayText;

#[derive(Clone, Debug)]
pub struct DemoBoard {
    pub root: Entity,
    pub layout: HexLayout,
    pub coords: Vec<AxialHex>,
    pub cells: HashMap<AxialHex, Entity>,
}

pub fn spawn_camera(commands: &mut Commands, title: &str) {
    commands.spawn((Name::new(title.to_string()), Camera2d));
}

pub fn spawn_overlay(commands: &mut Commands, title: impl Into<String>) -> Entity {
    commands
        .spawn((
            Name::new("Overlay"),
            OverlayText,
            Text::new(title.into()),
            TextFont {
                font_size: 15.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Node {
                position_type: PositionType::Absolute,
                left: px(18.0),
                top: px(16.0),
                width: px(420.0),
                padding: UiRect::axes(px(12.0), px(10.0)),
                border_radius: BorderRadius::all(px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.07, 0.09, 0.88)),
        ))
        .id()
}

pub fn board_cell_transform(layout: HexLayout, hex: AxialHex) -> Transform {
    Transform::from_translation(layout.hex_to_world(hex).extend(0.0))
        .with_rotation(board_rotation(layout.orientation))
        .with_scale(Vec3::new(
            layout.hex_size.x * 0.92,
            layout.hex_size.y * 0.92,
            1.0,
        ))
}

pub fn board_rotation(orientation: HexOrientation) -> Quat {
    match orientation {
        HexOrientation::FlatTop => Quat::IDENTITY,
        HexOrientation::PointyTop => Quat::from_rotation_z(FRAC_PI_6),
    }
}

pub fn cursor_world(
    window: &Window,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Option<Vec2> {
    window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor).ok())
}

pub fn spawn_demo_board(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    board: BoardKind,
    name: &str,
    layout: HexLayout,
    coords: impl IntoIterator<Item = AxialHex>,
    base_color: Color,
) -> DemoBoard {
    let root = commands
        .spawn((
            Name::new(name.to_string()),
            Transform::default(),
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            ViewVisibility::default(),
        ))
        .id();

    let mesh = meshes.add(RegularPolygon::new(1.0, 6));
    let material = materials.add(base_color);
    let title_offset = layout.rect_size().y * 3.0;
    let title_position = layout.origin + Vec2::new(0.0, title_offset);
    let title = name.to_string();
    let mut cells = HashMap::new();
    let mut coords_vec = Vec::new();

    commands.entity(root).with_children(|parent| {
        parent.spawn((
            Text2d::new(title.clone()),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::srgb(0.94, 0.94, 0.97)),
            Transform::from_translation(title_position.extend(2.0)),
        ));

        for hex in coords {
            coords_vec.push(hex);
            let entity = parent
                .spawn((
                    DemoHexCell { board, hex },
                    Mesh2d(mesh.clone()),
                    MeshMaterial2d(material.clone()),
                    board_cell_transform(layout, hex),
                ))
                .id();
            cells.insert(hex, entity);
        }
    });

    DemoBoard {
        root,
        layout,
        coords: coords_vec,
        cells,
    }
}
