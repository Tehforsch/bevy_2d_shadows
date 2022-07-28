use bevy::core_pipeline::draw_2d_graph;
use bevy::core_pipeline::draw_3d_graph;
use bevy::core_pipeline::node;
use bevy::core_pipeline::AlphaMask3d;
use bevy::core_pipeline::Opaque3d;
use bevy::core_pipeline::RenderTargetClearColors;
use bevy::core_pipeline::Transparent2d;
use bevy::core_pipeline::Transparent3d;
use bevy::ecs::system::lifetimeless::SRes;
use bevy::ecs::system::SystemParamItem;
use bevy::prelude::shape::Quad;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::camera::ActiveCamera;
use bevy::render::camera::CameraProjection;
use bevy::render::camera::CameraTypePlugin;
use bevy::render::camera::DepthCalculation;
use bevy::render::camera::RenderTarget;
use bevy::render::primitives::Frustum;
use bevy::render::render_asset::PrepareAssetError;
use bevy::render::render_asset::RenderAsset;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_graph::Node;
use bevy::render::render_graph::NodeRunError;
use bevy::render::render_graph::RenderGraph;
use bevy::render::render_graph::RenderGraphContext;
use bevy::render::render_graph::SlotValue;
use bevy::render::render_phase::RenderPhase;
use bevy::render::render_resource::std140::AsStd140;
use bevy::render::render_resource::std140::Std140;
use bevy::render::render_resource::BindGroup;
use bevy::render::render_resource::BindGroupDescriptor;
use bevy::render::render_resource::BindGroupEntry;
use bevy::render::render_resource::BindGroupLayout;
use bevy::render::render_resource::BindGroupLayoutDescriptor;
use bevy::render::render_resource::BindGroupLayoutEntry;
use bevy::render::render_resource::BindingResource;
use bevy::render::render_resource::BindingType;
use bevy::render::render_resource::BufferBindingType;
use bevy::render::render_resource::BufferInitDescriptor;
use bevy::render::render_resource::BufferSize;
use bevy::render::render_resource::BufferUsages;
use bevy::render::render_resource::Extent3d;
use bevy::render::render_resource::SamplerBindingType;
use bevy::render::render_resource::ShaderStages;
use bevy::render::render_resource::TextureDescriptor;
use bevy::render::render_resource::TextureDimension;
use bevy::render::render_resource::TextureFormat;
use bevy::render::render_resource::TextureSampleType;
use bevy::render::render_resource::TextureUsages;
use bevy::render::render_resource::TextureViewDimension;
use bevy::render::renderer::RenderContext;
use bevy::render::renderer::RenderDevice;
use bevy::render::view::RenderLayers;
use bevy::render::view::VisibleEntities;
use bevy::render::RenderApp;
use bevy::render::RenderStage;
use bevy::sprite::Material2d;
use bevy::sprite::Material2dPipeline;
use bevy::sprite::Material2dPlugin;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::sprite::Mesh2dHandle;

// The name of the final node of the first pass.
pub const FIRST_PASS_DRIVER: &str = "first_pass_driver";

#[derive(AsStd140, Clone, Debug, Default)]
pub struct MyData {
    x: f32,
}

#[derive(Debug, Clone, TypeUuid)]
#[uuid = "106b9f9a-bf10-11ec-9d64-0242ac120002"]
pub struct MyMaterial {
    pub texture: Handle<Image>,
    pub light_map: Handle<Image>,
    pub shader_data: MyData,
}

impl MyMaterial {
    fn new(texture: Handle<Image>, light_map: Handle<Image>) -> Self {
        let shader_data = MyData { x: 0.0 };
        Self {
            texture,
            light_map,
            shader_data,
        }
    }
}

#[derive(Clone)]
pub struct MyGpuMaterial {
    bind_group: BindGroup,
}

fn print_image_system(images: Res<Assets<Image>>, light_map: Res<RememberLightMap>) {
    let image = images.get(light_map.0.clone().unwrap()).unwrap();
    println!("{}", image.data.iter().map(|x| *x as i32).sum::<i32>());
}

