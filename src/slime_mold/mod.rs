use bevy::{prelude::*, render::{extract_resource::ExtractResourcePlugin, RenderApp, Render, render_graph::RenderGraph, RenderSet}};

use self::{texture::{SlimeMoldImage, setup_texture}, buffers::{SettingsBuffer, extract_time, prepare_settings_buffer, SlimeMoldAgentsBuffer}, compute::{queue_bind_group, SlimeMoldNode, SlimeMoldPipeline}};

pub mod compute;
pub mod texture;
pub mod buffers;


// const TEXTURE_SIZE: (u32, u32) = (3840, 2160);
pub const TEXTURE_SIZE: (u32, u32) = (2560, 1440);
// const TEXTURE_SIZE: (u32, u32) = (1280, 720);
pub const NUM_AGENTS: u32 = 200_000;
pub const WORKGROUP_SIZE: u32 = 8;


pub struct SlimeMoldComputePlugin;

impl Plugin for SlimeMoldComputePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_texture);
        app.add_plugins(ExtractResourcePlugin::<SlimeMoldImage>::default());
        
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<SettingsBuffer>()
            .init_resource::<Time>()
            .add_systems(ExtractSchedule, extract_time)
            .add_systems(Render, prepare_settings_buffer.in_set(RenderSet::Prepare))
            .add_systems(Render, queue_bind_group.in_set(RenderSet::Queue));
        
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