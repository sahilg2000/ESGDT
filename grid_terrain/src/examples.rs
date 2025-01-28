use std::f64::consts::PI as PI64;
use crate::{
    function::Function, mirror::Mirror, plane::Plane, rotate::Rotate, step::Step,
    step_slope::StepSlope, GridElement,
};

/// Creates a table-top pattern using steps and slopes arranged in a 2x3 grid
/// size: Dimension of each grid element
/// height: Height of the steps and slopes
pub fn table_top(size: f64, height: f64) -> Vec<Vec<Box<dyn GridElement + 'static>>> {
    let grid_elements: Vec<Vec<Box<dyn GridElement + 'static>>> = vec![
        // First row: slope -> step -> mirrored slope
        vec![
            Box::new(StepSlope {
                size,
                height,
                mirror: Mirror::None,
                rotate: Rotate::Ninety,      // Rotated 90 degrees
            }),
            Box::new(Step {
                size,
                height,
                mirror: Mirror::None,
                rotate: Rotate::Ninety,      // Rotated 90 degrees
            }),
            Box::new(StepSlope {
                size,
                height,
                mirror: Mirror::YZ,          // Mirrored across YZ plane
                rotate: Rotate::TwoSeventy,  // Rotated 270 degrees
            }),
        ],
        // Second row: mirrored slope -> step -> slope
        vec![
            Box::new(StepSlope {
                size,
                height,
                mirror: Mirror::YZ,          // Mirrored across YZ plane
                rotate: Rotate::Ninety,      // Rotated 90 degrees
            }),
            Box::new(Step {
                size,
                height,
                mirror: Mirror::None,
                rotate: Rotate::TwoSeventy,  // Rotated 270 degrees
            }),
            Box::new(StepSlope {
                size,
                height,
                mirror: Mirror::None,
                rotate: Rotate::TwoSeventy,  // Rotated 270 degrees
            }),
        ],
    ];
    grid_elements
}

/// Creates a series of steps with varying heights
/// size: Dimension of each step
/// heights: Vector of heights for each row of steps
pub fn steps(size: f64, heights: Vec<f64>) -> Vec<Vec<Box<dyn GridElement + 'static>>> {
    let mut grid_elements: Vec<Vec<Box<dyn GridElement + 'static>>> = Vec::new();
    for height in heights {
        // For each height, create a row with: step -> rotated step -> plane
        grid_elements.push(vec![
            Box::new(Step {
                size,
                height,
                ..Default::default()
            }),
            Box::new(Step {
                size,
                height,
                rotate: Rotate::OneEighty,  // Rotated 180 degrees
                ..Default::default()
            }),
            Box::new(Plane {
                size: [size, size],
                subdivisions: 1,
            }),
        ]);
    }
    grid_elements
}

const TAU64: f64 = 2. * PI64;  // Full circle in radians (2π)

/// Creates a wave pattern using mathematical functions
/// size: Dimension of each grid element
/// height: Amplitude of the wave
/// wave_length: Length of one complete wave cycle
pub fn wave(size: f64, height: f64, wave_length: f64) -> Vec<Vec<Box<dyn GridElement + 'static>>> {
    // Parameter mapping functions for grid boundaries
    let x_start = Box::new(move |x: f64, _y: f64| x / size);        // Maps x to [0,1] at start
    let x_end = Box::new(move |x: f64, _y: f64| 1.0 - x / size);    // Maps x to [1,0] at end
    let y_start = Box::new(move |_x: f64, y: f64| y / size);        // Maps y to [0,1] at start
    let y_end = Box::new(move |_x: f64, y: f64| 1.0 - y / size);    // Maps y to [1,0] at end

    // Derivatives of parameter mapping functions
    let dx_start = Box::new(move |_x: f64, _y: f64| (1.0 / size, 0.));     // Derivative of x_start
    let dx_end = Box::new(move |_x: f64, _y: f64| (-1.0 / size, 0.));      // Derivative of x_end
    let dy_start = Box::new(move |_x: f64, _y: f64| (0., 1.0 / size));     // Derivative of y_start
    let dy_end = Box::new(move |_x: f64, _y: f64| (0., -1.0 / size));      // Derivative of y_end

    // Wave function and its derivative
    let z_fun = Box::new(move |x: f64, _y: f64| height * (TAU64 / wave_length * x).cos());
    let z_der = Box::new(move |x: f64, _y: f64| {
        (
            -height * TAU64 / wave_length * (TAU64 / wave_length * x).sin(),
            0.,
        )
    });

    let size = [size, size];

    // Create 3x3 grid of wave functions with appropriate boundary conditions
    let grid_elements: Vec<Vec<Box<dyn GridElement + 'static>>> = vec![
        // Top row (y_start boundary)
        vec![
            Box::new(Function {
                size,
                functions: vec![z_fun.clone(), x_start.clone(), y_start.clone()],
                derivatives: vec![z_der.clone(), dx_start.clone(), dy_start.clone()],
            }),
            Box::new(Function {
                size,
                functions: vec![z_fun.clone(), y_start.clone()],
                derivatives: vec![z_der.clone(), dy_start.clone()],
            }),
            Box::new(Function {
                size,
                functions: vec![z_fun.clone(), x_end.clone(), y_start.clone()],
                derivatives: vec![z_der.clone(), dx_end.clone(), dy_start.clone()],
            }),
        ],
        // Middle row
        vec![
            Box::new(Function {
                size,
                functions: vec![z_fun.clone(), x_start.clone()],
                derivatives: vec![z_der.clone(), dx_start.clone()],
            }),
            Box::new(Function {
                size,
                functions: vec![z_fun.clone()],
                derivatives: vec![z_der.clone()],
            }),
            Box::new(Function {
                size,
                functions: vec![z_fun.clone(), x_end.clone()],
                derivatives: vec![z_der.clone(), dx_end.clone()],
            }),
        ],
        // Bottom row (y_end boundary)
        vec![
            Box::new(Function {
                size,
                functions: vec![z_fun.clone(), x_start.clone(), y_end.clone()],
                derivatives: vec![z_der.clone(), dx_start.clone(), dy_end.clone()],
            }),
            Box::new(Function {
                size,
                functions: vec![z_fun.clone(), y_end.clone()],
                derivatives: vec![z_der.clone(), dy_end.clone()],
            }),
            Box::new(Function {
                size,
                functions: vec![z_fun.clone(), x_end.clone(), y_end.clone()],
                derivatives: vec![z_der.clone(), dx_end.clone(), dy_end.clone()],
            }),
        ],
    ];

    grid_elements
}