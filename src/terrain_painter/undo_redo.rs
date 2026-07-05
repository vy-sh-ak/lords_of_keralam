use std::collections::HashMap;
use bevy::prelude::*;


use crate::terrain::endless_terrain::{EndlessTerrainState};
use crate::ui_editor::UIKeyboardCapture;

use super::{SculptMap, SyncGridRequest, BrushConfig};

#[derive(Clone)]
pub struct UndoEntry {
    pub old_chunks: HashMap<IVec2, Option<Vec<Vec<f32>>>>,
}

#[derive(Resource)]
pub struct UndoStack {
    pub undo_entries: Vec<UndoEntry>,
    pub redo_entries: Vec<UndoEntry>,
    max_entries: usize,
}

impl Default for UndoStack {
    fn default() -> Self {
        Self {
            undo_entries: Vec::new(),
            redo_entries: Vec::new(),
            max_entries: 100,
        }
    }
}

impl UndoStack {
    pub fn push(&mut self, entry: UndoEntry) {
        self.redo_entries.clear();
        self.undo_entries.push(entry);
        if self.undo_entries.len() > self.max_entries {
            self.undo_entries.remove(0);
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_entries.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_entries.is_empty()
    }
}

pub fn undo_redo_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut sculpt_map: ResMut<SculptMap>,
    mut undo_stack: ResMut<UndoStack>,
    mut endless_state: ResMut<EndlessTerrainState>,
    mut sync_request: ResMut<SyncGridRequest>,
    brush_config: Res<BrushConfig>,
    keyboard_capture: Option<Res<UIKeyboardCapture>>,
) {
    if !brush_config.active {
        return;
    }
    if let Some(capture) = keyboard_capture.as_ref() {
        if capture.is_typing {
            return;
        }
    }

    let ctrl = keyboard.pressed(KeyCode::ControlLeft)
        || keyboard.pressed(KeyCode::ControlRight);
    let shift = keyboard.pressed(KeyCode::ShiftLeft)
        || keyboard.pressed(KeyCode::ShiftRight);

    if ctrl && !shift && keyboard.just_pressed(KeyCode::KeyZ) {
        // Undo
        let entry = match undo_stack.undo_entries.pop() {
            Some(e) => e,
            None => return,
        };
        // Save current state of affected chunks for redo
        let mut redo_chunks = HashMap::new();
        for &coord in entry.old_chunks.keys() {
            redo_chunks.insert(coord, sculpt_map.chunks.get(&coord).cloned());
        }
        undo_stack.redo_entries.push(UndoEntry {
            old_chunks: redo_chunks,
        });
        // Restore old state
        for (coord, old_data) in entry.old_chunks {
            match old_data {
                Some(data) => {
                    sculpt_map.chunks.insert(coord, data);
                }
                None => {
                    sculpt_map.chunks.remove(&coord);
                }
            }
            endless_state.mark_chunk_dirty(coord);
        }
        sync_request.0 = true;
    } else if ctrl && shift && keyboard.just_pressed(KeyCode::KeyZ) {
        // Redo
        let entry = match undo_stack.redo_entries.pop() {
            Some(e) => e,
            None => return,
        };
        let mut undo_chunks = HashMap::new();
        for &coord in entry.old_chunks.keys() {
            undo_chunks.insert(coord, sculpt_map.chunks.get(&coord).cloned());
        }
        undo_stack.undo_entries.push(UndoEntry {
            old_chunks: undo_chunks,
        });
        for (coord, old_data) in entry.old_chunks {
            match old_data {
                Some(data) => {
                    sculpt_map.chunks.insert(coord, data);
                }
                None => {
                    sculpt_map.chunks.remove(&coord);
                }
            }
            endless_state.mark_chunk_dirty(coord);
        }
        sync_request.0 = true;
    }
}
