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
pub struct StepSlope {
    pub size: f64,
    pub height: f64,
    pub rotate: Rotate,
    pub mirror: Mirror,
}

impl GridElement for StepSlope {
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
                position: point - point.z * Vector::z(),
                normal: Vector::z(),
            };
            interference.mirror(size, &self.mirror);
            interference.rotate(size, &self.rotate, RotationDirection::Forward);
            return Some(interference);
        }

        // point is in area, and on the step
        let top_normal = Vector::new(0., height, size).normalize();
        let top_corner = Vector::new(size / 2., 0., height);
        let normal_interference = -top_normal.dot(&(point - top_corner));
        if normal_interference < 0.0 {
            // point is above step, no contact possible
            return None;
        }

        // point is in contact with step
        let x_interference = point.x - size / 2.0;
        if x_interference > normal_interference {
            // point is closer to the top of the step than the side
            // point should be pushed up
            let mut interference = Interference {
                magnitude: normal_interference,
                position: point + normal_interference * top_normal,
                normal: top_normal,
            };
            interference.mirror(size, &self.mirror);
            interference.rotate(size, &self.rotate, RotationDirection::Forward);
            return Some(interference);
        }
        // point is closer to the side of the step than the top
        // point should be pushed back
        let mut interference = Interference {
            magnitude: x_interference,
            position: point - x_interference * Vector::x(),
            normal: -Vector::x(),
        };
        interference.mirror(size, &self.mirror);
        interference.rotate(size, &self.rotate, RotationDirection::Forward);
        return Some(interference);
    }

    fn mesh(&self) -> Mesh {
        let up = Vec3::Z.to_array();
        let back = (-Vec3::X).to_array();
        let slope_normal = Vec3::new(0., self.height as f32, self.size as f32)
            .normalize()
            .to_array();

        let size = self.size as f32;
        let height = self.height as f32;

        let mut positions: Vec<[f32; 3]> = vec![
            // face 1 - ground level
            [0., 0., 0.],
            [size / 2., 0., 0.],
            [size / 2., size, 0.],
            [0., size, 0.],
            // face 2 - vertical face
            [size / 2., 0., 0.],
            [size / 2., 0., height],
            // [size / 2., size, height],
            [size / 2., size, 0.],
            // face 3 - sloped face
            [size / 2., 0., height],
            [size, 0., height],
            [size, size, 0.],
            [size / 2., size, 0.],
        ];
        let mut normals = vec![
            // face 1
            up,
            up,
            up,
            up,
            // face 2
            back,
            back,
            back,
            // face 3
            slope_normal,
            slope_normal,
            slope_normal,
            slope_normal,
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
            // [2. / 3., 1.],
            [1. / 3., 1.],
            // face 3
            [2. / 3., 0.],
            [1., 0.],
            [1., 1.],
            [2. / 3., 1.],
        ];

        let mut indices = vec![
            // face 1
            [0, 1, 3],
            [2, 3, 1],
            // face 2
            [4, 5, 6],
            // face 3
            [7, 8, 10],
            [9, 10, 8],
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

        let indices: Vec<u32> = indices.into_iter().flatten().collect();

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(Indices::U32(indices)));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh
    }
}
