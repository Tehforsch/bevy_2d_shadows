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
    for (i, mut transform) in query.iter_mut().enumerate() {
        transform.translation.x = mouse_pos.world.x + i as f32 * 400.0;
        transform.translation.y = mouse_pos.world.y;
    }
}

pub fn spawn_lights_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let mesh = meshes.add(Mesh::from(Quad::new(Vec2::new(800.0, 800.0))));

    for color in [Color::WHITE, Color::RED] {
        let mat = materials.add(ColorMaterial {
            color,
            texture: Some(asset_server.load("light.png")),
            ..Default::default()
        });

        commands
            .spawn_bundle(ColorMesh2dBundle {
                mesh: Mesh2dHandle(mesh.clone()),
                material: mat.clone(),
                ..default()
            })
            .insert(Light)
            .insert(LIGHT_PASS_LAYER);
    }
}
