use bevy::prelude::*;             
use grid_terrain::GridTerrain; 

use cameras::camera_az_el::{AzElCamera, PointerOverUi};

// A resource that tracks whether the user is in "line-draw mode" (`enabled`),
// along with the last 3D point (`last_point`) we clicked.
#[derive(Resource)]
pub struct LineDrawState {
    pub last_point: Option<Vec3>,
}

impl Default for LineDrawState {
    fn default() -> Self {
        Self {
            last_point: None,
        }
    }
}

// A system that responds to user clicks (left mouse button) when in line-draw mode.
// 
// 1) A ray from the camera is casted into the 3D world to find where it hits the terrain.
// 2) If there exist a previously clicked point (`last_point`), subdivide the line from that point
//    to the new point in many segments so it sticks to the terrain's uneven surface.
// 3) Place small rectangular boxes for each line segment.
pub fn line_draw_system(
    windows: Query<&Window>,                              // Query for the primary window
    mouse: Res<Input<MouseButton>>,                      // Tracks mouse button presses
    keyboard: Res<Input<KeyCode>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<AzElCamera>>, 
    mut line_draw_state: ResMut<LineDrawState>,          // Our resource controlling line-draw mode
    mut commands: Commands,                              // For spawning 3D objects
    pointer_over_ui: Res<PointerOverUi>,                 // If pointer is over UI, we ignore clicks
    mut meshes: ResMut<Assets<Mesh>>,                    // Asset storage for Mesh objects
    mut materials: ResMut<Assets<StandardMaterial>>,      // Asset storage for Material objects
    grid_terrain: Res<GridTerrain>,                      // The terrain resource for collision
) {

    // Check if user presses 'R' to reset line drawing
    if keyboard.just_pressed(KeyCode::R) {
        line_draw_state.last_point = None;
        return;
    }

    // If pointer is over UI, do nothing
    if pointer_over_ui.check() {
        return;
    }

    let Ok(window) = windows.get_single() else { return; };

    // Attempt to get the camera entity (with AzElCamera) and its transform
    let Ok((camera, camera_transform)) = camera_query.get_single() else { return; };

    // On LEFT mouse click, we start a raycast
    if mouse.just_pressed(MouseButton::Right) {
        // Grab the 2D cursor position in window space
        if let Some(cursor_pos) = window.cursor_position() {
            // Convert from 2D cursor position -> a ray (origin, direction) in 3D
            if let Some((origin, dir)) = screen_to_world_ray(cursor_pos, window, camera, camera_transform) {
                if let Some(world_pos) = raycast_terrain(origin, dir, 200.0, 0.05, &grid_terrain) {
                    // If we had a previous point, create a line from that old point to the new one
                    if let Some(prev_point) = line_draw_state.last_point {
                        spawn_line_hugging_terrain(
                            &mut commands,
                            prev_point,
                            world_pos,
                            &grid_terrain,
                            &mut meshes,
                            &mut materials,
                        );
                    }
                    // Store this new point for future line segments
                    line_draw_state.last_point = Some(world_pos);
                }
            }
        }
    }
}

