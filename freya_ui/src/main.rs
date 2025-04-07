mod components;
mod player_ref;

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
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            players: Default::default(),
            top_group: Default::default(),
            groups: Default::default(),
            global_paused: Default::default(),
            master_volume: 1.0,
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
