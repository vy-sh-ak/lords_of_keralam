use std::collections::HashMap;

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use bevy_persistent::Persistent;

use crate::terrain::{DrawMode, MapConfigAutosave, MapConfigs};

pub mod widgets;

use self::widgets::{property_grid, stepper_input_row_enabled};

const GRID_ID: &str = "ui_editor_map_config_grid";
const HEIGHT_CURVE_GRID_ID: &str = "ui_editor_height_curve_grid";
const REGION_GRID_ID_PREFIX: &str = "ui_editor_region_grid";

#[derive(Resource, Clone)]
pub struct UIEditor {
    panel_id: String,
    panel_title: String,
    toggle_key: KeyCode,
    starts_open: bool,
}

impl Default for UIEditor {
    fn default() -> Self {
        Self {
            panel_id: "ui_editor_panel".to_string(),
            panel_title: "UI Editor".to_string(),
            toggle_key: KeyCode::F1,
            starts_open: true,
        }
    }
}

impl UIEditor {
    pub fn plugin(self) -> UIEditorPlugin {
        UIEditorPlugin { editor: self }
    }

    pub fn with_toggle_key(mut self, toggle_key: KeyCode) -> Self {
        self.toggle_key = toggle_key;
        self
    }

    pub fn starts_open(mut self, starts_open: bool) -> Self {
        self.starts_open = starts_open;
        self
    }

    pub fn with_panel_title(mut self, panel_title: impl Into<String>) -> Self {
        self.panel_title = panel_title.into();
        self
    }
}

pub struct UIEditorPlugin {
    editor: UIEditor,
}

#[derive(Resource, Default)]
pub struct UIKeyboardCapture {
    pub is_typing: bool,
    pub wants_pointer_input: bool,
}

#[derive(Default)]
struct FieldBuffer {
    input: String,
    has_focus: bool,
}

#[derive(Resource)]
struct UIEditorState {
    is_open: bool,
    field_buffers: HashMap<EditorField, FieldBuffer>,
}

impl UIEditorState {
    fn from_editor(editor: &UIEditor) -> Self {
        let mut field_buffers = HashMap::new();
        for field in ScalarEditorField::ALL {
            field_buffers.insert(EditorField::Scalar(field), FieldBuffer::default());
        }

        Self {
            is_open: editor.starts_open,
            field_buffers,
        }
    }

    fn sync_from_map_configs(
        &mut self,
        map_configs: &Persistent<MapConfigs>,
        has_external_change: bool,
    ) {
        for field in ScalarEditorField::ALL {
            self.sync_buffer(
                EditorField::Scalar(field),
                field.display_value(map_configs),
                has_external_change,
            );
        }

        self.field_buffers
            .retain(|field, _| {
                field.should_keep(
                    map_configs.regions.len(),
                    map_configs.height_curve.points.len(),
                    map_configs.endless_lod_bands.len(),
                )
            });

        for index in 0..map_configs.height_curve.points.len() {
            self.sync_buffer(
                EditorField::CurvePointInput(index),
                format_decimal(map_configs.height_curve.points[index].input as f64),
                has_external_change,
            );
            self.sync_buffer(
                EditorField::CurvePointOutput(index),
                format_decimal(map_configs.height_curve.points[index].output as f64),
                has_external_change,
            );
        }

        for index in 0..map_configs.endless_lod_bands.len() {
            self.sync_buffer(
                EditorField::LodBandDistance(index),
                format_decimal(map_configs.endless_lod_bands[index].visible_distance as f64),
                has_external_change,
            );
            self.sync_buffer(
                EditorField::LodBandLevel(index),
                map_configs.endless_lod_bands[index].level_of_detail.to_string(),
                has_external_change,
            );
        }

        for index in 0..map_configs.regions.len() {
            self.sync_buffer(
                EditorField::RegionName(index),
                map_configs.regions[index].name.clone(),
                has_external_change,
            );
            self.sync_buffer(
                EditorField::RegionHeight(index),
                format_decimal(map_configs.regions[index].height),
                has_external_change,
            );
        }
    }

    fn sync_field_from_map_configs(
        &mut self,
        field: EditorField,
        map_configs: &Persistent<MapConfigs>,
    ) {
        self.field_mut(field).input = field.display_value(map_configs);
    }

