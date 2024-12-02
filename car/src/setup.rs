#![allow(dead_code)]

use bevy::prelude::*;
use bevy_integrator::{PhysicsSchedule, PhysicsSet};
use rigid_body::joint::Joint;

use crate::{
    control::user_control_system,
    physics::{
        brake_wheel_system, driven_wheel_lookup_system, steering_curvature_system, steering_system,
        suspension_system,
    },
    tire::point_tire_system,
};

use super::control::CarControl;
use cameras::{
    camera_az_el::{self, camera_builder},
    control::camera_parent_system,
};

pub fn simulation_setup(app: &mut App) {
    app.add_systems(
        PhysicsSchedule,
        (steering_system, steering_curvature_system).in_set(PhysicsSet::Pre),
    )
    .add_systems(
        PhysicsSchedule,
        (
            suspension_system,
            point_tire_system,
            driven_wheel_lookup_system,
            brake_wheel_system,
        )
            .in_set(PhysicsSet::Evaluate),
    )
    .add_systems(Update, (user_control_system,))
    .init_resource::<CarControl>();
}

pub fn camera_setup(app: &mut App) {
    app.add_systems(
        Startup,
        camera_builder(
            Vec3 {
                x: 0.,
                y: 0.,
                z: 1.,
            },
            -90.0_f32.to_radians(),
            10.0_f32.to_radians(),
            20.,
            camera_az_el::UpDirection::Z,
        ),
    )
    .add_systems(Update, (camera_az_el::az_el_camera, camera_parent_system)); // setup the camera
}

pub fn hud_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(TextBundle {
        style: Style {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        text: Text::from_section(
            "RPM: --", // Placeholder text
            TextStyle {
                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                font_size: 30.0,
                color: Color::WHITE,
            },
        ),
        ..default()
    });
}

// Updates the text with current RPM
pub fn update_hud_system(
    mut query: Query<&mut Text>,
    joints: Query<&Joint>,                      // Query for wheel joints to calculate RPM
) {
    let mut total_rpm = 0.0;
    let count = joints.iter().count() as f64;

    for joint in joints.iter() {
        total_rpm += (joint.qd * 60.0) / (2.0 * std::f64::consts::PI);      // Calculate RPM for each wheel
    }

    let average_rpm = total_rpm / count; // Average RPM across all wheels

    for mut text in query.iter_mut() {
        text.sections[0].value = format!("RPM: {:.1}", average_rpm);        // Update with current average RPM
    }
}
