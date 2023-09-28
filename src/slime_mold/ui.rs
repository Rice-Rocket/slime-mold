use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};


#[derive(Resource, Default, PartialEq, Clone)]
pub enum UIVisibility {
    #[default]
    Visible, 
    Hidden,
}

#[derive(Resource, Clone)]
pub struct UISettings {
    pub move_speed: f32,
    pub turn_speed: f32,

    pub trail_weight: f32,
    pub decay_rate: f32,
    pub diffuse_rate: f32,

    pub sensor_angle_spacing: f32,
    pub sensor_offset_dst: f32,
    pub sensor_size: i32,

    pub color_a: [f32; 3],
    pub color_b: [f32; 3],

    pub running: bool,
}

impl Default for UISettings {
    fn default() -> Self {
        Self {
            move_speed: 100.0,
            turn_speed: 10.0,

            trail_weight: 50.0,
            decay_rate: 0.25,
            diffuse_rate: 5.0,

            sensor_angle_spacing: 15.0,
            sensor_offset_dst: 15.0,
            sensor_size: 3,

            color_a: [1.0, 1.0, 1.0],
            color_b: [0.0, 0.0, 0.0],

            running: false,
        }
    }
}


pub fn ui_update(
    mut contexts: EguiContexts,
    mut ui_visibility: ResMut<UIVisibility>,
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut settings: ResMut<UISettings>,
) {
    if keyboard.just_pressed(KeyCode::Tab) {
        *ui_visibility = match ui_visibility.clone() {
            UIVisibility::Visible => UIVisibility::Hidden,
            UIVisibility::Hidden => UIVisibility::Visible,
        }
    }
    if ui_visibility.clone() == UIVisibility::Hidden { return; }

    egui::Window::new("Settings").show(contexts.ctx_mut(), |ui| {
        ui.label(format!("FPS: {:.1}", 1.0 / time.delta_seconds()));
        ui.label("Press [TAB] to Toggle UI");

        ui.separator();

        ui.add(egui::widgets::DragValue::new(&mut settings.move_speed).prefix("Move Speed: ").speed(0.1));
        ui.add(egui::widgets::DragValue::new(&mut settings.turn_speed).prefix("Turn Speed: ").speed(0.02));

        ui.separator();

        ui.add(egui::widgets::DragValue::new(&mut settings.trail_weight).prefix("Trail Weight: ").speed(0.1));
        ui.add(egui::widgets::DragValue::new(&mut settings.decay_rate).prefix("Decay Rate: ").speed(0.01).clamp_range(0..=1));
        ui.add(egui::widgets::DragValue::new(&mut settings.diffuse_rate).prefix("Diffuse Rate: ").speed(0.02));

        ui.separator();

        ui.add(egui::widgets::DragValue::new(&mut settings.sensor_angle_spacing).prefix("Sensor Angle Spacing: ").suffix("°").speed(0.1));
        ui.add(egui::widgets::DragValue::new(&mut settings.sensor_offset_dst).prefix("Sensor Offset: ").speed(0.05));
        ui.add(egui::widgets::DragValue::new(&mut settings.sensor_size).prefix("Sensor Size: ").speed(0.05).clamp_range(3..=7));

        ui.separator();

        ui.label("Primary Color");
        egui::widgets::color_picker::color_edit_button_rgb(ui, &mut settings.color_a);
        ui.label("Secondary Color");
        egui::widgets::color_picker::color_edit_button_rgb(ui, &mut settings.color_b);

        ui.separator();

        let button_text = match settings.running {
            true => "Pause Simulation",
            false => "Run Simulation",
        };
        if ui.button(button_text).clicked() {
            settings.running = !settings.running;
        }
    });
}