use bevy::{prelude::*, render::{extract_resource::ExtractResource, render_resource::{ShaderType, Buffer, UniformBuffer, BufferDescriptor, BufferUsages, BufferInitDescriptor}, Extract, renderer::{RenderDevice, RenderQueue}}};

use super::{NUM_AGENTS, TEXTURE_SIZE, ui::UISettings};


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
pub struct SlimeMoldAgentsBuffer {
    pub storage: Buffer,
    pub staging: Buffer,
    pub size: u64,
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



#[derive(Default, Clone, Resource, ExtractResource, Reflect, ShaderType)]
#[reflect(Resource)]
pub struct SettingsUniform {
    dim_x: i32,
    dim_y: i32,
    delta_time: f32,
    time: f32,

    pub move_speed: f32,
    pub turn_speed: f32,

    pub trail_weight: f32,
    pub decay_rate: f32,
    pub diffuse_rate: f32,

    pub sensor_angle_spacing: f32,
    pub sensor_offset_dst: f32,
    pub sensor_size: i32,

    color_a: Vec4,
    color_b: Vec4,
    
    // #[cfg(all(feature = "webgl", target_arch = "wasm32"))]
    // _padding: f32,
}

#[derive(Resource, Default)]
pub struct SettingsBuffer {
    pub buffer: UniformBuffer<SettingsUniform>,
}

pub fn extract_time(mut commands: Commands, time: Extract<Res<Time>>) {
    commands.insert_resource(time.clone());
}

pub fn extract_ui_settings(mut commands: Commands, settings: Extract<Res<UISettings>>) {
    commands.insert_resource(settings.clone());
}

pub fn prepare_settings_buffer(
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    mut settings_buffer: ResMut<SettingsBuffer>,
    settings: Res<UISettings>,
    time: Res<Time>,
) {
    let buffer = settings_buffer.buffer.get_mut();
    buffer.delta_time = time.delta_seconds();
    buffer.time = time.elapsed_seconds();
    buffer.dim_x = TEXTURE_SIZE.0 as i32;
    buffer.dim_y = TEXTURE_SIZE.1 as i32;
    buffer.move_speed = settings.move_speed;
    buffer.turn_speed = settings.turn_speed;
    buffer.trail_weight = settings.trail_weight;
    buffer.decay_rate = settings.decay_rate;
    buffer.diffuse_rate = settings.diffuse_rate;
    buffer.sensor_angle_spacing = settings.sensor_angle_spacing;
    buffer.sensor_offset_dst = settings.sensor_offset_dst;
    buffer.sensor_size = settings.sensor_size;
    buffer.color_a = Vec4::new(settings.color_a[0], settings.color_a[1], settings.color_a[2], 1.0);
    buffer.color_b = Vec4::new(settings.color_b[0], settings.color_b[1], settings.color_b[2], 1.0);

    settings_buffer.buffer.write_buffer(&device, &queue);
}