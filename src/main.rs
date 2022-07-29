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
use bevy::render::view::RenderLayers;
use bevy::render::RenderApp;
use bevy::render::RenderStage;
use bevy::sprite::Material2dPlugin;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::sprite::Mesh2dHandle;
use bevy::window::PresentMode;
use light_pass::LightPassCamera;
use light_pass::LIGHT_PASS_DRIVER;
use mouse_position::track_mouse_world_position_system;
use mouse_position::MousePosition;
use shadow_material::ShadowMaterial;

use crate::light_pass::new_light_camera;

mod light_pass;
mod mouse_position;
mod shadow_material;

#[derive(Component)]
pub struct WorldCamera;

fn setup(
    mut commands: Commands,
    mut custom_materials: ResMut<Assets<ShadowMaterial>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut clear_colors: ResMut<RenderTargetClearColors>,
    mut windows: ResMut<Windows>,
) {
    asset_server.watch_for_changes().unwrap();
    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(WorldCamera);
    let window = windows.get_primary_mut().unwrap();
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
    let shadow_map_handle = images.add(shadow_map);

    let mesh = meshes.add(Mesh::from(Quad::new(Vec2::new(300.0, 300.0))));

    // This specifies the layer used for the first pass, which will be attached to the first pass camera and cube.
    let first_pass_layer = RenderLayers::layer(1);

    let mat = materials.add(ColorMaterial {
        color: Color::WHITE,
        ..Default::default()
    });

    // The cube that will be rendered to the texture.
    commands
        .spawn_bundle(ColorMesh2dBundle {
            mesh: Mesh2dHandle(mesh),
            material: mat,
            ..default()
        })
        .insert(FirstPassCube)
        .insert(first_pass_layer);

    // First pass camera
    let render_target = RenderTarget::Image(shadow_map_handle.clone());
    clear_colors.insert(render_target.clone(), Color::rgb(0.7, 0.7, 0.7));
    commands
        .spawn_bundle(new_light_camera(shadow_map_handle.clone()))
        .insert(first_pass_layer);
    // NOTE: omitting the RenderLayers component for this camera may cause a validation error:
    //
    // thread 'main' panicked at 'wgpu error: Validation Error
    //
    //    Caused by:
    //        In a RenderPass
    //          note: encoder = `<CommandBuffer-(0, 1, Metal)>`
    //        In a pass parameter
    //          note: command buffer = `<CommandBuffer-(0, 1, Metal)>`
    //        Attempted to use texture (5, 1, Metal) mips 0..1 layers 0..1 as a combination of COLOR_TARGET within a usage scope.
    //
    // This happens because the texture would be written and read in the same frame, which is not allowed.
    // So either render layers must be used to avoid this, or the texture must be double buffered.

    // second pass stuff
    for i in -5..5i32 {
        for j in -5..5i32 {
            let mesh = Mesh::from(Quad::new(Vec2::new(150.0, 150.0)));
            commands.spawn_bundle(MaterialMesh2dBundle {
                mesh: Mesh2dHandle(meshes.add(mesh)),
                material: custom_materials.add(ShadowMaterial::new(
                    asset_server.load("tree.png"),
                    shadow_map_handle.clone(),
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
    app.add_plugins(DefaultPlugins)
        .add_plugin(Material2dPlugin::<ShadowMaterial>::default())
        .add_plugin(CameraTypePlugin::<LightPassCamera>::default())
        .add_system(move_light_system)
        .add_system(track_mouse_world_position_system)
        .add_plugin(FrameTimeDiagnosticsPlugin)
        .insert_resource(WindowDescriptor {
            present_mode: PresentMode::Immediate,
            ..default()
        })
        .add_plugin(LogDiagnosticsPlugin::default())
        .insert_resource(MousePosition::default())
        .add_startup_system(setup);

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

// Marks the first pass cube (rendered to a texture.)
#[derive(Component)]
struct FirstPassCube;

fn move_light_system(
    mouse_pos: Res<MousePosition>,
    mut query: Query<&mut Transform, With<FirstPassCube>>,
) {
    for mut transform in query.iter_mut() {
        transform.translation.x = mouse_pos.world.x;
        transform.translation.y = mouse_pos.world.y;
    }
}
