use bevy::prelude::*;
use bevy_hanabi::prelude::*;
use std::f32::consts::PI;
use bevy::pbr::{CascadeShadowConfigBuilder, DirectionalLightShadowMap};

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
        // Cycle to the next weather state
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
        // Update ambient light
        let (ambient_color, ambient_brightness) = match *weather {
            Weather::Sunny => (Color::rgb(1.0, 1.0, 1.0), 0.5),
            Weather::Cloudy => (Color::rgb(0.6, 0.6, 0.7), 0.3),
            Weather::Rain => (Color::rgb(0.6, 0.6, 0.7), 0.3),
            Weather::Night => (Color::rgb(0.2, 0.2, 0.3), 0.1),
        };
        ambient_light.color = ambient_color;
        ambient_light.brightness = ambient_brightness;

        // Update directional light
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

        println!("Environment updated for weather: {:?}", *weather);
    }
}

#[derive(Resource)]
pub struct RainEffect {
    pub entity: Entity,
}

pub fn setup_rain_system(
    mut commands: Commands,
    mut effects: ResMut<Assets<EffectAsset>>,
) {
    let mut module = Module::default();

    // Define expressions
    let _position = module.lit(Vec3::ZERO);
    let radius = module.lit(100.0);
    let center = module.lit(Vec3::ZERO);
    let velocity_center = module.lit(Vec3::new(0.0, -1.0, 0.0));
    let speed = module.lit(50.0);
    let accel = module.lit(Vec3::new(0.0, -9.81, 0.0));

    // Define the rain particle effect
    let effect = EffectAsset::new(
        32768,
        Spawner::rate(5000.0.into()),
        module,
    )
    .with_name("Rain".to_string())
    .init(SetPositionSphereModifier {
        center,
        radius,
        dimension: ShapeDimension::Surface,
    })
    .init(SetVelocitySphereModifier {
        center: velocity_center,
        speed,
    })
    .update(AccelModifier::new(accel))
    .render(BillboardModifier {})
    .render(ColorOverLifetimeModifier {
        gradient: Gradient::constant(Vec4::new(0.5, 0.5, 1.0, 0.3)),
    });
    .render(SizeOverLifetimeModifier {
        gradient: Gradient::constant(0.1), // Adjust size as needed
    });

    let effect_handle = effects.add(effect);

    let entity = commands
        .spawn((
            Name::new("RainEffect"),
            ParticleEffectBundle {
                effect: ParticleEffect::new(effect_handle),
                transform: Transform::from_translation(Vec3::new(0.0, 50.0, 0.0)),
                visibility: Visibility::Visible, 
                ..default()
            },
        ))
        .id();

    commands.insert_resource(RainEffect { entity });
}

pub fn toggle_rain_system(
    weather: Res<Weather>,
    rain_effect: Res<RainEffect>,
    mut query: Query<&mut Visibility>,
) {
    if weather.is_changed() {
        if let Ok(mut visibility) = query.get_mut(rain_effect.entity) {
            if *weather == Weather::Rain {
                *visibility = Visibility::Visible;
            } else {
                *visibility = Visibility::Hidden;
            }
        }
    }
}