use std::collections::HashMap;

use bevy::prelude::*;
use bevy_egui::egui;

use crate::terrain::endless_terrain::EndlessTerrainState;
use crate::terrain_painter::{BrushConfig, SculptMap, SyncGridRequest, TerrainTool, UndoEntry, UndoStack};

const TOOL_NAMES: &[(TerrainTool, &str)] = &[
    (TerrainTool::Raise, "Raise"),
    (TerrainTool::Lower, "Lower"),
    (TerrainTool::Flatten, "Flatten"),
    (TerrainTool::Smooth, "Smooth"),
];

pub fn tab_contents(world: &mut World, ui: &mut egui::Ui) {
    let mut brush_config = world.resource_mut::<BrushConfig>();

    ui.add_space(8.0);
    ui.heading("Terrain Painter");
    ui.separator();
    ui.horizontal(|ui| {
        ui.label("Tool:");
        let current_label = TOOL_NAMES
            .iter()
            .find(|(t, _)| *t == brush_config.tool)
            .map(|(_, n)| *n)
            .unwrap_or("Raise");
        egui::ComboBox::from_id_salt("painter_tool")
            .selected_text(current_label)
            .show_ui(ui, |ui| {
                for &(tool, label) in TOOL_NAMES {
                    ui.selectable_value(&mut brush_config.tool, tool, label);
                }
            });
    });

    ui.add(
        egui::Slider::new(&mut brush_config.radius, 0.5..=50.0)
            .text("Radius")
            .fixed_decimals(1),
    );
    ui.add(
        egui::Slider::new(&mut brush_config.strength, 0.1..=20.0)
            .text("Strength")
            .fixed_decimals(1),
    );

    if brush_config.tool == TerrainTool::Flatten {
        ui.separator();
        ui.horizontal(|ui| {
            if brush_config.flatten_sampling {
                ui.label("Click terrain to sample target height");
            } else {
                if ui.button("Sample Target Height").clicked() {
                    brush_config.flatten_sampling = true;
                }
            }
        });
        if let Some(ref mut target) = brush_config.flatten_target {
            ui.add(egui::Slider::new(target, -100.0..=200.0).text("Target"));
        }
    }

    ui.separator();
    let sculpt_map = world.resource::<SculptMap>();
    ui.label(format!("Sculpted chunks: {}", sculpt_map.chunks.len()));

    ui.separator();
    let (can_undo, can_redo) = {
        let stack = world.resource::<UndoStack>();
        (stack.can_undo(), stack.can_redo())
    };
    ui.horizontal(|ui| {
        if ui.add_enabled(can_undo, egui::Button::new("Undo")).clicked() {
            let entry = { world.resource_mut::<UndoStack>().undo_entries.pop() };
            if let Some(entry) = entry {
                let coords: Vec<IVec2> = entry.old_chunks.keys().copied().collect();
                let redo_chunks = {
                    let sculpt_map = world.resource::<SculptMap>();
                    coords.iter().map(|&c| (c, sculpt_map.chunks.get(&c).cloned())).collect::<HashMap<_, _>>()
                };
                world.resource_mut::<UndoStack>().redo_entries.push(UndoEntry {
                    old_chunks: redo_chunks,
                });
                {
                    let mut sculpt_map = world.resource_mut::<SculptMap>();
                    for (coord, old_data) in entry.old_chunks {
                        match old_data {
                            Some(data) => {
                                sculpt_map.chunks.insert(coord, data);
                            }
                            None => {
                                sculpt_map.chunks.remove(&coord);
                            }
                        }
                    }
                }
                {
                    let mut endless_state = world.resource_mut::<EndlessTerrainState>();
                    for coord in coords {
                        endless_state.mark_chunk_dirty(coord);
                    }
                }
                world.resource_mut::<SyncGridRequest>().0 = true;
            }
        }
        if ui.add_enabled(can_redo, egui::Button::new("Redo")).clicked() {
            let entry = { world.resource_mut::<UndoStack>().redo_entries.pop() };
            if let Some(entry) = entry {
                let coords: Vec<IVec2> = entry.old_chunks.keys().copied().collect();
                let undo_chunks = {
                    let sculpt_map = world.resource::<SculptMap>();
                    coords.iter().map(|&c| (c, sculpt_map.chunks.get(&c).cloned())).collect::<HashMap<_, _>>()
                };
                world.resource_mut::<UndoStack>().undo_entries.push(UndoEntry {
                    old_chunks: undo_chunks,
                });
                {
                    let mut sculpt_map = world.resource_mut::<SculptMap>();
                    for (coord, old_data) in entry.old_chunks {
                        match old_data {
                            Some(data) => {
                                sculpt_map.chunks.insert(coord, data);
                            }
                            None => {
                                sculpt_map.chunks.remove(&coord);
                            }
                        }
                    }
                }
                {
                    let mut endless_state = world.resource_mut::<EndlessTerrainState>();
                    for coord in coords {
                        endless_state.mark_chunk_dirty(coord);
                    }
                }
                world.resource_mut::<SyncGridRequest>().0 = true;
            }
        }
    });

    ui.separator();
    if ui.button("Sync Grid to Sculpt").clicked() {
        world.resource_mut::<SyncGridRequest>().0 = true;
    }
}