impl RenderAsset for MyMaterial {
    type ExtractedAsset = MyMaterial;
    type PreparedAsset = MyGpuMaterial;
    type Param = (
        SRes<RenderDevice>,
        SRes<RenderAssets<Image>>,
        SRes<Material2dPipeline<Self>>,
    );
    fn extract_asset(&self) -> Self::ExtractedAsset {
        self.clone()
    }

    fn prepare_asset(
        extracted_asset: Self::ExtractedAsset,
        (render_device, gpu_images, material_pipeline): &mut SystemParamItem<Self::Param>,
    ) -> Result<Self::PreparedAsset, PrepareAssetError<Self::ExtractedAsset>> {
        let gpu_image = match gpu_images.get(&extracted_asset.texture) {
            Some(gpu_image) => gpu_image,
            None => return Err(PrepareAssetError::RetryNextUpdate(extracted_asset)),
        };
        let light_map = match gpu_images.get(&extracted_asset.light_map) {
            Some(gpu_image) => gpu_image,
            None => return Err(PrepareAssetError::RetryNextUpdate(extracted_asset)),
        };

        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            contents: extracted_asset.shader_data.as_std140().as_bytes(),
            label: None,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&gpu_image.texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&gpu_image.sampler),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&light_map.texture_view),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::Sampler(&light_map.sampler),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: buffer.as_entire_binding(),
                },
            ],
            label: None,
            layout: &material_pipeline.material2d_layout,
        });

        Ok(MyGpuMaterial { bind_group })
    }
}

impl Material2d for MyMaterial {
    fn vertex_shader(asset_server: &AssetServer) -> Option<Handle<Shader>> {
        Some(asset_server.load("shaders/shader.wgsl"))
    }

    fn fragment_shader(asset_server: &AssetServer) -> Option<Handle<Shader>> {
        Some(asset_server.load("shaders/shader.wgsl"))
    }

    fn bind_group(render_asset: &<Self as RenderAsset>::PreparedAsset) -> &BindGroup {
        &render_asset.bind_group
    }

    fn bind_group_layout(render_device: &RenderDevice) -> BindGroupLayout {
        render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: BufferSize::new(MyData::std140_size_static() as u64),
                    },
                    count: None,
                },
            ],
            label: None,
        })
    }
}

#[derive(Component)]
struct FirstPassObject;

#[derive(Component, Default)]
pub struct FirstPassCamera;

pub fn new_2d(render_target: Handle<Image>) -> OrthographicCameraBundle<FirstPassCamera> {
    // we want 0 to be "closest" and +far to be "farthest" in 2d, so we offset
    // the camera's translation by far and use a right handed coordinate system
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
        marker: FirstPassCamera,
    }
}

struct RememberLightMap(Option<Handle<Image>>);

