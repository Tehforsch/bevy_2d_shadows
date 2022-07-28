use bevy::ecs::system::lifetimeless::SRes;
use bevy::ecs::system::SystemParamItem;
use bevy::prelude::shape::Quad;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::render_asset::PrepareAssetError;
use bevy::render::render_asset::RenderAsset;
use bevy::render::render_asset::RenderAssets;
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
use bevy::render::render_resource::SamplerBindingType;
use bevy::render::render_resource::ShaderStages;
use bevy::render::render_resource::TextureSampleType;
use bevy::render::render_resource::TextureViewDimension;
use bevy::render::renderer::RenderDevice;
use bevy::sprite::Material2d;
use bevy::sprite::Material2dPipeline;
use bevy::sprite::Material2dPlugin;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::sprite::Mesh2dHandle;

#[derive(AsStd140, Clone, Debug, Default)]
pub struct MyData {
    x: f32,
}

#[derive(Debug, Clone, TypeUuid)]
#[uuid = "106b9f9a-bf10-11ec-9d64-0242ac120002"]
pub struct MyMaterial {
    pub texture: Handle<Image>,
    pub shader_data: MyData,
}

impl From<Handle<Image>> for MyMaterial {
    fn from(texture: Handle<Image>) -> Self {
        let shader_data = MyData { x: 0.0 };
        Self {
            texture,
            shader_data,
        }
    }
}

#[derive(Clone)]
pub struct MyGpuMaterial {
    bind_group: BindGroup,
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

fn setup(
    mut commands: Commands,
    mut custom_materials: ResMut<Assets<MyMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
) {
    let mesh = Mesh::from(Quad::new(Vec2::new(600.0, 400.0)));
    commands.spawn_bundle(MaterialMesh2dBundle {
        mesh: Mesh2dHandle(meshes.add(mesh)),
        material: custom_materials.add(MyMaterial::from(asset_server.load("tree.png"))),
        ..default()
    });
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(Material2dPlugin::<MyMaterial>::default())
        .add_startup_system(setup)
        .run();
}