    fn set_focus(&mut self, field: EditorField, has_focus: bool) {
        self.field_mut(field).has_focus = has_focus;
    }

    fn input_mut(&mut self, field: EditorField) -> &mut String {
        &mut self.field_mut(field).input
    }

    fn field_mut(&mut self, field: EditorField) -> &mut FieldBuffer {
        self.field_buffers.entry(field).or_default()
    }

    fn sync_buffer(&mut self, field: EditorField, value: String, has_external_change: bool) {
        let buffer = self.field_mut(field);
        if buffer.input.is_empty() || (has_external_change && !buffer.has_focus) {
            buffer.input = value;
        }
    }

    fn is_any_field_focused(&self) -> bool {
        self.field_buffers.values().any(|buffer| buffer.has_focus)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum ScalarEditorField {
    MapChunkSize,
    LevelOfDetail,
    Scale,
    Seed,
    OffsetX,
    OffsetY,
    Persistence,
    Lacunarity,
    Frequency,
    HeightMultiplier,
    RegionsCount,
}

impl ScalarEditorField {
    const ALL: [Self; 11] = [
        Self::MapChunkSize,
        Self::LevelOfDetail,
        Self::Scale,
        Self::Seed,
        Self::OffsetX,
        Self::OffsetY,
        Self::Persistence,
        Self::Lacunarity,
        Self::Frequency,
        Self::HeightMultiplier,
        Self::RegionsCount,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::MapChunkSize => "Map Chunk Size",
            Self::LevelOfDetail => "Level Of Detail (0-6)",
            Self::Scale => "Scale",
            Self::Seed => "Seed",
            Self::OffsetX => "Offset X",
            Self::OffsetY => "Offset Y",
            Self::Persistence => "Persistence",
            Self::Lacunarity => "Lacunarity",
            Self::Frequency => "Frequency",
            Self::HeightMultiplier => "Height Multiplier",
            Self::RegionsCount => "Region Count",
        }
    }

    fn is_read_only(self) -> bool {
        matches!(self, Self::MapChunkSize)
    }

    fn display_value(self, map_configs: &Persistent<MapConfigs>) -> String {
        match self {
            Self::MapChunkSize => map_configs.map_chunk_size.to_string(),
            Self::LevelOfDetail => map_configs.level_of_detail.to_string(),
            Self::Scale => format_decimal(map_configs.scale),
            Self::Seed => map_configs.seed.to_string(),
            Self::OffsetX => format_decimal(map_configs.offset_x),
            Self::OffsetY => format_decimal(map_configs.offset_y),
            Self::Persistence => format_decimal(map_configs.persistence),
            Self::Lacunarity => format_decimal(map_configs.lacunarity),
            Self::Frequency => format_decimal(map_configs.frequency),
            Self::HeightMultiplier => format_decimal(map_configs.height_multiplier as f64),
            Self::RegionsCount => map_configs.regions.len().to_string(),
        }
    }

    fn decrement(self, map_configs: &mut MapConfigs) -> bool {
        match self {
            Self::MapChunkSize => map_configs.decrement_map_chunk_size(),
            Self::LevelOfDetail => map_configs.decrement_level_of_detail(),
            Self::Scale => map_configs.decrement_scale(),
            Self::Seed => map_configs.decrement_seed(),
            Self::OffsetX => map_configs.decrement_offset_x(),
            Self::OffsetY => map_configs.decrement_offset_y(),
            Self::Persistence => map_configs.decrement_persistence(),
            Self::Lacunarity => map_configs.decrement_lacunarity(),
            Self::Frequency => map_configs.decrement_frequency(),
            Self::HeightMultiplier => map_configs.decrement_height_multiplier(),
            Self::RegionsCount => map_configs.decrement_region_count(),
        }
    }

    fn increment(self, map_configs: &mut MapConfigs) -> bool {
        match self {
            Self::MapChunkSize => map_configs.increment_map_chunk_size(),
            Self::LevelOfDetail => map_configs.increment_level_of_detail(),
            Self::Scale => map_configs.increment_scale(),
            Self::Seed => map_configs.increment_seed(),
            Self::OffsetX => map_configs.increment_offset_x(),
            Self::OffsetY => map_configs.increment_offset_y(),
            Self::Persistence => map_configs.increment_persistence(),
            Self::Lacunarity => map_configs.increment_lacunarity(),
            Self::Frequency => map_configs.increment_frequency(),
            Self::HeightMultiplier => map_configs.increment_height_multiplier(),
            Self::RegionsCount => map_configs.increment_region_count(),
        }
    }

    fn apply_input(self, input: &str, map_configs: &mut MapConfigs) -> bool {
        match self {
            Self::MapChunkSize => input
                .trim()
                .parse::<u32>()
                .ok()
                .is_some_and(|value| map_configs.set_map_chunk_size(value)),
            Self::LevelOfDetail => input
                .trim()
                .parse::<u32>()
                .ok()
                .is_some_and(|value| map_configs.set_level_of_detail(value)),
            Self::Scale => input
                .trim()
                .parse::<f64>()
                .ok()
                .is_some_and(|value| map_configs.set_scale(value)),
            Self::Seed => input
                .trim()
                .parse::<u32>()
                .ok()
                .is_some_and(|value| map_configs.set_seed(value)),
            Self::OffsetX => input
                .trim()
                .parse::<f64>()
                .ok()
                .is_some_and(|value| map_configs.set_offset_x(value)),
            Self::OffsetY => input
                .trim()
                .parse::<f64>()
                .ok()
                .is_some_and(|value| map_configs.set_offset_y(value)),
            Self::Persistence => input
                .trim()
                .parse::<f64>()
                .ok()
                .is_some_and(|value| map_configs.set_persistence(value)),
            Self::Lacunarity => input
                .trim()
                .parse::<f64>()
                .ok()
                .is_some_and(|value| map_configs.set_lacunarity(value)),
            Self::Frequency => input
                .trim()
                .parse::<f64>()
                .ok()
                .is_some_and(|value| map_configs.set_frequency(value)),
            Self::HeightMultiplier => input
                .trim()
                .parse::<f32>()
                .ok()
                .is_some_and(|value| map_configs.set_height_multiplier(value)),
            Self::RegionsCount => input
                .trim()
                .parse::<usize>()
                .ok()
                .is_some_and(|value| map_configs.set_region_count(value)),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum EditorField {
    Scalar(ScalarEditorField),
    CurvePointInput(usize),
    CurvePointOutput(usize),
    LodBandDistance(usize),
    LodBandLevel(usize),
    RegionName(usize),
    RegionHeight(usize),
}

impl EditorField {
    fn display_value(&self, map_configs: &Persistent<MapConfigs>) -> String {
        match self {
            Self::Scalar(field) => field.display_value(map_configs),
            Self::CurvePointInput(index) => map_configs
                .height_curve
                .points
                .get(*index)
                .map(|point| format_decimal(point.input as f64))
                .unwrap_or_default(),
            Self::CurvePointOutput(index) => map_configs
                .height_curve
                .points
                .get(*index)
                .map(|point| format_decimal(point.output as f64))
                .unwrap_or_default(),
            Self::LodBandDistance(index) => map_configs
                .endless_lod_bands
                .get(*index)
                .map(|band| format_decimal(band.visible_distance as f64))
                .unwrap_or_default(),
            Self::LodBandLevel(index) => map_configs
                .endless_lod_bands
                .get(*index)
                .map(|band| band.level_of_detail.to_string())
                .unwrap_or_default(),
            Self::RegionName(index) => map_configs
                .regions
                .get(*index)
                .map(|region| region.name.clone())
                .unwrap_or_default(),
            Self::RegionHeight(index) => map_configs
                .regions
                .get(*index)
                .map(|region| format_decimal(region.height))
                .unwrap_or_default(),
        }
    }

    fn apply_input(&self, input: &str, map_configs: &mut MapConfigs) -> bool {
        match self {
            Self::Scalar(field) => field.apply_input(input, map_configs),
            Self::CurvePointInput(index) => input
                .trim()
                .parse::<f32>()
                .ok()
                .is_some_and(|value| map_configs.set_height_curve_point_input(*index, value)),
            Self::CurvePointOutput(index) => input
                .trim()
                .parse::<f32>()
                .ok()
                .is_some_and(|value| map_configs.set_height_curve_point_output(*index, value)),
            Self::LodBandDistance(index) => input
                .trim()
                .parse::<f32>()
                .ok()
                .is_some_and(|value| map_configs.set_endless_lod_band_distance(*index, value)),
            Self::LodBandLevel(index) => input
                .trim()
                .parse::<u32>()
                .ok()
                .is_some_and(|value| map_configs.set_endless_lod_band_level_of_detail(*index, value)),
            Self::RegionName(index) => map_configs.set_region_name(*index, input.to_string()),
            Self::RegionHeight(index) => input
                .trim()
                .parse::<f64>()
                .ok()
                .is_some_and(|value| map_configs.set_region_height(*index, value)),
        }
    }

    fn should_keep(&self, region_count: usize, curve_point_count: usize, lod_band_count: usize) -> bool {
        match self {
            Self::Scalar(_) => true,
            Self::CurvePointInput(index) | Self::CurvePointOutput(index) => *index < curve_point_count,
            Self::LodBandDistance(index) | Self::LodBandLevel(index) => *index < lod_band_count,
            Self::RegionName(index) | Self::RegionHeight(index) => *index < region_count,
        }
    }
}

impl Plugin for UIEditorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.editor.clone())
            .insert_resource(UIKeyboardCapture::default())
            .insert_resource(UIEditorState::from_editor(&self.editor))
            .add_systems(Update, toggle_ui_editor)
            .add_systems(EguiPrimaryContextPass, render_ui_editor);
    }
}

fn toggle_ui_editor(
    keyboard: Res<ButtonInput<KeyCode>>,
    editor: Res<UIEditor>,
    mut state: ResMut<UIEditorState>,
) {
    if keyboard.just_pressed(editor.toggle_key) {
        state.is_open = !state.is_open;
    }
}

fn render_ui_editor(
    mut contexts: EguiContexts,
    editor: Res<UIEditor>,
    mut state: ResMut<UIEditorState>,
    mut keyboard_capture: ResMut<UIKeyboardCapture>,
    map_configs: Option<ResMut<Persistent<MapConfigs>>>,
    autosave: Option<ResMut<MapConfigAutosave>>,
) {
    keyboard_capture.is_typing = false;
    keyboard_capture.wants_pointer_input = false;

    if !state.is_open {
        return;
    }

    let (Some(mut map_configs), Some(mut autosave)) = (map_configs, autosave) else {
        return;
    };

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let has_external_change = map_configs.is_changed();
    state.sync_from_map_configs(&map_configs, has_external_change);

    egui::SidePanel::left(editor.panel_id.clone())
        .resizable(false)
        .default_width(280.0)
        .show(ctx, |ui| {
            ui.heading(editor.panel_title.as_str());
            ui.label("Development-only editor");
            ui.small(format!("Toggle: {:?}", editor.toggle_key));
            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("Terrain");
                ui.label("Map Configuration");
                ui.small("0 keeps the most detail. 6 simplifies the mesh the most.");
                ui.add_space(6.0);

                property_grid(ui, GRID_ID, |ui| {
                    for field in ScalarEditorField::ALL {
                        render_editor_field_row(
                            ui,
                            EditorField::Scalar(field),
                            field.label(),
                            &mut state,
                            &mut map_configs,
                            &mut autosave,
                        );
                    }

                    render_draw_mode_row(ui, &mut map_configs, &mut autosave);
                });

                render_endless_lod_bands_section(ui, &mut state, &mut map_configs, &mut autosave);

                render_height_curve_section(ui, &mut state, &mut map_configs, &mut autosave);

                ui.separator();
                ui.heading("Regions");
                ui.add_space(6.0);

                if map_configs.regions.is_empty() {
                    ui.small("No terrain regions configured.");
                } else {
                    for index in 0..map_configs.regions.len() {
                        render_region_section(
                            ui,
                            index,
                            &mut state,
                            &mut map_configs,
                            &mut autosave,
                        );
                        ui.add_space(8.0);
                    }
                }
            });
        });

    keyboard_capture.is_typing = state.is_any_field_focused();
    keyboard_capture.wants_pointer_input = ctx.wants_pointer_input();
}

