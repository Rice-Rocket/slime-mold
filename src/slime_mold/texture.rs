use bevy::{prelude::*, window::PrimaryWindow, render::{render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages}, extract_resource::ExtractResource}};

use super::TEXTURE_SIZE;

pub fn setup_texture(
    mut commands: Commands, 
    window_query: Query<&Window, With<PrimaryWindow>>, 
    mut images: ResMut<Assets<Image>>
) {
    let mut image = Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE.0,
            height: TEXTURE_SIZE.1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Rgba8Unorm,
    );
    image.texture_descriptor.usage =
        TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;
    let image = images.add(image);

    let window = window_query.get_single().unwrap();

    commands.spawn(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(window.width(), window.height())),
            ..default()
        },
        texture: image.clone(),
        ..default()
    });
    commands.spawn(Camera2dBundle::default());

    commands.insert_resource(SlimeMoldImage(image));
}


#[derive(Resource, Clone, Deref, ExtractResource)]
pub struct SlimeMoldImage(pub Handle<Image>);
