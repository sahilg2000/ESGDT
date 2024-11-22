use bevy::prelude::*;
use grid_terrain::GridTerrain;
use rigid_body::{
    joint::Joint,
    sva::{Force, Vector},
};

#[derive(Component)]
pub struct PointTire {
    joint_entity: Entity,
    joint_parent: Entity,
    points: Vec<Vector>,
    stiffness: [f64; 2],
    damping: f64,
    coefficient_of_friction: f64,
    normalized_slip_stiffness: f64,
    rolling_radius: f64,
    low_speed: f64,
    filter_time: f64,
    my_filtered: f64,
    activation_length: f64,
}

impl PointTire {
    pub fn new(
        joint_entity: Entity,
        joint_parent: Entity,
        stiffness: [f64; 2],
        damping: f64,
        coefficient_of_friction: f64,
        normalized_slip_stiffness: f64,
        rolling_radius: f64,
        low_speed: f64,
        radius: f64,
        width: f64,
        filter_time: f64,
        num_points_width: usize,
        num_points_radius: usize,
        activation_length: f64,
    ) -> Self {
        let mut points = Vec::new();
        let mut theta: f64 = 0.;
        let d_theta = 2. * std::f64::consts::PI / num_points_radius as f64;
        let half_width: f64;
        let y_step: f64;

        if num_points_width == 1 {
            half_width = 0.;
            y_step = 0.;
        } else {
            y_step = width / (num_points_width - 1) as f64;
            half_width = width / 2.;
        }
        for _ in 0..num_points_radius {
            let mut y_pos = -half_width;
            for width_ind in 0..num_points_width {
                let theta_point = theta + (width_ind as f64 / num_points_width as f64) * d_theta;
                let point = Vector::new(
                    radius * theta_point.sin(),
                    y_pos,
                    radius * theta_point.cos(),
                );
                points.push(point);
                y_pos += y_step;
            }
            theta += d_theta;
        }

        // build the tire
        Self {
            joint_entity,
            joint_parent,
            points,
            stiffness,
            damping,
            coefficient_of_friction,
            normalized_slip_stiffness,
            rolling_radius,
            low_speed,
            filter_time,
            my_filtered: 0.,
            activation_length,
        }
    }

    pub fn joint_entity(&self) -> Entity {
        self.joint_entity
    }

    pub fn points(&self) -> &Vec<Vector> {
        &self.points
    }
}

