use bevy::prelude::*;
use bevy_integrator::{SimTime, Solver};
use rigid_body::plugin::RigidBodyPlugin;

use car::{
    build::{build_car, car_startup_system},
    environment::build_environment,
    setup::{camera_setup, simulation_setup},
    ui::*,
    weather::*,
    logger::*,  // Add the logger module
};

fn main() {
    App::new()
        .add_plugins(RigidBodyPlugin {
            time: SimTime::new(0.002, 0.0, None),
            solver: Solver::RK4,
            simulation_setup: vec![simulation_setup],
            environment_setup: vec![camera_setup],
            name: "car_demo".to_string(),
        })
        .insert_resource(build_car())
        .insert_resource(Weather::Sunny)
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)))
        .insert_resource(InputLogger::new("car_inputs.log".to_string()))  // Add the logger resource
        .add_systems(Startup, (
            car_startup_system,
            build_environment,
            setup_lighting_system,
            setup_rain_system,
            hud_setup,
        ))
        .add_systems(Update, (
            update_speedometer_system,
            update_rpm_system,
            update_controls_system,
            cycle_weather_system,
            update_environment_system,
            toggle_rain_system,
            update_weather_system,
            input_logger_system,  // Add the logger system
        ))
        .run();
}