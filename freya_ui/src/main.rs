mod common_actions;
mod components;
mod player_ref;

use common_actions::GlobalModals;
use components::{
    player_view::PlayerView,
    top_controls::{AddPlayer, Load, MasterVolume, PausePlay, Save, Stop},
};
use freya::prelude::*;
use indexmap::{IndexMap, IndexSet};
use player_ref::PlayerRef;
use std::collections::HashMap;

/*
TODO: error handling
TODO: handle no path chosen
TODO: file extensions
TODO: grouping
TODO: icons
TODO: theming
TODO: layout
TODO: dark/light mode
TODO: player context menu (remove, add to group, cross fade, etc)
TODO: group context menu (remove, remove and remove players, combine with, cross fade, etc)
TODO: cross fade

You can cross fade from a context menu, in which case a cross fade of a default shape and length happens.
You can also create and save cross fades. You can select the shape and length of the cross fade.
This cross fade is then displayed as a button underneath the player/group
*/

#[derive(Debug)]
struct AppState {
    pub players: HashMap<String, PlayerRef>,
    pub top_group: IndexSet<String>,
    pub groups: IndexMap<String, IndexSet<String>>,
    pub saved: bool,
    pub global_paused: bool,
    pub master_volume: f32,
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
        GlobalModals { state,
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
}
