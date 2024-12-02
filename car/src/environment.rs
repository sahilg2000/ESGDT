use bevy::prelude::*;

use grid_terrain::{
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
}