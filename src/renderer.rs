use std::num::NonZero;
use bevy::{
    asset::*,
    image::*,
    prelude::*, 
    ecs::component::Tick,
    platform::collections::*,
    core_pipeline::core_3d::*,
    render::{
        *,
        view,
        view::*,
        texture::*,
        renderer::*,
        render_phase::*, 
        render_asset::*,
        render_resource::*,
        extract_resource::*,
        extract_component::*,
        render_resource::binding_types::*
    }
};
use super::stdb::{Block, ModelType};

pub struct Model;
impl Model {
    pub fn load(block: &Block, assets: &AssetServer) -> Option<Handle<Image>> {
        match &block.model {
            ModelType::Cube(path) => {
                Some(assets.load(format!("embedded://{}", path)))
            },
            _ => None
        }
    }
}

#[derive(Resource)]
pub struct TexturesHandler(pub Vec<Option<Handle<Image>>>);

#[derive(Resource)]
pub struct TexturesBindGroup(pub BindGroup);

#[derive(Clone, Resource, ExtractResource, Default)]
pub struct LoadBlocksHandler(pub HashMap<u16, Block>);

/// Default textures sampler
pub fn default_sampler() -> ImageSamplerDescriptor {
    ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        mag_filter: ImageFilterMode::Nearest,
        min_filter: ImageFilterMode::Linear,
        mipmap_filter: ImageFilterMode::Nearest,
        ..default()
    }
}

fn prepare_chunks_buffers(
    mut commands: Commands,
    render_queue: Res<RenderQueue>,
    render_device: Res<RenderDevice>,
    pipeline: Option<Res<ChunksPipeline>>,
    meshes: Query<(Entity, Ref<ChunkMesh>), Without<ChunkBuffers>>,
) {
    let Some(pipeline) = pipeline else { return };

    for (entity, mesh) in meshes {
        let transform = Mat4::from_translation(mesh.position);

        let mut vertices = BufferVec::new(BufferUsages::VERTEX);
        let mut indices = BufferVec::new(BufferUsages::INDEX);
        let mut uniform = DynamicUniformBuffer::new_with_alignment(64);

        for vertex in mesh.vertices.iter() {
            vertices.push(*vertex);
        }
        for index in mesh.indices.iter() {
            indices.push(*index);
        }

        uniform.push(&ChunkUniform { transform });

        vertices.write_buffer(&render_device, &render_queue);
        indices.write_buffer(&render_device, &render_queue);
        uniform.write_buffer(&render_device, &render_queue);

        info!("Prepare buffers...");
        info!("Vertices: {}", vertices.len());

        let bind = render_device.create_bind_group(
            "chunk_bind_group",
            &pipeline.chunk_layout,
            &BindGroupEntries::single(&uniform),
        );

        let buffers = ChunkBuffers { vertices, indices, bind };
        commands.entity(entity).insert(buffers);
    }
}

fn prepare_uniforms(
    mut commands: Commands,
    cameras: Query<(Entity, Ref<ExtractedView>)>
) {
    for (camera, view) in cameras.iter() {
        let world_from_view = view.world_from_view.compute_matrix();
        let uniform = CameraUniform {
            clip_from_view: view.clip_from_view,
            view_from_world: world_from_view.inverse()
        };

        commands.entity(camera).insert(uniform);
    }
}

fn prepare_pipeline(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    loader: Option<Res<ChunkShaderLoader>>,
    pipeline: Option<Res<ChunksPipeline>>,
    textures: Option<Res<TexturesHandler>>,
    camera_uniforms: Res<ComponentUniforms<CameraUniform>>,
    assets: Res<AssetServer>,
) {
    let Some(loader) = loader else { return };
    if pipeline.is_some() { return; }

    // Load textures
    let Some(textures) = textures else { return };

    let Some(camera_binding) = camera_uniforms.uniforms().binding() else { return };

    let camera_layout = render_device.create_bind_group_layout(
        "camera_layout",
        &BindGroupLayoutEntries::single(
            ShaderStages::VERTEX,
            uniform_buffer::<CameraUniform>(true),
        ),
    );

    let chunk_layout = render_device.create_bind_group_layout(
        "chunk_layout",
        &BindGroupLayoutEntries::single(
            ShaderStages::VERTEX,
            uniform_buffer::<ChunkUniform>(false),
        ),
    );

    let textures_layout = render_device.create_bind_group_layout(
        "textures_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                texture_2d(TextureSampleType::Float { filterable: false })
                    .count(NonZero::<u32>::new(textures.0.len() as u32).unwrap()),
                sampler(SamplerBindingType::NonFiltering),
            )
        ),
    );

    let camera_bind = render_device.create_bind_group(
        "camera_bind_group",
        &camera_layout,
        &BindGroupEntries::single(camera_binding)
    );

    let sampler = render_device.create_sampler(&SamplerDescriptor {
        mag_filter: FilterMode::Nearest,
        min_filter: FilterMode::Nearest,
        mipmap_filter: FilterMode::Nearest,
        ..Default::default()
    });

    info!("Main chunk shader loaded");
    let shader = assets.load(loader.0);
    commands.insert_resource(ChunksPipeline { 
        shader, 
        camera_layout, 
        camera_bind,
        chunk_layout, 
        textures_layout,
        sampler
    });

    commands.init_resource::<SpecializedRenderPipelines<ChunksPipeline>>()
}

