use bevy::core_pipeline::draw_2d_graph;
use bevy::core_pipeline::node;
use bevy::core_pipeline::Transparent2d;
use bevy::prelude::*;
use bevy::render::camera::ActiveCamera;
use bevy::render::camera::CameraTypePlugin;
use bevy::render::render_graph::Node;
use bevy::render::render_graph::NodeRunError;
use bevy::render::render_graph::RenderGraph;
use bevy::render::render_graph::RenderGraphContext;
use bevy::render::render_graph::SlotValue;
use bevy::render::render_phase::RenderPhase;
use bevy::render::renderer::RenderContext;
use bevy::render::RenderApp;
use bevy::render::RenderStage;
use bevy::sprite::Material2dPlugin;

use crate::setup_shadow_pass_system;
use crate::shadow_material::ShadowMaterial;
use crate::shadow_pass::LightPassCamera;
use crate::shadow_pass::ShadowMap;
use crate::shadow_pass::LIGHT_PASS_DRIVER;

pub struct ShadowPlugin;

impl Plugin for ShadowPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(Material2dPlugin::<ShadowMaterial>::default())
            .add_plugin(CameraTypePlugin::<LightPassCamera>::default())
            .insert_resource(ShadowMap(None))
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
    }
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
