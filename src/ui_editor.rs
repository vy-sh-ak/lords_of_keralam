use bevy::camera::Viewport;
use bevy::prelude::*;
use bevy::reflect::TypeRegistry;
use bevy::window::{PrimaryWindow, Window};
use bevy_egui::egui::{self, LayerId};
use bevy_egui::{EguiPrimaryContextPass, PrimaryEguiContext};
use bevy_inspector_egui::bevy_egui::EguiContextSettings;
use bevy_inspector_egui::bevy_inspector::hierarchy::{SelectedEntities, hierarchy_ui};
use bevy_inspector_egui::bevy_inspector::{self, ui_for_entity_with_children};
use bevy_inspector_egui::reflect_inspector;
use bevy_persistent::Persistent;
use egui_dock::{DockArea, DockState, NodeIndex, Style};
use std::any::TypeId;

use crate::editor_config::EditorState;
use crate::terrain::{DrawMode, FallOffGenerator, MapGenerator};
use crate::terrain_painter;
use crate::world_grid_config::WorldGrid;
use curve_editor::height_curve_editor;

pub mod widgets;
pub mod curve_editor;

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

#[derive(Debug, PartialEq, Eq, Clone)]
enum EguiWindow {
    GameView,
    Hierarchy,
    Inspector,
    Resources,
    MapGenerator,
    TerrainPainter,
}

#[derive(Eq, PartialEq)]
enum InspectorSelection {
    Entities,
    Resource(TypeId, String),
}
#[derive(Resource)]
pub struct UiState {
    dock_state: DockState<EguiWindow>,
    viewport_rect: egui::Rect,
    selected_entities: SelectedEntities,
    selection: InspectorSelection,
    pointer_in_viewport: bool,
    is_open: bool,
}

impl UiState {
    fn new(starts_open: bool) -> Self {
        let initial_tabs = if starts_open {
            vec![
                EguiWindow::GameView,
                EguiWindow::Hierarchy,
                EguiWindow::Inspector,
                EguiWindow::Resources,
                EguiWindow::MapGenerator,
                EguiWindow::TerrainPainter,
            ]
        } else {
            vec![EguiWindow::GameView]
        };

        let mut dock_state = DockState::new(initial_tabs);

        if starts_open {
            let tree = dock_state.main_surface_mut();
            // GameView takes most space; Inspector on the right
            let [game, _inspector] =
                tree.split_right(NodeIndex::root(), 0.75, vec![EguiWindow::Inspector,EguiWindow::MapGenerator,EguiWindow::TerrainPainter]);
            // Hierarchy on the left
            let [game, _hierarchy] = tree.split_left(game, 0.2, vec![EguiWindow::Hierarchy]);
            // TerrainConfig and Resources at the bottom (collapsed by default)
            let [_game, bottom] = tree.split_below(game, 0.7, vec![EguiWindow::Resources]);
            tree[bottom].set_collapsed(true);
        }

        Self {
            dock_state,
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

        // Terrain painter toolbar (shown when sculpt mode is active)
        let show_toolbar = world
            .get_resource::<terrain_painter::BrushConfig>()
            .is_some_and(|b| b.active);
        if show_toolbar {
            egui::TopBottomPanel::top("terrain_toolbar")
                .min_height(0.0)
                .show(ctx, |ui| {
                    terrain_painter::ui::toolbar_contents(world, ui);
                });
        }

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
        let mut tab_viewer = TabViewer {
            world,
            viewport_rect: &mut self.viewport_rect,
            selected_entities: &mut self.selected_entities,
            selection: &mut self.selection,
            pointer_in_viewport: &mut self.pointer_in_viewport,
        };
        DockArea::new(&mut self.dock_state)
            .style(Style::from_egui(ctx.style().as_ref()))
            .show(ctx, &mut tab_viewer);
    }
}

struct TabViewer<'a> {
    world: &'a mut World,
    selected_entities: &'a mut SelectedEntities,
    selection: &'a mut InspectorSelection,
    viewport_rect: &'a mut egui::Rect,
    pointer_in_viewport: &'a mut bool,
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = EguiWindow;

