use std::collections::HashMap;

use bevy::math::Vec3Swizzles;
use bevy::math::Vec4Swizzles;
use bevy::prelude::shape::Quad;
use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::mesh::PrimitiveTopology;
use bevy::render::mesh::VertexAttributeValues;
use bevy::sprite::Mesh2dHandle;
use ordered_float::OrderedFloat;

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
    for (mut shadow_caster, shadow_caster_mesh_handle, shadow_caster_transform) in
        shadow_casters.iter_mut()
    {
        for (light_entity, light_transform, _) in lights.iter() {
            match shadow_caster.shadows.get(&light_entity) {
                Some(_shadow) => adjust_shadow_mesh(),
                None => {
                    let light_pos = light_transform.translation.xy();
                    let shadow_caster_mesh: &Mesh =
                        meshes.get(shadow_caster_mesh_handle.0.clone()).unwrap();
                    let shadow_mesh =
                        get_shadow_mesh(light_pos, &shadow_caster_mesh, shadow_caster_transform);
                    let handle = meshes.add(shadow_mesh);
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

fn get_shadow_mesh(
    light_pos: Vec2,
    shadow_caster_mesh: &Mesh,
    shadow_caster_transform: &Transform,
) -> Mesh {
    let points = shadow_caster_mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .unwrap();
    let get_angle = |point: &Vec4| -> OrderedFloat<f32> {
        let dist = point.xy() - light_pos;
        OrderedFloat(dist.x.atan2(dist.y))
    };
    let matrix = shadow_caster_transform.compute_matrix();
    if let VertexAttributeValues::Float32x3(points) = points {
        let transformed = points
            .iter()
            .map(|point| matrix * Vec4::new(point[0], point[1], 0.0, 1.0));
        let a = transformed.max_by_key(get_angle).unwrap();
        let transformed = points
            .iter()
            .map(|point| matrix * Vec4::new(point[0], point[1], 0.0, 1.0));
        let b = transformed.min_by_key(get_angle).unwrap();
        get_infinity_projection_quad(light_pos, Vec2::new(a[0], a[1]), Vec2::new(b[0], b[1]))
    } else {
        unreachable!()
    }
}

fn get_infinity_projection_quad(light_pos: Vec2, a: Vec2, b: Vec2) -> Mesh {
    // We don't take infinity all that literally here. Gotta move this to homogeneous coordinates if I ever understand how to
    let a_infinity = light_pos + (a - light_pos) * 100.0;
    let b_infinity = light_pos + (b - light_pos) * 100.0;

    let (u_left, u_right) = (1.0, 0.0);
    let vertices = [
        (a, [0.0, 0.0, 1.0], [u_left, 1.0]),
        (b, [0.0, 0.0, 1.0], [u_left, 0.0]),
        (b_infinity, [0.0, 0.0, 1.0], [u_right, 0.0]),
        (a_infinity, [0.0, 0.0, 1.0], [u_right, 1.0]),
    ];

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
