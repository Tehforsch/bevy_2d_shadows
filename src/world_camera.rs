use bevy::prelude::Camera2dBundle;
use bevy::prelude::Commands;
use bevy::prelude::Component;

#[derive(Component)]
pub struct WorldCamera;

pub fn setup_camera_system(mut commands: Commands) {
    commands
        .spawn(Camera2dBundle::default())
        .insert(WorldCamera);
}