// Converts a 2D screen coordinate into a ray in world space.
// 
// - `camera_transform`: The camera's current position & rotation in the world
// 
// Returns `None` if the projection matrix cannot be inverted or if we can't determine
// a valid world position.
fn screen_to_world_ray(
    screen_pos: Vec2,
    window: &Window,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Option<(Vec3, Vec3)> {
    // Convert from pixel coords to Normalized Device Coordinates (NDC)
    let wnd_size = Vec2::new(window.width(), window.height());
    let ndc = (screen_pos / wnd_size) * 2.0 - Vec2::ONE;
    let ndc = Vec3::new(ndc.x, -ndc.y, 1.0);

    // Combine the camera's global transform with the inverse of its projection
    let cam_matrix = camera_transform.compute_matrix();
    let proj_matrix = camera.projection_matrix();
    let ndc_to_world = cam_matrix * proj_matrix.inverse();

    // Multiply (x, y, 1.0, 1.0) in homogeneous coords
    let world_pos4 = ndc_to_world * ndc.extend(1.0);

    // If w is near zero, the point is not in front of the camera
    if world_pos4.w.abs() < f32::EPSILON {
        return None;
    }

    let world_pos3 = world_pos4.truncate() / world_pos4.w;

    // The 'origin' is the camera's position,
    // The 'direction' is the vector from camera to that 3D point
    let origin = camera_transform.translation();
    let direction = (world_pos3 - origin).normalize();
    Some((origin, direction))
}

// Move along a ray from `origin` in `dir` increments of `step_size`, up to `max_dist`.
// Calls `terrain.interference_f32(...)` to see if there's a collision at each step.
// If collision is found, we compute the collision point and move it up a little
// to avoid z-fighting, returning it as our final intersection point.
fn raycast_terrain(
    origin: Vec3,
    dir: Vec3,
    max_dist: f32,
    step_size: f32,
    terrain: &GridTerrain,
) -> Option<Vec3> {
    let mut dist = 0.0;
    while dist < max_dist {
        // The point in space we want to test
        let test_point = origin + dir * dist;

        // Convert Vec3 into Vector for interference()
        let test_point_vector = rigid_body::sva::Vector::new(
            test_point.x as f64,
            test_point.y as f64,
            test_point.z as f64,
        );

        // Ask the terrain if there's a collision at that point
        if let Some(inter) = terrain.interference(test_point_vector) {
            let offset = 0.01; // nudge above the surface by 1 cm
            let collision_point = test_point
                + Vec3::new(
                    inter.magnitude as f32 * inter.normal.x as f32,
                    inter.magnitude as f32 * inter.normal.y as f32,
                    inter.magnitude as f32 * inter.normal.z as f32,
                );
            let normal = Vec3::new(
                inter.normal.x as f32,
                inter.normal.y as f32,
                inter.normal.z as f32,
            );

            // Push the contact point slightly above the collision normal
            let final_point = collision_point + offset * normal.normalize();
            return Some(final_point);
        }

        // Advance the ray by step_size and keep checking
        dist += step_size;
    }
    // No collision found within max_dist
    None
}

/// Subdivides the line from `p1` to `p2` into segments so each piece can follow
/// the terrain's ups and downs. Each sub-segment is individually "snapped" onto the terrain
/// so the entire line sticks to the ground or slopes/bumps.
fn spawn_line_hugging_terrain(
    commands: &mut Commands,
    p1: Vec3,
    p2: Vec3,
    terrain: &GridTerrain,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    // We subdivide into this many small line pieces
    let total_subdiv = 100;
    let segment = p2 - p1;

    // Define how far apart the parallel lines
    let offset_distance = 2.0; 

    let mut last_left = None;
    let mut last_right = None;

    for i in 0..=total_subdiv {
        let t = i as f32 / total_subdiv as f32;
        let rough = p1 + segment * t;

        // Compute perpendicular vector based on previous segment to avoid misalignment
        let segment_dir = (p2 - p1).normalize();
        let perpendicular = Vec3::new(-segment_dir.y, segment_dir.x, 0.0).normalize();

        let rough_left = rough + perpendicular * (offset_distance * 0.5);
        let rough_right = rough - perpendicular * (offset_distance * 0.5);

        // Snap both approximate points onto the terrain
        if let Some(surf_left) = snap_point_to_terrain(rough_left, terrain) {
            if let Some(prev_left) = last_left {
                spawn_line_segment(commands, prev_left, surf_left, meshes, materials);
            }
            last_left = Some(surf_left);
        }

        if let Some(surf_right) = snap_point_to_terrain(rough_right, terrain) {
            if let Some(prev_right) = last_right {
                spawn_line_segment(commands, prev_right, surf_right, meshes, materials);
            }
            last_right = Some(surf_right);
        }
    }
}

// Given an approximate point in the air (`rough`), cast a small ray downward
// so we can pin it exactly to the terrain surface. 
//
/// This is done by calling `raycast_terrain` from 2 units above to 10 units below.
fn snap_point_to_terrain(
    rough: Vec3,
    terrain: &GridTerrain,
) -> Option<Vec3> {
    let above = rough + Vec3::new(0., 0., 2.0);
    let dir_down = Vec3::new(0., 0., -1.);
    let max_dist = 10.0;
    let step = 0.05;
    // If there's a collision, we get a snapped point
    raycast_terrain(above, dir_down, max_dist, step, terrain)
}

// Spawns a single rectangular "line segment" between p1 and p2.
fn spawn_line_segment(
    commands: &mut Commands,
    p1: Vec3,
    p2: Vec3,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let segment = p2 - p1;
    let length = segment.length();
    // If there's no length, skip
    if length < f32::EPSILON {
        return;
    }

    // Midpoint for translation
    let mid = p1 + 0.5 * segment;
    // Determine how much to rotate in XY plane so the box aligns with p1->p2
    let angle_xy = segment.y.atan2(segment.x);
    let rotation = Quat::from_rotation_z(angle_xy);

    // A small thickness for the line, and a Z scale of 0.02 for a slightly raised effect
    let thickness = 0.2; 
    // We use a unit shape::Box, then scale it to length × thickness × 0.02
    let mesh_handle = meshes.add(Mesh::from(shape::Box::new(1.0, 1.0, 0.02)));

    // Simple white material so the line is visible
    let material_handle = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        ..default()
    });

    commands.spawn(PbrBundle {
        transform: Transform {
            translation: mid,                          // The center in world coords
            rotation,                                  // Rotates box so it lines up with p1->p2
            scale: Vec3::new(length, thickness, 0.02), // Scale in X to line length, Y to thickness
        },
        mesh: mesh_handle,
        material: material_handle,
        ..default()
    });
}
