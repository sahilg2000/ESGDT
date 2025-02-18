use bevy::prelude::*;
use bevy::pbr::{CascadeShadowConfigBuilder, DirectionalLightShadowMap};
use std::f32::consts::PI;

#[derive(Resource, PartialEq, Eq, Clone, Copy, Debug)]
pub enum Weather {
    Sunny,
    Cloudy,
    Rain,
    Night,
}

pub fn setup_lighting_system(
    mut commands: Commands,
    weather: Res<Weather>,
) {
    // Set ambient light
    let (ambient_color, ambient_brightness) = match *weather {
        Weather::Sunny => (Color::rgb(1.0, 1.0, 1.0), 0.5),
        Weather::Cloudy => (Color::rgb(0.6, 0.6, 0.7), 0.3),
        Weather::Rain => (Color::rgb(0.6, 0.6, 0.7), 0.3),
        Weather::Night => (Color::rgb(0.2, 0.2, 0.3), 0.1),
    };
    commands.insert_resource(AmbientLight {
        color: ambient_color,
        brightness: ambient_brightness,
    });

    // Set directional light
    let (directional_light_color, illuminance) = match *weather {
        Weather::Sunny => (Color::rgb(1.0, 1.0, 0.9), 100000.0),
        Weather::Cloudy => (Color::rgb(0.7, 0.7, 0.8), 50000.0),
        Weather::Rain => (Color::rgb(0.7, 0.7, 0.8), 50000.0),
        Weather::Night => (Color::rgb(0.2, 0.2, 0.5), 5000.0),
    };
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: directional_light_color,
            shadows_enabled: true,
            illuminance,
            shadow_depth_bias: 0.3,
            shadow_normal_bias: 1.0,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, 10.0),
            rotation: Quat::from_rotation_x(-PI / 4.) * Quat::from_rotation_y(-PI / 4.),
            ..default()
        },
        cascade_shadow_config: CascadeShadowConfigBuilder {
            num_cascades: 4,
            minimum_distance: 1.,
            maximum_distance: 300.0,
            first_cascade_far_bound: 5.0,
            overlap_proportion: 0.3,
        }
        .into(),
        ..default()
    });

    // Insert the shadow map resource
    commands.insert_resource(DirectionalLightShadowMap { size: 4 * 1024 });
}

pub fn cycle_weather_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut weather: ResMut<Weather>,
) {
    if keyboard_input.just_pressed(KeyCode::P) {
        *weather = match *weather {
            Weather::Sunny => Weather::Cloudy,
            Weather::Cloudy => Weather::Rain,
            Weather::Rain => Weather::Night,
            Weather::Night => Weather::Sunny,
        };
        println!("Weather changed to: {:?}", *weather);
    }
}

pub fn update_environment_system(
    weather: Res<Weather>,
    mut ambient_light: ResMut<AmbientLight>,
    mut query: Query<&mut DirectionalLight>,
) {
    if weather.is_changed() {
        let (ambient_color, ambient_brightness) = match *weather {
            Weather::Sunny => (Color::rgb(1.0, 1.0, 1.0), 0.5),
            Weather::Cloudy => (Color::rgb(0.6, 0.6, 0.7), 0.3),
            Weather::Rain => (Color::rgb(0.6, 0.6, 0.7), 0.3),
            Weather::Night => (Color::rgb(0.2, 0.2, 0.3), 0.1),
        };
        ambient_light.color = ambient_color;
        ambient_light.brightness = ambient_brightness;

        let (directional_light_color, illuminance) = match *weather {
            Weather::Sunny => (Color::rgb(1.0, 1.0, 0.9), 100000.0),
            Weather::Cloudy => (Color::rgb(0.7, 0.7, 0.8), 50000.0),
            Weather::Rain => (Color::rgb(0.7, 0.7, 0.8), 50000.0),
            Weather::Night => (Color::rgb(0.2, 0.2, 0.5), 5000.0),
        };
        for mut dir_light in query.iter_mut() {
            dir_light.color = directional_light_color;
            dir_light.illuminance = illuminance;
        }
    }
}

// Placeholder functions to maintain API compatibility
pub fn setup_rain_system() {}
pub fn toggle_rain_system() {}