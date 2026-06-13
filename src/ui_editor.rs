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
use serde::de::DeserializeOwned;
use serde::{Serialize};
use std::any::TypeId;

use crate::editor_config::EditorState;
use crate::terrain::MapGenerator;

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

#[derive(Debug, PartialEq, Eq, Clone)]
enum EguiWindow {
    GameView,
    Hierarchy,
    Inspector,
    Resources,
    MapGenerator,
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
            ]
        } else {
            vec![EguiWindow::GameView]
        };

        let mut dock_state = DockState::new(initial_tabs);

        if starts_open {
            let tree = dock_state.main_surface_mut();
            // GameView takes most space; Inspector on the right
            let [game, _inspector] =
                tree.split_right(NodeIndex::root(), 0.75, vec![EguiWindow::Inspector,EguiWindow::MapGenerator]);
            // Hierarchy on the left
            let [game, _hierarchy] = tree.split_left(game, 0.2, vec![EguiWindow::Hierarchy]);
            // TerrainConfig and Resources at the bottom
            let [_game, _bottom] = tree.split_below(game, 0.7, vec![EguiWindow::Resources]);
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
        ui_state.ui(world, &mut egui_context.get_mut());
        let ctx = egui_context.get_mut();
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
                    render_editor::<MapGenerator>(ui, self.world);
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
        }
    }

    fn clear_background(&self, window: &Self::Tab) -> bool {
        !matches!(window, EguiWindow::GameView)
    }
}

fn render_editor<T>(
    ui: &mut egui::Ui,
    world: &mut World,
)
where
    T: Resource
        + Clone
        + Reflect
        + Serialize
        + DeserializeOwned,
{
    let type_registry = world.resource::<AppTypeRegistry>().0.clone();
    let type_registry = type_registry.read();

    let mut editor =
        world.resource_mut::<EditorState<T>>();

    let _ = reflect_inspector::ui_for_value(
        &mut editor.edited,
        ui,
        &type_registry,
    );

    let edited = editor.edited.clone();
    drop(editor);

    let mut persistent = world.resource_mut::<Persistent<T>>();

    let current = &**persistent as &dyn Reflect;
    if !edited.reflect_partial_eq(current).unwrap_or(true) {
        *persistent.get_mut() = edited;
        persistent.set_changed();
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
        if ui.selectable_label(selected, name).clicked() {
            *selection = InspectorSelection::Resource(type_id, name.to_string());
        }
    }
}