    fn ui(&mut self, ui: &mut egui::Ui, window: &mut Self::Tab) {
        match window {
            EguiWindow::GameView => {
                *self.viewport_rect = ui.clip_rect();
            }
            EguiWindow::Hierarchy => {
                ui.push_id("hierarchy_tab", |ui| {
                    let selected = hierarchy_ui(self.world, ui, self.selected_entities);
                    if selected {
                        *self.selection = InspectorSelection::Entities;
                    }
                });
            }
            EguiWindow::Inspector => {
                ui.push_id("inspector_tab", |ui| {
                    render_inspector_tab(ui, self.world, self.selected_entities, self.selection);
                });
            }
            EguiWindow::Resources => {
                ui.push_id("resources_tab", |ui| {
                    let type_registry = self.world.resource::<AppTypeRegistry>().0.clone();
                    let type_registry = type_registry.read();
                    render_resources_tab(ui, &type_registry, self.selection);
                });
            }
            EguiWindow::MapGenerator => {
                ui.push_id("map_generator_tab", |ui| {
                    render_map_generator_editor(ui, self.world);
                });
            }
            EguiWindow::TerrainPainter => {
                ui.push_id("terrain_painter_tab", |ui| {
                    terrain_painter::ui::tab_contents(self.world, ui);
                });
            }
        }

        *self.pointer_in_viewport = ui
            .ctx()
            .rect_contains_pointer(LayerId::background(), self.viewport_rect.shrink(16.));
    }

    fn title(&mut self, window: &mut Self::Tab) -> egui_dock::egui::WidgetText {
        match window {
            EguiWindow::GameView => "Game View".into(),
            EguiWindow::Hierarchy => "Hierarchy".into(),
            EguiWindow::Inspector => "Inspector".into(),
            EguiWindow::Resources => "Resources".into(),
            EguiWindow::MapGenerator => "Map Generator".into(),
            EguiWindow::TerrainPainter => "Terrain Painter".into(),
        }
    }

    fn clear_background(&self, window: &Self::Tab) -> bool {
        !matches!(window, EguiWindow::GameView)
    }
}

fn render_map_generator_editor(ui: &mut egui::Ui, world: &mut World) {
    let type_registry = world.resource::<AppTypeRegistry>().0.clone();
    let type_registry = type_registry.read();

    // Scope the editor borrow so it's dropped before we access WorldGrid below.
    let changed = {
        let mut editor = world.resource_mut::<EditorState<MapGenerator>>();
        let mut changed = false;

        // ---- Main Settings ----
        egui::Grid::new("map_gen_main").num_columns(2).spacing([12.0, 8.0]).show(ui, |ui| {
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
                            .selectable_value(&mut editor.edited.draw_mode, mode, mode.label())
                            .changed();
                    }
                });
            ui.end_row();

            ui.label("Show UV Wireframe");
            changed |= ui.checkbox(&mut editor.edited.show_uv_wireframe, "").changed();
            ui.end_row();
        });

        ui.add_space(8.0);

        // ---- Noise Data ----
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

        // ---- Terrain Data ----
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

        // ---- Texture Data ----
        egui::CollapsingHeader::new("Texture Data")
            .default_open(false)
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
                                        egui::ComboBox::from_id_salt(format!("tex_combo_{}", i))
                                            .selected_text(&layer.texture_path)
                                            .show_ui(ui, |ui| {
                                                for tex in &textures {
                                                    changed |= ui
                                                        .selectable_value(
                                                            &mut layer.texture_path,
                                                            tex.clone(),
                                                            tex,
                                                        )
                                                        .changed();
                                                }
                                            });
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
                                        .add(egui::Slider::new(&mut layer.start_height, 0.0..=1.0))
                                        .changed();
                                });

                                ui.horizontal(|ui| {
                                    ui.label("Blend");
                                    changed |= ui
                                        .add(egui::Slider::new(&mut layer.blend_strength, 0.0..=1.0))
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
                                        .add(egui::Slider::new(&mut layer.tint_strength, 0.0..=1.0))
                                        .changed();
                                });

                                ui.horizontal(|ui| {
                                    ui.label("Tex Scale");
                                    changed |= ui
                                        .add(egui::Slider::new(&mut layer.texture_scale, 0.1..=500.0))
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
                            let default_tex = textures.first().cloned().unwrap_or_else(|| "textures/grass.png".to_string());
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

        // ---- Height Curve ----
        egui::CollapsingHeader::new("Height Curve")
            .default_open(false)
            .show(ui, |ui| {
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    changed |= height_curve_editor(ui, &mut editor.edited.terrain_data.height_curve);
                });
            });

        // ---- Endless LOD Bands ----
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

    // ---- Grid Settings ----
    ui.add_space(8.0);
    egui::CollapsingHeader::new("Grid Settings")
        .default_open(true)
        .show(ui, |ui| {
            egui::Frame::group(ui.style()).show(ui, |ui| {
                let mut grid = world.resource_mut::<WorldGrid>();
                ui.horizontal(|ui| {
                    ui.label("Show Grid");
                    ui.checkbox(&mut grid.show_grid, "");
                });
            });
        });

    // ---- Live preview sync ----
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
        if ui.selectable_label(selected, name).clicked() {
            *selection = InspectorSelection::Resource(type_id, name.to_string());
        }
    }
}

