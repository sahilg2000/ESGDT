#![allow(dead_code)]
use bevy::{
    core_pipeline::clear_color::ClearColorConfig, input::mouse::MouseWheel, prelude::*,
    render::camera::Projection,
};
use std::f32::consts::PI;

#[derive(Resource)]
pub struct PointerOverUi(bool);

impl PointerOverUi {
    pub fn new() -> Self {
        PointerOverUi(false)
    }

    pub fn set(&mut self, value: bool) {
        self.0 = value;
    }

    pub fn or(&mut self, value: bool) {
        self.0 |= value;
    }

    pub fn check(&self) -> bool {
        self.0
    }
}
impl Default for PointerOverUi {
    fn default() -> Self {
        PointerOverUi(false)
    }
}

// This started as a copy paste from
// https://bevy-cheatbook.github.io/cookbook/pan-orbit-camera.html

#[derive(Clone)]
pub enum UpDirection {
    X,
    Y,
    Z,
}

#[derive(Component)]
pub struct AzElCamera {
    pub focus: Vec3,
    pub radius: f32,
    pub up_direction: UpDirection,
    pub azimuth: f32,
    pub elevation: f32,
}

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

pub fn az_el_camera(
    windows: Query<&mut Window>,
    mut cursor_moved: EventReader<CursorMoved>,
    mut ev_scroll: EventReader<MouseWheel>,
    input_mouse: Res<Input<MouseButton>>,
    mut query: Query<(&mut AzElCamera, &mut Transform, &Projection)>,
    pointer_over_ui: Res<PointerOverUi>,
    mut last_position: Local<Vec2>,
) {
    // get first cursor position
    let current_position = if let Some(cursor) = cursor_moved.iter().next() {
        cursor.position
    } else {
        *last_position
    };
    let delta = current_position - *last_position;
    *last_position = current_position;

    if pointer_over_ui.check() {
        return;
    }
    let cursor_sensitivity = 0.5;

    // change input mapping for orbit and panning here
    let orbit_button = MouseButton::Left;
    let pan_button = MouseButton::Middle;

    let mut pan = Vec2::ZERO;
    let mut rotation_move = Vec2::ZERO;
    let mut scroll = 0.0;

    // Handle user input here
    if input_mouse.pressed(orbit_button) {
        rotation_move += delta * cursor_sensitivity;
    } else if input_mouse.pressed(pan_button) {
        // Pan only if we're not rotating at the moment
        pan += delta * cursor_sensitivity;
    }
    for ev in ev_scroll.iter() {
        scroll += ev.y;
    }

    // update cameras
    for (mut az_el, mut transform, projection) in query.iter_mut() {
        let mut any_changes = false; // has anything changed?

        if rotation_move.length_squared() > 0.0 {
            any_changes = true;
            let window = get_primary_window_size(&windows);
            let delta_x = rotation_move.x / window.x * PI * 2.0;
            let delta_y = rotation_move.y / window.y * PI;

            az_el.azimuth -= delta_x;
            az_el.elevation += delta_y;

            az_el.elevation = az_el.elevation.max(-PI / 2.).min(PI / 2.);
            transform.rotation =
                az_el_rotation(az_el.azimuth, az_el.elevation, &az_el.up_direction);
        }

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

        if scroll.abs() > 0.0 {
            any_changes = true;
            az_el.radius -= scroll * az_el.radius * 0.2;
            // don't allow zoom to reach zero or you get stuck
            az_el.radius = az_el.radius.max(0.05);
        }

        if any_changes {
            transform.translation = az_el_translation(az_el.focus, transform.rotation, az_el.radius)
        }
    }
}

fn az_el_rotation(az: f32, el: f32, up_direction: &UpDirection) -> Quat {
    match up_direction {
        UpDirection::X => {
            let yaw = Quat::from_rotation_x(az + PI);
            let pitch = Quat::from_rotation_y(el);
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

fn az_el_translation(focus: Vec3, rotation: Quat, radius: f32) -> Vec3 {
    focus + rotation * Vec3::new(0.0, 0.0, radius)
}

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
