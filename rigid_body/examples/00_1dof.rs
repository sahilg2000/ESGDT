use std::f32::consts::PI;

use bevy::prelude::*;

use bevy_integrator::{PhysicsSchedule, PhysicsSet, SimTime, Solver};
use cameras::camera_az_el::{self, camera_builder};
use rigid_body::{
    definitions::{MeshDef, MeshTypeDef, TransformDef},
    joint::{Base, Joint},
    plugin::RigidBodyPlugin,
    sva::{Inertia, Matrix, Motion, Vector, Xform},
};

// Main function
fn main() {
    // Create App
    App::new()
        .add_plugins(RigidBodyPlugin {
            time: SimTime::new(0.002, 0.0, Some(10.)),
            solver: Solver::RK4,
            simulation_setup: vec![],
            environment_setup: vec![camera_setup],
            name: "example 00_1dof".to_string(),
        })
        .add_systems(
            PhysicsSchedule,
            (spring_damper_system,).in_set(PhysicsSet::Evaluate),
        )
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
                z: 1.,
            },
            -90.0_f32.to_radians(),
            10.0_f32.to_radians(),
            10.,
            camera_az_el::UpDirection::Z,
        ),
    )
    .add_systems(Update, (camera_az_el::az_el_camera,)); // setup the camera
}

fn startup_system(mut commands: Commands) {
    let base = Joint::base(Motion::new([0., 0., 9.81], [0., 0., 0.]));
    let base_id = commands.spawn((base, Base)).id();

    let mass: f64 = 10.;
    let stiffness = 100.;
    let damping = 0.1 * 2. * (mass * stiffness).sqrt();

    let inertia = Inertia::new(
        mass,
        Vector::new(0.0, 0.0, 0.0),
        Matrix::from_diagonal(&Vector::new(10., 10., 10.)),
    );
    let px = Joint::pz("body_pz".to_string(), inertia, Xform::identity());
    let mut px_e = commands.spawn((
        px,
        SpringDamper::new(stiffness, damping),
        MeshDef {
            mesh_type: MeshTypeDef::Box {
                dimensions: [1., 1., 1.],
            },
            transform: TransformDef::Identity,
            color: Color::rgb(0.0, 0.0, 1.0),
        },
    ));
    px_e.set_parent(base_id);
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
            rotation: Quat::from_rotation_x(-PI / 4.) * Quat::from_rotation_y(-PI / 4.),

            ..default()
        },

        ..default()
    });
}

#[derive(Component)]
pub struct SpringDamper {
    stiffness: f64,
    damping: f64,
}

impl SpringDamper {
    pub fn new(stiffness: f64, damping: f64) -> Self {
        Self { stiffness, damping }
    }
}

pub fn spring_damper_system(mut joints: Query<(&mut Joint, &SpringDamper)>) {
    for (mut joint, spring_damper) in joints.iter_mut() {
        joint.tau -= spring_damper.stiffness * joint.q + spring_damper.damping * joint.qd;
    }
}
