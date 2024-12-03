#![allow(dead_code)]

use bevy::prelude::*;
use rigid_body::joint::Joint;
use crate::weather::Weather;

#[derive(Component)]
pub struct RpmText;

#[derive(Component)]
pub struct WeatherText;


pub fn hud_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                ..default()
            },
            background_color: Color::NONE.into(),
            ..default()
        })
        .with_children(|parent| {
            // RPM Text
            parent.spawn((
                TextBundle {
                    text: Text::from_section(
                        "RPM: --", // RPM UI initial text
                        TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 30.0,
                            color: Color::WHITE,
                        },
                    ),
                    ..default()
                },
                RpmText,
            ));
            // Weather Text
            parent.spawn((
                TextBundle {
                    text: Text::from_section(
                        "Weather: --", // Weather UI initial text
                        TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 30.0,
                            color: Color::WHITE,
                        },
                    ),
                    ..default()
                },
                WeatherText,
            ));
        });
}

pub fn update_hud_system(
    mut query: Query<&mut Text, With<RpmText>>,
    joints: Query<&Joint>,                              // Query for wheel joints to calculate RPM
) {
    let mut total_rpm = 0.0;
    let count = joints.iter().count() as f64;

    for joint in joints.iter() {
        total_rpm += (joint.qd * 60.0) / (2.0 * std::f64::consts::PI);  // Calculate RPM for each wheel
    }

    let average_rpm = total_rpm / count;                           // Average RPM across all wheels

    for mut text in query.iter_mut() {
        text.sections[0].value = format!("RPM: {:.1}", average_rpm);    // Update with current average RPM
    }
}

pub fn update_weather_hud_system(
    weather: Res<Weather>,
    mut query: Query<&mut Text, With<WeatherText>>,
) {
    if weather.is_changed() {
        for mut text in query.iter_mut() {
            text.sections[0].value = format!("Weather: {:?}", *weather); // Update with current weather
        }
    }
}