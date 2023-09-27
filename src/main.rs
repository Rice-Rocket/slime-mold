mod slime_mold;
use slime_mold::*;


#[allow(unused_imports)]
use bevy::{
    prelude::*,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_asset::RenderAssets,
        render_graph::{self, RenderGraph},
        render_resource::*,
        renderer::{RenderContext, RenderDevice, RenderQueue},
        Render, RenderApp, RenderSet, Extract,
    },
    window::{WindowPlugin, PrimaryWindow},
};



fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    // uncomment for unthrottled FPS
                    // present_mode: bevy::window::PresentMode::AutoNoVsync,
                    title: String::from("Physarum (Slime Mold)"),
                    ..default()
                }),
                ..default()
            }).set(ImagePlugin::default_nearest()),
            SlimeMoldComputePlugin,
        ))
        .run();
}


