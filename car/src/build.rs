use bevy::prelude::*;

use cameras::control::CameraParentList;
use rigid_body::{
    definitions::{MeshDef, MeshTypeDef, TransformDef},
    joint::{Base, Joint},
    sva::{Inertia, Matrix, Motion, Vector, Xform},
};

use crate::{
    physics::{
        BrakeWheel, DriveType, DrivenWheelLookup, SteeringCurvature, SteeringType,
        SuspensionComponent,
    },
    tire::PointTire,
};

#[derive(Resource)]
pub struct CarDefinition {
    chassis: Chassis,
    suspension: Vec<Suspension>,
    wheel: Wheel,
    drives: Vec<DriveType>,
    brake: Brake,
}

const CHASSIS_MASS: f64 = 1000.;
const SUSPENSION_MASS: f64 = 20.;
const GRAVITY: f64 = 9.81;

pub fn build_car() -> CarDefinition {
    // Chassis
    let mass = 1000.;
    let dimensions = [3.0_f64, 1.2, 0.4]; // shape of rectangular chassis
    let moi = [
        dimensions[1].powi(2) + dimensions[2].powi(2),
        dimensions[2].powi(2) + dimensions[0].powi(2),
        dimensions[0].powi(2) + dimensions[1].powi(2),
    ]
    .map(|x| mass * (1. / 12.) * x);

    let chassis = Chassis {
        mass,
        cg_position: [0., 0., 0.],
        moi,
        dimensions,
        position: [0., 0., 0.],
        initial_position: [-5., 20., 0.3 + 0.25],
        initial_orientation: [0., 0., 0.],
        mesh_file: None,
    };

    // Suspension
    let suspension_mass = 20.;
    let suspension_size = 0.025_f64;
    let suspension_stiffness = mass * (GRAVITY / 4.) / 0.1;
    let suspension_damping = 0.25 * 2. * (suspension_stiffness * (1000. / 4.) as f64).sqrt();
    let suspension_preload = mass * (GRAVITY / 4.);
    let suspension_moi = (2. / 3.) * suspension_mass * suspension_size.powi(2);

    let suspension_names = ["fl", "fr", "rl", "rr"].map(|name| name.to_string());
    let suspension_locations = [
        [1.25, 0.75, -0.2],
        [1.25, -0.75, -0.2],
        [-1.25, 0.75, -0.2],
        [-1.25, -0.75, -0.2],
    ];

    let suspension: Vec<Suspension> = suspension_locations
        .iter()
        .zip(suspension_names.clone())
        .enumerate()
        .map(|(ind, (location, name))| {
            let steering = if ind < 2 {
                // SteeringType::Angle(Steering {
                //     max_angle: 30.0_f64.to_radians(),
                // })
                SteeringType::Curvature(SteeringCurvature {
                    x: suspension_locations[ind][0] - suspension_locations[ind + 2][0],
                    y: suspension_locations[ind][1],
                    max_curvature: 1. / 5.0,
                })
            } else {
                SteeringType::None
            };
            Suspension {
                name,
                mass: suspension_mass,
                steering,
                stiffness: suspension_stiffness,
                damping: suspension_damping,
                preload: suspension_preload,
                moi: suspension_moi,
                location: *location,
            }
        })
        .collect();

    // Wheel
    let wheel = build_wheel();

    // // Drive and Brake
    let drive_speeds = vec![0., 25., 50., 75.];
    let drive_torques = vec![1000., 1000., 600., 250.];

    let rear_drive = DriveType::DrivenWheelLookup(DrivenWheelLookup::new(
        "fl".to_string(),
        drive_speeds.clone(),
        drive_torques.clone(),
    ));

    let drives = vec![
        DriveType::None,
        DriveType::None,
        rear_drive.clone(),
        rear_drive.clone(),
    ];

    let brake = Brake {
        front_torque: 800.,
        rear_torque: 400.,
    };

    CarDefinition {
        chassis,
        suspension,
        wheel,
        drives,
        brake,
    }
}

pub fn build_wheel() -> Wheel {
    let wheel_mass = 20.;
    let wheel_radius = 0.325_f64;
    let wheel_moi_y = wheel_mass * wheel_radius.powi(2);
    let wheel_moi_xz = 1. / 12. * 10. * (3. * wheel_radius.powi(2));
    let corner_mass = CHASSIS_MASS / 4. + SUSPENSION_MASS + wheel_mass;
    let wheel_stiffness = corner_mass * GRAVITY / 0.005;
    let wheel_damping = 0.01 * 2. * (wheel_stiffness * wheel_mass).sqrt();
    Wheel {
        mass: wheel_mass,
        radius: wheel_radius,
        width: 0.2_f64,
        moi_y: wheel_moi_y,
        moi_xz: wheel_moi_xz,
        stiffness: [wheel_stiffness, 0.],
        damping: wheel_damping,
        coefficient_of_friction: 0.8,
        rolling_radius: 0.315,
        low_speed: 1.0,
        normalized_slip_stiffness: 20.0,
        filter_time: 0.005,
    }
}

