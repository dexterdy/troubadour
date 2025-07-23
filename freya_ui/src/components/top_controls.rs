use crate::{common_actions::ShowError, player_ref::PlayerRef, AppState};
use anyhow::Error;
use freya::prelude::*;
use rfd::AsyncFileDialog;
use std::cell::Cell;
use std::{collections::HashMap, path::PathBuf};
use troubadour_lib::player::Player;

#[component]
pub fn AddPlayer(state: Signal<AppState>) -> Element {
    let mut path = use_signal::<Option<PathBuf>>(|| None);
    let mut show_name_dialogue = use_signal(|| false);
    let mut name = use_signal(|| "".to_string());
    let mut show_pick_file = use_signal(|| false);
    let mut show_error_popup = use_context::<UsePopup<Error, ShowError>>();

    let pick_file = move |_| {
        if !*show_pick_file.read() {
            show_pick_file.set(true);
            spawn(async move {
                AsyncFileDialog::new()
                    .add_filter("sound", &["flac", "mp3", "mp4", "ogg", "wav", "aac", "pcm"])
                    .pick_file()
                    .await
                    .map(|file| path.set(Some(file.path().to_path_buf())));
                if path.read().is_some() {
                    show_name_dialogue.set(true);
                }
                show_pick_file.set(false);
            });
        }
    };

    let done = move |_| {
        show_name_dialogue.set(false);
        let mut s = state.write();
        let name = name.read().clone();
        let path = path.read().clone().expect("path doesn't exist");
        if s.players.contains_key(&name) {
            spawn(async move {
                let error_string =
                    format!("error: you cannot use the name '{name}', because it is already used.");
                show_error_popup.open(Some(Error::msg(error_string))).await;
            });
            return;
        }
        let new_player = Player::new(path, name.clone()).unwrap();
        s.players.insert(name.clone(), PlayerRef::new(new_player));
        s.top_group.insert(name.clone());
        s.saved = false;
    };

    rsx! {
        Button {
            onpress: pick_file,
            theme: theme_with!(ButtonTheme { padding : "4 8".into() }),
            svg {
                width: "20",
                height: "20",
                svg_data: static_bytes(include_bytes!("../../icons/list-add-symbolic.svg")),
            }
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
                    Button { onpress: done,
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
    let mut show_error_popup = use_context::<UsePopup<Error, ShowError>>();

    let get_prev_state = move || {
        state
            .read()
            .players
            .iter()
            .map(|(name, p)| (name.clone(), p.read().get_is_playing()))
            .collect::<HashMap<String, bool>>()
    };

    let mut pause = move || {
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
    };

    let mut play = move |prev: Signal<HashMap<String, bool>>| {
        state.with_mut(|s| {
            for (n, p) in &s.players {
                let error = Cell::new(None);
                p.with_mut(|p| {
                    if let Some(true) = prev.read().get(n) {
                        if let Err(e) = p.play() {
                            error.set(Some(e));
                        }
                    }
                });
                if let Some(e) = error.take() {
                    pause();
                    spawn(async move {
                        show_error_popup.open(Some(e.into())).await;
                    });
                    break;
                }
            }
            s.global_paused = false;
        })
    };

    let pause_or_play = move |_| {
        if !state.read().global_paused {
            prev_player_play_states.set(get_prev_state());
            pause()
        } else {
            play(prev_player_play_states)
        }
    };

    rsx! {
        Button {
            onpress: pause_or_play,
            theme: theme_with!(ButtonTheme { padding : "4 8".into() }),
            if state.read().global_paused {
                svg {
                    width: "20",
                    height: "20",
                    svg_data: static_bytes(include_bytes!("../../icons/media-playback-start-symbolic.svg")),
                }
            } else {
                svg {
                    width: "20",
                    height: "20",
                    svg_data: static_bytes(include_bytes!("../../icons/media-playback-pause-symbolic.svg")),
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
        Button {
            onpress: stop,
            theme: theme_with!(ButtonTheme { padding : "4 8".into() }),
            svg {
                width: "20",
                height: "20",
                svg_data: static_bytes(include_bytes!("../../icons/media-playback-stop-symbolic.svg")),
            }
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
            min_width: "150",
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
