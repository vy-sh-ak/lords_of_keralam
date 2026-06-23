use bevy::camera::Viewport;
use bevy::prelude::*;
use bevy::reflect::TypeRegistry;
use bevy::window::{PrimaryWindow, Window};
use bevy_egui::egui::{self, LayerId};
use bevy_egui::{EguiPrimaryContextPass, PrimaryEguiContext};
use bevy_inspector_egui::bevy_egui::EguiContextSettings;
use bevy_inspector_egui::bevy_inspector::hierarchy::{SelectedEntities, hierarchy_ui};
use bevy_inspector_egui::bevy_inspector::{self, ui_for_entity_with_children};
use std::any::TypeId;

use bevy_persistent::Persistent;

use crate::editor_config::EditorState;
use crate::terrain::{FallOffGenerator, MapGenerator};
use crate::terrain_painter::BrushConfig;
use crate::world_grid_config::WorldGrid;

pub mod curve_editor;
pub mod map_generator_ui;
pub mod terrain_sculpting_ui;
pub mod widgets;

#[derive(Resource, Clone)]
pub struct UIEditor {
    toggle_key: KeyCode,
    starts_open: bool,
}

impl Default for UIEditor {
    fn default() -> Self {
        Self {
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
}

pub struct UIEditorPlugin {
    editor: UIEditor,
}

#[derive(Resource, Default)]
pub struct UIKeyboardCapture {
    pub is_typing: bool,
    pub wants_pointer_input: bool,
    pub pointer_in_viewport: bool,
}

// ---- Navigation enums ----

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MainSection {
    Map,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LeftPanelKind {
    Hierarchy,
    Resources,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RightPanelKind {
    MapGenerator,
    TerrainPainter,
    Inspector,
}

#[derive(Eq, PartialEq)]
enum InspectorSelection {
    Entities,
    Resource(TypeId, String),
}

#[derive(Resource)]
pub struct UiState {
    selected_section: MainSection,
    active_left_panel: Option<LeftPanelKind>,
    active_right_panel: Option<RightPanelKind>,
    viewport_rect: egui::Rect,
    selected_entities: SelectedEntities,
    selection: InspectorSelection,
    pointer_in_viewport: bool,
    is_open: bool,
}

impl UiState {
    fn new(starts_open: bool) -> Self {
        Self {
            selected_section: MainSection::Map,
            active_left_panel: None,
            active_right_panel: None,
            viewport_rect: egui::Rect::NOTHING,
            selected_entities: SelectedEntities::default(),
            selection: InspectorSelection::Entities,
            pointer_in_viewport: false,
            is_open: starts_open,
        }
    }
}

// ---------------------------------------------------------------------------
// Plugin impl
// ---------------------------------------------------------------------------

impl Plugin for UIEditorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.editor.clone())
            .insert_resource(UIKeyboardCapture::default())
            .insert_resource(UiState::new(self.editor.starts_open))
            .add_systems(Update, toggle_ui_editor)
            .add_systems(EguiPrimaryContextPass, show_ui_system)
            .add_systems(PostUpdate, set_camera_viewport.after(show_ui_system));
    }
}

fn toggle_ui_editor(
    keyboard: Res<ButtonInput<KeyCode>>,
    editor: Res<UIEditor>,
    mut ui_state: ResMut<UiState>,
) {
    if keyboard.just_pressed(editor.toggle_key) {
        ui_state.is_open = !ui_state.is_open;
    }
}

fn show_ui_system(world: &mut World) {
    let Ok(egui_context) = world
        .query_filtered::<&mut bevy_egui::EguiContext, With<PrimaryEguiContext>>()
        .single(world)
    else {
        return;
    };
    let mut egui_context = egui_context.clone();

    world.resource_scope::<UiState, _>(|world, mut ui_state| {
        if !ui_state.is_open {
            let ctx = egui_context.get_mut();
            ui_state.viewport_rect = ctx.input(|i| i.content_rect());
            let mut kb = world.resource_mut::<UIKeyboardCapture>();
            kb.wants_pointer_input = false;
            kb.is_typing = false;
            kb.pointer_in_viewport = true;
            return;
        }

        let ctx = egui_context.get_mut();

        ui_state.ui(world, ctx);
        let in_viewport = ui_state.pointer_in_viewport;
        let mut kb = world.resource_mut::<UIKeyboardCapture>();
        kb.pointer_in_viewport = in_viewport;
        kb.wants_pointer_input = !in_viewport && ctx.wants_pointer_input();
        kb.is_typing = ctx.wants_keyboard_input();
    });
}

fn set_camera_viewport(
    ui_state: Res<UiState>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut cam: Single<&mut Camera, Without<PrimaryEguiContext>>,
    egui_settings: Single<&EguiContextSettings>,
) {
    if !ui_state.is_open {
        cam.viewport = None;
        return;
    }
    let scale_factor = window.scale_factor() * egui_settings.scale_factor;

    let viewport_pos = ui_state.viewport_rect.left_top().to_vec2() * scale_factor;
    let viewport_size = ui_state.viewport_rect.size() * scale_factor;

    let physical_position = UVec2::new(viewport_pos.x as u32, viewport_pos.y as u32);
    let physical_size = UVec2::new(viewport_size.x as u32, viewport_size.y as u32);

    let rect = physical_position + physical_size;
    let window_size = window.physical_size();
    if rect.x <= window_size.x && rect.y <= window_size.y {
        cam.viewport = Some(Viewport {
            physical_position,
            physical_size,
            depth: 0.0..1.0,
        });
    }
}

impl UiState {
    fn ui(&mut self, world: &mut World, ctx: &mut egui::Context) {
        // ---- Top bar: sections row + editor buttons row ----
        egui::TopBottomPanel::top("editor_top_bar")
            .min_height(80.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let is_map = self.selected_section == MainSection::Map;
                    if ui.add(egui::Button::new("Map").selected(is_map)).clicked() {
                        self.selected_section = MainSection::Map;
                    }
                });

