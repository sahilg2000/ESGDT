use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use rigid_body::sva::Vector;

use crate::{
    rotate::{rotate_mesh, rotate_point},
    GridElement, Interference, Rotate, RotationDirection,
};

#[derive(Default)]
pub struct Slope {
    pub size: f64,
    pub height: f64,
    pub rotate: Rotate,
}

impl GridElement for Slope {
    fn interference(&self, mut point: Vector) -> Option<Interference> {
        rotate_point(
            &mut point,
            self.size,
            &self.rotate,
            RotationDirection::Reverse,
        );
        let size = self.size;
        let height = self.height;

        // point is above slope, no contact possible
        if point.z > height {
            return None;
        }
        // point is outside of area
        if point.x < 0.0 || point.x > size || point.y < 0.0 || point.y > size {
            return None;
        }

        // point is in area
        let top_normal = Vector::new(0., height, size).normalize();
        let top_point = Vector::new(0., 0., height);
        let normal_interference = -top_normal.dot(&(point - top_point));
        if normal_interference < 0.0 {
            // point is not in contact with slope
            return None;
        } else {
            let mut interference = Interference {
                magnitude: normal_interference,
                position: point - normal_interference * top_normal,
                normal: top_normal,
            };
            interference.rotate(size, &self.rotate, RotationDirection::Forward);
            return Some(interference);
        }
    }

    fn mesh(&self) -> Mesh {
        let slope_normal = Vec3::new(0., self.height as f32, self.size as f32)
            .normalize()
            .to_array();

        let size = self.size as f32;
        let height = self.height as f32;

        let mut positions: Vec<[f32; 3]> = vec![
            // face 1
            [0., 0., height],
            [size, 0., height],
            [size, size, 0.],
            [0., size, 0.],
        ];
        let mut normals = vec![
            // face 1
            slope_normal,
            slope_normal,
            slope_normal,
            slope_normal,
        ];
        let mut uvs = vec![
            // face 1
            [0., 0.],
            [1., 0.],
            [1., 1.],
            [0., 1.],
        ];

        let indices = vec![
            // face 1
            [0, 1, 3],
            [2, 3, 1],
        ];

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