fn render_editor_field_row(
    ui: &mut egui::Ui,
    field: EditorField,
    label: &str,
    state: &mut UIEditorState,
    map_configs: &mut Persistent<MapConfigs>,
    autosave: &mut MapConfigAutosave,
) {
    let enabled = !field.is_read_only();
    let response = {
        let buffer = state.input_mut(field);
        stepper_input_row_enabled(ui, label, buffer, enabled)
    };

    if response.decrement_clicked() && field.decrement(map_configs.get_mut()) {
        autosave.mark_dirty();
        state.sync_field_from_map_configs(field, map_configs);
    }

    if response.changed() {
        let input = state.input_mut(field).clone();
        if field.apply_input(input.as_str(), map_configs.get_mut()) {
            autosave.mark_dirty();
        }
    }

    if response.lost_focus() {
        state.sync_field_from_map_configs(field, map_configs);
    }

    state.set_focus(field, response.has_focus());

    if response.increment_clicked() && field.increment(map_configs.get_mut()) {
        autosave.mark_dirty();
        state.sync_field_from_map_configs(field, map_configs);
    }
}

fn render_draw_mode_row(
    ui: &mut egui::Ui,
    map_configs: &mut Persistent<MapConfigs>,
    autosave: &mut MapConfigAutosave,
) {
    let mut draw_mode = map_configs.draw_mode;

    ui.label("Draw Mode");
    egui::ComboBox::from_id_salt("ui_editor_draw_mode")
        .selected_text(draw_mode.label())
        .width(96.0)
        .show_ui(ui, |ui| {
            for mode in DrawMode::ALL {
                ui.selectable_value(&mut draw_mode, mode, mode.label());
            }
        });
    ui.label("");
    ui.label("");
    ui.end_row();

    if map_configs.get_mut().set_draw_mode(draw_mode) {
        autosave.mark_dirty();
    }

    render_uv_wireframe_row(ui, map_configs, autosave);
}

