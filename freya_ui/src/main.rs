mod common_actions;
mod components;

use crate::common_actions::Unsaved;
use crate::components::player_view::EditPlayerPanel;
use crate::components::save_load::{Load, Save};
use common_actions::use_polling;
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
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::LazyLock;
use std::time::Duration;
use tokio::sync::Notify;
use troubadour_lib::player::Player;

/*
TODO: drag and drop
TODO: grouping
TODO: file drop
TODO: progress indicator
TODO: file extensions (save files)

TODO: Effect panel
TODO: cross fade
TODO: animate volume
TODO: panning
TODO: EQ
TODO: ctrl+z
TODO: keyboard shortcuts
FIXME: icons in buttons shift up in small window
*/

type PlayerId = String;
type GroupID = String;

static MASTER_VOLUME: GlobalSignal<f32> = Signal::global(|| 1.0);
static GLOBAL_PAUSED: GlobalSignal<bool> = Signal::global(|| false);
static HAS_SAVED: GlobalSignal<bool> = Signal::global(|| true);
static SELECTED_PLAYER: GlobalSignal<Option<PlayerId>> = Signal::global(|| None);
static CLOSING_NOTIFY: LazyLock<Notify> = LazyLock::new(|| Notify::new());
static CLOSING_APPROVED: AtomicBool = AtomicBool::new(false);

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
    launch_cfg(
        LaunchConfig::new().with_window(WindowConfig::new(app).with_title("Troubadour").on_close(
            |_| {
                if !CLOSING_APPROVED.load(Ordering::Relaxed) {
                    CLOSING_NOTIFY.notify_one();
                    OnCloseResponse::NotClose
                } else {
                    OnCloseResponse::Close
                }
            },
        )),
    );
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
                CloseHandler {}
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
                                PlayerView {
                                    key: player_id,
                                    player_id: player_id.clone(),
                                }
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

#[component]
fn CloseHandler() -> Element {
    let mut unsaved_modal = use_context::<UsePopup<(), Unsaved>>();
    let platform = use_platform();

    use_future(move || async move {
        loop {
            CLOSING_NOTIFY.notified().await;
            if !*HAS_SAVED.read() {
                let result = unsaved_modal.open(()).await;
                match result.as_ref().unwrap() {
                    Unsaved::Saved | Unsaved::NotSaved => {
                        CLOSING_APPROVED.store(true, Ordering::Relaxed);
                        platform.close_window();
                    }
                    Unsaved::Cancelled => { /*do nothing*/ }
                }
            } else {
                CLOSING_APPROVED.store(true, Ordering::Relaxed);
                platform.close_window();
            }
        }
    });

    rsx! {}
}
