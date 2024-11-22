use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use rigid_body::sva::Vector;

use crate::{GridElement, Interference};

pub struct Function {
    pub size: [f64; 2],
    pub functions: Vec<Box<dyn Fn(f64, f64) -> f64>>,
    pub derivatives: Vec<Box<dyn Fn(f64, f64) -> (f64, f64)>>,
}

impl Default for Function {
    fn default() -> Self {
        Self {
            size: [10.0, 10.],
            functions: vec![Box::new(|x, _y| x.cos())],
            derivatives: vec![Box::new(|x, _y| (-x.sin(), 0.))],
        }
    }
}

fn evaluate(
    functions: &Vec<Box<dyn Fn(f64, f64) -> f64>>,
    derivatives: &Vec<Box<dyn Fn(f64, f64) -> (f64, f64)>>,
    point: Vector,
) -> (f64, f64, f64) {
    let mut height = 1.0;
    let mut derivative_parts_x: Vec<f64> = functions.iter().map(|_x| 1.0_f64).collect();
    let mut derivative_parts_y: Vec<f64> = functions.iter().map(|_y| 1.0_f64).collect();

    for (i_fun, (fun, der)) in functions.iter().zip(derivatives.iter()).enumerate() {
        let fun_val = (fun)(point.x, point.y);
        let der_val = (der)(point.x, point.y);

        height *= fun_val;
        for i in 0..functions.len() {
            if i == i_fun {
                derivative_parts_x[i] *= der_val.0;
                derivative_parts_y[i] *= der_val.1;
            } else {
                derivative_parts_x[i] *= fun_val;
                derivative_parts_y[i] *= fun_val;
            }
        }
    }

    let derivative_x: f64 = derivative_parts_x.iter().sum();
    let derivative_y: f64 = derivative_parts_y.iter().sum();
    (height, derivative_x, derivative_y)
}

impl GridElement for Function {
    fn interference(&self, point: Vector) -> Option<Interference> {
        let size = self.size;

        // point is outside of area
        if point.x < 0.0 || point.x > size[0] || point.y < 0.0 || point.y > size[1] {
            // should be unreachable. Not great that it fails silently...
            return None;
        }

        let (height, dx, dy) = evaluate(&self.functions, &self.derivatives, point);

        if point.z > height {
            // return immediately if point is above surface
            return None;
        }

        let interference_magnitude = height - point.z;
        let contact_point = Vector::new(point.x, point.y, height);
        let normal = Vector::new(-dx, -dy, 1.).normalize();

        // // iterate to improve contact_point and normal (no significant improvement)
        // for _ in 0..0 {
        //     let (height, dx, dy) = evaluate(&self.functions, &self.derivatives, contact_point);
        //     normal = Vector::new(-dx, -dy, 1.).normalize();
        //     let function_point = Vector::new(contact_point.x, contact_point.y, height);
        //     let separation = function_point - point;
        //     interference_magnitude = separation.dot(&normal);
        //     contact_point = point + normal * interference_magnitude;
        // }

        Some(Interference {
            magnitude: interference_magnitude,
            position: contact_point,
            normal,
        })
    }

    fn mesh(&self) -> Mesh {
        let size = [self.size[0] as f32, self.size[1] as f32];
        let x_vertex_count = 100;
        let y_vertex_count = 100;

        let num_vertices = (y_vertex_count * x_vertex_count) as usize;
        let num_indices = ((y_vertex_count - 1) * (x_vertex_count - 1) * 6) as usize;

        let mut positions: Vec<[f32; 3]> = Vec::with_capacity(num_vertices);
        let mut normals: Vec<[f32; 3]> = Vec::with_capacity(num_vertices);
        let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(num_vertices);
        let mut indices: Vec<u32> = Vec::with_capacity(num_indices);

        for y_vert in 0..y_vertex_count {
            for x_vert in 0..x_vertex_count {
                let x_normalized = x_vert as f32 / (x_vertex_count - 1) as f32;
                let y_normalized = y_vert as f32 / (y_vertex_count - 1) as f32;

                let x = x_normalized * size[0];
                let y = y_normalized * size[1];
                let (height, dx, dy) = evaluate(
                    &self.functions,
                    &self.derivatives,
                    Vector::new(x as f64, y as f64, 0.),
                );

                let normal = Vec3::new(-dx as f32, dy as f32, 1.).normalize().to_array();
                positions.push([x, y, height as f32]);
                normals.push(normal);
                uvs.push([x_normalized, 1. - y_normalized]);
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