fn render_height_curve_section(
    ui: &mut egui::Ui,
    state: &mut UIEditorState,
    map_configs: &mut Persistent<MapConfigs>,
    autosave: &mut MapConfigAutosave,
) {
    ui.separator();
    ui.heading("Height Curve");
    ui.small("Applies only to Mesh draw mode.");
    ui.add_space(6.0);

    ui.horizontal(|ui| {
        if ui.button("Add Point").clicked() && map_configs.get_mut().add_height_curve_point() {
            autosave.mark_dirty();
            state.sync_from_map_configs(map_configs, true);
        }

        if ui.button("Reset Curve").clicked() && map_configs.get_mut().reset_height_curve() {
            autosave.mark_dirty();
            state.sync_from_map_configs(map_configs, true);
        }
    });

    ui.add_space(6.0);

    egui::Grid::new(HEIGHT_CURVE_GRID_ID)
        .num_columns(4)
        .spacing([8.0, 10.0])
        .show(ui, |ui| {
            ui.strong("Point");
            ui.strong("Input");
            ui.strong("Output");
            ui.label("");
            ui.end_row();

            let point_count = map_configs.height_curve.points.len();
            for index in 0..point_count {
                if render_curve_point_row(ui, index, state, map_configs, autosave) {
                    break;
                }
            }
        });
}

