mod common_actions;
mod components;
mod player_ref;

use crate::components::save_load::{Load, Save};
use common_actions::GlobalModals;
use components::{
    player_view::PlayerView,
    top_controls::{AddPlayer, MasterVolume, PausePlay, Stop},
    Orientation, Separator,
};
use freya::prelude::*;
use indexmap::{IndexMap, IndexSet};
use player_ref::PlayerRef;
use std::collections::HashMap;
use std::fmt::Debug;

/*
TODO: error handling
TODO: handle no path chosen
TODO: file extensions
TODO: grouping
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
    let config: LaunchConfig<'_, ()> = LaunchConfig::new()
        .with_title("Troubadour")
        .with_min_size(615.0, 800.0);
    launch_cfg(app, config);
}

fn app() -> Element {
    let state = use_signal(|| AppState::default());

    let state_lock = state.read();

    let theme = use_get_theme();

    rsx! {
        GlobalModals { state,
            rect {
                direction: "horizontal",
                width: "fill",
                spacing: "5",
                padding: "6",
                background: "{theme.colors.neutral_surface}",
                AddPlayer { state }
                PausePlay { state }
                Stop { state }
                Save { state }
                Load { state }
                MasterVolume { state }
            }
            Separator { orientation: Orientation::Horizontal }
            ScrollView { padding: "6",
                for (_ , p) in state_lock.players.clone() {
                    PlayerView { player: p, state }
                }
            }
        }
    }
}
