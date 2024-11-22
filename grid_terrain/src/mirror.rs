use rigid_body::sva::Vector;

#[derive(Default)]
pub enum Mirror {
    #[default]
    None,
    XZ,
    YZ,
}

pub fn mirror_mesh(
    size: f32,
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    indices: &mut Vec<[u32; 3]>,
    _uvs: &mut Vec<[f32; 2]>,
    mirror: &Mirror,
) {
    match mirror {
        Mirror::None => {}
        Mirror::XZ => {
            for i in 0..positions.len() {
                positions[i][1] = -positions[i][1] + size;
                normals[i][1] = -normals[i][1];
            }
            for i in 0..indices.len() {
                let ind1 = indices[i][1];
                let ind2 = indices[i][2];
                indices[i][1] = ind2;
                indices[i][2] = ind1;
            }
        }
        Mirror::YZ => {
            for i in 0..positions.len() {
                positions[i][0] = -positions[i][0] + size;
                normals[i][0] = -normals[i][0];
            }
            for i in 0..indices.len() {
                let ind1 = indices[i][1];
                let ind2 = indices[i][2];
                indices[i][1] = ind2;
                indices[i][2] = ind1;
            }
        }
    }
}

pub fn mirror_point(point: &mut Vector, size: f64, mirror: &Mirror) {
    match mirror {
        Mirror::None => {}
        Mirror::XZ => {
            point.y = size - point.y;
        }
        Mirror::YZ => {
            point.x = size - point.x;
        }
    }
}
