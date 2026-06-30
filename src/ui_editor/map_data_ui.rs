use bevy::prelude::*;
use bevy_egui::egui;
use bevy_persistent::Persistent;

use crate::map_asset::{ActiveMap, DefaultMapConfig, MapAsset, MapAssetService};
use crate::terrain::MapGenerator;
use crate::terrain_painter::SculptMap;

#[derive(Resource, Default)]
struct MapDataUiState {
    new_map_name: String,
    selected_map: Option<String>,
}

pub fn render(ui: &mut egui::Ui, world: &mut World) {
    let maps = MapAssetService::list();

    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.set_min_height(400.0);
        ui.heading("Maps");
        ui.separator();

        let mut message: Option<(String, egui::Color32)> = None;

        // --- Create New ---
        {
            let mut st = world.get_resource_or_insert_with::<MapDataUiState>(Default::default);
            ui.label("Map Name");
            ui.add(
                egui::TextEdit::singleline(&mut st.new_map_name)
                    .hint_text("Enter map name")
                    .desired_width(f32::INFINITY),
            );
            if ui
                .add_enabled(!st.new_map_name.is_empty(), egui::Button::new("Create New"))
                .clicked()
            {
                let name = st.new_map_name.trim().to_string();
                st.new_map_name.clear();
                drop(st);

                let map_gen = world.resource::<Persistent<MapGenerator>>();
                let sculpt = world.resource::<SculptMap>();
                let asset = MapAsset {
                    metadata: crate::map_asset::MapMetadata { name: name.clone() },
                    map_generator: (**map_gen).clone(),
                    sculpt_map: SculptMap {
                        chunks: sculpt.chunks.clone(),
                    },
                };
                let _ = map_gen;
                let _ = sculpt;

                match MapAssetService::save(&asset) {
                    Ok(()) => {
                        world.resource_mut::<ActiveMap>().current_map = Some(name.clone());
                        message = Some((format!("Map '{name}' created"), egui::Color32::GREEN));
                    }
                    Err(e) => {
                        eprintln!("[map_data] Create New error: {e}");
                        message = Some((e, egui::Color32::RED));
                    }
                }
            }
        }

        ui.separator();

        // --- Save ---
        {
            let active_map = world.resource::<ActiveMap>();
            match &active_map.current_map {
                Some(name) => {
                    ui.label(format!("Current Map: {name}"));
                    if ui.button("Save").clicked() {
                        let name = name.clone();
                        let _ = active_map;

                        let map_gen = world.resource::<Persistent<MapGenerator>>();
                        let sculpt = world.resource::<SculptMap>();
                        let asset = MapAsset {
                            metadata: crate::map_asset::MapMetadata { name: name.clone() },
                            map_generator: (**map_gen).clone(),
                            sculpt_map: SculptMap {
                                chunks: sculpt.chunks.clone(),
                            },
                        };
                        let _ = map_gen;
                        let _ = sculpt;

                        match MapAssetService::save(&asset) {
                            Ok(()) => {
                                message =
                                    Some((format!("Map '{name}' saved"), egui::Color32::GREEN));
                            }
                            Err(e) => {
                                eprintln!("[map_data] Save error: {e}");
                                message = Some((e, egui::Color32::RED));
                            }
                        }
                    }
                }
                None => {
                    ui.label("Current Map: None");
                }
            }
        }

        ui.separator();

        // --- Available Maps ---
        ui.label("Available Maps");
        if maps.is_empty() {
            ui.label("(no maps saved)");
        } else {
            let default_map_name = {
                let config = world.resource::<Persistent<DefaultMapConfig>>();
                config.default_map.clone()
            };

            for map_name in &maps {
                let selected = {
                    let st = world.resource::<MapDataUiState>();
                    st.selected_map.as_deref() == Some(map_name.as_str())
                };
                let is_default = default_map_name.as_deref() == Some(map_name.as_str());
                let display_name = if is_default {
                    format!("{map_name} (default)")
                } else {
                    map_name.clone()
                };
                let resp = ui.selectable_label(selected, display_name);
                if resp.clicked() {
                    let mut st = world.resource_mut::<MapDataUiState>();
                    st.selected_map = Some(map_name.clone());
                }
            }

            let mut load_clicked = false;
            let mut delete_clicked = false;
            let mut set_default_clicked = false;
            {
                let st = world.resource::<MapDataUiState>();
                let has_selection = st.selected_map.is_some();
                let is_default_selected =
                    default_map_name.as_deref() == st.selected_map.as_deref();

                ui.horizontal(|ui| {
                    load_clicked = ui
                        .add_enabled(has_selection, egui::Button::new("Load"))
                        .clicked();

                    let set_default_text =
                        if is_default_selected { "Unset Default" } else { "Set Default" };
                    set_default_clicked = ui
                        .add_enabled(has_selection, egui::Button::new(set_default_text))
                        .clicked();

                    delete_clicked = ui
                        .add_enabled(
                            has_selection && !is_default_selected,
                            egui::Button::new("Delete"),
                        )
                        .clicked();
                });
            }

            if load_clicked {
                let name = {
                    let st = world.resource::<MapDataUiState>();
                    st.selected_map.clone()
                };
                if let Some(ref name) = name {
                    match MapAssetService::load(name) {
                        Ok(asset) => {
                            crate::map_asset::apply_map_asset_to_world(world, asset, name);
                            message = Some((
                                format!("Map '{name}' loaded"),
                                egui::Color32::GREEN,
                            ));
                        }
                        Err(e) => {
                            eprintln!("[map_data] Load error: {e}");
                            message = Some((e, egui::Color32::RED));
                        }
                    }
                }
            }

            if set_default_clicked {
                let name = {
                    let st = world.resource::<MapDataUiState>();
                    st.selected_map.clone()
                };
                if let Some(ref name) = name {
                    let mut config = world.resource_mut::<Persistent<DefaultMapConfig>>();
                    if config.default_map.as_deref() == Some(name.as_str()) {
                        config.default_map = None;
                        message = Some((
                            format!("Default map '{name}' unset"),
                            egui::Color32::GREEN,
                        ));
                    } else {
                        config.default_map = Some(name.clone());
                        message = Some((
                            format!("Default map set to '{name}'"),
                            egui::Color32::GREEN,
                        ));
                    }
                    config.set_changed();
                }
            }

            if delete_clicked {
                let name = {
                    let st = world.resource::<MapDataUiState>();
                    st.selected_map.clone()
                };
                if let Some(ref name) = name {
                    match MapAssetService::delete(name) {
                        Ok(()) => {
                            let mut active = world.resource_mut::<ActiveMap>();
                            if active.current_map.as_deref() == Some(name.as_str()) {
                                active.current_map = None;
                            }
                            drop(active);

                            let mut config =
                                world.resource_mut::<Persistent<DefaultMapConfig>>();
                            if config.default_map.as_deref() == Some(name.as_str()) {
                                config.default_map = None;
                                config.set_changed();
                            }
                            drop(config);

                            let mut st = world.resource_mut::<MapDataUiState>();
                            st.selected_map = None;
                            message = Some((
                                format!("Map '{name}' deleted"),
                                egui::Color32::GREEN,
                            ));
                        }
                        Err(e) => {
                            eprintln!("[map_data] Delete error: {e}");
                            message = Some((e, egui::Color32::RED));
                        }
                    }
                }
            }
        }

        if let Some((msg, color)) = &message {
            ui.separator();
            ui.colored_label(*color, msg);
        }
    });
}
