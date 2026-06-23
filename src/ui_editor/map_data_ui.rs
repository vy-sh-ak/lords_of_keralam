use bevy::prelude::*;
use bevy_egui::egui;
use bevy_persistent::Persistent;

use crate::map_asset::{ActiveMap, MapAsset, MapAssetService};
use crate::terrain::endless_terrain::EndlessTerrainState;
use crate::terrain::MapGenerator;
use crate::terrain_painter::{SculptMap, SyncGridRequest, UndoStack};

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
            for map_name in &maps {
                let selected = {
                    let st = world.resource::<MapDataUiState>();
                    st.selected_map.as_deref() == Some(map_name.as_str())
                };
                let resp = ui.selectable_label(selected, map_name);
                if resp.clicked() {
                    let mut st = world.resource_mut::<MapDataUiState>();
                    st.selected_map = Some(map_name.clone());
                }
            }

            let load_clicked;
            let delete_clicked;
            {
                let st = world.resource::<MapDataUiState>();
                let has_selection = st.selected_map.is_some();
                load_clicked = ui
                    .add_enabled(has_selection, egui::Button::new("Load"))
                    .clicked();
                delete_clicked = ui
                    .add_enabled(has_selection, egui::Button::new("Delete"))
                    .clicked();
            }

            if load_clicked {
                let name = {
                    let st = world.resource::<MapDataUiState>();
                    st.selected_map.clone()
                };
                if let Some(ref name) = name {
                    match MapAssetService::load(name) {
                        Ok(asset) => {
                            let mut persistent =
                                world.resource_mut::<Persistent<MapGenerator>>();
                            *persistent.get_mut() = asset.map_generator;
                            persistent.set_changed();

                            let mut sculpt = world.resource_mut::<SculptMap>();
                            sculpt.chunks = asset.sculpt_map.chunks;
                            let chunk_keys: Vec<_> = sculpt.chunks.keys().copied().collect();
                            drop(sculpt);

                            let mut endless = world.resource_mut::<EndlessTerrainState>();
                            for coord in chunk_keys {
                                endless.mark_chunk_dirty(coord);
                            }

                            world.resource_mut::<UndoStack>().undo_entries.clear();
                            world.resource_mut::<UndoStack>().redo_entries.clear();
                            world.resource_mut::<SyncGridRequest>().0 = true;
                            world.resource_mut::<ActiveMap>().current_map = Some(name.clone());

                            message =
                                Some((format!("Map '{name}' loaded"), egui::Color32::GREEN));
                        }
                        Err(e) => {
                            eprintln!("[map_data] Load error: {e}");
                            message = Some((e, egui::Color32::RED));
                        }
                    }
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
