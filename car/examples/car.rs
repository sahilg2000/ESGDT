use bevy::prelude::*;
use bevy_hanabi::prelude::*;
use bevy_integrator::{SimTime, Solver};
use rigid_body::plugin::RigidBodyPlugin;

use car::build::{build_car, car_startup_system};
use car::environment::build_environment;
use car::ui::{hud_setup, update_hud_system, update_weather_hud_system};
use car::setup::{camera_setup, simulation_setup};
use car::weather::{
    cycle_weather_system, setup_lighting_system, setup_rain_system, toggle_rain_system, update_environment_system, Weather,
};

fn main() {
    let car_definition = build_car();
    App::new()
        .add_plugins((
            RigidBodyPlugin {
                time: SimTime::new(0.002, 0.0, None),
                solver: Solver::RK4,
                simulation_setup: vec![simulation_setup],
                environment_setup: vec![camera_setup],
                name: "car_demo".to_string(),
            },
            HanabiPlugin,
        ))
        .insert_resource(car_definition)
        .insert_resource(Weather::Sunny)

        // build.rs
        .add_systems(Startup, car_startup_system)

        .add_systems(Update, update_hud_system) // Add this line to your app setup

        // environment.rs
        .add_systems(Startup, build_environment)
        
        // weather.rs
        .add_systems(Startup, setup_lighting_system) // Added lighting setup here

        // setup.rs
        .add_systems(Startup, hud_setup)
        .add_systems(Update, (update_hud_system, update_weather_hud_system))

        // environment.rs
        .add_systems(Startup, build_environment)
        
        // weather.rs
        .add_systems(Startup, setup_lighting_system)
        .add_systems(Startup, setup_rain_system)
        .add_systems(Update, cycle_weather_system)
        .add_systems(Update, update_environment_system)
        .add_systems(Update, toggle_rain_system)

        // setup.rs
        .add_systems(Startup, hud_setup)

        .run();
}