                ui.separator();

                ui.horizontal(|ui| {
                    // Quick actions
                    ui.vertical(|ui| {
                        ui.set_width(60.0);
                        let mut editor = world.resource_mut::<EditorState<MapGenerator>>();
                        let mut uv = editor.edited.show_uv_wireframe;
                        if ui.checkbox(&mut uv, "UV").changed()
                            && uv != editor.edited.show_uv_wireframe
                        {
                            editor.edited.show_uv_wireframe = uv;
                            let edited = editor.edited.clone();
                            drop(editor);
                            let mut persistent = world.resource_mut::<Persistent<MapGenerator>>();
                            *persistent.get_mut() = edited;
                            if persistent.terrain_data.use_falloff_map
                                && persistent.falloff_map.is_empty()
                            {
                                let map_size = persistent.map_chunk_size as usize + 2;
                                persistent.falloff_map =
                                    FallOffGenerator::generate_fall_off_map(map_size);
                            }
                            persistent.set_changed();
                        } else {
                            drop(editor);
                        }

                        let mut grid = world.resource_mut::<WorldGrid>();
                        ui.checkbox(&mut grid.show_grid, "Grid");
                    });

                    ui.separator();

                    // Editor buttons (64px tall)
                    let is_active = self.active_right_panel == Some(RightPanelKind::MapGenerator);
                    if ui
                        .add(
                            egui::Button::new("Map Gen")
                                .selected(is_active)
                                .min_size(egui::vec2(0.0, 45.0)),
                        )
                        .clicked()
                    {
                        self.active_right_panel = if is_active {
                            None
                        } else {
                            Some(RightPanelKind::MapGenerator)
                        };
                    }
                    let is_active = self.active_right_panel == Some(RightPanelKind::TerrainPainter);
                    if ui
                        .add(
                            egui::Button::new("Terrain Paint")
                                .selected(is_active)
                                .min_size(egui::vec2(0.0, 45.0)),
                        )
                        .clicked()
                    {
                        self.active_right_panel = if is_active {
                            None
                        } else {
                            Some(RightPanelKind::TerrainPainter)
                        };
                    }

                    let mut brush_config = world.resource_mut::<BrushConfig>();
                    brush_config.active = self.active_right_panel == Some(RightPanelKind::TerrainPainter);
                });
            });

        // ---- Left sidebar: H / R buttons ----
        egui::SidePanel::left("left_sidebar")
            .resizable(false)
            .default_width(32.0)
            .width_range(32.0..=32.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(8.0);
                    let is_h = self.active_left_panel == Some(LeftPanelKind::Hierarchy);
                    if ui.add(egui::Button::new("H").selected(is_h)).clicked() {
                        self.active_left_panel = if is_h {
                            None
                        } else {
                            Some(LeftPanelKind::Hierarchy)
                        };
                    }
                    ui.add_space(4.0);
                    let is_r = self.active_left_panel == Some(LeftPanelKind::Resources);
                    if ui.add(egui::Button::new("R").selected(is_r)).clicked() {
                        self.active_left_panel = if is_r {
                            None
                        } else {
                            Some(LeftPanelKind::Resources)
                        };
                    }
                });
            });

        // ---- Left panel (conditional) ----
        let left_kind = self.active_left_panel;
        if let Some(kind) = left_kind {
            egui::SidePanel::left("left_panel")
                .resizable(true)
                .default_width(200.0)
                .width_range(80.0..=500.0)
                .show(ctx, |ui| match kind {
                    LeftPanelKind::Hierarchy => {
                        ui.push_id("hierarchy_panel", |ui| {
                            let selected = hierarchy_ui(world, ui, &mut self.selected_entities);
                            if selected {
                                self.selection = InspectorSelection::Entities;
                                self.active_right_panel = Some(RightPanelKind::Inspector);
                            }
                        });
                    }
                    LeftPanelKind::Resources => {
                        ui.push_id("resources_panel", |ui| {
                            let type_registry = world.resource::<AppTypeRegistry>().0.clone();
                            let type_registry = type_registry.read();
                            render_resources_tab(ui, &type_registry, &mut self.selection);
                        });
                        if matches!(self.selection, InspectorSelection::Resource(_, _)) {
                            self.active_right_panel = Some(RightPanelKind::Inspector);
                        }
                    }
                });
        }

        // ---- Right panel (conditional) ----
        let right_kind = self.active_right_panel;
        if let Some(kind) = right_kind {
            egui::SidePanel::right("right_panel")
                .resizable(true)
                .default_width(400.0)
                .width_range(200.0..=800.0)
                .show(ctx, |ui| match kind {
                    RightPanelKind::MapGenerator => {
                        ui.push_id("map_gen_editor", |ui| {
                            map_generator_ui::render_map_generator_editor(ui, world);
                        });
                    }
                    RightPanelKind::TerrainPainter => {
                        ui.push_id("terrain_painter_editor", |ui| {
                            terrain_sculpting_ui::tab_contents(world, ui);
                        });
                    }
                    RightPanelKind::Inspector => {
                        ui.push_id("inspector_panel", |ui| {
                            render_inspector_tab(
                                ui,
                                world,
                                &self.selected_entities,
                                &self.selection,
                            );
                        });
                    }
                });
        }

        // ---- Central game view (transparent frame so 3D viewport shows through) ----
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                self.viewport_rect = ui.clip_rect();
            });

        self.pointer_in_viewport =
            ctx.rect_contains_pointer(LayerId::background(), self.viewport_rect.shrink(16.));
    }
}

