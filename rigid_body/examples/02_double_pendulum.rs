use std::f32::consts::PI as PI32;
use std::f64::consts::PI as PI64;

use bevy::prelude::*;

use bevy_integrator::{SimTime, Solver};
use cameras::camera_az_el::{self, camera_builder};
use rigid_body::{
    definitions::{MeshDef, MeshTypeDef, TransformDef},
    // forces::spring_damper_system,
    joint::{Base, Joint},
    plugin::RigidBodyPlugin,
    sva::{Inertia, Matrix, Motion, Vector, Xform},
};

// Main function
fn main() {
    // Create App
    App::new()
        .add_plugins(RigidBodyPlugin {
            time: SimTime::new(0.002, 0.0, Some(60.)),
            solver: Solver::RK4,
            simulation_setup: vec![],
            environment_setup: vec![camera_setup],
            name: "example 02_double_pendulum".to_string(),
        })
        .add_systems(Startup, startup_system)
        .add_systems(Startup, environment_startup_system)
        .run();
}

pub fn camera_setup(app: &mut App) {
    app.add_systems(
        Startup,
        camera_builder(
            Vec3 {
                x: 0.,
                y: 0.,
                z: -1.,
            },
            180.0_f32.to_radians(),
            0.0_f32.to_radians(),
            3.5,
            camera_az_el::UpDirection::Z,
        ),
    )
    .add_systems(Update, (camera_az_el::az_el_camera,)); // setup the camera
}

fn startup_system(mut commands: Commands) {
    let base = Joint::base(Motion::new([0., 0., 9.81], [0., 0., 0.]));
    let base_id = commands.spawn((base, Base)).id();

    let mass: f64 = 1.;
    let width: f64 = 0.05;
    let length: f64 = 1.0;
    let moi_z = 1. / 12. * mass * 2. * width.powi(2);
    let moi_xy = 1. / 12. * mass * (width.powi(2) + length.powi(2));

    let inertia = Inertia::new(
        mass,
        Vector::new(0.0, 0.0, -length / 2.),
        Matrix::from_diagonal(&Vector::new(moi_xy, moi_xy, moi_z)),
    );

    // first pendulum link
    let mut ry0 = Joint::ry("body_ry0".to_string(), inertia.clone(), Xform::identity());
    ry0.q = 0.5 * PI64;
    let mesh_def = MeshDef {
        mesh_type: MeshTypeDef::Box {
            dimensions: [width as f32, width as f32, length as f32],
        },
        transform: TransformDef::Position {
            x: 0.,
            y: 0.,
            z: -length / 2.,
        },
        color: Color::rgb(1.0, 0.0, 0.0),
    };
    let mut ry0_e = commands.spawn((ry0, mesh_def));
    ry0_e.set_parent(base_id);

    let ry0_id = ry0_e.id();

    // second pendulum link
    let transform = Xform::posz(-1.0);
    let ry1 = Joint::ry("body_ry1".to_string(), inertia.clone(), transform);
    let mesh_def_ry1 = MeshDef {
        mesh_type: MeshTypeDef::Box {
            dimensions: [width as f32, width as f32, length as f32],
        },
        transform: TransformDef::Position {
            x: 0.,
            y: 0.,
            z: -length / 2.,
        },
        color: Color::rgb(0.0, 0.0, 1.0),
    };
    let mut ry1_e = commands.spawn((ry1, mesh_def_ry1));
    ry1_e.set_parent(ry0_id);
}

fn environment_startup_system(mut commands: Commands) {
    commands.insert_resource(AmbientLight {
        color: Color::rgb(0.9, 0.9, 1.0),
        brightness: 0.4,
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            illuminance: 10000.0, // lux
            shadow_depth_bias: 0.3,
            shadow_normal_bias: 1.0,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, 10.0),
            rotation: Quat::from_rotation_x(-PI32 / 4.) * Quat::from_rotation_y(-PI32 / 4.),

            ..default()
        },

        ..default()
    });
}