pub fn point_tire_system(
    mut tire_query: Query<&mut PointTire>,
    mut query_joints: Query<&mut Joint>,
    grid_terrain: Res<GridTerrain>,
) {
    let terrain = grid_terrain.as_ref();
    for mut tire in tire_query.iter_mut() {
        if let Ok([mut joint, parent]) =
            query_joints.get_many_mut([tire.joint_entity, tire.joint_parent])
        {
            let mut f_ext = Force::zero();
            let x0i = joint.x.inverse(); // spatial transform from the wheel joint to absolute coordinates
            let v0 = x0i * joint.v; // spatial velocity of the wheel joint in absolute coordinates
            let xp0 = parent.x.inverse(); // spatial transform from the parent joint to absolute coordinates
            let vp0 = xp0 * parent.v; // spatial velocity of the parent joint in absolute coordinates
            let center_abs = xp0.transform_point(Vector::zeros()); // center of the tire in absolute coordinates
            let lateral_abs = x0i * Vector::y(); // tire lateral direction in absolute coordinates

            // identify points in contact with the terrain
            let mut contacts = Vec::new();
            let mut active_points = 0.0;
            for point in tire.points.iter() {
                let point_abs = x0i.transform_point(*point); // point in absolute coordinates
                if let Some(contact) = terrain.interference(point_abs) {
                    let active = (contact.magnitude / tire.activation_length).clamp(0.0, 1.0);
                    contacts.push((contact, point_abs, active));
                    active_points += active;
                }
            }

            // calculate forces for each contact point
            for (contact, point_abs, active) in contacts {
                // critical directions - all in absolute coordinates
                let contact_lateral =
                    (lateral_abs - contact.normal.dot(&lateral_abs) * contact.normal).normalize();
                let contact_longitudinal = contact_lateral.cross(&contact.normal).normalize();
                let tire_up = contact_longitudinal.cross(&lateral_abs).normalize(); // vertical in the plane of the tire

                let mut radial = point_abs - center_abs;
                radial = (radial - radial.dot(&lateral_abs) * lateral_abs).normalize();

                // Calculate slip
                let rolling_radius_point =
                    center_abs + radial * tire.rolling_radius / -tire_up.dot(&radial);

                let vel_abs_rolling = v0.velocity_point(rolling_radius_point);
                let plane_velocity_rolling =
                    vel_abs_rolling.vel - vel_abs_rolling.vel.dot(&contact.normal) * contact.normal;

                let vel_abs_contact = v0.velocity_point(contact.position);
                let plane_velocity_contact =
                    vel_abs_contact.vel - vel_abs_contact.vel.dot(&contact.normal) * contact.normal;

                let vel_abs_parent = vp0.velocity_point(contact.position);
                let normal_velocity_parent = vel_abs_parent.vel.dot(&contact.normal);
                let plane_velocity_parent =
                    vel_abs_parent.vel - normal_velocity_parent * contact.normal;

                // slip angle and slip ratio calculation
                let ground_speed_lat = plane_velocity_contact.dot(&contact_lateral);
                let ground_speed_long = plane_velocity_rolling.dot(&contact_longitudinal);
                let ground_speed_parent_long = plane_velocity_parent.dot(&contact_longitudinal);

                let ground_speed_parent_long_abs =
                    ground_speed_parent_long.abs().max(tire.low_speed);

                let slip_ratio_point = -ground_speed_long / ground_speed_parent_long_abs;
                let slip_angle_point = -ground_speed_lat / ground_speed_parent_long_abs;

                // Calculate forces

                // normal force
                let stiffness_force_magnitude = (tire.stiffness[0] * contact.magnitude
                    + tire.stiffness[1] * contact.magnitude.powi(2))
                    / active_points;

                let normal_speed_parent = vel_abs_parent.vel.dot(&contact.normal);
                let damping_force_magnitude = (-tire.damping / active_points * normal_speed_parent)
                    .clamp(-stiffness_force_magnitude / 2., stiffness_force_magnitude);

                let normal_force_magnitude = stiffness_force_magnitude + damping_force_magnitude;
                let normal_force = normal_force_magnitude * contact.normal;

                // in plane forces
                let normalized_long_force =
                    (slip_ratio_point * tire.normalized_slip_stiffness).clamp(-1., 1.);
                let normalized_lat_force =
                    (slip_angle_point * tire.normalized_slip_stiffness).clamp(-1., 1.);

                let long_force =
                    normalized_long_force * normal_force_magnitude * tire.coefficient_of_friction;

                let lat_force =
                    normalized_lat_force * normal_force_magnitude * tire.coefficient_of_friction;

                let plane_force = lat_force * contact_lateral + long_force * contact_longitudinal;

                let force = active * (normal_force + plane_force);
                f_ext += Force::force_point(force, contact.position);
            }

            // Y Moment Filter (otherwise the wheel oscillates, it is too stiff for the solver)
            let mut f_ext_parent = parent.x * f_ext; // resolve the force about the axle
            let weight = 0.5_f64.powf(1. / (tire.filter_time / (0.002 / 4.))); // hard coded time step
            tire.my_filtered = tire.my_filtered * weight + f_ext_parent.m.y * (1. - weight);
            f_ext_parent.m.y = tire.my_filtered;
            f_ext = parent.x.inverse() * f_ext_parent;

            // apply the force to the joint
            joint.f_ext += f_ext;
        }
    }
}
