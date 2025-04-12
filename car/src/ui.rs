use bevy::prelude::*;
use rigid_body::joint::Joint;
use crate::{
    control::CarControl,
    weather::Weather,
};

#[derive(Component)]
pub struct SpeedometerText;

#[derive(Component)]
pub struct RpmText;

#[derive(Component)]
pub struct ControlsText;

#[derive(Component)]
pub struct WeatherText;
//
pub fn hud_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Px(170.0),  
                height: Val::Px(180.0),
                ..default()
            },
            background_color: Color::BLACK.into(),
            ..default()
        })
        .with_children(|parent| {
            // Stats panel
            parent.spawn(NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    left: Val::Px(10.0),
                    top: Val::Px(10.0),
                    width: Val::Px(200.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(5.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
                ..default()
            })
            .with_children(|parent| {
                // Speed display
                parent.spawn((
                    TextBundle::from_section(
                        "0 MPH",
                        TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 30.0,
                            color: Color::rgb(0.0, 0.5, 1.0),
                        },
                    ),
                    SpeedometerText,
                ));

                // RPM display
                parent.spawn((
                    TextBundle::from_section(
                        "0 RPM",
                        TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 30.0,
                            color: Color::rgb(0.0, 0.5, 1.0),
                        },
                    ),
                    RpmText,
                ));

                // Controls
                parent.spawn((
                    TextBundle::from_sections([
                        TextSection::new(
                            "Controls:\n",
                            TextStyle {
                                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 20.0,
                                color: Color::WHITE,
                            },
                        ),
                        TextSection::new(
                            "",
                            TextStyle {
                                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 16.0,
                                color: Color::GOLD,
                            },
                        ),
                    ]),
                    ControlsText,
                ));

                // Weather
                parent.spawn((
                    TextBundle::from_sections([
                        TextSection::new(
                            "Weather: ",
                            TextStyle {
                                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 20.0,
                                color: Color::WHITE,
                            },
                        ),
                        TextSection::new(
                            "",
                            TextStyle {
                                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 20.0,
                                color: Color::GOLD,
                            },
                        ),
                    ]),
                    WeatherText,
                ));
            });
        });
}

pub fn update_speedometer_system(
    mut query: Query<&mut Text, With<SpeedometerText>>,
    wheel_query: Query<&Joint>,
) {
    for mut text in query.iter_mut() {
        let mut total_speed = 0.0;
        let wheel_count = wheel_query.iter().count() as f64;
        
        for joint in wheel_query.iter() {
            let wheel_speed = joint.qd * 0.3 * 2.237;
            total_speed += wheel_speed;
        }
        
        let average_speed = total_speed / wheel_count;
        text.sections[0].value = format!("{:.1} MPH", average_speed);
    }
}

pub fn update_rpm_system(
    mut query: Query<&mut Text, With<RpmText>>,
    joints: Query<&Joint>,
) {
    for mut text in query.iter_mut() {
        let mut total_rpm = 0.0;
        let count = joints.iter().count() as f64;
        
        for joint in joints.iter() {
            total_rpm += (joint.qd * 60.0) / (2.0 * std::f64::consts::PI);
        }
        
        let average_rpm = total_rpm / count;
        text.sections[0].value = format!("{:.0} RPM", average_rpm);
    }
}

pub fn update_controls_system(
    mut query: Query<&mut Text, With<ControlsText>>,
    control: Res<CarControl>,
) {
    for mut text in query.iter_mut() {
        text.sections[1].value = format!(
            "Throttle: {:.0}%\nBrake: {:.0}%\nSteering: {:.0}°",
            control.throttle * 100.0,
            control.brake * 100.0,
            control.steering * 30.0
        );
    }
}

pub fn update_weather_system(
    weather: Res<Weather>,
    mut query: Query<&mut Text, With<WeatherText>>,
) {
    if weather.is_changed() {
        for mut text in query.iter_mut() {
            text.sections[1].value = format!("{:?}", *weather);
        }
    }
}