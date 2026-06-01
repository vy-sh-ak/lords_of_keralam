use std::collections::HashSet;

use crate::camera_plugin::CameraSystems;
use bevy::{
    math::primitives::{Cuboid, Plane3d},
    prelude::*,
    window::PrimaryWindow,
};

pub struct RegularGridPlugin;

#[derive(Resource, Clone, Copy)]
pub struct RegularGridSettings {
    pub cells_x: u32,
    pub cells_z: u32,
    pub spacing: f32,
    pub y: f32,
}

#[derive(Resource, Clone)]
pub struct GridGraph {
    pub points: Vec<Vec3>,
    pub edges: Vec<(usize, usize)>,
}

#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct GridHoverState {
    pub is_over_grid: bool,
    pub cell: Option<IVec2>,
}

#[derive(Resource, Default)]
pub struct PlacedCells {
    pub cells: HashSet<IVec2>,
}

#[derive(Resource, Clone)]
struct GridRenderAssets {
    cube_mesh: Handle<Mesh>,
    cube_material: Handle<StandardMaterial>,
}

#[derive(Component)]
struct HoverCellFill;

#[derive(Component)]
struct PlacedCube {
    cell: IVec2,
}

impl Plugin for RegularGridPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RegularGridSettings {
            cells_x: 10,
            cells_z: 10,
            spacing: 10.0,
            y: 0.02,
        })
        .insert_resource(GridHoverState::default())
        .insert_resource(PlacedCells::default())
        .add_systems(
            Startup,
            (
                build_regular_grid_system,
                setup_grid_visuals_system,
            ),
        )
        .add_systems(
            Update,
            (
                draw_regular_grid_system,
                (
                    hover_grid_cell.after(CameraSystems::UpdateState),
                    place_cube_on_click,
                    draw_hover_cell,
                )
                    .chain(),
            ),
        );
    }
}

fn build_regular_grid_system(mut commands: Commands, settings: Res<RegularGridSettings>) {
    commands.insert_resource(build_regular_grid(*settings));
}

fn setup_grid_visuals_system(
    mut commands: Commands,
    settings: Res<RegularGridSettings>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cube_size = settings.spacing;

    commands.insert_resource(GridRenderAssets {
        cube_mesh: meshes.add(Cuboid::from_size(Vec3::splat(cube_size))),
        cube_material: materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 1.0, 1.0),
            perceptual_roughness: 0.95,
            ..default()
        }),
    });

    commands.spawn((
        Name::new("HoverCellFill"),
        HoverCellFill,
        Mesh3d(meshes.add(Plane3d::default().mesh().size(settings.spacing, settings.spacing))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(0.15, 0.35, 1.0, 0.45),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            cull_mode: None,
            ..default()
        })),
        Transform::from_xyz(0.0, settings.y + 0.03, 0.0),
        Visibility::Hidden,
    ));
}

fn build_regular_grid(settings: RegularGridSettings) -> GridGraph {
    let stride = settings.cells_x + 1;
    let total_width = settings.cells_x as f32 * settings.spacing;
    let total_depth = settings.cells_z as f32 * settings.spacing;

    let origin = Vec3::new(-total_width * 0.5, settings.y, -total_depth * 0.5);

    let mut points =
        Vec::with_capacity(((settings.cells_x + 1) * (settings.cells_z + 1)) as usize);
    let mut edges = Vec::new();

    for z in 0..=settings.cells_z {
        for x in 0..=settings.cells_x {
            let point = origin
                + Vec3::new(
                    x as f32 * settings.spacing,
                    0.0,
                    z as f32 * settings.spacing,
                );

            points.push(point);

            let current = point_index(x, z, stride);

            if x < settings.cells_x {
                edges.push((current, point_index(x + 1, z, stride)));
            }

            if z < settings.cells_z {
                edges.push((current, point_index(x, z + 1, stride)));
            }
        }
    }

    GridGraph { points, edges }
}

fn point_index(x: u32, z: u32, stride: u32) -> usize {
    (z * stride + x) as usize
}

fn draw_regular_grid_system(mut gizmos: Gizmos, grid: Res<GridGraph>) {
    for &(a, b) in &grid.edges {
        gizmos.line(
            grid.points[a],
            grid.points[b],
            Color::srgb(0.1, 0.1, 0.1),
        );
    }
}

