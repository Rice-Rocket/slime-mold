//! A compute shader that simulates Conway's Game of Life.
//!
//! Compute shaders use the GPU for computing arbitrary information, that may be independent of what
//! is rendered to the screen.

use bevy::{
    prelude::*,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_asset::RenderAssets,
        render_graph::{self, RenderGraph},
        render_resource::*,
        renderer::{RenderContext, RenderDevice, RenderQueue},
        Render, RenderApp, RenderSet,
    },
    window::WindowPlugin,
};
use std::borrow::Cow;

const SIZE: (u32, u32) = (640, 360);
const NUM_AGENTS: u32 = 1000;
const WORKGROUP_SIZE: u32 = 8;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    // uncomment for unthrottled FPS
                    // present_mode: bevy::window::PresentMode::AutoNoVsync,
                    ..default()
                }),
                ..default()
            }),
            SlimeMoldComputePlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let mut image = Image::new_fill(
        Extent3d {
            width: SIZE.0,
            height: SIZE.1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Rgba8Unorm,
    );
    image.texture_descriptor.usage =
        TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;
    let image = images.add(image);

    // let buf = StorageBuffer::from(AgentsArray { arr: [[0.0; 3]; NUM_AGENTS as usize] });
    // let buf: BufferVec<[f32; 3]> = BufferVec::new(BufferUsages::STORAGE);

    commands.spawn(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(SIZE.0 as f32, SIZE.1 as f32)),
            ..default()
        },
        texture: image.clone(),
        ..default()
    });
    commands.spawn(Camera2dBundle::default());

    commands.insert_resource(SlimeMoldImage(image));
    // commands.insert_resource(SlimeMoldAgentsBuffer(buf));
}

pub struct SlimeMoldComputePlugin;

impl Plugin for SlimeMoldComputePlugin {
    fn build(&self, app: &mut App) {
        // Extract the slime mold image resource from the main world into the render world
        // for operation on by the compute shader and display on the sprite.
        app.add_plugins(ExtractResourcePlugin::<SlimeMoldImage>::default());
        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_systems(Render, queue_bind_group.in_set(RenderSet::Queue));
        
        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        render_graph.add_node("slime_mold", SlimeMoldNode::default());
        render_graph.add_node_edge(
            "slime_mold",
            bevy::render::main_graph::node::CAMERA_DRIVER,
        );
    }
    
    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<SlimeMoldAgentsBuffer>();
        render_app.init_resource::<SlimeMoldPipeline>();
    }
}

#[derive(Resource, Clone, Deref, ExtractResource)]
struct SlimeMoldImage(Handle<Image>);

#[derive(Clone, Resource, ExtractResource, Reflect, ShaderType)]
#[reflect(Resource)]
struct AgentsArray {
    arr: [[f32; 3]; NUM_AGENTS as usize]
}

impl Default for AgentsArray {
    fn default() -> Self {
        Self { arr: [[0.0; 3]; NUM_AGENTS as usize] }
    }
}

#[derive(Resource)]
struct SlimeMoldAgentsBuffer {
    storage: Buffer,
    staging: Buffer,
    size: u64,
}

impl FromWorld for SlimeMoldAgentsBuffer {
    fn from_world(world: &mut World) -> Self {
        let device = world.resource::<RenderDevice>();
        let size = (NUM_AGENTS * 3 * std::mem::size_of::<f32>() as u32) as u64;
        
        let staging = device.create_buffer(&BufferDescriptor {
            label: None,
            size,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let storage = device.create_buffer_with_data(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[[0.0; 3]; NUM_AGENTS as usize]),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
        });

        Self {
            storage,
            staging,
            size,
        }
    }
}

#[derive(Resource)]
struct SlimeMoldBindGroups(BindGroup, BindGroup);

fn queue_bind_group(
    mut commands: Commands,
    pipeline: Res<SlimeMoldPipeline>,
    gpu_images: Res<RenderAssets<Image>>,
    slime_mold_image: Res<SlimeMoldImage>,
    slime_mold_agents_buf: Res<SlimeMoldAgentsBuffer>,
    render_device: Res<RenderDevice>,
) {
    let view = &gpu_images[&slime_mold_image.0];
    let bind_group_tex = render_device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &pipeline.texture_bind_group_layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: BindingResource::TextureView(&view.texture_view),
        }],
    });
    let bind_group_buf = render_device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &pipeline.agent_buf_bind_group_layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: slime_mold_agents_buf.storage.as_entire_binding(),
            // resource: slime_mold_agents_buf.0.binding().unwrap(),
        }],
    });
    commands.insert_resource(SlimeMoldBindGroups(bind_group_tex, bind_group_buf));
}

