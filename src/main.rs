use bevy::asset::AssetServerSettings;
use bevy::core_pipeline::draw_2d_graph;
use bevy::core_pipeline::node;
use bevy::core_pipeline::RenderTargetClearColors;
use bevy::core_pipeline::Transparent2d;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::diagnostic::LogDiagnosticsPlugin;
use bevy::prelude::shape::Quad;
use bevy::prelude::*;
use bevy::render::camera::ActiveCamera;
use bevy::render::camera::CameraTypePlugin;
use bevy::render::camera::RenderTarget;
use bevy::render::render_graph::Node;
use bevy::render::render_graph::NodeRunError;
use bevy::render::render_graph::RenderGraph;
use bevy::render::render_graph::RenderGraphContext;
use bevy::render::render_graph::SlotValue;
use bevy::render::render_phase::RenderPhase;
use bevy::render::render_resource::Extent3d;
use bevy::render::render_resource::TextureDescriptor;
use bevy::render::render_resource::TextureDimension;
use bevy::render::render_resource::TextureFormat;
use bevy::render::render_resource::TextureUsages;
use bevy::render::renderer::RenderContext;
use bevy::render::RenderApp;
use bevy::render::RenderStage;
use bevy::sprite::Material2dPlugin;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::sprite::Mesh2dHandle;
use bevy::window::PresentMode;
use light::move_light_system;
use light::setup_lights_system;
use mouse_position::track_mouse_world_position_system;
use mouse_position::MousePosition;
use shadow_material::ShadowMaterial;
use shadow_pass::LightPassCamera;
use shadow_pass::ShadowMap;
use shadow_pass::LIGHT_PASS_DRIVER;
use shadow_pass::LIGHT_PASS_LAYER;
use world_camera::setup_camera_system;

use crate::shadow_pass::new_light_camera;

mod light;
mod mouse_position;
mod shadow_material;
mod shadow_pass;
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
    asset_server: Res<AssetServer>,
    shadow_map: Res<ShadowMap>,
) {
    for i in -5..5i32 {
        for j in -5..5i32 {
            let mesh = Mesh::from(Quad::new(Vec2::new(150.0, 150.0)));
            commands.spawn_bundle(MaterialMesh2dBundle {
                mesh: Mesh2dHandle(meshes.add(mesh)),
                material: custom_materials.add(ShadowMaterial::new(
                    asset_server.load("tree.png"),
                    shadow_map.0.clone().unwrap(),
                )),
                transform: Transform {
                    translation: Vec3::new(160.0 * i as f32, 160.0 * j as f32, 0.0),
                    ..default()
                },
                ..default()
            });
        }
    }
}

fn main() {
    let mut app = App::new();

    app.insert_resource(AssetServerSettings {
        watch_for_changes: true,
        ..default()
    })
    .add_plugins(DefaultPlugins)
    .add_plugin(Material2dPlugin::<ShadowMaterial>::default())
    .add_plugin(CameraTypePlugin::<LightPassCamera>::default())
    .insert_resource(ShadowMap(None))
    .add_system(move_light_system)
    .add_startup_system(spawn_objects_system.after(setup_shadow_pass_system))
    .add_system(track_mouse_world_position_system)
    .add_startup_system(setup_camera_system)
    .add_plugin(FrameTimeDiagnosticsPlugin)
    .insert_resource(WindowDescriptor {
        present_mode: PresentMode::Immediate,
        ..default()
    })
    .add_plugin(LogDiagnosticsPlugin::default())
    .insert_resource(MousePosition::default())
    .add_startup_system(setup_lights_system)
    .add_startup_system(setup_shadow_pass_system);

    let render_app = app.sub_app_mut(RenderApp);
    let driver = FirstPassCameraDriver::new(&mut render_app.world);
    // This will add 3D render phases for the new camera.
    render_app.add_system_to_stage(RenderStage::Extract, extract_first_pass_camera_phases);

    let mut graph = render_app.world.resource_mut::<RenderGraph>();

    // Add a node for the first pass.
    graph.add_node(LIGHT_PASS_DRIVER, driver);

    // The first pass's dependencies include those of the main pass.
    graph
        .add_node_edge(node::MAIN_PASS_DEPENDENCIES, LIGHT_PASS_DRIVER)
        .unwrap();

    // Insert the first pass node: CLEAR_PASS_DRIVER -> FIRST_PASS_DRIVER -> MAIN_PASS_DRIVER
    graph
        .add_node_edge(node::CLEAR_PASS_DRIVER, LIGHT_PASS_DRIVER)
        .unwrap();
    graph
        .add_node_edge(LIGHT_PASS_DRIVER, node::MAIN_PASS_DRIVER)
        .unwrap();
    // bevy_mod_debugdump::print_render_graph(&mut app);
    app.run();
}

// Add 3D render phases for FIRST_PASS_CAMERA.
fn extract_first_pass_camera_phases(
    mut commands: Commands,
    active: Res<ActiveCamera<LightPassCamera>>,
) {
    if let Some(entity) = active.get() {
        commands
            .get_or_spawn(entity)
            .insert_bundle((RenderPhase::<Transparent2d>::default(),));
    }
}

// A node for the first pass camera that runs draw_3d_graph with this camera.
struct FirstPassCameraDriver {
    query: QueryState<Entity, With<LightPassCamera>>,
}

impl FirstPassCameraDriver {
    pub fn new(render_world: &mut World) -> Self {
        Self {
            query: QueryState::new(render_world),
        }
    }
}
impl Node for FirstPassCameraDriver {
    fn update(&mut self, world: &mut World) {
        self.query.update_archetypes(world);
    }

    fn run(
        &self,
        graph: &mut RenderGraphContext,
        _render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        for camera in self.query.iter_manual(world) {
            graph.run_sub_graph(draw_2d_graph::NAME, vec![SlotValue::Entity(camera)])?;
        }
        Ok(())
    }
}
