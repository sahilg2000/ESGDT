use std::collections::HashMap;

use bevy::prelude::*;

use rigid_body::joint::Joint;

use crate::interpolate::Interpolator1D;

use super::control::CarControl;

#[derive(Component)]
pub struct SuspensionComponent {
    stiffness: f64,
    damping: f64,
    preload: f64,
}

impl SuspensionComponent {
    pub fn new(stiffness: f64, damping: f64, preload: f64) -> Self {
        Self {
            stiffness,
            damping,
            preload,
        }
    }
}

pub fn suspension_system(mut joints: Query<(&mut Joint, &SuspensionComponent)>) {
    for (mut joint, suspension) in joints.iter_mut() {
        joint.tau -=
            suspension.stiffness * joint.q + suspension.damping * joint.qd + suspension.preload;
    }
}

#[derive(Clone)]
pub enum SteeringType {
    None,
    Curvature(SteeringCurvature),
    Angle(Steering),
}

#[derive(Component, Clone)]
pub struct Steering {
    pub max_angle: f64,
}

impl Steering {
    pub fn new(max_angle: f64) -> Self {
        Self { max_angle }
    }
}

pub fn steering_system(mut joints: Query<(&mut Joint, &Steering)>, control: Res<CarControl>) {
    for (mut joint, steering) in joints.iter_mut() {
        joint.q = control.steering as f64 * steering.max_angle;
    }
}

#[derive(Component, Clone)]
pub struct SteeringCurvature {
    pub x: f64,
    pub y: f64,
    pub max_curvature: f64,
}

impl SteeringCurvature {
    pub fn new(max_curvature: f64, x: f64, y: f64) -> Self {
        Self {
            x,
            y,
            max_curvature,
        }
    }
}

pub fn steering_curvature_system(
    mut joints: Query<(&mut Joint, &SteeringCurvature)>,
    control: Res<CarControl>,
) {
    for (mut joint, steering) in joints.iter_mut() {
        let vehicle_curvature_target = steering.max_curvature * control.steering as f64;
        let wheel_curvature_target =
            vehicle_curvature_target / (1.0 - vehicle_curvature_target * steering.y);
        joint.q = (wheel_curvature_target * steering.x).atan();
    }
}

#[derive(Clone)]
pub enum DriveType {
    None,
    DrivenWheel(DrivenWheel),
    DrivenWheelLookup(DrivenWheelLookup),
}

#[derive(Component, Clone)]
pub struct DrivenWheel {
    pub max_torque: f64,
    pub max_speed: f64,
    pub max_power: f64,
}

impl DrivenWheel {
    pub fn new(max_torque: f64, max_speed: f64, max_power: f64) -> Self {
        Self {
            max_torque,
            max_speed,
            max_power,
        }
    }
}

pub fn driven_wheel_system(
    mut joints: Query<(&mut Joint, &DrivenWheel)>,
    control: Res<CarControl>,
) {
    for (mut joint, driven_wheel) in joints.iter_mut() {
        let power_limited_torque = (driven_wheel.max_power / joint.qd).abs();
        if joint.qd.abs() < driven_wheel.max_speed {
            joint.tau +=
                control.throttle as f64 * driven_wheel.max_torque.min(power_limited_torque);
        }
    }
}

#[derive(Component, Clone)]
pub struct DrivenWheelLookup {
    pub name: String,
    pub torque_lookup: Interpolator1D,
    pub max_speed: f64,
    pub max_speed_power: f64,
    pub outputs: HashMap<String, f64>,
}

impl DrivenWheelLookup {
    pub fn new(name: String, speeds: Vec<f64>, torques: Vec<f64>) -> Self {
        let max_speed = speeds[speeds.len() - 1];
        let max_speed_power = torques[torques.len() - 1] * max_speed;
        Self {
            name,
            torque_lookup: Interpolator1D::new(speeds, torques),
            max_speed,
            max_speed_power,
            outputs: HashMap::new(),
        }
    }

    pub fn limit_torque(&self, speed: f64) -> f64 {
        let mut sign = speed.signum();
        if sign == 0. {
            sign = 1.;
        }
        let abs_speed = speed.abs();
        if abs_speed > self.max_speed {
            return 0.0;
        }

        let limit_torque = if abs_speed < self.max_speed {
            sign * self.torque_lookup.interpolate(abs_speed)
        } else {
            sign * self.max_speed_power / abs_speed
        };
        limit_torque
    }
}

pub fn driven_wheel_lookup_system(
    mut joints: Query<(&mut Joint, &mut DrivenWheelLookup)>,
    control: Res<CarControl>,
) {
    for (mut joint, mut driven_wheel) in joints.iter_mut() {
        let torque_limit = driven_wheel.limit_torque(joint.qd).abs();
        let commanded_torque = control.throttle as f64 * torque_limit;
        joint.tau += commanded_torque;
        driven_wheel
            .outputs
            .insert("torque".to_string(), commanded_torque);
        driven_wheel
            .outputs
            .insert("torque_limit".to_string(), torque_limit);
    }
}

#[derive(Component)]
pub struct BrakeWheel {
    pub max_torque: f64,
}

impl BrakeWheel {
    pub fn new(max_torque: f64) -> Self {
        Self { max_torque }
    }
}

pub fn brake_wheel_system(mut joints: Query<(&mut Joint, &BrakeWheel)>, control: Res<CarControl>) {
    for (mut joint, brake_wheel) in joints.iter_mut() {
        // TODO: make better? What to do around zero speed?
        joint.tau += -control.brake as f64 * brake_wheel.max_torque * joint.qd.min(1.).max(-1.);
    }
}
