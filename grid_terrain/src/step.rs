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

// Defines a step with specified size, height, rotation, and mirroring properties
#[derive(Default)]
pub struct Step {
    pub size: f64,       // Base size of the step
    pub height: f64,     // Height of the step
    pub rotate: Rotate,  // Rotation properties
    pub mirror: Mirror,  // Mirroring properties
}

impl GridElement for Step {
    // Calculates the interference of a point with the step
    fn interference(&self, mut point: Vector) -> Option<Interference> {
        // Apply rotation and mirroring to the point
        rotate_point(
            &mut point,
            self.size,
            &self.rotate,
            RotationDirection::Reverse,
        );
        mirror_point(&mut point, self.size, &self.mirror);

        let size = self.size;
        let height = self.height;

        // Check if the point is above the step's height
        if point.z > height {
            return None;
        }

        // Check if the point is outside the horizontal boundaries of the step
        if point.x < 0.0 || point.x > size || point.y < 0.0 || point.y > size {
            return None;
        }

        // Handle points in the area but not on the step
        if point.x < size / 2.0 {
            if point.z > 0.0 {
                return None;
            }
            // Interference when the point is below the ground plane
            let mut interference = Interference {
                magnitude: -point.z,
                position: Vector::new(point.x, point.y, 0.0),
                normal: Vector::z(), // Normal pointing up
            };
            // Apply mirroring and rotation to the interference
            interference.mirror(size, &self.mirror);
            interference.rotate(size, &self.rotate, RotationDirection::Forward);
            return Some(interference);
        }

        // Handle points in the area and on the step
        let z_interference = height - point.z;
        let x_interference = point.x - size / 2.0;
        let yp_interference = size - point.y; // Distance to the +y edge
        let yn_interference = point.y;       // Distance to the -y edge

        // Point is closer to the top of the step than the side
        if (x_interference > z_interference)
            & (yp_interference > z_interference)
            & (yn_interference > z_interference)
        {
            let mut interference = Interference {
                magnitude: z_interference,
                position: Vector::new(point.x, point.y, height),
                normal: Vector::z(),
            };
            interference.mirror(size, &self.mirror);
            interference.rotate(size, &self.rotate, RotationDirection::Forward);
            return Some(interference);
        }

        // Point is closer to the x side of the step than the top
        if (yp_interference > x_interference) & (yn_interference > x_interference) {
            let mut interference = Interference {
                magnitude: x_interference,
                position: Vector::new(size / 2.0, point.y, point.z),
                normal: -Vector::x(),
            };
            interference.mirror(size, &self.mirror);
            interference.rotate(size, &self.rotate, RotationDirection::Forward);
            return Some(interference);
        };

        // Handle points closer to the +y or -y sides
        if yp_interference > yn_interference {
            let mut interference = Interference {
                magnitude: yn_interference,
                position: Vector::new(point.x, 0.0, point.z),
                normal: -Vector::y(),
            };
            interference.mirror(size, &self.mirror);
            interference.rotate(size, &self.rotate, RotationDirection::Forward);
            return Some(interference);
        } else {
            let mut interference = Interference {
                magnitude: yp_interference,
                position: Vector::new(point.x, size, point.z),
                normal: Vector::y(),
            };
            interference.mirror(size, &self.mirror);
            interference.rotate(size, &self.rotate, RotationDirection::Forward);
            return Some(interference);
        }
    }

    // Generates the mesh representation of the step
    fn mesh(&self) -> Mesh {
        // Normals for the mesh faces
        let up = Vec3::Z.to_array();         // Normal pointing up
        let backwards = (-Vec3::X).to_array(); // Normal pointing backward
        let side_py = Vec3::Y.to_array();    // +Y normal
        let side_ny = (-Vec3::Y).to_array(); // -Y normal

        let size = self.size as f32;
        let height = self.height as f32;

        // Vertex positions for the mesh
        let mut positions: Vec<[f32; 3]> = vec![
            // Bottom face
            [0., 0., 0.],
            [size / 2., 0., 0.],
            [size / 2., size, 0.],
            [0., size, 0.],
            // Vertical face
            [size / 2., 0., 0.],
            [size / 2., 0., height],
            [size / 2., size, height],
            [size / 2., size, 0.],
            // Top face
            [size / 2., 0., height],
            [size, 0., height],
            [size, size, height],
            [size / 2., size, height],
            // -Y face
            [size, 0., 0.],
            [size, 0., height],
            [size / 2., 0., height],
            [size / 2., 0., 0.],
            // +Y face
            [size, size, 0.],
            [size / 2., size, 0.],
            [size / 2., size, height],
            [size, size, height],
        ];

        // Normals for each vertex
        let mut normals = vec![
            up, up, up, up,                 // Bottom face
            backwards, backwards, backwards, backwards, // Vertical face
            up, up, up, up,                 // Top face
            side_ny, side_ny, side_ny, side_ny, // -Y face
            side_py, side_py, side_py, side_py, // +Y face
        ];

        // UV coordinates for texturing
        let mut uvs = vec![
            [0., 0.], [1. / 3., 0.], [1. / 3., 1.], [0., 1.], // Bottom face
            [1. / 3., 0.], [2. / 3., 0.], [2. / 3., 1.], [1. / 3., 1.], // Vertical face
            [2. / 3., 0.], [1., 0.], [1., 1.], [2. / 3., 1.],           // Top face
            [1., 0.], [1., 1.], [2. / 3., 1.], [2. / 3., 0.],           // -Y face
            [1. / 3., 0.], [1. / 3., 1.], [2. / 3., 1.], [2. / 3., 0.], // +Y face
        ];

        // Indices for rendering triangles
        let mut indices = vec![
            [0, 1, 3], [2, 3, 1],     // Bottom face
            [4, 5, 7], [6, 7, 5],     // Vertical face
            [8, 9, 11], [10, 11, 9],  // Top face
            [12, 13, 15], [14, 15, 13], // -Y face
            [16, 17, 19], [18, 19, 17], // +Y face
        ];

        // Apply transformations to the mesh
        mirror_mesh(
            size,
            &mut positions,
            &mut normals,
            &mut indices,
            &mut uvs,
            &self.mirror,
        );
        rotate_mesh(size, &mut positions, &mut normals, &mut uvs, &self.rotate);

        // Flatten indices for the mesh
        let indices: Vec<u32> = indices.into_iter().flatten().map(|x| x as u32).collect();

        // Create the mesh and set its attributes
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(Indices::U32(indices)));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh
    }
}
