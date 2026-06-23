use bevy::prelude::*;
use bevy_egui::egui;
use bevy_inspector_egui::reflect_inspector;
use bevy_persistent::Persistent;

use crate::editor_config::EditorState;
use crate::terrain::{DrawMode, FallOffGenerator, MapGenerator};

use super::curve_editor::height_curve_editor;

pub fn render_map_generator_editor(ui: &mut egui::Ui, world: &mut World) {
    let type_registry = world.resource::<AppTypeRegistry>().0.clone();
    let type_registry = type_registry.read();
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.set_min_height(400.0);
        ui.set_max_height(800.0);
        let changed = {
            let mut editor = world.resource_mut::<EditorState<MapGenerator>>();
            let mut changed = false;

            ui.add_space(8.0);
            egui::Grid::new("map_gen_main")
                .num_columns(2)
                .spacing([12.0, 8.0])
                .show(ui, |ui| {
                    ui.label("Level of Detail");
                    changed |= ui
                        .add(egui::Slider::new(&mut editor.edited.level_of_detail, 0..=6))
                        .changed();
                    ui.end_row();

                    ui.label("Draw Mode");
                    egui::ComboBox::from_id_salt("draw_mode")
                        .selected_text(editor.edited.draw_mode.label())
                        .show_ui(ui, |ui| {
                            for &mode in &DrawMode::ALL {
                                changed |= ui
                                    .selectable_value(
                                        &mut editor.edited.draw_mode,
                                        mode,
                                        mode.label(),
                                    )
                                    .changed();
                            }
                        });
                    ui.end_row();
                });

            ui.add_space(8.0);

            egui::CollapsingHeader::new("Noise Data")
                .default_open(true)
                .show(ui, |ui| {
                    egui::Frame::group(ui.style()).show(ui, |ui| {
                        changed |= reflect_inspector::ui_for_value(
                            &mut editor.edited.noise_data,
                            ui,
                            &type_registry,
                        );
                    });
                });

            ui.add_space(8.0);

            egui::CollapsingHeader::new("Terrain Data")
                .default_open(false)
                .show(ui, |ui| {
                    egui::Frame::group(ui.style()).show(ui, |ui| {
                        changed |= reflect_inspector::ui_for_value(
                            &mut editor.edited.terrain_data,
                            ui,
                            &type_registry,
                        );
                    });
                });

            ui.add_space(8.0);

            egui::CollapsingHeader::new("Texture Data")
                .default_open(true)
                .show(ui, |ui| {
                    egui::Frame::group(ui.style()).show(ui, |ui| {
                        let tex = &mut editor.edited.texture_data;

                        ui.horizontal(|ui| {
                            ui.label("Min Height");
                            changed |= ui
                                .add(egui::Slider::new(&mut tex.min_height, -100.0..=100.0))
                                .changed();
                        });
                        ui.horizontal(|ui| {
                            ui.label("Max Height");
                            changed |= ui
                                .add(egui::Slider::new(&mut tex.max_height, -100.0..=100.0))
                                .changed();
                        });

                        ui.separator();
                        ui.strong("Texture Layers");
                        ui.add_space(4.0);

                        let mut remove_idx: Option<usize> = None;

                        for i in 0..tex.layers.len() {
                            let _ = egui::Frame::group(ui.style())
                                .inner_margin(egui::Margin::symmetric(8, 4))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.strong(format!("#{}", i + 1));
                                        if tex.layers.len() > 1
                                            && ui.add(egui::Button::new("✕").small()).clicked()
                                        {
                                            remove_idx = Some(i);
                                        }
                                    });

                                    let layer = &mut tex.layers[i];

                                    ui.horizontal(|ui| {
                                        ui.label("Texture");
                                        let textures = list_texture_files();
                                        if !textures.is_empty() {
                                            egui::ComboBox::from_id_salt(format!(
                                                "tex_combo_{}",
                                                i
                                            ))
                                            .selected_text(&layer.texture_path)
                                            .show_ui(
                                                ui,
                                                |ui| {
                                                    for tex in &textures {
                                                        changed |= ui
                                                            .selectable_value(
                                                                &mut layer.texture_path,
                                                                tex.clone(),
                                                                tex,
                                                            )
                                                            .changed();
                                                    }
                                                },
                                            );
                                        }
                                        changed |= ui
                                            .add(
                                                egui::TextEdit::singleline(&mut layer.texture_path)
                                                    .desired_width(140.0),
                                            )
                                            .changed();
                                    });

                                    ui.horizontal(|ui| {
                                        ui.label("Start Height");
                                        changed |= ui
                                            .add(egui::Slider::new(
                                                &mut layer.start_height,
                                                0.0..=1.0,
                                            ))
                                            .changed();
                                    });

                                    ui.horizontal(|ui| {
                                        ui.label("Blend");
                                        changed |= ui
                                            .add(egui::Slider::new(
                                                &mut layer.blend_strength,
                                                0.0..=1.0,
                                            ))
                                            .changed();
                                    });

                                    ui.horizontal(|ui| {
                                        ui.label("Tint");
                                        let mut ec = egui::Rgba::from_rgba_unmultiplied(
                                            layer.tint.red,
                                            layer.tint.green,
                                            layer.tint.blue,
                                            layer.tint.alpha,
                                        );
                                        changed |= egui::color_picker::color_edit_button_rgba(
                                            ui,
                                            &mut ec,
                                            egui::color_picker::Alpha::Opaque,
                                        )
                                        .changed();
                                        layer.tint =
                                            LinearRgba::new(ec.r(), ec.g(), ec.b(), ec.a());
                                    });

                                    ui.horizontal(|ui| {
                                        ui.label("Tint Strength");
                                        changed |= ui
                                            .add(egui::Slider::new(
                                                &mut layer.tint_strength,
                                                0.0..=1.0,
                                            ))
                                            .changed();
                                    });

                                    ui.horizontal(|ui| {
                                        ui.label("Tex Scale");
                                        changed |= ui
                                            .add(egui::Slider::new(
                                                &mut layer.texture_scale,
                                                0.1..=500.0,
                                            ))
                                            .changed();
                                    });
                                });
                        }

                        if let Some(idx) = remove_idx {
                            tex.layers.remove(idx);
                            changed = true;
                        }

                        ui.add_space(4.0);
                        if tex.layers.len() < 8 {
                            if ui.button("＋ Add Layer").clicked() {
                                let last = tex.layers.last().map(|l| l.start_height).unwrap_or(0.0);
                                let textures = list_texture_files();
                                let default_tex = textures
                                    .first()
                                    .cloned()
                                    .unwrap_or_else(|| "textures/grass.png".to_string());
                                tex.layers.push(crate::terrain::data::TextureLayerConfig {
                                    texture_path: default_tex,
                                    start_height: (last + 1.0) * 0.5,
                                    blend_strength: 0.1,
                                    tint_strength: 0.0,
                                    texture_scale: 10.0,
                                    tint: LinearRgba::new(0.5, 0.5, 0.5, 1.0),
                                });
                                changed = true;
                            }
                        }
                    });
                });

            ui.add_space(8.0);

            egui::CollapsingHeader::new("Height Curve")
                .default_open(false)
                .show(ui, |ui| {
                    egui::Frame::group(ui.style()).show(ui, |ui| {
                        changed |=
                            height_curve_editor(ui, &mut editor.edited.terrain_data.height_curve);
                    });
                });

            ui.add_space(8.0);
            egui::CollapsingHeader::new("LOD Bands")
                .default_open(false)
                .show(ui, |ui| {
                    egui::Frame::group(ui.style()).show(ui, |ui| {
                        changed |= reflect_inspector::ui_for_value(
                            &mut editor.edited.endless_lod_bands,
                            ui,
                            &type_registry,
                        );
                    });
                });

            changed
        };

        if !changed {
            return;
        }

        let editor = world.resource_mut::<EditorState<MapGenerator>>();
        let edited = editor.edited.clone();
        drop(editor);

        let mut persistent = world.resource_mut::<Persistent<MapGenerator>>();
        *persistent.get_mut() = edited;
        if persistent.terrain_data.use_falloff_map && persistent.falloff_map.is_empty() {
            let map_size = persistent.map_chunk_size as usize + 2;
            persistent.falloff_map = FallOffGenerator::generate_fall_off_map(map_size);
        }
        persistent.set_changed();
    });
}

fn list_texture_files() -> Vec<String> {
    let assets_dir = std::path::Path::new("assets/textures");
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(assets_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "png") {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    files.push(format!("textures/{}", name));
                }
            }
        }
    }
    files.sort();
    files
}
