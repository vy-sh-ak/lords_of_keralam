use bevy::prelude::*;
use bevy_egui::egui;

use crate::terrain_painter::{BrushConfig, SculptMap, SyncGridRequest, TerrainTool};

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
    if ui.button("Sync Grid to Sculpt").clicked() {
        world.resource_mut::<SyncGridRequest>().0 = true;
    }
}
