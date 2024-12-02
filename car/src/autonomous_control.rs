use bevy::prelude::*;
use std::f32::consts::PI;
use crate::control::CarControl;

#[derive(Resource, Default)]
pub struct VehicleState {
    pub position: Vec3,
    pub velocity: Vec3,
    pub heading: f32,
    pub angular_velocity: f32,
}

// Autonomous Control Structures
#[derive(PartialEq, Clone, Debug)]
pub enum ControlMode {
    PositionTracking,
    VelocityTracking,
    HeadingTracking,
}

#[derive(Resource)]
pub struct AutonomousControl {
    pub mode: ControlMode,
    pub target_position: Vec3,
    pub target_velocity: f32,
    pub target_heading: f32,
    position_controller: PositionController,
    velocity_controller: VelocityController,
    heading_controller: HeadingController,
}

impl Default for AutonomousControl {
    fn default() -> Self {
        Self {
            mode: ControlMode::PositionTracking,
            target_position: Vec3::ZERO,
            target_velocity: 0.0,
            target_heading: 0.0,
            position_controller: PositionController::new(
                Vec3::new(1.0, 1.0, 1.0),
                Vec3::new(0.1, 0.1, 0.1),
                Vec3::new(0.5, 0.5, 0.5),
                Vec3::new(10.0, 10.0, 10.0),
            ),
            velocity_controller: VelocityController::new(
                1.0, 0.1, 0.5, 1.0
            ),
            heading_controller: HeadingController::new(
                1.0, 0.1, 0.5, 1.0
            ),
        }
    }
}

#[derive(Clone)]
pub struct PositionController {
    pub kp: Vec3,
    pub ki: Vec3,
    pub kd: Vec3,
    integral: Vec3,
    last_error: Vec3,
    max_integral: Vec3,
    max_output: Vec3,
}

impl PositionController {
    pub fn new(kp: Vec3, ki: Vec3, kd: Vec3, max_output: Vec3) -> Self {
        Self {
            kp,
            ki,
            kd,
            integral: Vec3::ZERO,
            last_error: Vec3::ZERO,
            max_integral: max_output * 0.5,
            max_output,
        }
    }

    pub fn compute(&mut self, current: Vec3, target: Vec3, dt: f32) -> Vec3 {
        let error = target - current;
        self.integral += error * dt;
        self.integral = Vec3::new(
            self.integral.x.clamp(-self.max_integral.x, self.max_integral.x),
            self.integral.y.clamp(-self.max_integral.y, self.max_integral.y),
            self.integral.z.clamp(-self.max_integral.z, self.max_integral.z),
        );

        let derivative = if dt > 0.0 {
            (error - self.last_error) / dt
        } else {
            Vec3::ZERO
        };
        
        self.last_error = error;

        let output = self.kp * error + self.ki * self.integral + self.kd * derivative;
        Vec3::new(
            output.x.clamp(-self.max_output.x, self.max_output.x),
            output.y.clamp(-self.max_output.y, self.max_output.y),
            output.z.clamp(-self.max_output.z, self.max_output.z),
        )
    }
}

#[derive(Clone)]
pub struct VelocityController {
    pub kp: f32,
    pub ki: f32,
    pub kd: f32,
    integral: f32,
    last_error: f32,
    max_integral: f32,
    max_output: f32,
}

impl VelocityController {
    pub fn new(kp: f32, ki: f32, kd: f32, max_output: f32) -> Self {
        Self {
            kp,
            ki,
            kd,
            integral: 0.0,
            last_error: 0.0,
            max_integral: max_output * 0.5,
            max_output,
        }
    }

    pub fn compute(&mut self, current: f32, target: f32, dt: f32) -> f32 {
        let error = target - current;
        self.integral += error * dt;
        self.integral = self.integral.clamp(-self.max_integral, self.max_integral);

        let derivative = if dt > 0.0 {
            (error - self.last_error) / dt
        } else {
            0.0
        };
        
        self.last_error = error;
        
        let output = self.kp * error + self.ki * self.integral + self.kd * derivative;
        output.clamp(-self.max_output, self.max_output)
    }
}