#[derive(Resource)]
pub struct SlimeMoldPipeline {
    texture_bind_group_layout: BindGroupLayout,
    agent_buf_bind_group_layout: BindGroupLayout,
    init_pipeline: CachedComputePipelineId,
    update_agents_pipeline: CachedComputePipelineId,
    update_trailmap_pipeline: CachedComputePipelineId,
}

impl FromWorld for SlimeMoldPipeline {
    fn from_world(world: &mut World) -> Self {
        let texture_bind_group_layout =
            world
                .resource::<RenderDevice>()
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::ReadWrite,
                            format: TextureFormat::Rgba8Unorm,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    }],
                });
        let agent_buf_bind_group_layout = 
            world
                .resource::<RenderDevice>()
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage {
                                read_only: false,
                            },
                            has_dynamic_offset: false,
                            // min_binding_size: None,
                            min_binding_size: BufferSize::new((NUM_AGENTS * 3 * std::mem::size_of::<f32>() as u32) as u64),
                            // min_binding_size: std::num::NonZeroU64::new(16u64),
                        },
                        count: None,
                    }]
                });
        let shader = world
            .resource::<AssetServer>()
            .load("shaders/slime_mold.wgsl");
        let pipeline_cache = world.resource::<PipelineCache>();
        let init_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![texture_bind_group_layout.clone(), agent_buf_bind_group_layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: Cow::from("initAgents"),
        });
        let update_agents_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![texture_bind_group_layout.clone(), agent_buf_bind_group_layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: Cow::from("updateAgents"),
        });
        let update_trailmap_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![texture_bind_group_layout.clone(), agent_buf_bind_group_layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: Cow::from("updateTrailmap"),
        });

        SlimeMoldPipeline {
            texture_bind_group_layout,
            agent_buf_bind_group_layout,
            init_pipeline,
            update_agents_pipeline,
            update_trailmap_pipeline,
        }
    }
}

enum SlimeMoldState {
    Loading,
    Init,
    Update,
}

struct SlimeMoldNode {
    state: SlimeMoldState,
}

impl Default for SlimeMoldNode {
    fn default() -> Self {
        Self {
            state: SlimeMoldState::Loading,
        }
    }
}

impl render_graph::Node for SlimeMoldNode {
    fn update(&mut self, world: &mut World) {
        let pipeline = world.resource::<SlimeMoldPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        // if the corresponding pipeline has loaded, transition to the next stage
        match self.state {
            SlimeMoldState::Loading => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.init_pipeline)
                {
                    self.state = SlimeMoldState::Init;
                }
            }
            SlimeMoldState::Init => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.update_agents_pipeline)
                {
                    self.state = SlimeMoldState::Update;
                }
            }
            SlimeMoldState::Update => {}
        }
    }

    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let texture_bind_group = &world.resource::<SlimeMoldBindGroups>().0;
        let agents_buf_bind_group = &world.resource::<SlimeMoldBindGroups>().1;
        let agents_buf = &world.resource::<SlimeMoldAgentsBuffer>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<SlimeMoldPipeline>();

        let encoder = render_context.command_encoder();
        {
            let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());

            pass.set_bind_group(0, texture_bind_group, &[]);
            pass.set_bind_group(1, agents_buf_bind_group, &[]);

            // select the pipeline based on the current state
            match self.state {
                SlimeMoldState::Loading => {}
                SlimeMoldState::Init => {
                    let init_pipeline = pipeline_cache
                        .get_compute_pipeline(pipeline.init_pipeline)
                        .unwrap();
                    pass.set_pipeline(init_pipeline);
                    pass.dispatch_workgroups(NUM_AGENTS / (WORKGROUP_SIZE * 2), 1, 1);
                }
                SlimeMoldState::Update => {
                    let update_agents_pipeline = pipeline_cache
                        .get_compute_pipeline(pipeline.update_agents_pipeline)
                        .unwrap();
                    pass.set_pipeline(update_agents_pipeline);
                    pass.dispatch_workgroups(NUM_AGENTS / (WORKGROUP_SIZE * 2), 1, 1);

                    let update_trailmap_pipeline = pipeline_cache
                        .get_compute_pipeline(pipeline.update_trailmap_pipeline)
                        .unwrap();
                    pass.set_pipeline(update_trailmap_pipeline);
                    pass.dispatch_workgroups(SIZE.0 / WORKGROUP_SIZE, SIZE.1 / WORKGROUP_SIZE, 1);
                }
            }
        }

        encoder.copy_buffer_to_buffer(&agents_buf.storage, 0, &agents_buf.staging, 0, agents_buf.size);

        Ok(())
    }
}