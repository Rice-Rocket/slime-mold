use std::borrow::Cow;

use bevy::{prelude::*, render::{render_resource::{BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, BindGroupLayout, CachedComputePipelineId, BindGroupLayoutDescriptor, BindGroupLayoutEntry, ShaderStages, BindingType, StorageTextureAccess, TextureFormat, TextureViewDimension, BufferBindingType, BufferSize, PipelineCache, ComputePipelineDescriptor, CachedPipelineState, ComputePassDescriptor}, render_asset::RenderAssets, renderer::{RenderDevice, RenderContext}, render_graph}};

use super::{NUM_AGENTS, WORKGROUP_SIZE, TEXTURE_SIZE, texture::SlimeMoldImage, buffers::{SlimeMoldAgentsBuffer, SettingsBuffer}};


#[derive(Resource)]
struct SlimeMoldBindGroups(BindGroup, BindGroup, BindGroup);

pub fn queue_bind_group(
    mut commands: Commands,
    pipeline: Res<SlimeMoldPipeline>,
    gpu_images: Res<RenderAssets<Image>>,
    slime_mold_image: Res<SlimeMoldImage>,
    slime_mold_agents_buf: Res<SlimeMoldAgentsBuffer>,
    slime_mold_settings: Res<SettingsBuffer>,
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
        }],
    });
    let bind_group_settings = render_device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &pipeline.settings_bind_group_layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: slime_mold_settings.buffer.binding().unwrap(),
        }],
    });
    commands.insert_resource(SlimeMoldBindGroups(bind_group_tex, bind_group_buf, bind_group_settings));
}

#[derive(Resource)]
pub struct SlimeMoldPipeline {
    texture_bind_group_layout: BindGroupLayout,
    agent_buf_bind_group_layout: BindGroupLayout,
    settings_bind_group_layout: BindGroupLayout,
    init_pipeline: CachedComputePipelineId,
    update_agents_pipeline: CachedComputePipelineId,
    update_trailmap_pipeline: CachedComputePipelineId,
}

impl FromWorld for SlimeMoldPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let texture_bind_group_layout =
            render_device
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
            render_device
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
                            min_binding_size: BufferSize::new((NUM_AGENTS * 3 * std::mem::size_of::<f32>() as u32) as u64),
                        },
                        count: None,
                    }]
                });
        let settings_bind_group_layout = 
            render_device
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
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
            layout: vec![texture_bind_group_layout.clone(), agent_buf_bind_group_layout.clone(), settings_bind_group_layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: Cow::from("initAgents"),
        });
        let update_agents_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![texture_bind_group_layout.clone(), agent_buf_bind_group_layout.clone(), settings_bind_group_layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: Cow::from("updateAgents"),
        });
        let update_trailmap_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![texture_bind_group_layout.clone(), agent_buf_bind_group_layout.clone(), settings_bind_group_layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: Cow::from("updateTrailmap"),
        });

        SlimeMoldPipeline {
            texture_bind_group_layout,
            agent_buf_bind_group_layout,
            settings_bind_group_layout,
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

pub struct SlimeMoldNode {
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
        let settings_bind_group = &world.resource::<SlimeMoldBindGroups>().2;
        let agents_buf = &world.resource::<SlimeMoldAgentsBuffer>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<SlimeMoldPipeline>();

        let encoder = render_context.command_encoder();
        {
            let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());

            pass.set_bind_group(0, texture_bind_group, &[]);
            pass.set_bind_group(1, agents_buf_bind_group, &[]);
            pass.set_bind_group(2, settings_bind_group, &[]);

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
                    pass.dispatch_workgroups(TEXTURE_SIZE.0 / WORKGROUP_SIZE, TEXTURE_SIZE.1 / WORKGROUP_SIZE, 1);
                }
            }
        }

        encoder.copy_buffer_to_buffer(&agents_buf.storage, 0, &agents_buf.staging, 0, agents_buf.size);

        Ok(())
    }
}