fn render_inspector_tab(
    ui: &mut egui::Ui,
    world: &mut World,
    selected_entities: &SelectedEntities,
    selection: &InspectorSelection,
) {
    match selection {
        InspectorSelection::Entities => match selected_entities.as_slice() {
            &[entity] => ui_for_entity_with_children(world, entity, ui),
            entities => {
                bevy_inspector::ui_for_entities_shared_components(world, entities, ui);
            }
        },
        InspectorSelection::Resource(type_id, name) => {
            let type_registry = world.resource::<AppTypeRegistry>().0.clone();
            let type_registry = type_registry.read();
            ui.label(name);
            bevy_inspector::by_type_id::ui_for_resource(world, *type_id, ui, name, &type_registry);
        }
    }
}

fn render_resources_tab(
    ui: &mut egui::Ui,
    type_registry: &TypeRegistry,
    selection: &mut InspectorSelection,
) {
    let mut resources: Vec<_> = type_registry
        .iter()
        .filter(|reg| reg.data::<bevy::ecs::reflect::ReflectResource>().is_some())
        .map(|reg| {
            (
                reg.type_info().type_path_table().short_path(),
                reg.type_id(),
            )
        })
        .collect();
    resources.sort_by(|(a, _), (b, _)| a.cmp(b));

    for (name, type_id) in resources {
        let selected = matches!(selection, InspectorSelection::Resource(id, _) if *id == type_id);
        if ui.add(egui::Button::new(name).selected(selected)).clicked() {
            *selection = InspectorSelection::Resource(type_id, name.to_string());
        }
    }
}
