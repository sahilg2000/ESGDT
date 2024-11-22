use bevy::prelude::*;

use crate::camera_az_el::AzElCamera;

#[derive(Resource)]
pub struct CameraParentList {
    pub list: Vec<Entity>,
    pub active: usize,
}

pub fn camera_parent_system(
    mut commands: Commands,
    mut parent_list: ResMut<CameraParentList>,
    mut query: Query<Entity, With<AzElCamera>>,
    focused_windows: Query<(Entity, &Window)>,
    input: Res<Input<KeyCode>>,
) {
    for (_window, focus) in focused_windows.iter() {
        if !focus.focused {
            continue;
        }

        if parent_list.list.len() == 0 {
            continue;
        }

        if input.just_pressed(KeyCode::C) {
            parent_list.active = (parent_list.active + 1) % parent_list.list.len();
        }

        // update the parent on every frame...
        if let Ok(camera_entity) = query.get_single_mut() {
            let parent_entity = parent_list.list[parent_list.active];
            if commands.get_entity(parent_entity).is_some() {
                if let Some(mut camera_entity_commands) = commands.get_entity(camera_entity) {
                    camera_entity_commands.set_parent(parent_entity);
                }
            } else {
                if let Some(mut camera_entity_commands) = commands.get_entity(camera_entity) {
                    camera_entity_commands.remove_parent();
                }
            }
        }
    }
}
