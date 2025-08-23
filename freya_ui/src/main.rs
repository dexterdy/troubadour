mod common_actions;
mod components;
mod player_ref;

use crate::components::player_view::EditPlayerPanel;
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
TODO: file extensions (save files)
TODO: grouping
TODO: popup layout
TODO: drag and drop
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
    pub selected_player: Option<PlayerRef>,
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
            selected_player: None,
        }
    }
}

fn main() {
    launch_cfg(
        LaunchConfig::new().with_window(
            WindowConfig::new(app)
                .with_title("Troubadour")
                .with_min_size(615.0, 800.0),
        ),
    );
}

fn get_theme(preferred_theme: PreferredTheme) -> Theme {
    match preferred_theme {
        PreferredTheme::Dark => DARK_THEME,
        PreferredTheme::Light => LIGHT_THEME,
    }
}

fn app() -> Element {
    let state = use_signal(|| AppState::default());
    let preferred_theme = use_preferred_theme();
    let mut current_theme = use_init_theme(|| get_theme(*preferred_theme.peek()));

    use_effect(move || {
        let theme = get_theme(preferred_theme());
        if theme != *current_theme.peek() {
            current_theme.set(theme);
        }
    });

    rsx! {
        Body {
            GlobalModals { state,
                rect { height: "100v", content: "flex",
                    rect {
                        direction: "horizontal",
                        width: "fill",
                        spacing: "5",
                        padding: "6",
                        background: "{current_theme.read().colors.neutral_surface}",
                        AddPlayer { state }
                        PausePlay { state }
                        Stop { state }
                        Save { state }
                        Load { state }
                        MasterVolume { state }
                    }
                    Separator { orientation: Orientation::Horizontal }
                    ScrollView {
                        padding: "6",
                        height: "flex(1)",
                        rect {
                            spacing: "10",
                            for (_ , p) in state.read().players.clone() {
                                PlayerView { player: p, state }
                            }
                        }
                    }
                    if let Some(player) = &state.read().selected_player {
                        Separator { orientation: Orientation::Horizontal }
                        EditPlayerPanel { player: player.clone(), state }
                    }
                }
            }
        }
    }
}
