use crate::sva::Xform;
use bevy::prelude::{Color, Component, Transform};

#[derive(Component, Debug)]
pub struct MeshDef {
    pub mesh_type: MeshTypeDef,
    pub transform: TransformDef,
    pub color: Color,
}

#[derive(Debug, Clone)]
pub enum MeshTypeDef {
    Box { dimensions: [f32; 3] },
    Cylinder { height: f32, radius: f32 },
    Wheel { radius: f32, width: f32 },
    File { file_name: String },
    // Sphere { radius: f64 },
    // Mesh { filename: String },
}

#[derive(Debug, Clone)]
pub enum TransformDef {
    Identity,
    Position { x: f64, y: f64, z: f64 },
    Quaternion { x: f64, y: f64, z: f64, w: f64 },
    RotationX(f64),
    RotationY(f64),
    RotationZ(f64),
}

impl Default for TransformDef {
    fn default() -> Self {
        Self::Identity
    }
}

impl From<&TransformDef> for Xform {
    fn from(transform: &TransformDef) -> Self {
        match transform {
            TransformDef::Identity => Xform::identity(),
            TransformDef::Position { x, y, z } => Xform::pos(*x, *y, *z),
            TransformDef::Quaternion { x, y, z, w } => Xform::quaternion(*x, *y, *z, *w),
            TransformDef::RotationX(angle) => Xform::rotx(*angle),
            TransformDef::RotationY(angle) => Xform::roty(*angle),
            TransformDef::RotationZ(angle) => Xform::rotz(*angle),
            // TransformDef::RotationVector(vector) => Xform::from_rotation_vector(vector),
        }
    }
}

impl From<&TransformDef> for Transform {
    fn from(transform: &TransformDef) -> Self {
        match transform {
            TransformDef::Identity => Transform::IDENTITY,
            TransformDef::Position { x, y, z } => {
                Transform::from_xyz(*x as f32, *y as f32, *z as f32)
            }
            TransformDef::Quaternion { x, y, z, w } => {
                let quat = bevy::math::Quat::from_xyzw(*w as f32, *x as f32, *y as f32, *z as f32);
                Transform::from_rotation(quat)
            }
            TransformDef::RotationX(angle) => {
                let mut transform = Transform::IDENTITY;
                transform.rotate_local_x(*angle as f32);
                transform
            }
            TransformDef::RotationY(angle) => {
                let mut transform = Transform::IDENTITY;
                transform.rotate_local_y(*angle as f32);
                transform
            }
            TransformDef::RotationZ(angle) => {
                let mut transform = Transform::IDENTITY;
                transform.rotate_local_z(*angle as f32);
                transform
            } // TransformDef::RotationVector(vector) => Xform::from_rotation_vector(vector),
        }
    }
}

impl TransformDef {
    pub fn from_position(position: [f64; 3]) -> Self {
        Self::Position {
            x: position[0],
            y: position[1],
            z: position[2],
        }
    }

    pub fn from_quaternion(quaternion: [f64; 4]) -> Self {
        Self::Quaternion {
            x: quaternion[0],
            y: quaternion[1],
            z: quaternion[2],
            w: quaternion[3],
        }
    }
}
