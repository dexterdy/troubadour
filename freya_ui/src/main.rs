mod common_actions;
mod components;
mod player_ref;

use common_actions::{GlobalModals, ModalConfig, UnsavedModal};
use components::{
    player_view::PlayerView,
    top_controls::{AddPlayer, Load, MasterVolume, PausePlay, Save, Stop},
};
use freya::prelude::*;
use indexmap::{IndexMap, IndexSet};
use player_ref::PlayerRef;
use std::collections::HashMap;

// TODO: error handling
// TODO: handle no path chosen
// TODO: saving and loading
// TODO: remove a player
// TODO: groups
// TODO: icons
// TODO: theming
// TODO: layout

struct AppState {
    pub players: HashMap<String, PlayerRef>,
    pub top_group: IndexSet<String>,
    pub groups: IndexMap<String, IndexSet<String>>,
    pub saved: bool,
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
            saved: true,
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
    let state = use_signal(|| AppState::default());

    let state_lock = state.read();

    rsx! {
        UnsavedModal { state }
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
