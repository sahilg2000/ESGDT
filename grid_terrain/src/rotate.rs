use rigid_body::sva::Vector;

pub enum RotationDirection {
    Forward,
    Reverse,
}

#[derive(Default)]
pub enum Rotate {
    #[default]
    Zero,
    Ninety,
    OneEighty,
    TwoSeventy,
}

pub fn rotate_mesh(
    size: f32,
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    rotation: &Rotate,
) {
    match rotation {
        Rotate::Zero => {}
        Rotate::Ninety => {
            for position in positions.iter_mut() {
                let x = position[0];
                let y = position[1];
                position[0] = size - y;
                position[1] = x;
            }
            for normal in normals.iter_mut() {
                let x = normal[0];
                let y = normal[1];
                normal[0] = -y;
                normal[1] = x;
            }
            for uv in uvs.iter_mut() {
                let x = uv[0];
                let y = uv[1];
                uv[0] = -y;
                uv[1] = x;
            }
        }
        Rotate::OneEighty => {
            for position in positions.iter_mut() {
                let x = position[0];
                let y = position[1];
                position[0] = -x + size;
                position[1] = -y + size;
            }
            for normal in normals.iter_mut() {
                let x = normal[0];
                let y = normal[1];
                normal[0] = -x;
                normal[1] = -y;
            }
            for uv in uvs.iter_mut() {
                let x = uv[0];
                let y = uv[1];
                uv[0] = -x;
                uv[1] = -y;
            }
        }
        Rotate::TwoSeventy => {
            for position in positions.iter_mut() {
                let x = position[0];
                let y = position[1];
                position[0] = y;
                position[1] = size - x;
            }
            for normal in normals.iter_mut() {
                let x = normal[0];
                let y = normal[1];
                normal[0] = y;
                normal[1] = -x;
            }
            for uv in uvs.iter_mut() {
                let x = uv[0];
                let y = uv[1];
                uv[0] = -y;
                uv[1] = x;
            }
        }
    }
}

pub fn rotate_point(point: &mut Vector, size: f64, rotate: &Rotate, direction: RotationDirection) {
    match (rotate, direction) {
        (Rotate::Zero, _) => {}
        (Rotate::Ninety, RotationDirection::Forward)
        | (Rotate::TwoSeventy, RotationDirection::Reverse) => {
            let x = point.x;
            let y = point.y;
            point.x = size - y;
            point.y = x;
        }
        (Rotate::OneEighty, _) => {
            point.x = size - point.x;
            point.y = size - point.y;
        }
        (Rotate::TwoSeventy, RotationDirection::Forward)
        | (Rotate::Ninety, RotationDirection::Reverse) => {
            let x = point.x;
            let y = point.y;
            point.x = y;
            point.y = size - x;
        }
    }
}
