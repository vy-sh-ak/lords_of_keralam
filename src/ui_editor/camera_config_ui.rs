use bevy::prelude::*;
use bevy_egui::egui;
use bevy_persistent::Persistent;

use crate::camera_config::CameraSettings;
use crate::editor_config::EditorState;

pub fn render_camera_config_editor(ui: &mut egui::Ui, world: &mut World) {
    let changed = {
        let mut editor = world.resource_mut::<EditorState<CameraSettings>>();
        let mut changed = false;

        ui.add_space(8.0);

        egui::CollapsingHeader::new("RTS Mode")
            .default_open(true)
            .show(ui, |ui| {
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    changed |= ui
                        .checkbox(&mut editor.edited.rts_enabled, "Enable RTS Camera")
                        .changed();
                });
            });

        ui.add_space(8.0);

        egui::CollapsingHeader::new("Initial Camera Position")
            .default_open(true)
            .show(ui, |ui| {
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    ui.label("Start Position");
                    ui.horizontal(|ui| {
                        ui.label("X:");
                        changed |= ui
                            .add(egui::DragValue::new(&mut editor.edited.start_position.x).speed(0.1))
                            .changed();
                        ui.label("Y:");
                        changed |= ui
                            .add(egui::DragValue::new(&mut editor.edited.start_position.y).speed(0.1))
                            .changed();
                        ui.label("Z:");
                        changed |= ui
                            .add(egui::DragValue::new(&mut editor.edited.start_position.z).speed(0.1))
                            .changed();
                    });

                    ui.separator();

                    ui.label("Start Look At");
                    ui.horizontal(|ui| {
                        ui.label("X:");
                        changed |= ui
                            .add(egui::DragValue::new(&mut editor.edited.start_look_at.x).speed(0.1))
                            .changed();
                        ui.label("Y:");
                        changed |= ui
                            .add(egui::DragValue::new(&mut editor.edited.start_look_at.y).speed(0.1))
                            .changed();
                        ui.label("Z:");
                        changed |= ui
                            .add(egui::DragValue::new(&mut editor.edited.start_look_at.z).speed(0.1))
                            .changed();
                    });
                });
            });

        ui.add_space(8.0);

        egui::CollapsingHeader::new("Zoom Settings")
            .default_open(true)
            .show(ui, |ui| {
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    changed |= ui
                        .add(
                            egui::Slider::new(&mut editor.edited.zoom_speed, 0.01..=0.2)
                                .text("Zoom Speed"),
                        )
                        .changed();

                    changed |= ui
                        .add(
                            egui::Slider::new(&mut editor.edited.zoom_smoothness, 1.0..=30.0)
                                .text("Smoothness"),
                        )
                        .changed();

                    changed |= ui
                        .add(
                            egui::Slider::new(&mut editor.edited.min_distance, 1.0..=50.0)
                                .text("Min Distance"),
                        )
                        .changed();

                    changed |= ui
                        .add(
                            egui::Slider::new(&mut editor.edited.max_distance, 100.0..=1000.0)
                                .text("Max Distance"),
                        )
                        .changed();
                });
            });

        ui.add_space(8.0);

        egui::CollapsingHeader::new("Movement Settings")
            .default_open(false)
            .show(ui, |ui| {
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    changed |= ui
                        .add(
                            egui::Slider::new(&mut editor.edited.move_speed_zoomed_in, 10.0..=100.0)
                                .text("Speed (Zoomed In)"),
                        )
                        .changed();

                    changed |= ui
                        .add(
                            egui::Slider::new(&mut editor.edited.move_speed_zoomed_out, 100.0..=500.0)
                                .text("Speed (Zoomed Out)"),
                        )
                        .changed();

                    changed |= ui
                        .add(
                            egui::Slider::new(&mut editor.edited.focus_height, 0.0..=20.0)
                                .text("Focus Height"),
                        )
                        .changed();
                });
            });

        ui.add_space(8.0);

        egui::CollapsingHeader::new("Orbit Settings")
            .default_open(false)
            .show(ui, |ui| {
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    changed |= ui
                        .add(
                            egui::Slider::new(&mut editor.edited.orbit_rotate_sensitivity, 0.001..=0.1)
                                .text("Rotate Sensitivity"),
                        )
                        .changed();

                    changed |= ui
                        .add(
                            egui::Slider::new(&mut editor.edited.vertical_rotate_sensitivity, 0.005..=0.1)
                                .text("Vertical Sensitivity"),
                        )
                        .changed();

                    changed |= ui
                        .add(
                            egui::Slider::new(&mut editor.edited.min_elevation, 0.0..=1.0)
                                .text("Min Elevation"),
                        )
                        .changed();

                    changed |= ui
                        .add(
                            egui::Slider::new(&mut editor.edited.max_elevation, 0.0..=1.5)
                                .text("Max Elevation"),
                        )
                        .changed();
                });
            });

        changed
    };

    if !changed {
        return;
    }

    let editor = world.resource_mut::<EditorState<CameraSettings>>();
    let edited = editor.edited.clone();
    drop(editor);

    let mut persistent = world.resource_mut::<Persistent<CameraSettings>>();
    *persistent.get_mut() = edited;
    persistent.set_changed();
}