fn render_endless_lod_bands_section(
    ui: &mut egui::Ui,
    state: &mut UIEditorState,
    map_configs: &mut Persistent<MapConfigs>,
    autosave: &mut MapConfigAutosave,
) {
    ui.separator();
    ui.heading("Endless Terrain LOD");
    ui.small("Used only in Endless Terrain mode. Distances are maximum visible ranges for each LOD band.");
    ui.add_space(6.0);

    ui.horizontal(|ui| {
        if ui.button("Add Band").clicked() && map_configs.get_mut().add_endless_lod_band() {
            autosave.mark_dirty();
            state.sync_from_map_configs(map_configs, true);
        }

        if ui.button("Reset Bands").clicked() && map_configs.get_mut().reset_endless_lod_bands() {
            autosave.mark_dirty();
            state.sync_from_map_configs(map_configs, true);
        }
    });

    ui.add_space(6.0);

    egui::Grid::new("ui_editor_endless_lod_grid")
        .num_columns(4)
        .spacing([8.0, 10.0])
        .show(ui, |ui| {
            ui.strong("Band");
            ui.strong("Max Distance");
            ui.strong("LOD");
            ui.label("");
            ui.end_row();

            for index in 0..map_configs.endless_lod_bands.len() {
                if render_lod_band_row(ui, index, state, map_configs, autosave) {
                    break;
                }
            }
        });
}

