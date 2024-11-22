#![allow(dead_code)]

use crate::{
    joint::{bevy_joint_positions, Joint},
    rendering::startup_rendering,
    structure::{apply_external_forces, loop_1, loop_23},
};
use bevy::{app::AppExit, prelude::*};
use bevy_integrator::{
    initialize_state, integrator_schedule, ExitEvent, PhysicsSchedule, PhysicsScheduleExt, SimTime,
    Solver,
};
use bevy_obj::ObjPlugin;

#[derive(Clone)]
pub struct RigidBodyPlugin {
    pub time: SimTime,
    pub simulation_setup: Vec<fn(&mut App)>,
    pub environment_setup: Vec<fn(&mut App)>,
    pub solver: Solver,
    pub name: String,
}

impl RigidBodyPlugin {
    pub fn setup_physics_simulation(&self, app: &mut App) {
        let schedule = create_physics_schedule();
        app.add_schedule(PhysicsSchedule, schedule)
            .insert_resource(self.time.clone())
            .insert_resource(self.solver)
            .insert_resource(FixedTime::new_from_secs(self.time.dt as f32))
            .add_systems(FixedUpdate, integrator_schedule::<Joint>);
    }
}

impl Plugin for RigidBodyPlugin {
    fn build(&self, app: &mut App) {
        self.setup_physics_simulation(app);
        app.add_event::<ExitEvent>();

        app.add_systems(
            Update,
            (time_exit_system, esc_exit_system, exit_system).chain(),
        );

        for setup in self.simulation_setup.iter() {
            setup(app);
        }
        for setup in self.environment_setup.iter() {
            setup(app);
        }

        app.add_plugins((
            DefaultPlugins.build().set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: (1920., 1080.).into(),
                    title: self.name.clone(),
                    resizable: true,
                    ..default()
                }),
                ..default()
            }),
            ObjPlugin,
        ));
        app.add_systems(PostStartup, startup_rendering)
            .add_systems(Update, bevy_joint_positions);

        app.add_systems(PostStartup, initialize_state::<Joint>);
    }
}

fn create_physics_schedule() -> Schedule {
    let mut physics_schedule = Schedule::new();
    physics_schedule
        .add_physics_systems::<Joint, _, _>((loop_1,), (apply_external_forces, loop_23).chain());

    physics_schedule
}

fn time_exit_system(time: Res<SimTime>, mut exit: EventWriter<ExitEvent>) {
    if time.is_complete() {
        exit.send(ExitEvent);
    }
}

fn esc_exit_system(
    windows: Query<&Window>,
    input: Res<Input<KeyCode>>,
    mut exit: EventWriter<ExitEvent>,
) {
    for window in windows.iter() {
        if !window.focused {
            continue;
        }

        if input.just_pressed(KeyCode::Escape) {
            exit.send(ExitEvent);
        }
    }
}

fn exit_system(mut exit: EventWriter<AppExit>, exit_request: EventReader<ExitEvent>) {
    if !exit_request.is_empty() {
        exit.send(AppExit);
    }
}
