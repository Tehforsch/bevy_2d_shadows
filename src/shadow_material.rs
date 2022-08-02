use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::render_resource::AsBindGroup;
use bevy::sprite::Material2d;

#[derive(Debug, Clone, TypeUuid, AsBindGroup)]
#[uuid = "106b9f9a-bf10-11ec-9d64-0242ac120002"]
pub struct ShadowMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub texture: Handle<Image>,
    #[texture(2)]
    #[sampler(3)]
    pub shadow_map: Handle<Image>,
}

impl ShadowMaterial {
    pub fn new(texture: Handle<Image>, shadow_map: Handle<Image>) -> Self {
        Self {
            texture,
            shadow_map,
        }
    }
}

impl Material2d for ShadowMaterial {
    fn vertex_shader() -> bevy::render::render_resource::ShaderRef {
        "shaders/shader.wgsl".into()
    }

    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        "shaders/shader.wgsl".into()
    }
}
