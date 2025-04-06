use crate::{components::prelude::SplitButton, player_ref::PlayerRef, AppState};
use anyhow::Error;
use freya::prelude::*;
use rfd::FileDialog;
use std::{collections::HashMap, path::PathBuf};
use troubadour_lib::{player::Player, save};

#[component]
pub fn AddPlayer(state: Signal<AppState>) -> Element {
    let mut path = use_signal::<Option<PathBuf>>(|| None);
    let mut show_name_dialogue = use_signal(|| false);
    let mut name = use_signal(|| "".to_string());
    let mut show_pick_file = use_signal(|| false);

    let pick_file = move |_| {
        if !*show_pick_file.read() {
            show_pick_file.toggle();
            spawn(async move {
                let file = FileDialog::new().pick_file();
                path.set(file);
                if path.read().is_some() {
                    show_name_dialogue.set(true);
                }
                show_pick_file.toggle();
            });
        }
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
pub fn PausePlay(state: Signal<AppState>) -> Element {
    let mut prev_player_play_states: Signal<HashMap<String, bool>> = use_signal(|| HashMap::new());
    let get_prev_state = move || {
        state
            .read()
            .players
            .iter()
            .map(|(name, p)| (name.clone(), p.read().get_is_playing()))
            .collect::<HashMap<String, bool>>()
    };
    let pause_or_play = move |_| {
        if !state.read().global_paused {
            prev_player_play_states.set(get_prev_state());
            state.with_mut(|s| {
                for (_, p) in &s.players {
                    p.with_mut(|p| {
                        if p.get_is_playing() {
                            p.pause();
                        }
                    });
                }
                s.global_paused = true;
            })
        } else {
            let prev = prev_player_play_states.read();
            state.with_mut(|s| {
                for (n, p) in &s.players {
                    p.with_mut(|p| {
                        if let Some(true) = prev.get(n) {
                            let _ = p.play();
                        }
                    });
                }
                s.global_paused = false;
            })
        }
    };

    rsx! {
        Button { onclick: pause_or_play,
            label {
                if state.read().global_paused {
                    "Play"
                } else {
                    "Pause"
                }
            }
        }
    }
}

#[component]
pub fn Stop(state: Signal<AppState>) -> Element {
    let stop = move |_| {
        state.with_mut(|s| {
            for (_, p) in &s.players {
                p.with_mut(|p| {
                    p.stop();
                });
            }
        })
    };

    rsx! {
        Button { onclick: stop,
            label { "Stop" }
        }
    }
}

#[component]
pub fn Save(state: Signal<AppState>) -> Element {
    let save = move |_| {
        spawn(async move {
            let file = FileDialog::new().save_file();
            if let Some(path) = file {
                let s = state.read();
                let _ = save(
                    s.players
                        .iter()
                        .map(|(n, p)| (n.clone(), p.clone()))
                        .collect(),
                    &s.top_group,
                    &s.groups,
                    &path,
                );
            }
        });
    };

    rsx! {
        Button { onclick: save,
            label { "Save" }
        }
    }
}

#[component]
pub fn Load(state: Signal<AppState>) -> Element {
    let load = move |_| {};
    let merge_load = move |_: ()| {};

    rsx! {
        SplitButton {
            onpress: load,
            options: vec![
                (
                    EventHandler::new(merge_load),
                    rsx! {
                        label { "add to soundscape" }
                    },
                ),
                (
                    EventHandler::new(merge_load),
                    rsx! {
                        label { "add to soundscape" }
                    },
                ),
                (
                    EventHandler::new(merge_load),
                    rsx! {
                        label { "add to soundscape" }
                    },
                ),
            ],
            label { "load" }
        }
    }
}

#[component]
pub fn MasterVolume(state: Signal<AppState>) -> Element {
    let mut master_volume = use_signal(|| 50.0);

    let set_master_volume = move |new_master_volume| {
        master_volume.set(new_master_volume);
        let new_master_volume = (new_master_volume * 0.02) as f32;
        state.with_mut(|s| {
            s.master_volume = new_master_volume;
            for (_, p) in &s.players {
                p.with_mut(|p| {
                    let player_volume = p.volume;
                    p.volume(player_volume, new_master_volume);
                });
            }
        });
    };

    rsx! {
        rect {
            width: "fill",
            height: "29",
            main_align: "center",
            cross_align: "end",
            Slider {
                size: "150",
                value: *master_volume.read(),
                onmoved: set_master_volume,
            }
        }
    }
}
