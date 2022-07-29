use bevy::prelude::*;
use bevy::render::camera::CameraProjection;
use bevy::render::camera::DepthCalculation;
use bevy::render::camera::RenderTarget;
use bevy::render::primitives::Frustum;
use bevy::render::view::VisibleEntities;

// The name of the final node of the pass.
pub const LIGHT_PASS_DRIVER: &str = "light_pass_driver";

#[derive(Component, Default)]
pub struct LightPassCamera;

pub fn new_light_camera(render_target: Handle<Image>) -> OrthographicCameraBundle<LightPassCamera> {
    let far = 1000.0;
    let orthographic_projection = OrthographicProjection {
        far,
        depth_calculation: DepthCalculation::ZDifference,
        ..Default::default()
    };
    let transform = Transform::from_xyz(0.0, 0.0, far - 0.1);
    let view_projection =
        orthographic_projection.get_projection_matrix() * transform.compute_matrix().inverse();
    let frustum = Frustum::from_view_projection(
        &view_projection,
        &transform.translation,
        &transform.back(),
        orthographic_projection.far(),
    );
    OrthographicCameraBundle {
        camera: Camera {
            near: orthographic_projection.near,
            far: orthographic_projection.far,
            target: RenderTarget::Image(render_target),
            ..Default::default()
        },
        orthographic_projection,
        visible_entities: VisibleEntities::default(),
        frustum,
        transform,
        global_transform: Default::default(),
        marker: LightPassCamera,
    }
}
