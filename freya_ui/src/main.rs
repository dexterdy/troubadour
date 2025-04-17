mod common_actions;
mod components;
mod player_ref;

use common_actions::{save, GlobalModals, ModalConfig, UnsavedModalResults};
use components::{
    player_view::PlayerView,
    top_controls::{AddPlayer, Load, MasterVolume, PausePlay, Save, Stop},
};
use freya::prelude::*;
use indexmap::{IndexMap, IndexSet};
use player_ref::PlayerRef;
use std::collections::HashMap;

// TODO: saving and loading
// TODO: groups
// TODO: icons
// TODO: theming
// TODO: layout

struct AppState {
    pub players: HashMap<String, PlayerRef>,
    pub top_group: IndexSet<String>,
    pub groups: IndexMap<String, IndexSet<String>>,
    pub global_paused: bool,
    pub master_volume: f32,
    pub global_modals: GlobalModals,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            players: Default::default(),
            top_group: Default::default(),
            groups: Default::default(),
            global_paused: Default::default(),
            master_volume: 1.0,
            global_modals: GlobalModals {
                unsaved_modal: ModalConfig {
                    continuations: Vec::new(),
                    shown: false,
                },
            },
        }
    }
}

fn main() {
    launch(app);
}

fn app() -> Element {
    let mut state = use_signal(|| AppState::default());

    let state_lock = state.read();

    let save = move |_| {
        spawn(async move {
            let _ = save(state).await;
            let mut writable_state = state.write();
            for cont in &mut writable_state.global_modals.unsaved_modal.continuations {
                (cont)(UnsavedModalResults::Saved)
            }
            writable_state
                .global_modals
                .unsaved_modal
                .continuations
                .clear();
        });
    };

    rsx! {
        if state.read().global_modals.unsaved_modal.shown {
            Popup {
                PopupTitle {
                    label { "You have unsaved changes. Do you want to save?" }
                }
                PopupContent {
                    label { "Unsaved changes will be lost." }
                    Button { onclick: save,
                        label { "Save" }
                    }
                }
            }
        }
        rect { direction: "horizontal", width: "fill",
            AddPlayer { state }
            PausePlay { state }
            Stop { state }
            Save { state }
            Load { state }
            MasterVolume { state }
        }
        for (_ , p) in state_lock.players.clone() {
            PlayerView { player: p, state }
        }
    }
}
