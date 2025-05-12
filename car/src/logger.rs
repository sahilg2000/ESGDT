use bevy::prelude::*;
use std::fs::OpenOptions;
use std::io::Write;
use serde::{Serialize, Deserialize};
use chrono::Local;

use crate::control::CarControl;

#[derive(Resource)]
pub struct InputLogger {
    log_file: String,
    last_throttle: f32,
    last_brake: f32,
    last_steering: f32,
    throttle_pressed: bool,
    brake_pressed: bool,
    steering_pressed: bool,
}

#[derive(Serialize, Deserialize)]
struct LogEvent {
    time: String,
    control_type: String,
    event_type: String,  // Added to distinguish press/release
    value: f32,
}

impl Default for InputLogger {
    fn default() -> Self {
        Self {
            log_file: "car_inputs.log".to_string(),
            last_throttle: 0.0,
            last_brake: 0.0,
            last_steering: 0.0,
            throttle_pressed: false,
            brake_pressed: false,
            steering_pressed: false,
        }
    }
}

impl InputLogger {
    pub fn new(log_file: String) -> Self {
        println!("Initializing logger with file: {}", log_file);
        Self {
            log_file,
            ..default()
        }
    }

    pub fn log_event(&self, control_type: &str, event_type: &str, value: f32) {
        let time = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        
        // Convert value to percentage and round to 3 decimal places
        let percentage_value = if control_type == "steering" {
            (value * 100.0 * 1000.0).round() / 1000.0
        } else {
            (value * 100.0 * 1000.0).round() / 1000.0
        };

        let event = LogEvent {
            time,
            control_type: control_type.to_string(),
            event_type: event_type.to_string(),
            value: percentage_value,
        };

        if let Ok(event_json) = serde_json::to_string(&event) {
            if let Ok(mut file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.log_file) 
            {
                if let Err(e) = writeln!(file, "{}", event_json) {
                    println!("Failed to write to log file: {}", e);
                }
            }
        }
    }
}

pub fn input_logger_system(
    mut logger: ResMut<InputLogger>,
    control: Res<CarControl>,
) {
    // Throttle Press and Release
    if control.throttle > 0.01 && !logger.throttle_pressed {
        logger.log_event("throttle", "press", control.throttle);
        logger.throttle_pressed = true;
    } else if control.throttle <= 0.01 && logger.throttle_pressed {
        logger.log_event("throttle", "release", logger.last_throttle);
        logger.throttle_pressed = false;
    }
    logger.last_throttle = control.throttle;

    // Brake Press and Release
    if control.brake > 0.01 && !logger.brake_pressed {
        logger.log_event("brake", "press", control.brake);
        logger.brake_pressed = true;
    } else if control.brake <= 0.01 && logger.brake_pressed {
        logger.log_event("brake", "release", logger.last_brake);
        logger.brake_pressed = false;
    }
    logger.last_brake = control.brake;

    // Steering Press and Release
    if control.steering.abs() > 0.01 && !logger.steering_pressed {
        logger.log_event("steering", "press", control.steering);
        logger.steering_pressed = true;
    } else if control.steering.abs() <= 0.01 && logger.steering_pressed {
        logger.log_event("steering", "release", logger.last_steering);
        logger.steering_pressed = false;
    }
    logger.last_steering = control.steering;
}