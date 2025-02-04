#![allow(dead_code)]
use bevy::{
    core_pipeline::clear_color::ClearColorConfig, input::mouse::MouseWheel, prelude::*,
    render::camera::Projection,
};
use std::f32::consts::PI;
use crate::line_draw::LineDrawState; 

// A resource to track if the pointer (mouse) is over a UI element, could be used to prevent camera movement when over future implemented UI.
#[derive(Resource)]
pub struct PointerOverUi(bool);

impl PointerOverUi {
    // Initialize with "not over UI"
    pub fn new() -> Self {
        PointerOverUi(false) 
    }

    // Set the state to the given value "over UI" or "not over UI" 
    pub fn set(&mut self, value: bool) {
        self.0 = value;  
        }

    // Logical OR with the current value, useful for UI checks from multiple sources.
    pub fn or(&mut self, value: bool) {
        self.0 |= value;
    }

    // Retrieve the current value
    pub fn check(&self) -> bool {
        self.0
    }
}

// Default state is "not over UI"
impl Default for PointerOverUi {
    fn default() -> Self {
        PointerOverUi(false)
    }
}

// This started as a copy paste from
// https://bevy-cheatbook.github.io/cookbook/pan-orbit-camera.html

// Define the possible "up" directions for the camera
#[derive(Clone)]
pub enum UpDirection {
    X,
    Y,
    Z,
}

// Component representing the state and properties of the camera
#[derive(Component)]
pub struct AzElCamera {
    pub focus: Vec3, // Point the camera is focused on
    pub radius: f32, // Distance from the focus point
    pub up_direction: UpDirection, // Which axes is "up" for the camera
    pub azimuth: f32, // Horizontal rotation angle
    pub elevation: f32, // Vertical rotation angle
    
}

 // Default values for the camera
impl Default for AzElCamera {
    fn default() -> Self {
        AzElCamera {
            focus: Vec3::ZERO,
            radius: 10.,
            up_direction: UpDirection::Y,
            azimuth: 0.,
            elevation: 0.,
        }
    }
}

// Main function handling camera movement, such as rotation, panning, and zoom
pub fn az_el_camera(
    windows: Query<&mut Window>,
    mut cursor_moved: EventReader<CursorMoved>, // Tracks mouse movement
    mut ev_scroll: EventReader<MouseWheel>, // Tracks mouse scroll
    input_mouse: Res<Input<MouseButton>>, // Tracks mouse button presses
    mut query: Query<(&mut AzElCamera, &mut Transform, &Projection)>,
    pointer_over_ui: Res<PointerOverUi>, // Tracks if pointer is over UI
    mut last_position: Local<Vec2>, // Tracks last known cursor position
    line_draw_state: Res<LineDrawState>,
) {

    // If line-draw is enabled, skip camera orbit entirely
    if line_draw_state.enabled {
        return;
    }
    
    // Retrieve the current mouse cursor position.
    // If a cursor movement event exists (`CursorMoved`), use its position.
    // Otherwise, return to the cursor last position .
    let current_position = if let Some(cursor) = cursor_moved.iter().next() {
        cursor.position
    } else {
        *last_position
    };

    // Calculate the change (delta) in cursor position, this represents how much the cursor moved.
    // Update the last known cursor position.
    let delta = current_position - *last_position;
    *last_position = current_position;

    // Exit if the pointer is over UI
    if pointer_over_ui.check() {
        return;
    }
    
    let cursor_sensitivity = 0.5;

    // change input mapping for orbit and panning here
    let orbit_button = MouseButton::Left;
    let pan_button = MouseButton::Middle;

    let mut pan = Vec2::ZERO; // Track panning offset
    let mut rotation_move = Vec2::ZERO; // Track rotation movement
    let mut scroll = 0.0; // Track zoom level changes

    // Handle input for orbiting or panning
    if input_mouse.pressed(orbit_button) {
        rotation_move += delta * cursor_sensitivity; 
    } else if input_mouse.pressed(pan_button) {
        // Pan only if we're not rotating at the moment
        pan += delta * cursor_sensitivity;
    }

    // Handle zoom input
    for ev in ev_scroll.iter() {
        scroll += ev.y;
    }

    // update cameras
    for (mut az_el, mut transform, projection) in query.iter_mut() {
        let mut any_changes = false; // Tracks if updates are made to the camera

        // Handle rotation based on cursor movement
        if rotation_move.length_squared() > 0.0 {
            any_changes = true;
            let window = get_primary_window_size(&windows);  // Get the primary window size to normalize rotation based on screen dimensions
            let delta_x = rotation_move.x / window.x * PI * 2.0;
            let delta_y = rotation_move.y / window.y * PI;

            az_el.azimuth -= delta_x;
            az_el.elevation += delta_y;

            az_el.elevation = az_el.elevation.max(-PI / 2.).min(PI / 2.); // Max elevation to avoid camera from flipping
            transform.rotation =
                az_el_rotation(az_el.azimuth, az_el.elevation, &az_el.up_direction); // Update rotation
        }
        
        // Handle panning based on cursor movement
        if pan.length_squared() > 0.0 {
            any_changes = true;
            // make panning distance independent of resolution and FOV,
            let window = get_primary_window_size(&windows);
            if let Projection::Perspective(projection) = projection {
                pan *= Vec2::new(projection.fov * projection.aspect_ratio, projection.fov) / window;
            }
            // translate by local axes
            let mat = Mat3::from_quat(transform.rotation);
            let left = -mat.x_axis * pan.x;
            let up = mat.y_axis * pan.y;
            // make panning proportional to distance away from focus point
            let translation = (left + up) * az_el.radius;
            az_el.focus += translation;
        }

        // Handle zooming
        if scroll.abs() > 0.0 {
            any_changes = true;
            // Adjust radius based on scroll input
            az_el.radius -= scroll * az_el.radius * 0.2;
            // don't allow zoom to reach zero or you get stuck
            az_el.radius = az_el.radius.max(0.05);
        }

        // Update camera position if any changes were made
        if any_changes {
            transform.translation = az_el_translation(az_el.focus, transform.rotation, az_el.radius)
        }
    }
}