pub fn car_startup_system(mut commands: Commands, car: ResMut<CarDefinition>) {
    let base = Joint::base(Motion::new([0., 0., 9.81], [0., 0., 0.]));
    let base_id = commands.spawn((base, Base)).id();

    // Chassis
    let chassis_ids = car
        .chassis
        .build(&mut commands, Color::rgb(0.9, 0.1, 0.2), base_id);
    let chassis_id = chassis_ids[3]; // ids are not ordered by parent child order!!! "3" is rx, the last joint in the chain

    let camera_parent_list = vec![
        chassis_ids[5], // follow x, y and z and yaw of chassis
        // chassis_ids[0], // only follow x of chassis (why would you do that?)
        chassis_ids[1], // follow x and y of chassis
        chassis_ids[2], // follow x, y and z of chassis
        chassis_ids[3], // follow all motion of chassis
        base_id,        // stationary camera
                        // chassis_ids[4],
    ];

    commands.insert_resource(CameraParentList {
        list: camera_parent_list,
        active: 0, // start with following x, y, z and yaw of chassis
    });

    for (ind, susp) in car.suspension.iter().enumerate() {
        let braked_wheel = if ind < 2 {
            Some(BrakeWheel {
                max_torque: car.brake.front_torque,
            })
        } else {
            Some(BrakeWheel {
                max_torque: car.brake.rear_torque,
            })
        };
        let id_susp = susp.build(&mut commands, chassis_id, &susp.location);
        let _wheel_id = car.wheel.build(
            &mut commands,
            &susp.name,
            id_susp,
            car.drives[ind].clone(),
            braked_wheel,
            0.,
        );
    }
}

#[derive(Clone)]
pub struct Chassis {
    pub mass: f64,
    pub cg_position: [f64; 3],
    pub moi: [f64; 3],
    pub dimensions: [f64; 3],
    pub position: [f64; 3],
    pub initial_position: [f64; 3],
    pub initial_orientation: [f64; 3],
    pub mesh_file: Option<String>,
}

impl Chassis {
    pub fn build(&self, commands: &mut Commands, color: Color, parent_id: Entity) -> Vec<Entity> {
        // x degree of freedom (absolute coordinate system, not relative to car)
        let mut px = Joint::px("chassis_px".to_string(), Inertia::zero(), Xform::identity());
        px.q = self.initial_position[0];
        let mut px_e = commands.spawn((px,));
        px_e.set_parent(parent_id);
        let px_id = px_e.id();

        // y degree of freedom (absolute coordinate system, not relative to car)
        let mut py = Joint::py("chassis_py".to_string(), Inertia::zero(), Xform::identity());
        py.q = self.initial_position[1];
        let mut py_e = commands.spawn((py,));
        py_e.set_parent(px_id);
        let py_id = py_e.id();

        // z degree of freedom (always points "up", relative to absolute coordinate system)
        let mut pz = Joint::pz("chassis_pz".to_string(), Inertia::zero(), Xform::identity());
        pz.q = self.initial_position[2];
        let mut pz_e = commands.spawn((pz,));
        pz_e.set_parent(py_id);
        let pz_id = pz_e.id();

        // yaw degree of freedom (rotation around z axis)
        let mut rz = Joint::rz("chassis_rz".to_string(), Inertia::zero(), Xform::identity());
        rz.q = self.initial_orientation[2];
        let mut rz_e = commands.spawn((rz,));
        rz_e.set_parent(pz_id);
        let rz_id = rz_e.id();

        // pitch degree of freedom (rotation around y axis)
        let mut ry = Joint::ry("chassis_ry".to_string(), Inertia::zero(), Xform::identity());
        ry.q = self.initial_orientation[1];
        let mut ry_e = commands.spawn((ry,));
        ry_e.set_parent(rz_id);
        let ry_id = ry_e.id();

        // roll degree of freedom (rotation around x axis)
        // this is the body of the car!
        let mass = self.mass;
        let cg_position = self.cg_position;
        let moi = self.moi;
        let position = self.position;
        let dimensions = self.dimensions;
        let inertia = Inertia::new(
            mass,
            Vector::new(cg_position[0], cg_position[1], cg_position[2]),
            Matrix::from_diagonal(&Vector::new(moi[0], moi[1], moi[2])),
        );

        let mut rx = Joint::rx("chassis_rx".to_string(), inertia, Xform::identity());
        rx.q = self.initial_orientation[0];
        let mut rx_e = commands.spawn((rx,));
        rx_e.set_parent(ry_id);
        let rx_id = rx_e.id();
        if let Some(chassis_file) = &self.mesh_file {
            rx_e.insert(MeshDef {
                mesh_type: MeshTypeDef::File {
                    file_name: chassis_file.to_string(),
                },
                transform: TransformDef::from_position(position),
                color,
            });
        } else {
            rx_e.insert(MeshDef {
                mesh_type: MeshTypeDef::Box {
                    dimensions: [
                        dimensions[0] as f32,
                        dimensions[1] as f32,
                        dimensions[2] as f32,
                    ],
                },
                transform: TransformDef::from_position(position),
                color,
            });
        }

        let chassis_ids = vec![px_id, py_id, pz_id, rx_id, ry_id, rz_id];
        // return id the last joint in the chain. It will be the parent of the suspension / wheels
        chassis_ids
    }
}

