use bevy::asset::AssetServerSettings;
use bevy::core_pipeline::RenderTargetClearColors;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::diagnostic::LogDiagnosticsPlugin;
use bevy::prelude::shape::Quad;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::render::render_resource::Extent3d;
use bevy::render::render_resource::TextureDescriptor;
use bevy::render::render_resource::TextureDimension;
use bevy::render::render_resource::TextureFormat;
use bevy::render::render_resource::TextureUsages;
use bevy::sprite::Material2dPlugin;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::sprite::Mesh2dHandle;
use bevy::window::PresentMode;
use light::move_light_system;
use light::spawn_lights_system;
use mouse_position::track_mouse_world_position_system;
use mouse_position::MousePosition;
use shadow_caster::ShadowCaster;
use shadow_material::ShadowMaterial;
use shadow_pass::ShadowMap;
use shadow_pass::LIGHT_PASS_LAYER;
use shadow_plugin::ShadowPlugin;
use world_camera::setup_camera_system;

use crate::shadow_pass::new_light_camera;

mod light;
mod mouse_position;
mod shadow_caster;
mod shadow_material;
mod shadow_pass;
mod shadow_plugin;
mod world_camera;

fn get_shadow_map(window: &Window) -> Image {
    println!("Window size was: {},{}", window.width(), window.height());
    let size = Extent3d {
        width: window.width() as u32,
        height: window.height() as u32,
        ..default()
    };
    let mut shadow_map = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
        },
        ..default()
    };
    shadow_map.resize(size);
    shadow_map
}

fn setup_shadow_pass_system(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut clear_colors: ResMut<RenderTargetClearColors>,
    mut windows: ResMut<Windows>,
    mut shadow_map: ResMut<ShadowMap>,
) {
    let window = windows.get_primary_mut().unwrap();
    let shadow_map_handle = images.add(get_shadow_map(window));
    *shadow_map = ShadowMap(Some(shadow_map_handle.clone()));

    let render_target = RenderTarget::Image(shadow_map_handle.clone());
    clear_colors.insert(render_target.clone(), Color::rgb(0.7, 0.7, 0.7));
    commands
        .spawn_bundle(new_light_camera(shadow_map_handle))
        .insert(LIGHT_PASS_LAYER);
}

fn spawn_objects_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut custom_materials: ResMut<Assets<ShadowMaterial>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    shadow_map: Res<ShadowMap>,
) {
    let mesh = Mesh::from(Quad::new(Vec2::new(1000.0, 1000.0)));
    commands.spawn_bundle(MaterialMesh2dBundle {
        mesh: Mesh2dHandle(meshes.add(mesh)),
        material: custom_materials.add(ShadowMaterial::new(
            asset_server.load("tree.png"),
            shadow_map.0.clone().unwrap(),
        )),
        ..default()
    });
    let mesh = Mesh::from(Quad::new(Vec2::new(50.0, 50.0)));
    let color_material = ColorMaterial {
        color: Color::RED,
        ..default()
    };
    commands
        .spawn_bundle(MaterialMesh2dBundle::<ColorMaterial> {
            mesh: Mesh2dHandle(meshes.add(mesh)),
            material: color_materials.add(color_material),
            transform: Transform {
                translation: Vec3::new(200.0 as f32, 0.0, 0.1),
                ..default()
            },
            ..default()
        })
        .insert(ShadowCaster::new());
}

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_plugin(FrameTimeDiagnosticsPlugin)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(ShadowPlugin)
        .insert_resource(AssetServerSettings {
            watch_for_changes: true,
            ..default()
        })
        .insert_resource(WindowDescriptor {
            present_mode: PresentMode::Immediate,
            ..default()
        })
        .insert_resource(MousePosition::default())
        .add_startup_system(spawn_objects_system.after(setup_shadow_pass_system))
        .add_startup_system(setup_camera_system)
        .add_startup_system(spawn_lights_system)
        .add_system(move_light_system)
        .add_system(track_mouse_world_position_system);

    app.run();
}
