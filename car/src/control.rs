use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct CarControl {
    pub throttle: f32,
    pub steering: f32,
    pub brake: f32,
}

pub fn user_control_system(
    keyboard_input: Res<Input<KeyCode>>,
    gamepads: Res<Gamepads>,
    button_axes: Res<Axis<GamepadButton>>,
    axes: Res<Axis<GamepadAxis>>,
    mut control: ResMut<CarControl>,
) {
    // gamepad controls
    for gamepad in gamepads.iter() {
        // trigger controls
        let throttle = button_axes
            .get(GamepadButton::new(
                gamepad,
                GamepadButtonType::RightTrigger2,
            ))
            .unwrap();

        if throttle > 0.01 {
            control.throttle = throttle;
        }

        let brake = button_axes
            .get(GamepadButton::new(gamepad, GamepadButtonType::LeftTrigger2))
            .unwrap();

        if brake > 0.01 {
            control.brake = brake;
        }

        // right stick throttle/brake
        let throttle_brake = axes
            .get(GamepadAxis::new(gamepad, GamepadAxisType::RightStickY))
            .unwrap();
        if throttle_brake > 0.01 {
            control.throttle = throttle_brake;
        }
        if throttle_brake < -0.01 {
            control.brake = -throttle_brake;
        }

        // left stick steering
        let steering = -axes
            .get(GamepadAxis::new(gamepad, GamepadAxisType::LeftStickX))
            .unwrap();
        if steering.abs() > 0.01 {
            control.steering = steering;
        }
    }

    // Keyboard controls - these are rate controlled to make them feel more natural.
    // When a key is pressed, the control value is increased at a constant rate.
    // When a key is released, the control value is decreased at a constant rate.
    // The control value is clamped between 0 and const MAX_SPEED for throttle and brake, 
    // and between -1 and 1 for steering.
    
    let acceleration_response_time = 0.01;
    let brake_response_time = 0.2;

    let accel_const: f32 =  1. / (acceleration_response_time * 60.);
    let brake_const: f32 =  1. / (brake_response_time * 60.);
    
    // Acceleration

    // Define constants at the beginning of your function
    const MAX_SPEED: f32 = 1.0;                                 // Maximum throttle value

    // Forward Acceleration - Key W
    if keyboard_input.pressed(KeyCode::W) {
        // Clamp acceleration at top speed (chooses min of max_speed and curr speed)
        control.throttle += accel_const;
        control.throttle = control.throttle.min(MAX_SPEED);
    } else {
        // If throttle is not active, set throttle to 0 and start applying brake (simplified friction)
        control.throttle -= accel_const;
        control.throttle = control.throttle.max(0.0);
    }
        

    // Brake Control - Key S
    if keyboard_input.pressed(KeyCode::S) {
        control.brake += brake_const;
        control.brake = control.brake.min(MAX_SPEED * 10.0);    // Braking is greater than max speed for use of quick braking
    } else {
        control.brake -= brake_const;
        control.brake = control.brake.max(0.0);
    }


    // Steering
    // gradual adjustment controls
    const MAX_STEERING: f32 = 1.0;                 // Max steering angle constant (Affects animation and physics)
    let steer_increment = 0.03;             // Adjust this value for faster/slower steering response
    let return_to_zero_increment = 0.09;    // Controls how fast the car will return to 0 or no steering after 'a' and 'd' are released
    let mut steer_active = false;


    // Steer Left - Key A
    if keyboard_input.pressed(KeyCode::A) {
        steer_active = true;
        if control.steering < MAX_STEERING {
            control.steering += steer_increment;
            if control.steering > MAX_STEERING {
                control.steering = MAX_STEERING;    // Clamp to max
            }
        }
    }


    // Steer Right - Key D
    if keyboard_input.pressed(KeyCode::D) {
        steer_active = true;
        if control.steering > -MAX_STEERING {
            control.steering -= steer_increment;
            if control.steering < -MAX_STEERING {
                control.steering = -MAX_STEERING;   // Clamp to min
            }
        }
    }


    // Activity and Idle controls
    if !steer_active {
        if control.steering.abs() < return_to_zero_increment {
            control.steering = 0.0;                         // Reset to zero if close enough
        } else if control.steering > 0.0 {                 // Reset via return_to_zero_increment, quicker return to 'no steer'
            control.steering -= return_to_zero_increment;
        } else {
            control.steering += return_to_zero_increment;
        }
        
        // Clamp to ensure it stays within bounds
        if control.steering > MAX_STEERING {
            control.steering = MAX_STEERING;
        } else if control.steering < -MAX_STEERING {
            control.steering = -MAX_STEERING;
        }
    }
}