fn render_uv_wireframe_row(
    ui: &mut egui::Ui,
    map_configs: &mut Persistent<MapConfigs>,
    autosave: &mut MapConfigAutosave,
) {
    let enabled = matches!(map_configs.draw_mode, DrawMode::Mesh | DrawMode::EndlessTerrain);
    let mut show_uv_wireframe = map_configs.show_uv_wireframe;

    ui.label("Show UV Wireframe");
    ui.add_enabled_ui(enabled, |ui| {
        ui.checkbox(&mut show_uv_wireframe, "");
    });
    ui.label("");
    ui.end_row();

    if enabled && map_configs.get_mut().set_show_uv_wireframe(show_uv_wireframe) {
        autosave.mark_dirty();
    }
}

fn render_lod_band_row(
    ui: &mut egui::Ui,
    index: usize,
    state: &mut UIEditorState,
    map_configs: &mut Persistent<MapConfigs>,
    autosave: &mut MapConfigAutosave,
) -> bool {
    ui.label(format!("Band {}", index + 1));
    render_lod_band_cell(
        ui,
        EditorField::LodBandDistance(index),
        state,
        map_configs,
        autosave,
        80.0,
    );
    render_lod_band_cell(
        ui,
        EditorField::LodBandLevel(index),
        state,
        map_configs,
        autosave,
        48.0,
    );

    let remove_clicked = ui
        .add_enabled(
            map_configs.endless_lod_bands.len() > 1,
            egui::Button::new("Remove"),
        )
        .clicked();
    ui.end_row();

    if remove_clicked && map_configs.get_mut().remove_endless_lod_band(index) {
        autosave.mark_dirty();
        state.sync_from_map_configs(map_configs, true);
        return true;
    }

    false
}

fn render_lod_band_cell(
    ui: &mut egui::Ui,
    field: EditorField,
    state: &mut UIEditorState,
    map_configs: &mut Persistent<MapConfigs>,
    autosave: &mut MapConfigAutosave,
    width: f32,
) {
    let response = {
        let buffer = state.input_mut(field);
        ui.add(
            egui::TextEdit::singleline(buffer)
                .desired_width(width)
                .horizontal_align(egui::Align::Center),
        )
    };

    if response.changed() {
        let input = state.input_mut(field).clone();
        if field.apply_input(input.as_str(), map_configs.get_mut()) {
            autosave.mark_dirty();
        }
    }

    if response.lost_focus() {
        state.sync_field_from_map_configs(field, map_configs);
    }

    state.set_focus(field, response.has_focus());
}

fn render_curve_point_row(
    ui: &mut egui::Ui,
    index: usize,
    state: &mut UIEditorState,
    map_configs: &mut Persistent<MapConfigs>,
    autosave: &mut MapConfigAutosave,
) -> bool {
    let point_count = map_configs.height_curve.points.len();
    let is_endpoint = index == 0 || index + 1 == point_count;

    ui.label(format!("Point {}", index + 1));
    render_curve_point_cell(
        ui,
        EditorField::CurvePointInput(index),
        state,
        map_configs,
        autosave,
        !is_endpoint,
    );
    render_curve_point_cell(
        ui,
        EditorField::CurvePointOutput(index),
        state,
        map_configs,
        autosave,
        true,
    );

    let remove_clicked = ui.add_enabled(!is_endpoint, egui::Button::new("Remove")).clicked();
    ui.end_row();

    if remove_clicked && map_configs.get_mut().remove_height_curve_point(index) {
        autosave.mark_dirty();
        state.sync_from_map_configs(map_configs, true);
        return true;
    }

    false
}

fn render_curve_point_cell(
    ui: &mut egui::Ui,
    field: EditorField,
    state: &mut UIEditorState,
    map_configs: &mut Persistent<MapConfigs>,
    autosave: &mut MapConfigAutosave,
    editable: bool,
) {
    if !editable {
        state.set_focus(field, false);
        ui.label(field.display_value(map_configs));
        return;
    }

    let response = {
        let buffer = state.input_mut(field);
        ui.add(
            egui::TextEdit::singleline(buffer)
                .desired_width(64.0)
                .horizontal_align(egui::Align::Center),
        )
    };

    if response.changed() {
        let input = state.input_mut(field).clone();
        if field.apply_input(input.as_str(), map_configs.get_mut()) {
            autosave.mark_dirty();
        }
    }

    if response.lost_focus() {
        state.sync_field_from_map_configs(field, map_configs);
    }

    state.set_focus(field, response.has_focus());
}

