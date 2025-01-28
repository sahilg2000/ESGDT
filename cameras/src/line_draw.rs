use bevy::prelude::*;
use crate::camera_az_el::{AzElCamera, PointerOverUi};

#[derive(Resource)]
pub struct LineDrawState {
    pub enabled: bool,
    pub last_point: Option<Vec3>,
}

impl Default for LineDrawState {
    fn default() -> Self {
        Self {
            enabled: false,
            last_point: None,
        }
    }
}

/// Toggle line-draw mode on/off
pub fn toggle_line_draw_mode_system(
    keyboard: Res<Input<KeyCode>>,
    mut line_draw_state: ResMut<LineDrawState>,
) {
    if keyboard.just_pressed(KeyCode::L) {
        line_draw_state.enabled = !line_draw_state.enabled;
        if !line_draw_state.enabled {
            line_draw_state.last_point = None;
        }
        info!(
            "Line draw mode: {}",
            if line_draw_state.enabled { "ON" } else { "OFF" }
        );
    }
}

/// If line-draw mode is on, skip orbit camera, and detect
/// clicks to draw white line segments on z=0
pub fn line_draw_system(
    windows: Query<&Window>,
    mouse: Res<Input<MouseButton>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<AzElCamera>>,
    mut line_draw_state: ResMut<LineDrawState>,
    mut commands: Commands,
    pointer_over_ui: Res<PointerOverUi>,

    // IMPORTANT: We need write-access to Mesh & Material assets
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // If not in line-draw mode or pointer is over UI, do nothing
    if !line_draw_state.enabled || pointer_over_ui.check() {
        return;
    }

    // Get the primary window
    let Ok(window) = windows.get_single() else { return; };

    // We only handle one AzElCamera. If multiple, adapt this approach.
    let Ok((camera, camera_transform)) = camera_query.get_single() else { return; };

    // On left click, compute ray-plane intersection
    if mouse.just_pressed(MouseButton::Left) {
        if let Some(cursor_pos) = window.cursor_position() {
            if let Some(world_pos) = screen_to_world_z0(cursor_pos, window, camera, camera_transform) {
                // If we have a "last_point," draw a segment
                if let Some(prev_point) = line_draw_state.last_point {
                    spawn_line_segment(
                        &mut commands,
                        prev_point,
                        world_pos,
                        &mut meshes,
                        &mut materials,
                    );
                }
                // Update last_point
                line_draw_state.last_point = Some(world_pos);
            }
        }
    }
}

/// Convert 2D screen coords -> a 3D point on z=0
fn screen_to_world_z0(
    screen_pos: Vec2,
    window: &Window,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Option<Vec3> {
    let wnd_size = Vec2::new(window.width(), window.height());
    let ndc = (screen_pos / wnd_size) * 2.0 - Vec2::ONE;
    let ndc = Vec3::new(ndc.x, -ndc.y, 1.0); // flip Y

    let camera_matrix = camera_transform.compute_matrix();
    let proj_matrix = camera.projection_matrix();
    let ndc_to_world = camera_matrix * proj_matrix.inverse();
    let world_pos4 = ndc_to_world * ndc.extend(1.0);

    if world_pos4.w.abs() < f32::EPSILON {
        return None;
    }
    let world_pos3 = world_pos4.truncate() / world_pos4.w;

    let origin = camera_transform.translation();
    let dir = (world_pos3 - origin).normalize();

    if dir.z.abs() < f32::EPSILON {
        return None; // parallel to plane
    }
    let t = -origin.z / dir.z;
    if t < 0.0 {
        return None; // behind camera
    }
    Some(origin + dir * t)
}

/// Spawns a thin rectangular "line" from p1->p2
fn spawn_line_segment(
    commands: &mut Commands,
    p1: Vec3,
    p2: Vec3,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let segment = p2 - p1;
    let length = segment.length();
    if length < f32::EPSILON {
        return;
    }
    let mid = p1 + 0.5 * segment;
    let angle = segment.y.atan2(segment.x);
    let rotation = Quat::from_rotation_z(angle);

    let thickness = 0.5;

    // CREATE a Mesh handle from shape::Box
    let mesh_handle = meshes.add(Mesh::from(bevy::prelude::shape::Box::new(1.0, 1.0, 0.02)));

    // CREATE a Material handle for a white color
    let material_handle = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        ..default()
    });

    commands.spawn((
        PbrBundle {
            transform: Transform {
                translation: mid,
                rotation,
                scale: Vec3::new(length, thickness, 1.0),
            },
            mesh: mesh_handle,         
            material: material_handle, 
            ..default()
        },
        Name::new("LineSegment"),
    ));
}
