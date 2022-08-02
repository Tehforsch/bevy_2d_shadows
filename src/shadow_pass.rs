use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::render::view::RenderLayers;

use crate::DARK_COLOR;

pub const LIGHT_PASS_LAYER: RenderLayers = RenderLayers::layer(1);

#[derive(Component, Default)]
pub struct LightPassCamera;

#[derive(Component, Default)]
pub struct ShadowMap(pub Option<Handle<Image>>);

pub fn new_light_camera(render_target: Handle<Image>) -> Camera2dBundle {
    Camera2dBundle {
        camera: Camera {
            priority: -1,
            target: RenderTarget::Image(render_target),
            ..Default::default()
        },
        camera_2d: Camera2d {
            clear_color: ClearColorConfig::Custom(DARK_COLOR),
            ..Default::default()
        },
        ..Default::default()
    }
}
