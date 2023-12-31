use bevy::{prelude::*, render::{extract_resource::ExtractResourcePlugin, RenderApp, Render, render_graph::RenderGraph, RenderSet}};

use self::{texture::{SlimeMoldImage, setup_texture}, buffers::{SettingsBuffer, extract_time, prepare_settings_buffer, SlimeMoldAgentsBuffer, extract_ui_settings}, compute::{queue_bind_group, SlimeMoldNode, SlimeMoldPipeline}, ui::UISettings};

pub mod compute;
pub mod texture;
pub mod buffers;
pub mod ui;


pub const TEXTURE_SIZE: (u32, u32) = (2560, 1440);
pub const NUM_AGENTS: u32 = 1_000_000;
pub const TEX_WORKGROUP_SIZE: u32 = 8;
pub const AGENTS_WORKGROUP_SIZE: u32 = 16;
pub const INITIAL_STATE: &str = "initAgentsInwardRing";


#[derive(States, Default, Debug, Hash, Eq, PartialEq, Clone)]
pub enum SimulationState {
    #[default]
    Uninitialized,
    Started,
}


pub struct SlimeMoldComputePlugin;

impl Plugin for SlimeMoldComputePlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<SimulationState>();
        app.add_systems(Startup, setup_texture);
        app.add_plugins(ExtractResourcePlugin::<SlimeMoldImage>::default());

        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<SettingsBuffer>()
            .init_resource::<Time>()
            .init_resource::<UISettings>()
            .add_state::<SimulationState>()
            .add_systems(ExtractSchedule, (extract_time, extract_ui_settings))
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