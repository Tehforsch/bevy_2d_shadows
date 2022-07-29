use bevy::math::Vec2;
use bevy::prelude::*;

use crate::world_camera::WorldCamera;

#[derive(Debug, Default, Clone)]
pub struct MousePosition {
    pub window: Vec2,
    pub world: Vec4,
}

pub fn track_mouse_world_position_system(
    mut events_reader_cursor: EventReader<CursorMoved>,
    mut position: ResMut<MousePosition>,
    windows: Res<Windows>,
    camera_query: Query<&Transform, With<WorldCamera>>,
) {
    let camera_transform = camera_query.single();
    let window = windows.get_primary().unwrap();
    if let Some(cursor_pos_window) = events_reader_cursor.iter().next() {
        let size = Vec2::new(window.width() as f32, window.height() as f32);
        let p = cursor_pos_window.position - size / 2.0;
        position.world = camera_transform.compute_matrix() * p.extend(0.0).extend(1.0);

        position.window = cursor_pos_window.position;
    }
}