fn prepare_textures(
    mut commands: Commands,
    assets: Res<AssetServer>,
    blocks: Option<Res<LoadBlocksHandler>>,
    textures: Option<Res<TexturesHandler>>,
) {
    let Some(blocks) = blocks else { return };
    if textures.is_some() { return };
 
    let l = blocks.0.len();
    let mut textures = Vec::with_capacity(blocks.0.len());

    for i in 0..l as u16 {
        let Some(block) = blocks.0.get(&i) else { return };

        textures.push(Model::load(block, &assets));
    }
    
    commands.insert_resource(TexturesHandler(textures));
}

fn prepare_textures_bind(
    mut commands: Commands,
    fallback: Res<FallbackImage>,
    render_device: Res<RenderDevice>,
    bind: Option<Res<TexturesBindGroup>>,
    pipeline: Option<Res<ChunksPipeline>>,
    handler: Option<Res<TexturesHandler>>,
    gpu_images: Res<RenderAssets<GpuImage>>,
) {
    let Some(pipeline) = pipeline else { return };
    let Some(handler) = handler else { return };
    if bind.is_some() { return };

    let mut images = Vec::with_capacity(handler.0.len());

    for handle_opt in handler.0.iter() {
        let Some(handle) = handle_opt else {
            images.push(None);
            continue;
        };

        match gpu_images.get(handle.id()) {
            Some(image) => images.push(Some(image)),
            None => return,
        }
    }

    let fallback_image = &fallback.d2;
    let textures = vec![&fallback_image.texture_view; images.len()];
    let mut textures: Vec<_> = textures.into_iter().map(|texture| &**texture).collect();
    for (id, image_opt) in images.into_iter().enumerate() {
        if let Some(image) = image_opt {
            textures[id] = &*image.texture_view;
        }
    }

    let bind_group = render_device.create_bind_group(
        "textures_bind_group",
        &pipeline.textures_layout,
        &BindGroupEntries::sequential((&textures[..], &pipeline.sampler)),
    );

    commands.insert_resource(TexturesBindGroup(bind_group));
}

fn queue_chunks(
    pipeline_cache: Res<PipelineCache>,
    pipeline: Option<Res<ChunksPipeline>>,
    textures: Option<Res<TexturesBindGroup>>,
    mut opaque_render_phases: ResMut<ViewBinnedRenderPhases<Opaque3d>>,
    opaque_draw_functions: Res<DrawFunctions<Opaque3d>>,
    render_pipelines: Option<ResMut<SpecializedRenderPipelines<ChunksPipeline>>>,
    views: Query<(&ExtractedView, &RenderVisibleEntities, &Msaa)>,
    mut next_tick: Local<Tick>,
) {
    // If unprepared - skip
    let Some(pipeline) = pipeline else { return };
    if textures.is_none() { return };

    let mut render_pipelines = render_pipelines.unwrap();

    let draw_chunk = opaque_draw_functions
        .read()
        .id::<DrawChunkCommands>();

    for (view, view_visible_entities, msaa) in views.iter() {
        let Some(opaque_phase) = opaque_render_phases.get_mut(&view.retained_view_entity) else {
            continue;
        };

        for &entity in view_visible_entities.get::<ChunkMesh>().iter() {
            let pipeline_id = render_pipelines.specialize(
                &pipeline_cache,
                &pipeline,
                *msaa,
            );

            let this_tick = next_tick.get() + 1;
            next_tick.set(this_tick);

            opaque_phase.add(
                Opaque3dBatchSetKey {
                    draw_function: draw_chunk,
                    pipeline: pipeline_id,
                    material_bind_group_index: None,
                    lightmap_slab: None,
                    vertex_slab: default(),
                    index_slab: None,
                },
                Opaque3dBinKey {
                    asset_id: AssetId::<Mesh>::invalid().untyped(),
                },
                entity,
                InputUniformIndex::default(),
                BinnedRenderPhaseType::NonMesh,
                *next_tick,
            );
        }
    }
}

#[derive(Clone, Component, ExtractComponent)]
#[require(VisibilityClass)]
#[component(on_add = view::add_visibility_class::<ChunkMesh>)]
pub struct ChunkMesh {
    position: Vec3,
    vertices: Vec<u32>,
    indices: Vec<u32>,
}

impl ChunkMesh {
    pub fn new(position: Vec3, vertices: Vec<u32>, indices: Vec<u32>) -> Self {
        Self { position, vertices, indices }
    }
}

#[derive(Component)]
pub struct ChunkBuffers {
    vertices: BufferVec<u32>,
    indices: BufferVec<u32>,
    bind: BindGroup
}

