use bevy::prelude::Commands;
use bevy::prelude::Component;
use bevy::prelude::OrthographicCameraBundle;

#[derive(Component)]
pub struct WorldCamera;

pub fn setup_camera(mut commands: Commands) {
    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(WorldCamera);
}
