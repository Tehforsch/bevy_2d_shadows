use std::collections::HashMap;

use bevy::math::Vec3Swizzles;
use bevy::prelude::shape::Quad;
use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::mesh::PrimitiveTopology;
use bevy::sprite::Mesh2dHandle;

use crate::light::Light;
use crate::shadow_pass::LIGHT_PASS_LAYER;

#[derive(Component)]
pub struct Shadow;

#[derive(Component)]
pub struct ShadowCaster {
    shadows: HashMap<Entity, Entity>,
}

impl ShadowCaster {
    pub fn new() -> Self {
        Self {
            shadows: HashMap::new(),
        }
    }
}

fn adjust_shadow_mesh() {}

pub fn spawn_shadows_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    lights: Query<(Entity, &Transform, &Light)>,
    mut shadow_casters: Query<(&mut ShadowCaster, &Mesh2dHandle, &Transform)>,
    _shadows: Query<&Shadow>,
) {
    for (mut shadow_caster, _mesh, caster_transform) in shadow_casters.iter_mut() {
        for (light_entity, light_transform, _) in lights.iter() {
            match shadow_caster.shadows.get(&light_entity) {
                Some(_shadow) => adjust_shadow_mesh(),
                None => {
                    let size = 50.0;
                    let light_pos = light_transform.translation.xy();
                    let caster_pos = caster_transform.translation.xy();
                    let dir = (light_pos - caster_pos).normalize();
                    let normal = Vec2::new(-dir.y, dir.x);
                    let a = caster_pos + normal * size;
                    let b = caster_pos - normal * size;
                    let handle = meshes.add(get_mesh(light_pos, a, b));
                    let mat = materials.add(ColorMaterial {
                        color: Color::BLUE,
                        ..Default::default()
                    });

                    let entity = commands
                        .spawn_bundle(ColorMesh2dBundle {
                            mesh: Mesh2dHandle(handle.clone()),
                            material: mat.clone(),
                            ..default()
                        })
                        .insert(LIGHT_PASS_LAYER)
                        .id();
                    shadow_caster.shadows.insert(light_entity, entity);
                }
            }
        }
    }
}

fn get_mesh(light_pos: Vec2, a: Vec2, b: Vec2) -> Mesh {
    let size = 150.0;
    let extent_x = size / 2.0;
    let extent_y = size / 2.0;

    // We don't take infinity all that literally here. Gotta move this to homogeneous coordinates if I ever understand how to
    dbg!(light_pos, a, b);
    let a_infinity = light_pos + (a - light_pos) * 2.0;
    let b_infinity = light_pos + (b - light_pos) * 2.0;

    let (u_left, u_right) = (1.0, 0.0);
    let vertices = [
        (a, [0.0, 0.0, 1.0], [u_left, 1.0]),
        (b, [0.0, 0.0, 1.0], [u_left, 0.0]),
        (b_infinity, [0.0, 0.0, 1.0], [u_right, 0.0]),
        (a_infinity, [0.0, 0.0, 1.0], [u_right, 1.0]),
    ];
    dbg!(vertices);

    let indices = Indices::U32(vec![0, 2, 1, 0, 3, 2]);

    let mut positions = Vec::<[f32; 3]>::new();
    let mut normals = Vec::<[f32; 3]>::new();
    let mut uvs = Vec::<[f32; 2]>::new();
    for (position, normal, uv) in &vertices {
        positions.push([position.x, position.y, 0.0]);
        normals.push(*normal);
        uvs.push(*uv);
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(indices));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh
}
