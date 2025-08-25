mod common_actions;
mod components;

use crate::components::player_view::EditPlayerPanel;
use crate::components::save_load::{Load, Save};
use crate::components::use_polling;
use common_actions::GlobalModals;
use components::{
    player_view::PlayerView,
    top_controls::{AddPlayer, MasterVolume, PausePlay, Stop},
    Orientation, Separator,
};
use dark_light;
use dioxus_radio::prelude::{use_init_radio_station, use_radio, RadioChannel};
use freya::prelude::reexports::winit;
use freya::prelude::*;
use indexmap::{IndexMap, IndexSet};
use std::collections::HashMap;
use std::fmt::Debug;
use std::time::Duration;
use troubadour_lib::player::Player;

/*
TODO: player context menu (remove, add to group, cross fade, etc)
TODO: exit unsaved changes
TODO: loader when loading save

TODO: drag and drop
TODO: grouping
TODO: group context menu (remove, remove and remove players, combine with, cross fade, etc)
TODO: cross fade
TODO: animate volume
TODO: file drop
TODO: progress indicator
TODO: file extensions (save files)
TODO: panning
TODO: EQ

You can cross fade from a context menu, in which case a cross fade of a default shape and length happens.
You can also create and save cross fades. You can select the shape and length of the cross fade.
This cross fade is then displayed as a button underneath the player/group
*/

type PlayerId = String;
type GroupID = String;

static MASTER_VOLUME: GlobalSignal<f32> = Signal::global(|| 1.0);
static GLOBAL_PAUSED: GlobalSignal<bool> = Signal::global(|| false);
static HAS_SAVED: GlobalSignal<bool> = Signal::global(|| true);
static SELECTED_PLAYER: GlobalSignal<Option<PlayerId>> = Signal::global(|| None);

#[derive(Debug)]
struct AppState {
    pub players: HashMap<PlayerId, Player>,
    pub top_group: IndexSet<PlayerId>,
    pub groups: IndexMap<GroupID, IndexSet<PlayerId>>,
}

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub enum StateChannel {
    NoUpdate,
    AddOrRemove,
    GlobalPaused,
    SpecificPlayer(PlayerId),
    SpecificPlayerPaused(PlayerId),
}

impl RadioChannel<AppState> for StateChannel {
    fn derive_channel(self, _radio: &AppState) -> Vec<Self> {
        match self.clone() {
            StateChannel::SpecificPlayerPaused(id) => {
                vec![
                    self,
                    StateChannel::SpecificPlayer(id),
                    StateChannel::GlobalPaused,
                ]
            }
            _ => vec![self],
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            players: HashMap::new(),
            top_group: IndexSet::new(),
            groups: IndexMap::new(),
        }
    }
}

fn main() {
    launch_cfg(LaunchConfig::new().with_window(WindowConfig::new(app).with_title("Troubadour")));
}

fn get_theme(preferred_theme: dark_light::Mode) -> Theme {
    match preferred_theme {
        dark_light::Mode::Dark => DARK_THEME,
        dark_light::Mode::Light => LIGHT_THEME,
        _ => LIGHT_THEME,
    }
}

fn get_winit_theme(preferred_theme: dark_light::Mode) -> winit::window::Theme {
    match preferred_theme {
        dark_light::Mode::Dark => winit::window::Theme::Dark,
        dark_light::Mode::Light => winit::window::Theme::Light,
        _ => winit::window::Theme::Light,
    }
}

fn app() -> Element {
    use_init_radio_station::<AppState, StateChannel>(AppState::default);
    let state = use_radio::<AppState, StateChannel>(StateChannel::AddOrRemove);
    let preferred_theme = use_polling(
        || dark_light::detect().unwrap_or(dark_light::Mode::Unspecified),
        dark_light::detect().unwrap_or(dark_light::Mode::Unspecified),
        Duration::from_millis(1000),
    );
    let mut current_theme = use_init_theme(|| get_theme(*preferred_theme.peek()));
    let platform = use_platform();

    use_hook(|| {
        let winit_theme = get_winit_theme(*preferred_theme.peek());
        platform.with_window(move |w| w.set_theme(Some(winit_theme)));
    });
    use_effect(move || {
        let theme = get_theme(*preferred_theme.read());
        if theme != *current_theme.peek() {
            current_theme.set(theme);
            let winit_theme = get_winit_theme(*preferred_theme.peek());
            platform.with_window(move |w| w.set_theme(Some(winit_theme)));
        }
    });

    rsx! {
        Body {
            GlobalModals {
                rect { height: "100v", content: "flex",
                    rect {
                        direction: "horizontal",
                        width: "fill",
                        spacing: "5",
                        padding: "6",
                        background: "{current_theme.read().colors.neutral_surface}",
                        AddPlayer {}
                        PausePlay {}
                        Stop {}
                        Save {}
                        Load {}
                        MasterVolume {}
                    }
                    Separator { orientation: Orientation::Horizontal }
                    ScrollView { padding: "6", height: "flex(1)",
                        rect {
                            width: "fill",
                            direction: "horizontal",
                            wrap_content: "wrap",
                            spacing: "10",
                            for player_id in state.read().top_group.clone() {
                                PlayerView { player_id }
                            }
                        }
                    }
                    if let Some(player_id) = SELECTED_PLAYER.read().clone() {
                        Separator { orientation: Orientation::Horizontal }
                        EditPlayerPanel { player_id: player_id.clone() }
                    }
                }
            }
        }
    }
}
