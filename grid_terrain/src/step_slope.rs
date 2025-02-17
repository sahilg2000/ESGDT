use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use rigid_body::sva::Vector;

use crate::{
    mirror::{mirror_mesh, mirror_point},
    rotate::{rotate_mesh, rotate_point},
    GridElement, Interference, Mirror, Rotate, RotationDirection,
};

// Represents a step slope, which is a grid element with specified size, height, rotation, and mirroring.
#[derive(Default)]
pub struct StepSlope {
    pub size: f64,       // Base size of the step slope
    pub height: f64,     // Height of the step
    pub rotate: Rotate,  // Rotation properties
    pub mirror: Mirror,  // Mirroring properties
}

impl GridElement for StepSlope {
    // Determines interference of a given point with the step slope
    fn interference(&self, mut point: Vector) -> Option<Interference> {
        // Rotate and mirror the point based on the step slope's properties
        rotate_point(
            &mut point,
            self.size,
            &self.rotate,
            RotationDirection::Reverse,
        );
        mirror_point(&mut point, self.size, &self.mirror);

        let size = self.size;
        let height = self.height;

        // If the point is above the height of the step, there is no contact
        if point.z > height {
            return None;
        }

        // If the point is outside the base size area, there is no contact
        if point.x < 0.0 || point.x > size || point.y < 0.0 || point.y > size {
            return None;
        }

        // If the point is in the area but not on the step
        if point.x < size / 2.0 {
            if point.z > 0.0 {
                return None;
            }
            // Generate interference for the point being below the ground
            let mut interference = Interference {
                magnitude: -point.z,
                position: point - point.z * Vector::z(),
                normal: Vector::z(),
            };
            // Apply mirroring and rotation transformations
            interference.mirror(size, &self.mirror);
            interference.rotate(size, &self.rotate, RotationDirection::Forward);
            return Some(interference);
        }

        // If the point is on the step, calculate contact with the slope
        let top_normal = Vector::new(0., height, size).normalize(); // Normal of the sloped face
        let top_corner = Vector::new(size / 2., 0., height);        // Top corner of the step
        let normal_interference = -top_normal.dot(&(point - top_corner));

        if normal_interference < 0.0 {
            // Point is above the step slope, no contact possible
            return None;
        }

        // If the point is closer to the sloped face than the side
        let x_interference = point.x - size / 2.0;
        if x_interference > normal_interference {
            let mut interference = Interference {
                magnitude: normal_interference,
                position: point + normal_interference * top_normal,
                normal: top_normal,
            };
            // Apply transformations
            interference.mirror(size, &self.mirror);
            interference.rotate(size, &self.rotate, RotationDirection::Forward);
            return Some(interference);
        }

        // If the point is closer to the side of the step than the sloped face
        let mut interference = Interference {
            magnitude: x_interference,
            position: point - x_interference * Vector::x(),
            normal: -Vector::x(),
        };
        // Apply transformations
        interference.mirror(size, &self.mirror);
        interference.rotate(size, &self.rotate, RotationDirection::Forward);
        return Some(interference);
    }

    // Generates the mesh representation of the step slope
    fn mesh(&self) -> Mesh {
        // Define direction vectors for normals
        let up = Vec3::Z.to_array();    // Normal pointing upward
        let back = (-Vec3::X).to_array(); // Normal pointing backward
        let slope_normal = Vec3::new(0., self.height as f32, self.size as f32)
            .normalize()
            .to_array(); // Normal of the sloped face

        let size = self.size as f32;
        let height = self.height as f32;

        // Define vertex positions, normals, and texture coordinates for the step slope
        let mut positions: Vec<[f32; 3]> = vec![
            // Face 1 - Ground level
            [0., 0., 0.],
            [size / 2., 0., 0.],
            [size / 2., size, 0.],
            [0., size, 0.],
            // Face 2 - Vertical face
            [size / 2., 0., 0.],
            [size / 2., 0., height],
            [size / 2., size, 0.],
            // Face 3 - Sloped face
            [size / 2., 0., height],
            [size, 0., height],
            [size, size, 0.],
            [size / 2., size, 0.],
        ];
        let mut normals = vec![
            up, up, up, up,           // Ground face normals
            back, back, back,         // Vertical face normals
            slope_normal, slope_normal, slope_normal, slope_normal, // Sloped face normals
        ];
        let mut uvs = vec![
            [0., 0.], [1. / 3., 0.], [1. / 3., 1.], [0., 1.], // Ground face UVs
            [1. / 3., 0.], [2. / 3., 0.], [1. / 3., 1.],      // Vertical face UVs
            [2. / 3., 0.], [1., 0.], [1., 1.], [2. / 3., 1.], // Sloped face UVs
        ];

        // Define indices for triangle rendering
        let mut indices = vec![
            [0, 1, 3], [2, 3, 1], // Ground face
            [4, 5, 6],            // Vertical face
            [7, 8, 10], [9, 10, 8], // Sloped face
        ];

        // Apply mirroring and rotation transformations to the mesh
        mirror_mesh(
            size,
            &mut positions,
            &mut normals,
            &mut indices,
            &mut uvs,
            &self.mirror,
        );
        rotate_mesh(size, &mut positions, &mut normals, &mut uvs, &self.rotate);

        // Flatten the indices
        let indices: Vec<u32> = indices.into_iter().flatten().collect();

        // Create the mesh and set attributes
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(Indices::U32(indices)));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

        mesh
    }
}