use bevy::prelude::shape::Quad;
use bevy::prelude::*;
use bevy::sprite::Mesh2dHandle;

use crate::mouse_position::MousePosition;
use crate::shadow_pass::LIGHT_PASS_LAYER;

#[derive(Component)]
pub struct Light;

pub fn move_light_system(
    mouse_pos: Res<MousePosition>,
    mut query: Query<&mut Transform, With<Light>>,
) {
    for mut transform in query.iter_mut() {
        transform.translation.x = mouse_pos.world.x;
        transform.translation.y = mouse_pos.world.y;
    }
}

pub fn setup_lights_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mesh = meshes.add(Mesh::from(Quad::new(Vec2::new(300.0, 300.0))));

    let mat = materials.add(ColorMaterial {
        color: Color::WHITE,
        ..Default::default()
    });

    commands
        .spawn_bundle(ColorMesh2dBundle {
            mesh: Mesh2dHandle(mesh),
            material: mat,
            ..default()
        })
        .insert(Light)
        .insert(LIGHT_PASS_LAYER);
}
