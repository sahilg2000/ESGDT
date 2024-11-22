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

#[derive(Default)]
pub struct Step {
    pub size: f64,
    pub height: f64,
    pub rotate: Rotate,
    pub mirror: Mirror,
}

impl GridElement for Step {
    fn interference(&self, mut point: Vector) -> Option<Interference> {
        rotate_point(
            &mut point,
            self.size,
            &self.rotate,
            RotationDirection::Reverse,
        );
        mirror_point(&mut point, self.size, &self.mirror);
        let size = self.size;
        let height = self.height;

        // point is above step, no contact possible
        if point.z > height {
            return None;
        }
        // point is outside of area
        if point.x < 0.0 || point.x > size || point.y < 0.0 || point.y > size {
            return None;
        }
        // point is in the area, but not on the step
        if point.x < size / 2.0 {
            if point.z > 0.0 {
                return None;
            }
            let mut interference = Interference {
                magnitude: -point.z,
                position: Vector::new(point.x, point.y, 0.0),
                normal: Vector::z(),
            };
            interference.mirror(size, &self.mirror);
            interference.rotate(size, &self.rotate, RotationDirection::Forward);
            return Some(interference);
        }

        // point is in area, and on the step
        let z_interference = height - point.z;
        let x_interference = point.x - size / 2.0;
        let yp_interference = size - point.y;
        let yn_interference = point.y;

        if (x_interference > z_interference)
            & (yp_interference > z_interference)
            & (yn_interference > z_interference)
        {
            // point is closer to the top of the step than the side
            // point should be pushed up
            let mut interference = Interference {
                magnitude: z_interference,
                position: Vector::new(point.x, point.y, height),
                normal: Vector::z(),
            };
            interference.mirror(size, &self.mirror);
            interference.rotate(size, &self.rotate, RotationDirection::Forward);
            return Some(interference);
        }
        // point is closer to the x side of the step than the top
        // point should be pushed back
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

    fn mesh(&self) -> Mesh {
        let up = Vec3::Z.to_array();
        let backwards = (-Vec3::X).to_array();

        let side_py = Vec3::Y.to_array();
        let side_ny = (-Vec3::Y).to_array();

        let size = self.size as f32;
        let height = self.height as f32;

        let mut positions: Vec<[f32; 3]> = vec![
            // face 1 (bottom)
            [0., 0., 0.],
            [size / 2., 0., 0.],
            [size / 2., size, 0.],
            [0., size, 0.],
            // face 2 (vertical)
            [size / 2., 0., 0.],
            [size / 2., 0., height],
            [size / 2., size, height],
            [size / 2., size, 0.],
            // face 3 (top)
            [size / 2., 0., height],
            [size, 0., height],
            [size, size, height],
            [size / 2., size, height],
            // -y face
            [size, 0., 0.],
            [size, 0., height],
            [size / 2., 0., height],
            [size / 2., 0., 0.],
            // +y face
            [size, size, 0.],
            [size / 2., size, 0.],
            [size / 2., size, height],
            [size, size, height],
        ];
        let mut normals = vec![
            up, up, up, up, // face 1
            backwards, backwards, backwards, backwards, // face 2
            up, up, up, up, // face 3
            side_ny, side_ny, side_ny, side_ny, // -y sides
            side_py, side_py, side_py, side_py, // +y sides
        ];
        let mut uvs = vec![
            // face 1
            [0., 0.],
            [1. / 3., 0.],
            [1. / 3., 1.],
            [0., 1.],
            // face 2
            [1. / 3., 0.],
            [2. / 3., 0.],
            [2. / 3., 1.],
            [1. / 3., 1.],
            // face 3
            [2. / 3., 0.],
            [1., 0.],
            [1., 1.],
            [2. / 3., 1.],
            // -y face (not sure what these should be)
            [1., 0.],
            [1., 1.],
            [2. / 3., 1.],
            [2. / 3., 0.],
            // +y face  (not sure what these should be)
            [1. / 3., 0.],
            [1. / 3., 1.],
            [2. / 3., 1.],
            [2. / 3., 0.],
        ];

        let mut indices = vec![
            // face 1
            [0, 1, 3],
            [2, 3, 1],
            // face 2
            [4, 5, 7],
            [6, 7, 5],
            // face 3
            [8, 9, 11],
            [10, 11, 9],
            // -y face
            [12, 13, 15],
            [14, 15, 13],
            // +y face
            [16, 17, 19],
            [18, 19, 17],
        ];

        mirror_mesh(
            size,
            &mut positions,
            &mut normals,
            &mut indices,
            &mut uvs,
            &self.mirror,
        );
        rotate_mesh(size, &mut positions, &mut normals, &mut uvs, &self.rotate);

        let indices: Vec<u32> = indices.into_iter().flatten().map(|x| x as u32).collect();

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(Indices::U32(indices)));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh
    }
}
