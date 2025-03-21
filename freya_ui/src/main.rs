mod components {
    pub mod player_view;
    pub mod top_controls;
}
mod player_ref;

use components::{
    player_view::PlayerView,
    top_controls::{AddPlayer, MasterVolume, PausePlay, Stop},
};
use freya::prelude::*;
use indexmap::{IndexMap, IndexSet};
use player_ref::PlayerRef;
use std::collections::HashMap;

#[derive(Default)]
struct AppState {
    pub players: HashMap<String, PlayerRef>,
    pub top_group: IndexSet<String>,
    pub groups: IndexMap<String, IndexSet<String>>,
    pub global_paused: bool,
    pub master_volume: f32,
}

fn main() {
    launch(app);
}

fn app() -> Element {
    let state = use_signal(|| AppState::default());

    let state_lock = state.read();

    rsx! {
        rect { direction: "horizontal", width: "fill", margin: "8",
            AddPlayer { state }
            PausePlay { state }
            Stop { state }
            MasterVolume { state }
        }
        for (_ , p) in state_lock.players.clone() {
            PlayerView { player: p, state }
        }
    }
}