fn render_region_section(
    ui: &mut egui::Ui,
    index: usize,
    state: &mut UIEditorState,
    map_configs: &mut Persistent<MapConfigs>,
    autosave: &mut MapConfigAutosave,
) {
    let title = map_configs
        .regions
        .get(index)
        .map(|region| {
            if region.name.trim().is_empty() {
                format!("Terrain Type {}", index + 1)
            } else {
                format!("Terrain Type {}: {}", index + 1, region.name)
            }
        })
        .unwrap_or_else(|| format!("Terrain Type {}", index + 1));

    ui.group(|ui| {
        ui.label(title);
        ui.add_space(6.0);

        egui::Grid::new(format!("{REGION_GRID_ID_PREFIX}_{index}"))
            .num_columns(2)
            .spacing([8.0, 10.0])
            .show(ui, |ui| {
                render_region_text_row(
                    ui,
                    EditorField::RegionName(index),
                    "Name",
                    state,
                    map_configs,
                    autosave,
                );
                render_region_text_row(
                    ui,
                    EditorField::RegionHeight(index),
                    "Height",
                    state,
                    map_configs,
                    autosave,
                );
                render_region_color_row(ui, index, map_configs, autosave);
            });
    });
}

fn render_region_text_row(
    ui: &mut egui::Ui,
    field: EditorField,
    label: &str,
    state: &mut UIEditorState,
    map_configs: &mut Persistent<MapConfigs>,
    autosave: &mut MapConfigAutosave,
) {
    ui.label(label);
    let response = {
        let buffer = state.input_mut(field.clone());
        ui.add(egui::TextEdit::singleline(buffer).desired_width(150.0))
    };
    ui.end_row();

    if response.changed() {
        let input = state.input_mut(field.clone()).clone();
        if field.apply_input(input.as_str(), map_configs.get_mut()) {
            autosave.mark_dirty();
        }
    }

    if response.lost_focus() {
        state.sync_field_from_map_configs(field.clone(), map_configs);
    }

    state.set_focus(field, response.has_focus());
}

fn render_region_color_row(
    ui: &mut egui::Ui,
    index: usize,
    map_configs: &mut Persistent<MapConfigs>,
    autosave: &mut MapConfigAutosave,
) {
    let Some(region) = map_configs.regions.get(index) else {
        return;
    };

    let mut color = [region.color.0, region.color.1, region.color.2];

    ui.label("Color");
    if ui.color_edit_button_srgb(&mut color).changed()
        && map_configs
            .get_mut()
            .set_region_color(index, (color[0], color[1], color[2]))
    {
        autosave.mark_dirty();
    }
    ui.end_row();
}

impl EditorField {
    fn is_read_only(&self) -> bool {
        matches!(self, Self::Scalar(field) if field.is_read_only())
    }

    fn decrement(&self, map_configs: &mut MapConfigs) -> bool {
        match self {
            Self::Scalar(field) => field.decrement(map_configs),
            Self::CurvePointInput(_)
            | Self::CurvePointOutput(_)
            | Self::LodBandDistance(_)
            | Self::LodBandLevel(_)
            | Self::RegionName(_)
            | Self::RegionHeight(_) => false,
        }
    }

    fn increment(&self, map_configs: &mut MapConfigs) -> bool {
        match self {
            Self::Scalar(field) => field.increment(map_configs),
            Self::CurvePointInput(_)
            | Self::CurvePointOutput(_)
            | Self::LodBandDistance(_)
            | Self::LodBandLevel(_)
            | Self::RegionName(_)
            | Self::RegionHeight(_) => false,
        }
    }
}

fn format_decimal(value: f64) -> String {
    let mut formatted = format!("{value:.3}");

    while formatted.contains('.') && formatted.ends_with('0') {
        formatted.pop();
    }

    if formatted.ends_with('.') {
        formatted.pop();
    }

    formatted
}