fn hover_grid_cell(
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    settings: Res<RegularGridSettings>,
    placed_cells: Res<PlacedCells>,
    mut hover_state: ResMut<GridHoverState>,
    mut last_hovered: Local<Option<IVec2>>,
) {
    hover_state.is_over_grid = false;
    hover_state.cell = None;

    let Ok(window) = windows.single() else {
        *last_hovered = None;
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        *last_hovered = None;
        return;
    };

    let Ok((camera, camera_transform)) = camera_query.single() else {
        *last_hovered = None;
        return;
    };

    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        *last_hovered = None;
        return;
    };

    let ray_origin = ray.origin;
    let ray_direction = ray.direction.as_vec3();

    if ray_direction.y.abs() < f32::EPSILON {
        *last_hovered = None;
        return;
    }

    let t = (settings.y - ray_origin.y) / ray_direction.y;
    if t < 0.0 {
        *last_hovered = None;
        return;
    }

    let hit = ray_origin + ray_direction * t;

    let Some(cell) = world_to_cell(hit, &settings) else {
        *last_hovered = None;
        return;
    };

    if placed_cells.cells.contains(&cell) {
        *last_hovered = None;
        return;
    }

    hover_state.is_over_grid = true;
    hover_state.cell = Some(cell);

    if *last_hovered != Some(cell) {
        info!("Hovered cell: ({}, {})", cell.x, cell.y);
        *last_hovered = Some(cell);
    }
}

fn place_cube_on_click(
    mut commands: Commands,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut hover_state: ResMut<GridHoverState>,
    mut placed_cells: ResMut<PlacedCells>,
    render_assets: Res<GridRenderAssets>,
    settings: Res<RegularGridSettings>,
) {
    if !mouse_buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let Some(cell) = hover_state.cell else {
        return;
    };

    if !placed_cells.cells.insert(cell) {
        return;
    }

    let cube_size = settings.spacing * 0.8;
    let center = cell_center(cell, &settings, settings.y + cube_size * 0.5);

    commands.spawn((
        Name::new(format!("PlacedCube({}, {})", cell.x, cell.y)),
        PlacedCube { cell },
        Mesh3d(render_assets.cube_mesh.clone()),
        MeshMaterial3d(render_assets.cube_material.clone()),
        Transform::from_translation(center),
    ));

    hover_state.is_over_grid = false;
    hover_state.cell = None;

    info!("Placed cube at cell: ({}, {})", cell.x, cell.y);
}

fn draw_hover_cell(
    hover_state: Res<GridHoverState>,
    placed_cells: Res<PlacedCells>,
    settings: Res<RegularGridSettings>,
    mut fill_query: Query<(&mut Transform, &mut Visibility), With<HoverCellFill>>,
) {
    let Ok((mut transform, mut visibility)) = fill_query.single_mut() else {
        return;
    };

    let Some(cell) = hover_state.cell else {
        *visibility = Visibility::Hidden;
        return;
    };

    if !hover_state.is_over_grid || placed_cells.cells.contains(&cell) {
        *visibility = Visibility::Hidden;
        return;
    }

    transform.translation = cell_center(cell, &settings, settings.y + 0.03);
    *visibility = Visibility::Visible;
}

fn world_to_cell(hit: Vec3, settings: &RegularGridSettings) -> Option<IVec2> {
    let (origin_x, origin_z) = grid_origin(settings);

    let local_x = hit.x - origin_x;
    let local_z = hit.z - origin_z;

    if local_x < 0.0 || local_z < 0.0 {
        return None;
    }

    let cell_x = (local_x / settings.spacing).floor() as i32;
    let cell_z = (local_z / settings.spacing).floor() as i32;

    if cell_x < 0
        || cell_z < 0
        || cell_x >= settings.cells_x as i32
        || cell_z >= settings.cells_z as i32
    {
        return None;
    }

    Some(IVec2::new(cell_x, cell_z))
}

fn cell_center(cell: IVec2, settings: &RegularGridSettings, y: f32) -> Vec3 {
    let (origin_x, origin_z) = grid_origin(settings);

    Vec3::new(
        origin_x + (cell.x as f32 + 0.5) * settings.spacing,
        y,
        origin_z + (cell.y as f32 + 0.5) * settings.spacing,
    )
}

fn grid_origin(settings: &RegularGridSettings) -> (f32, f32) {
    let total_width = settings.cells_x as f32 * settings.spacing;
    let total_depth = settings.cells_z as f32 * settings.spacing;

    (-total_width * 0.5, -total_depth * 0.5)
}