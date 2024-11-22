use bevy::{
    prelude::{Mesh, Vec3},
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use rigid_body::sva::Vector;

use crate::{GridElement, Interference};

pub struct Plane {
    pub size: [f64; 2],
    pub subdivisions: u32,
}

impl GridElement for Plane {
    fn interference(&self, point: Vector) -> Option<Interference> {
        if point.z < 0. {
            return Some(Interference {
                magnitude: -point.z,
                position: Vector::new(point.x, point.y, 0.),
                normal: Vector::z(),
            });
        } else {
            return None;
        }
    }

    fn mesh(&self) -> Mesh {
        let y_vertex_count = self.subdivisions + 2;
        let x_vertex_count = self.subdivisions + 2;
        let num_vertices = (y_vertex_count * x_vertex_count) as usize;
        let num_indices = ((y_vertex_count - 1) * (x_vertex_count - 1) * 6) as usize;
        let up = Vec3::Z.to_array();

        let mut positions: Vec<[f32; 3]> = Vec::with_capacity(num_vertices);
        let mut normals: Vec<[f32; 3]> = Vec::with_capacity(num_vertices);
        let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(num_vertices);
        let mut indices: Vec<u32> = Vec::with_capacity(num_indices);

        for y in 0..y_vertex_count {
            for x in 0..x_vertex_count {
                let tx = x as f32 / (x_vertex_count - 1) as f32;
                let ty = y as f32 / (y_vertex_count - 1) as f32;
                positions.push([tx * self.size[0] as f32, ty * self.size[1] as f32, 0.0]);
                normals.push(up);
                uvs.push([tx, 1.0 - ty]);
            }
        }

        for y in 0..y_vertex_count - 1 {
            for x in 0..x_vertex_count - 1 {
                let quad = y * x_vertex_count + x;
                indices.push(quad);
                indices.push(quad + 1);
                indices.push(quad + x_vertex_count);
                indices.push(quad + x_vertex_count + 1);
                indices.push(quad + x_vertex_count);
                indices.push(quad + 1);
            }
        }

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(Indices::U32(indices)));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh
    }
}
