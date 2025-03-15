use std::{cell::RefCell, collections::HashMap, path::PathBuf, rc::Rc};

use anyhow::Error;
use freya::prelude::*;
use indexmap::{IndexMap, IndexSet};
use rfd::AsyncFileDialog;
use troubadour_lib::player::Player;

#[derive(Clone)]
struct PlayerRef {
    inner: Rc<RefCell<Player>>,
}

impl PlayerRef {
    fn new(player: Player) -> Self {
        PlayerRef {
            inner: Rc::new(RefCell::new(player)),
        }
    }
}

impl PartialEq for PlayerRef {
    fn eq(&self, other: &Self) -> bool {
        let sb = self.inner.borrow();
        let ob = other.inner.borrow();
        sb.name == ob.name
            && sb.base_length == ob.base_length
            && sb.group == ob.group
            && sb.get_is_playing() == ob.get_is_playing()
            && sb.get_is_paused() == ob.get_is_paused()
            && sb.volume == ob.volume
            && sb.looping == ob.looping
            && sb.loop_gap == ob.loop_gap
            && sb.delay_length == ob.delay_length
            && sb.cut_end == ob.cut_end
            && sb.cut_start == ob.cut_start
    }
}

#[derive(Default)]
struct AppState {
    pub players: HashMap<String, PlayerRef>,
    pub top_group: IndexSet<String>,
    pub groups: IndexMap<String, IndexSet<String>>,
}

fn main() {
    launch(app);
}

fn app() -> Element {
    let state = use_signal(|| AppState::default());

    let state_lock = state.read();

    rsx! {
        AddPlayer { state }
        for (_ , p) in state_lock.players.clone() {
            PlayerComponent { player: p }
        }
    }
}

#[component]
fn AddPlayer(state: Signal<AppState>) -> Element {
    let mut path = use_signal::<Option<PathBuf>>(|| None);
    let mut show_name_dialogue = use_signal(|| false);
    let mut name = use_signal(|| "".to_string());

    let pick_file = move |_| {
        spawn(async move {
            let file = AsyncFileDialog::new().pick_file().await;
            path.set(file.map(|f| f.path().to_path_buf()));
            if path.read().is_some() {
                show_name_dialogue.set(true);
            }
        });
    };

    let done = move |_| {
        show_name_dialogue.set(false);
        let _ = state.with_mut(|s| {
            let name = name.read().clone();
            let path = path.read().clone();
            if path.is_none() {
                return Err(Error::msg("error: no path selected"));
            }
            if s.players.contains_key(&name) {
                return Err(Error::msg(format!(
                    "error: you cannot use the name '{name}', because it is already used."
                )));
            }
            let new_player = Player::new(path.unwrap(), name.clone())?;
            s.players.insert(name.clone(), PlayerRef::new(new_player));
            s.top_group.insert(name.clone());
            Ok(())
        });
    };

    rsx! {
        Button { onclick: pick_file,
            label { "Add" }
        }
        if *show_name_dialogue.read() {
            Popup { oncloserequest: move |_| { show_name_dialogue.set(false) },
                PopupTitle {
                    label { "What should this player be called?" }
                }
                PopupContent {
                    label { "Name:" }
                    Input {
                        value: name.read().clone(),
                        onchange: move |e| { name.set(e) },
                    }
                    Button { onclick: done,
                        label { "Done" }
                    }
                }
            }
        }
    }
}

#[component]
fn PlayerComponent(player: PlayerRef) -> Element {
    let player_borrow = player.inner.borrow();
    rsx! {
        label { "{player_borrow.name}" }
    }
}
