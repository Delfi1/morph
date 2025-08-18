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
        renderer::*,
        render_phase::*, 
        render_resource::*,
        extract_resource::*,
        extract_component::*,
        render_resource::binding_types::*
    }
};

#[derive(Resource, Default)]
pub struct TexturesHandler(pub HashMap<u16, Option<Handle<Image>>>);

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
    meshes: Query<(Entity, Ref<ChunkMesh>), Without<ChunkBuffers>>,
) {
    for (entity, mesh) in meshes {
        let mut vertices = RawBufferVec::new(BufferUsages::VERTEX);
        let mut indices = RawBufferVec::new(BufferUsages::INDEX);

        for vertex in mesh.vertices.iter() {
            vertices.push(*vertex);
        }
        for index in mesh.indices.iter() {
            indices.push(*index);
        }

        vertices.write_buffer(&render_device, &render_queue);
        indices.write_buffer(&render_device, &render_queue);

        info!("Prepare buffers...");

        let buffers = ChunkBuffers { vertices, indices };

        commands.entity(entity).insert(buffers);
    }
}

fn prepare_pipeline(
    mut commands: Commands,
    loader: Option<Res<ChunkShaderLoader>>,
    pipeline: Option<Res<ChunksPipeline>>,
    assets: Res<AssetServer>,
) {
    if pipeline.is_some() { return; }
    let Some(loader) = loader else { return };

    info!("Main chunk shader loaded");
    let shader = assets.load(loader.0);
    commands.insert_resource(ChunksPipeline { shader });
    commands.init_resource::<SpecializedRenderPipelines<ChunksPipeline>>()
}

fn queue_chunks(
    pipeline_cache: Res<PipelineCache>,
    pipeline: Option<Res<ChunksPipeline>>,
    mut opaque_render_phases: ResMut<ViewBinnedRenderPhases<Opaque3d>>,
    opaque_draw_functions: Res<DrawFunctions<Opaque3d>>,
    render_pipelines: Option<ResMut<SpecializedRenderPipelines<ChunksPipeline>>>,
    views: Query<(&ExtractedView, &RenderVisibleEntities, &Msaa)>,
    mut next_tick: Local<Tick>,
) {
    let Some(pipeline) = pipeline else { return };
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

            info!("Test!");

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
#[require(Transform, VisibilityClass)]
#[component(on_add = view::add_visibility_class::<ChunkMesh>)]
pub struct ChunkMesh {
    vertices: Vec<u32>,
    indices: Vec<u32>,
}

impl ChunkMesh {
    pub fn new(vertices: Vec<u32>, indices: Vec<u32>) -> Self {
        Self { vertices, indices }
    }
}

#[derive(Component)]
pub struct ChunkBuffers {
    vertices: RawBufferVec<u32>,
    indices: RawBufferVec<u32>
}

// Main chunks shader path
#[derive(Clone, Resource, ExtractResource)]
pub struct ChunkShaderLoader(pub &'static str);

#[derive(Resource)]
struct ChunksPipeline {
    shader: Handle<Shader>
}

#[derive(Default)]
pub struct RenderingPlugin;
impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<ChunkMesh>::default())
            .add_plugins(ExtractResourcePlugin::<ChunkShaderLoader>::default());

        app.get_sub_app_mut(RenderApp)
            .unwrap()
            .add_render_command::<Opaque3d, DrawChunkCommands>()
            .add_systems(
                Render,
                (
                    prepare_pipeline,
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
            layout: vec![],
            push_constant_ranges: vec![],
            vertex: VertexState {
                shader: self.shader.clone(),
                shader_defs: vec![],
                entry_point: "vertex".into(),
                buffers: vec![VertexBufferLayout {
                    array_stride: size_of::<u32>() as u64,
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
                depth_write_enabled: false,
                depth_compare: CompareFunction::Always,
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
        info!("Rendering...");

        let Some(buffers) = mesh else { 
            info!("Skip...");
            return RenderCommandResult::Skip
        };

        let buffers = buffers.into_inner();

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
type DrawChunkCommands = (SetItemPipeline, DrawChunk);
