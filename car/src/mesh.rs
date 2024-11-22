use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};

pub fn cylinder_wedge(
    inner_radius: f32,
    outer_radius: f32,
    start_angle: f32,
    end_angle: f32,
    width: f32,
    subdivisions: usize,
) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    let hw = width / 2.;

    let angle_step = (end_angle - start_angle) / subdivisions as f32;
    let r_in = inner_radius;
    let r_out = outer_radius;

    let yp = Vec3::Y.to_array();
    let yn = (-Vec3::Y).to_array();
    let mut ind0 = 0;
    for i in 0..subdivisions {
        let angle_0 = start_angle + i as f32 * angle_step;
        let angle_1 = angle_0 + angle_step;
        let a0cos = angle_0.cos();
        let a0sin = angle_0.sin();
        let a1cos = angle_1.cos();
        let a1sin = angle_1.sin();

        positions.extend(vec![
            // +y face
            [r_in * a0cos, hw, r_in * a0sin],
            [r_in * a1cos, hw, r_in * a1sin],
            [r_out * a1cos, hw, r_out * a1sin],
            [r_out * a0cos, hw, r_out * a0sin],
            // -y face
            [r_in * a0cos, -hw, r_in * a0sin],
            [r_in * a1cos, -hw, r_in * a1sin],
            [r_out * a1cos, -hw, r_out * a1sin],
            [r_out * a0cos, -hw, r_out * a0sin],
            // inner face
            [r_in * a0cos, hw, r_in * a0sin],
            [r_in * a0cos, -hw, r_in * a0sin],
            [r_in * a1cos, -hw, r_in * a1sin],
            [r_in * a1cos, hw, r_in * a1sin],
            // outer face
            [r_out * a0cos, hw, r_out * a0sin],
            [r_out * a0cos, -hw, r_out * a0sin],
            [r_out * a1cos, -hw, r_out * a1sin],
            [r_out * a1cos, hw, r_out * a1sin],
        ]);

        let a0_dir = Vec3::new(a0cos, 0., a0sin);
        let a1_dir = Vec3::new(a1cos, 0., a1sin);

        let a0_in = (-a0_dir).to_array();
        let a0_out = a0_dir.to_array();
        let a1_in = (-a1_dir).to_array();
        let a1_out = a1_dir.to_array();

        normals.extend(vec![
            yp, yp, yp, yp, // +y face
            yn, yn, yn, yn, // -y face
            a0_in, a0_in, a1_in, a1_in, // inner face
            a0_out, a0_out, a1_out, a1_out, // outer face
        ]);

        let ui0 = i as f32 / subdivisions as f32;
        let ui1 = (i + 1) as f32 / subdivisions as f32;

        uvs.extend(vec![
            // not sure if these are correct, they don't matter for now
            [ui0, 0.],
            [ui1, 0.],
            [ui1, 1.],
            [ui0, 1.],
            [ui0, 0.],
            [ui1, 0.],
            [ui1, 1.],
            [ui0, 1.],
            [ui0, 0.],
            [ui1, 0.],
            [ui1, 1.],
            [ui0, 1.],
            [ui0, 0.],
            [ui1, 0.],
            [ui1, 1.],
            [ui0, 1.],
        ]);

        let mut ind: Vec<u32> = vec![
            // +y face
            [0, 1, 2],
            [2, 3, 0],
            // -y face
            [4, 7, 6],
            [6, 5, 4],
            // inner face
            [8, 9, 10],
            [10, 11, 8],
            // outer face
            [12, 15, 14],
            [14, 13, 12],
        ]
        .into_iter()
        .flatten()
        .collect();
        ind.iter_mut().for_each(|x| {
            *x += ind0;
        });
        indices.extend(ind);

        ind0 += 16;
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(Indices::U32(indices)));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh
}