// Calculate rotation for the camera based on azimuth and elevation
fn az_el_rotation(az: f32, el: f32, up_direction: &UpDirection) -> Quat {
    match up_direction {
        // If the X-axis is the "up" direction
        UpDirection::X => {
            // Rotate around the X-axis for azimuth (horizontal rotation)
            let yaw = Quat::from_rotation_x(az + PI);
            // Rotate around the Y-axis for elevation (vertical rotation)
            let pitch = Quat::from_rotation_y(el);
            // Combine yaw, pitch, and an additional rotation to align the Z-axis
            yaw * pitch * Quat::from_rotation_z(-PI / 2.)
        }
        UpDirection::Y => {
            let yaw = Quat::from_rotation_y(az);
            let pitch = Quat::from_rotation_z(-el);
            yaw * pitch * Quat::from_rotation_y(-PI / 2.)
        }
        UpDirection::Z => {
            let yaw = Quat::from_rotation_z(az);
            let pitch = Quat::from_rotation_x(PI / 2. - el);
            yaw * pitch
        }
    }
}

// Calculate the camera's position based on its focus, rotation, and radius
fn az_el_translation(focus: Vec3, rotation: Quat, radius: f32) -> Vec3 {
    focus + rotation * Vec3::new(0.0, 0.0, radius)
}

// Get the size of the primary window
fn get_primary_window_size(windows: &Query<&mut Window>) -> Vec2 {
    let window = windows.get_single().unwrap();
    let window = Vec2::new(window.width() as f32, window.height() as f32);
    window
}

// /// Spawn a camera like this
// pub fn spawn_camera(mut commands: Commands) {
//     let translation = Vec3::new(20.0, -40., 0.);
//     let focus = Vec3::new(20.0, 0., 0.);
//     let radius = translation.length();

//     commands
//         .spawn_bundle(Camera3dBundle {
//             transform: Transform::from_translation(translation).looking_at(focus, Vec3::Z),
//             ..Default::default()
//         })
//         .insert(AzElCamera {
//             radius,
//             focus,
//             ..Default::default()
//         });
// }

// Function to spawn a camera
pub fn camera_builder(
    focus: Vec3,
    az: f32,
    el: f32,
    radius: f32,
    up_direction: UpDirection,
) -> impl Fn(Commands) -> () {
    let spawn_camera = move |mut commands: Commands| {
        let rotation = az_el_rotation(az, el, &up_direction);
        let translation = az_el_translation(focus, rotation, radius);
        let transform = Transform {
            translation,
            rotation,
            ..default()
        };

        commands
            .spawn(Camera3dBundle {
                transform,
                camera_3d: Camera3d {
                    clear_color: ClearColorConfig::Custom(Color::BLACK),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(AzElCamera {
                radius,
                focus,
                up_direction: up_direction.clone(),
                azimuth: az,
                elevation: el,
            });

        commands.init_resource::<PointerOverUi>()
    };
    spawn_camera
}