#[derive(Clone)]
pub struct HeadingController {
    pub kp: f32,
    pub ki: f32,
    pub kd: f32,
    integral: f32,
    last_error: f32,
    max_integral: f32,
    max_output: f32,
}

impl HeadingController {
    pub fn new(kp: f32, ki: f32, kd: f32, max_output: f32) -> Self {
        Self {
            kp,
            ki,
            kd,
            integral: 0.0,
            last_error: 0.0,
            max_integral: max_output * 0.5,
            max_output,
        }
    }

    pub fn compute(&mut self, current: f32, target: f32, dt: f32) -> f32 {
        let mut error = target - current;
        while error > PI {
            error -= 2.0 * PI;
        }
        while error < -PI {
            error += 2.0 * PI;
        }
        
        self.integral += error * dt;
        self.integral = self.integral.clamp(-self.max_integral, self.max_integral);

        let derivative = if dt > 0.0 {
            (error - self.last_error) / dt
        } else {
            0.0
        };
        
        self.last_error = error;
        
        let output = self.kp * error + self.ki * self.integral + self.kd * derivative;
        output.clamp(-self.max_output, self.max_output)
    }
}

// Autonomous Decision Making System
pub fn autonomous_decision_system(
    mut auto_control: ResMut<AutonomousControl>,
    vehicle_state: Res<VehicleState>,
    time: Res<Time>,
) {
    let target_reached = (auto_control.target_position - vehicle_state.position).length() < 0.1;
    
    if target_reached {
        auto_control.target_position += Vec3::new(1.0, 0.0, 0.0); 
        auto_control.target_velocity = 5.0; 
        
        // Calculate desired heading
        auto_control.target_heading = (auto_control.target_position - vehicle_state.position)
            .y
            .atan2(vehicle_state.position.x);
    }
}

// Main Control System
pub fn autonomous_control_system(
    mut car_control: ResMut<CarControl>,
    mut auto_control: ResMut<AutonomousControl>,
    vehicle_state: Res<VehicleState>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();

    match auto_control.mode {
        ControlMode::PositionTracking => {
            let control_output = auto_control.position_controller.compute(
                vehicle_state.position,
                auto_control.target_position,
                dt
            );
            
            let desired_velocity = control_output;
            let velocity_error = desired_velocity - vehicle_state.velocity;
            
            if velocity_error.length() > 0.1 {
                car_control.throttle = (velocity_error.length() * 0.1).min(1.0);
                car_control.brake = 0.0;
            } else {
                car_control.throttle = 0.0;
                car_control.brake = 0.1;
            }
            
            let desired_heading = velocity_error.y.atan2(velocity_error.x);
            let heading_error = normalize_angle(desired_heading - vehicle_state.heading);
            car_control.steering = (heading_error * 0.5).clamp(-1.0, 1.0);
        },
        
        ControlMode::VelocityTracking => {
            let velocity_magnitude = vehicle_state.velocity.length();
            let control_output = auto_control.velocity_controller.compute(
                velocity_magnitude,
                auto_control.target_velocity,
                dt
            );
            
            if control_output > 0.0 {
                car_control.throttle = control_output;
                car_control.brake = 0.0;
            } else {
                car_control.throttle = 0.0;
                car_control.brake = -control_output;
            }
        },
        
        ControlMode::HeadingTracking => {
            let control_output = auto_control.heading_controller.compute(
                vehicle_state.heading,
                auto_control.target_heading,
                dt
            );
            
            car_control.steering = control_output;
        },
    }
}

fn normalize_angle(mut angle: f32) -> f32 {
    while angle > PI {
        angle -= 2.0 * PI;
    }
    while angle < -PI {
        angle += 2.0 * PI;
    }
    angle
}

pub struct AutonomousPlugin;

impl Plugin for AutonomousPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AutonomousControl>()
           .init_resource::<VehicleState>()
           .add_systems(Update, (
                autonomous_decision_system,
                autonomous_control_system,
            ));
    }
}