// Main chunks shader path
#[derive(Clone, Resource, ExtractResource)]
pub struct ChunkShaderLoader(pub &'static str);

#[derive(Resource)]
struct ChunksPipeline {
    shader: Handle<Shader>,
    camera_layout: BindGroupLayout,
    camera_bind: BindGroup,
    chunk_layout: BindGroupLayout,
    textures_layout: BindGroupLayout,
    sampler: Sampler
}

#[derive(Component, Default, Clone, Copy, ShaderType)]
pub struct CameraUniform {
    // Projection matrix
    clip_from_view: Mat4,
    // View matrix
    view_from_world: Mat4
}

#[derive(Default, Clone, Copy, ShaderType)]
pub struct ChunkUniform {
    transform: Mat4,
}

#[derive(Default)]
pub struct RenderingPlugin;
impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<ChunkMesh>::default())
            .add_plugins(UniformComponentPlugin::<CameraUniform>::default())
            .add_plugins(ExtractResourcePlugin::<ChunkShaderLoader>::default())
            .add_plugins(ExtractResourcePlugin::<LoadBlocksHandler>::default());

        app.get_sub_app_mut(RenderApp)
            .unwrap()
            .add_render_command::<Opaque3d, DrawChunkCommands>()
            .add_systems(
                Render,
                (
                    prepare_pipeline,
                    prepare_uniforms,
                    prepare_textures,
                    prepare_textures_bind,
                    prepare_chunks_buffers
                ).in_set(RenderSet::Prepare),
            )
            .add_systems(Render, queue_chunks.in_set(RenderSet::Queue));
    }
}

impl SpecializedRenderPipeline for ChunksPipeline {
    type Key = Msaa;

    fn specialize(&self, msaa: Self::Key) -> RenderPipelineDescriptor {
        RenderPipelineDescriptor {
            label: Some("chunk render pipeline".into()),
            layout: vec![self.camera_layout.clone(), self.textures_layout.clone(), self.chunk_layout.clone()],
            push_constant_ranges: vec![],
            vertex: VertexState {
                shader: self.shader.clone(),
                shader_defs: vec![],
                entry_point: "vertex".into(),
                buffers: vec![VertexBufferLayout {
                    array_stride: VertexFormat::Uint32.size(),
                    step_mode: VertexStepMode::Vertex,
                    attributes: vec![
                        VertexAttribute {
                            format: VertexFormat::Uint32,
                            offset: 0,
                            shader_location: 0,
                        },
                    ],
                }],
            },
            fragment: Some(FragmentState {
                shader: self.shader.clone(),
                shader_defs: vec![],
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: Some(DepthStencilState {
                format: CORE_3D_DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Greater,
                stencil: default(),
                bias: default(),
            }),
            multisample: MultisampleState {
                count: msaa.samples(),
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            zero_initialize_workgroup_memory: false,
        }
    }
}

struct BindChunk;
impl<P: PhaseItem> RenderCommand<P> for BindChunk {
    type Param = (
        Res<'static, ChunksPipeline>,
        Res<'static, TexturesBindGroup>,
    );

    type ViewQuery = (
        Ref<'static, CameraUniform>,
        Ref<'static, DynamicUniformIndex<CameraUniform>>,
    );

    type ItemQuery = ();

    fn render<'w>(
        _: &P,
        (_, camera_index): bevy::ecs::query::ROQueryItem<'w, Self::ViewQuery>,
        _: Option<bevy::ecs::query::ROQueryItem<'w, Self::ItemQuery>>,
        (pipeline, textures): bevy::ecs::system::SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let pipeline = pipeline.into_inner();
        let textures = textures.into_inner();

        pass.set_bind_group(
            0,
            &pipeline.camera_bind,
            &[camera_index.index()]
        );

        pass.set_bind_group(
            1,
            &textures.0,
            &[]
        );

        RenderCommandResult::Success
    }
}

// Main draw command
struct DrawChunk;
impl<P: PhaseItem> RenderCommand<P> for DrawChunk {
    type Param = ();

    type ViewQuery = ();

    type ItemQuery = Ref<'static, ChunkBuffers>;

    fn render<'w>(
        _: &P,
        _: bevy::ecs::query::ROQueryItem<'w, Self::ViewQuery>,
        mesh: Option<bevy::ecs::query::ROQueryItem<'w, Self::ItemQuery>>,
        _: bevy::ecs::system::SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(buffers) = mesh else { 
            return RenderCommandResult::Skip
        };

        let buffers = buffers.into_inner();

        pass.set_bind_group(
            2,
            &buffers.bind,
            &[]
        );

        pass.set_vertex_buffer(
            0,
            buffers
                .vertices
                .buffer()
                .unwrap()
                .slice(..),
        );

        pass.set_index_buffer(
            buffers
                .indices
                .buffer()
                .unwrap()
                .slice(..),
            0,
            IndexFormat::Uint32,
        );

        let l = buffers.indices.len() as u32;
        pass.draw_indexed(0..l, 0, 0..1);

        RenderCommandResult::Success
    }
}

// todo view bind command
type DrawChunkCommands = (SetItemPipeline, BindChunk, DrawChunk);
