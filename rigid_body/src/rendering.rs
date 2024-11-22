use crate::mesh::{add_obj_mesh, Mesh as RigidBodyMesh};
use crate::{definitions::MeshDef, joint::Joint};
use bevy::prelude::*;

pub fn startup_rendering(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut asset_server: Res<AssetServer>,
    mut joint_mesh_query: Query<(Entity, &MeshDef), (With<Joint>, With<MeshDef>)>,
    mut joint_no_mesh_query: Query<Entity, (With<Joint>, Without<MeshDef>)>,
) {
    for (entity, mesh_def) in joint_mesh_query.iter_mut() {
        let mut entity_commands = commands.entity(entity);
        entity_commands.insert(SpatialBundle::default());
        let rb_mesh = RigidBodyMesh::from_mesh_def(mesh_def);

        match rb_mesh {
            RigidBodyMesh::Box(box_mesh) => {
                let mesh = meshes.add(box_mesh.to_bevy_mesh());
                let mut entity_commands = commands.spawn(PbrBundle {
                    mesh: mesh,
                    material: materials.add(mesh_def.color.into()),
                    transform: Transform::from(&mesh_def.transform),
                    ..Default::default()
                });
                entity_commands.set_parent(entity);
            }
            RigidBodyMesh::Wheel(wheel_mesh) => {
                wheel_mesh.add_mesh(&mut commands, entity, &mut meshes, &mut materials, mesh_def)
            }
            RigidBodyMesh::Cylinder(cylinder_mesh) => {
                let mesh = meshes.add(cylinder_mesh.to_bevy_mesh());
                let mut entity_commands = commands.spawn(PbrBundle {
                    mesh: mesh,
                    material: materials.add(mesh_def.color.into()),
                    transform: Transform::from(&mesh_def.transform),
                    ..Default::default()
                });
                entity_commands.set_parent(entity);
            }
            RigidBodyMesh::File(file_name) => add_obj_mesh(
                &mut commands,
                entity,
                &mut materials,
                &mut asset_server,
                mesh_def,
                &file_name,
            ),
        };
    }

    // add spatial bundle to joints without meshes
    for entity in joint_no_mesh_query.iter_mut() {
        let mut entity_commands = commands.entity(entity);
        entity_commands.insert(SpatialBundle::default());
    }
}
