use crate::definitions::{MeshDef, MeshTypeDef};
use bevy::ecs::system::EntityCommands;
use bevy::prelude::Mesh as BevyMesh;
use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;

#[derive(Debug)]
pub struct BoxMesh {
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
    pub min_z: f32,
    pub max_z: f32,
}

impl BoxMesh {
    pub fn new(min_x: f32, max_x: f32, min_y: f32, max_y: f32, min_z: f32, max_z: f32) -> Self {
        Self {
            min_x,
            max_x,
            min_y,
            max_y,
            min_z,
            max_z,
        }
    }
    pub fn to_bevy_mesh(self) -> BevyMesh {
        BevyMesh::from(shape::Box {
            max_x: self.max_x,
            min_x: self.min_x,
            max_y: self.max_y,
            min_y: self.min_y,
            max_z: self.max_z,
            min_z: self.min_z,
        })
    }
}

#[derive(Debug)]
pub struct CylinderMesh {
    pub height: f32,
    pub radius: f32,
}

impl CylinderMesh {
    pub fn new(height: f32, radius: f32) -> Self {
        Self { height, radius }
    }

    pub fn to_bevy_mesh(self) -> BevyMesh {
        BevyMesh::from(shape::Cylinder {
            height: self.height,
            radius: self.radius,
            ..default()
        })
    }
}

#[derive(Debug)]
pub struct WheelMesh {
    pub radius: f32,
    pub width: f32,
}

impl WheelMesh {
    pub fn to_bevy_mesh(self) -> BevyMesh {
        BevyMesh::from(shape::Cylinder {
            height: self.width,
            radius: self.radius,
            ..default()
        })
    }

    pub fn add_mesh(
        &self,
        commands: &mut Commands,
        wheel_id: Entity,
        meshes: &mut ResMut<Assets<BevyMesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        _mesh_def: &MeshDef,
    ) {
        add_wheel_mesh(
            commands,
            wheel_id,
            meshes,
            materials,
            self.width,
            self.radius,
        );
    }
}

pub fn add_wheel_mesh(
    commands: &mut Commands,
    wheel_id: Entity,
    meshes: &mut ResMut<Assets<BevyMesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    width: f32,
    radius: f32,
) {
    let outer_radius = radius as f32;
    let inner_radius = 0.25 * outer_radius;
    let width = width as f32;
    let subdivisions = 10;

    // four quadrants of alternating black and white wedges
    for i in 0..4 {
        let start_angle = (i as f32 * 90.0).to_radians();
        let end_angle = ((i + 1) as f32 * 90.0).to_radians();
        let mesh = cylinder_wedge(
            inner_radius,
            outer_radius,
            start_angle,
            end_angle,
            width,
            subdivisions,
        );
        // alternate between white and black
        let color = if i % 2 == 0 {
            Color::rgba(1.0, 1.0, 1.0, 1.0)
        } else {
            Color::rgba(0.0, 0.0, 0.0, 1.0)
        };
        // add mesh
        commands
            .spawn(PbrBundle {
                mesh: meshes.add(mesh),
                material: materials.add(StandardMaterial {
                    base_color: color,
                    ..default()
                }),
                transform: Transform::from_xyz(0., 0., 0.),
                ..default()
            })
            .set_parent(wheel_id);
    }
}

#[derive(Debug)]
pub enum Mesh {
    Box(BoxMesh),
    Wheel(WheelMesh),
    Cylinder(CylinderMesh),
    File(String),
}

impl Mesh {
    pub fn from_mesh_def(mesh_def: &MeshDef) -> Self {
        match mesh_def.mesh_type.clone() {
            MeshTypeDef::Box {
                dimensions: [x, y, z],
            } => Self::Box(BoxMesh::new(
                -x / 2.,
                x / 2.,
                -y / 2.,
                y / 2.,
                -z / 2.,
                z / 2.,
            )),
            MeshTypeDef::Cylinder { height, radius } => {
                Self::Cylinder(CylinderMesh::new(height, radius))
            }
            MeshTypeDef::Wheel { radius, width } => Self::Wheel(WheelMesh { radius, width }),
            MeshTypeDef::File { file_name } => Self::File(file_name),
        }
    }
}

// should make this match the mesh type above
pub fn add_cube_mesh(
    entity: &mut EntityCommands,
    meshes: &mut ResMut<Assets<BevyMesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: [f64; 3],
    dimensions: [f64; 3],
    color: Color,
) {
    let pos_f32 = [position[0] as f32, position[1] as f32, position[2] as f32];
    let dims_f32 = [
        dimensions[0] as f32,
        dimensions[1] as f32,
        dimensions[2] as f32,
    ];
    entity.insert(PbrBundle {
        mesh: meshes.add(BevyMesh::from(shape::Box {
            max_x: pos_f32[0] + dims_f32[0] / 2.,
            min_x: pos_f32[0] - dims_f32[0] / 2.,
            max_y: pos_f32[1] + dims_f32[1] / 2.,
            min_y: pos_f32[1] - dims_f32[1] / 2.,
            max_z: pos_f32[2] + dims_f32[2] / 2.,
            min_z: pos_f32[2] - dims_f32[2] / 2.,
        })),
        material: materials.add(StandardMaterial {
            base_color: color,
            ..default()
        }),
        transform: Transform::from_xyz(0., 0., 0.),
        ..default()
    });
}

pub fn add_obj_mesh(
    commands: &mut Commands,
    parent: Entity,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: &mut Res<AssetServer>,
    mesh_def: &MeshDef,
    obj_file: &String,
) {
    commands
        .spawn(PbrBundle {
            mesh: asset_server.load(obj_file),
            material: materials.add(StandardMaterial {
                base_color: mesh_def.color,
                ..default()
            }),
            transform: Transform::from(&mesh_def.transform),
            ..default()
        })
        .set_parent(parent);
}

pub fn cylinder_wedge(
    inner_radius: f32,
    outer_radius: f32,
    start_angle: f32,
    end_angle: f32,
    width: f32,
    subdivisions: usize,
) -> BevyMesh {
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

    let mut mesh = BevyMesh::new(PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(Indices::U32(indices)));
    mesh.insert_attribute(BevyMesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(BevyMesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(BevyMesh::ATTRIBUTE_UV_0, uvs);
    mesh
}
