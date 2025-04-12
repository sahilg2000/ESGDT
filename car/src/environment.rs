use bevy::prelude::*;
use grid_terrain::GridTerrain;

/*use grid_terrain::{
    examples::{steps, table_top, wave},
    GridTerrain,
};

pub fn build_environment(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Ambient and directional lighting setup has been moved to `weather.rs`

    let size = 20.0;

    let height = 2.;
    let table_elements = table_top(size, height);

    let height = 0.3;
    let wave_length = 4.;
    let wave_elements = wave(size, height, wave_length);

    let step_elements = steps(size, vec![0.2, 0.4, 0.6]);

    let mut elements = table_elements;
    elements.extend(wave_elements);
    elements.extend(step_elements);

    let grid_terrain = GridTerrain::new(elements, [size, size]);
    let empty_parent = commands.spawn(SpatialBundle::default()).id();

    grid_terrain.build_meshes(&mut commands, &mut meshes, &mut materials, empty_parent);
    commands.insert_resource(grid_terrain);
}*/



pub fn build_environment(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) { 
    // Load texture and create a material with it
    let texture_handle = asset_server.load("texture/track.png");
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle),
        unlit: true, // ensures texture visibility
        ..default()
    });

    // Create a large plane mesh
    let mesh_handle = meshes.add(Mesh::from(shape::Plane {
        size: 200.0,
        subdivisions: 0, // 0 means no extra tessellation, just a single quad
    }));

    // Add the plane with the texture applied
    commands.spawn(PbrBundle {
        mesh: mesh_handle,
        material: material_handle,
        transform: Transform {
            translation: Vec3::new(90.0, -30.0, -0.05),
            rotation: Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
            ..default()
        },
        ..default()
    });


    commands.insert_resource(GridTerrain::new(vec![], [1.0, 1.0]));
}