#[derive(Clone)]
pub struct Suspension {
    pub name: String,
    pub mass: f64,
    pub steering: SteeringType,
    pub stiffness: f64,
    pub damping: f64,
    pub preload: f64,
    pub moi: f64,
    pub location: [f64; 3],
}

impl Suspension {
    pub fn build(
        &self,
        commands: &mut Commands,
        mut parent_id: Entity,
        location: &[f64; 3],
    ) -> Entity {
        // suspension transform
        let mut xt_susp = Xform::new(
            Vector::new(location[0], location[1], location[2]), // location of suspension relative to chassis
            Matrix::identity(),
        );

        // suspension mass
        let inertia = Inertia::new(
            self.mass,
            Vector::new(0., 0., 0.),       // center of mass
            self.moi * Matrix::identity(), // inertia
        );

        match self.steering.clone() {
            SteeringType::None => {}
            SteeringType::Curvature(steering) => {
                let steer_name = ("steer_".to_owned() + &self.name).to_string();
                let steer = Joint::rz(steer_name, Inertia::zero(), xt_susp);
                let mut steer_e = commands.spawn((steer, steering));
                steer_e.set_parent(parent_id);

                parent_id = steer_e.id();
                xt_susp = Xform::identity();
            }
            SteeringType::Angle(steering) => {
                // create suspension joint
                let steer_name = ("steer_".to_owned() + &self.name).to_string();
                let steer = Joint::rz(steer_name, Inertia::zero(), xt_susp);
                let mut steer_e = commands.spawn((steer, steering));
                steer_e.set_parent(parent_id);

                parent_id = steer_e.id();
                xt_susp = Xform::identity();
            }
        }

        // create suspension joint
        let name = ("susp_".to_owned() + &self.name).to_string();
        let susp = Joint::pz(name, inertia, xt_susp);

        // create suspension entity
        let mut susp_e = commands.spawn((
            susp,
            SpatialBundle::default(),
            SuspensionComponent::new(self.stiffness, self.damping, self.preload),
        ));
        susp_e.set_parent(parent_id);

        susp_e.id()
    }
}

#[derive(Resource, Clone)]
pub struct Wheel {
    pub mass: f64,
    pub radius: f64,
    pub width: f64,
    pub moi_y: f64,
    pub moi_xz: f64,
    pub stiffness: [f64; 2],
    pub damping: f64,
    pub coefficient_of_friction: f64,
    pub rolling_radius: f64,
    pub low_speed: f64,
    pub normalized_slip_stiffness: f64,
    pub filter_time: f64,
}

impl Wheel {
    pub fn build(
        &self,
        commands: &mut Commands,
        corner_name: &String,
        parent_id: Entity,
        driven_wheel: DriveType,
        braked_wheel: Option<BrakeWheel>,
        initial_speed: f64,
    ) -> Entity {
        // wheel inertia
        let inertia = Inertia::new(
            self.mass,
            Vector::new(0., 0., 0.),
            Matrix::from_diagonal(&Vector::new(self.moi_xz, self.moi_y, self.moi_xz)),
        );

        // create wheel joint
        let name = ("wheel_".to_owned() + corner_name).to_string();
        let mut ry = Joint::ry(name, inertia, Xform::identity());
        ry.qd = initial_speed;

        let mut wheel_e = commands.spawn((
            ry,
            MeshDef {
                mesh_type: MeshTypeDef::Wheel {
                    radius: self.radius as f32,
                    width: self.width as f32,
                },
                transform: TransformDef::Identity,
                color: Color::rgb(0.5, 0.5, 1.0),
            },
        ));

        // add driven and braked components
        match driven_wheel {
            DriveType::None => {}
            DriveType::DrivenWheelLookup(driven) => {
                wheel_e.insert(driven);
            }
            DriveType::DrivenWheel(driven) => {
                wheel_e.insert(driven);
            }
        }

        if let Some(braked) = braked_wheel {
            wheel_e.insert(braked);
        }

        // set parent
        wheel_e.set_parent(parent_id);
        let wheel_id = wheel_e.id();

        // add tire contact model
        commands.spawn(PointTire::new(
            wheel_id,
            parent_id,
            self.stiffness,
            self.damping,
            self.coefficient_of_friction,
            self.normalized_slip_stiffness,
            // self.rolling_resistance,
            self.rolling_radius,
            self.low_speed,
            self.radius,
            self.width,
            self.filter_time,
            5,
            51,
            0.01,
        ));
        wheel_id
    }
}

pub struct Brake {
    front_torque: f64,
    rear_torque: f64,
}