fn setup(
    mut commands: Commands,
    mut custom_materials: ResMut<Assets<MyMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut clear_colors: ResMut<RenderTargetClearColors>,
    mut remember_light_map: ResMut<RememberLightMap>,
) {
    asset_server.watch_for_changes().unwrap();
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let size = Extent3d {
        width: 512,
        height: 512,
        ..default()
    };
    let mut light_map = Image {
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

    light_map.resize(size);

    for (i, v) in light_map.data.chunks_exact_mut(4).enumerate() {
        let y = i / 512;
        let x = i.rem_euclid(512);
        let dist = ((x as f32 - 250.0).powi(2) + (y as f32 - 200.0).powi(2)).sqrt();
        let val = match dist < 100.0 {
            true => 1.0,
            false => (-(dist - 100.0) * 0.009).exp(),
        };
        v[0] = (val * 255.0) as u8;
        v[1] = (val * 255.0) as u8;
        v[2] = (val * 255.0) as u8;
        v[3] = (val * 255.0) as u8;
    }

    let light_map_handle = images.add(light_map);
    *remember_light_map = RememberLightMap(Some(light_map_handle.clone()));

    let cube_handle = meshes.add(Mesh::from(shape::Cube { size: 4.0 }));

    // This specifies the layer used for the first pass, which will be attached to the first pass camera and cube.
    let first_pass_layer = RenderLayers::layer(1);

    let mat = materials.add(StandardMaterial::default());

    // The cube that will be rendered to the texture.
    commands
        .spawn_bundle(PbrBundle {
            mesh: cube_handle,
            material: mat,
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
            ..default()
        })
        .insert(FirstPassCube)
        .insert(first_pass_layer);

    // Light
    // NOTE: Currently lights are shared between passes - see https://github.com/bevyengine/bevy/issues/3462
    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
        ..default()
    });

    // First pass camera
    let render_target = RenderTarget::Image(light_map_handle.clone());
    clear_colors.insert(render_target.clone(), Color::WHITE);
    commands
        .spawn_bundle(PerspectiveCameraBundle::<FirstPassCamera> {
            camera: Camera {
                target: render_target,
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 15.0))
                .looking_at(Vec3::default(), Vec3::Y),
            ..PerspectiveCameraBundle::new()
        })
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
    let mesh = Mesh::from(Quad::new(Vec2::new(600.0, 400.0)));
    commands.spawn_bundle(MaterialMesh2dBundle {
        mesh: Mesh2dHandle(meshes.add(mesh)),
        material: custom_materials.add(MyMaterial::new(
            asset_server.load("tree.png"),
            light_map_handle,
        )),
        ..default()
    });
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugin(Material2dPlugin::<MyMaterial>::default())
        .add_plugin(CameraTypePlugin::<FirstPassCamera>::default())
        .add_system(rotator_system)
        .insert_resource(RememberLightMap(None))
        .add_system(print_image_system)
        .add_startup_system(setup);

    let render_app = app.sub_app_mut(RenderApp);
    let driver = FirstPassCameraDriver::new(&mut render_app.world);
    // This will add 3D render phases for the new camera.
    render_app.add_system_to_stage(RenderStage::Extract, extract_first_pass_camera_phases);

    let mut graph = render_app.world.resource_mut::<RenderGraph>();

    // Add a node for the first pass.
    graph.add_node(FIRST_PASS_DRIVER, driver);

    // The first pass's dependencies include those of the main pass.
    graph
        .add_node_edge(node::MAIN_PASS_DEPENDENCIES, FIRST_PASS_DRIVER)
        .unwrap();

    // Insert the first pass node: CLEAR_PASS_DRIVER -> FIRST_PASS_DRIVER -> MAIN_PASS_DRIVER
    graph
        .add_node_edge(node::CLEAR_PASS_DRIVER, FIRST_PASS_DRIVER)
        .unwrap();
    graph
        .add_node_edge(FIRST_PASS_DRIVER, node::MAIN_PASS_DRIVER)
        .unwrap();
    // bevy_mod_debugdump::print_render_graph(&mut app);
    app.run();
}

// Add 3D render phases for FIRST_PASS_CAMERA.
fn extract_first_pass_camera_phases(
    mut commands: Commands,
    active: Res<ActiveCamera<FirstPassCamera>>,
) {
    if let Some(entity) = active.get() {
        commands.get_or_spawn(entity).insert_bundle((
            RenderPhase::<Opaque3d>::default(),
            RenderPhase::<AlphaMask3d>::default(),
            RenderPhase::<Transparent3d>::default(),
        ));
    }
}

// A node for the first pass camera that runs draw_3d_graph with this camera.
struct FirstPassCameraDriver {
    query: QueryState<Entity, With<FirstPassCamera>>,
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
            graph.run_sub_graph(draw_3d_graph::NAME, vec![SlotValue::Entity(camera)])?;
        }
        Ok(())
    }
}

// Marks the first pass cube (rendered to a texture.)
#[derive(Component)]
struct FirstPassCube;

fn rotator_system(time: Res<Time>, mut query: Query<&mut Transform, With<FirstPassCube>>) {
    for mut transform in query.iter_mut() {
        transform.rotation *= Quat::from_rotation_x(1.5 * time.delta_seconds());
        transform.rotation *= Quat::from_rotation_z(1.3 * time.delta_seconds());
    }
}
