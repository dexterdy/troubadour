use crate::{
    common_actions::ShowError, AppState, StateChannel, GLOBAL_PAUSED, HAS_SAVED, MASTER_VOLUME,
};
use anyhow::Error;
use dioxus_radio::hooks::use_radio;
use freya::prelude::*;
use rfd::AsyncFileDialog;
use std::{collections::HashMap, path::PathBuf};
use troubadour_lib::player::Player;

#[component]
pub fn AddPlayer() -> Element {
    let mut state = use_radio::<AppState, StateChannel>(StateChannel::AddOrRemove);
    let mut path = use_signal::<Option<PathBuf>>(|| None);
    let mut show_name_dialogue = use_signal(|| false);
    let mut name = use_signal(|| "".to_string());
    let mut show_pick_file = use_signal(|| false);
    let mut show_error_popup = use_context::<UsePopup<Error, ShowError>>();
    let theme = use_get_theme();

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
        s.players.insert(name.clone(), new_player);
        s.top_group.insert(name.clone());
        *HAS_SAVED.write() = false;
    };

    rsx! {
        Button {
            onpress: pick_file,
            theme: theme_with!(ButtonTheme { padding : "4 8".into() }),
            svg {
                width: "20",
                height: "20",
                fill: "{theme.colors.solid}",
                svg_data: static_bytes(include_bytes!("../../icons/list-add-symbolic.svg")),
            }
        }
        if *show_name_dialogue.read() {
            Popup { show_close_button: false, close_on_escape_key: false,
                rect { padding: "20",
                    label {
                        font_size: "18",
                        font_weight: "bold",
                        margin: "0 0 15 0",
                        "What should this player be called?"
                    }
                    label { margin: "0 0 5 0", "Name:" }
                    rect { margin: "0 0 10 0",
                        Input {
                            width: "fill",
                            value: name.read().clone(),
                            onchange: move |e| { name.set(e) },
                        }
                    }
                    rect {
                        width: "fill",
                        main_align: "end",
                        direction: "horizontal",
                        Button { onpress: done,
                            label { "Done" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn PausePlay() -> Element {
    let mut prev_player_play_states: Signal<HashMap<String, bool>> = use_signal(|| HashMap::new());
    let mut show_error_popup = use_context::<UsePopup<Error, ShowError>>();
    let theme = use_get_theme();
    let mut state = use_radio::<AppState, StateChannel>(StateChannel::GlobalPaused);

    let get_prev_state = move || {
        state
            .read()
            .players
            .iter()
            .map(|(name, p)| (name.clone(), p.get_is_playing()))
            .collect::<HashMap<String, bool>>()
    };

    let mut pause = move || {
        let keys: Vec<String> = state.read().players.keys().cloned().collect();
        for id in keys {
            let mut player_channel_state =
                state.write_channel(StateChannel::SpecificPlayerPaused(id.clone()));
            let player = player_channel_state.players.get_mut(&id).unwrap();
            if player.get_is_playing() {
                player.pause();
            }
        }
        *GLOBAL_PAUSED.write() = true;
    };

    let mut play = move |prev: Signal<HashMap<String, bool>>| {
        let keys: Vec<String> = state.read().players.keys().cloned().collect();
        for id in keys {
            let result = if prev.read().get(&id) == Some(&true) {
                let mut player_channel_state =
                    state.write_channel(StateChannel::SpecificPlayerPaused(id.clone()));
                let player = player_channel_state.players.get_mut(&id).unwrap();
                player.play()
            } else {
                Ok(())
            };

            if let Err(e) = result {
                pause();
                spawn(async move {
                    show_error_popup.open(Some(e.into())).await;
                });
                return;
            }
        }
        *GLOBAL_PAUSED.write() = false;
    };

    let pause_or_play = move |_| {
        if !*GLOBAL_PAUSED.read() {
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
            if *GLOBAL_PAUSED.read() {
                svg {
                    width: "20",
                    height: "20",
                    fill: "{theme.colors.solid}",
                    svg_data: static_bytes(include_bytes!("../../icons/media-playback-start-symbolic.svg")),
                }
            } else {
                svg {
                    width: "20",
                    height: "20",
                    fill: "{theme.colors.solid}",
                    svg_data: static_bytes(include_bytes!("../../icons/media-playback-pause-symbolic.svg")),
                }
            }
        }
    }
}

#[component]
pub fn Stop() -> Element {
    let theme = use_get_theme();
    let mut state = use_radio::<AppState, StateChannel>(StateChannel::NoUpdate);

    let stop = move |_| {
        let keys: Vec<String> = state.read().players.keys().cloned().collect();
        for id in keys {
            let mut player_channel_state =
                state.write_channel(StateChannel::SpecificPlayerPaused(id.clone()));
            let player = player_channel_state.players.get_mut(&id).unwrap();
            player.stop();
        }
        *GLOBAL_PAUSED.write() = false;
    };

    rsx! {
        Button {
            onpress: stop,
            theme: theme_with!(ButtonTheme { padding : "4 8".into() }),
            svg {
                width: "20",
                height: "20",
                fill: "{theme.colors.solid}",
                svg_data: static_bytes(include_bytes!("../../icons/media-playback-stop-symbolic.svg")),
            }
        }
    }
}

#[component]
pub fn MasterVolume() -> Element {
    let mut state = use_radio::<AppState, StateChannel>(StateChannel::NoUpdate);
    let mut master_volume_slider = use_signal(|| 50.0);

    let set_master_volume = move |new_master_volume| {
        master_volume_slider.set(new_master_volume);
        let new_master_volume = (new_master_volume * 0.02) as f32;
        *MASTER_VOLUME.write() = new_master_volume;

        let keys: Vec<String> = state.read().players.keys().cloned().collect();
        for id in keys {
            let mut player_channel_state =
                state.write_channel(StateChannel::SpecificPlayer(id.clone()));
            let player = player_channel_state.players.get_mut(&id).unwrap();
            let player_volume = player.volume;
            player.volume(player_volume, new_master_volume);
        }
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
                value: *master_volume_slider.read(),
                onmoved: set_master_volume,
            }
        }
    }
}
