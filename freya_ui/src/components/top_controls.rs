use crate::{
    common_actions::{save, UnsavedModalResult},
    components::SplitButton,
    player_ref::PlayerRef,
    AppState,
};
use freya::prelude::*;
use rfd::AsyncFileDialog;
use std::{collections::HashMap, path::PathBuf};
use troubadour_lib::{load, player::Player};

#[component]
pub fn AddPlayer(state: Signal<AppState>) -> Element {
    let mut path = use_signal::<Option<PathBuf>>(|| None);
    let mut show_name_dialogue = use_signal(|| false);
    let mut name = use_signal(|| "".to_string());
    let mut show_pick_file = use_signal(|| false);

    let pick_file = move |_| {
        if !*show_pick_file.read() {
            show_pick_file.set(true);
            spawn(async move {
                let file = AsyncFileDialog::new().pick_file().await.unwrap();
                path.set(Some(file.path().to_path_buf()));
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
        let path = path.read().clone();
        // TODO
        // if path.is_none() {
        //     return Err(Error::msg("error: no path selected"));
        // }
        // if s.players.contains_key(&name) {
        //     return Err(Error::msg(format!(
        //         "error: you cannot use the name '{name}', because it is already used."
        //     )));
        // }
        let new_player = Player::new(path.unwrap(), name.clone()).unwrap();
        s.players.insert(name.clone(), PlayerRef::new(new_player));
        s.top_group.insert(name.clone());
        s.saved = false;
    };

    rsx! {
        Button {
            onclick: pick_file,
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
        Button {
            onclick: pause_or_play,
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
            onclick: stop,
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
pub fn Save(state: Signal<AppState>) -> Element {
    let save = move |_| {
        spawn(async move {
            let _ = save(state).await;
        });
    };

    rsx! {
        Button {
            onclick: save,
            theme: theme_with!(ButtonTheme { padding : "4 8".into() }),
            svg {
                width: "20",
                height: "20",
                svg_data: static_bytes(include_bytes!("../../icons/save-symbolic.svg")),
            }
        }
    }
}

#[component]
pub fn Load(state: Signal<AppState>) -> Element {
    let mut unsaved_modal = use_context::<UsePopup<(), UnsavedModalResult>>();
    let mut name_conflict_popup = use_popup::<(String, String), NameResolution>();

    let load = async move || {
        let p = AsyncFileDialog::new().pick_file().await.unwrap();
        return load(p.path());
    };

    let replace_load = move || {
        spawn(async move {
            let new_state = load().await.unwrap();
            let mut s = state.write();
            s.players = new_state
                .0
                .into_iter()
                .map(|(n, p)| (n, PlayerRef::new(p)))
                .collect();
            s.top_group = new_state.1;
            s.groups = new_state.2;
        });
    };

    let merge_load = move || {
        spawn(async move {
            let mut state = state.write();
            let mut new_state = load().await.unwrap();

            for (n, mut p) in new_state.0 {
                if state.players.contains_key(&n) {
                    match name_conflict_popup
                        .open(("A player".to_string(), n.clone()))
                        .await
                        .clone()
                        .unwrap()
                    {
                        NameResolution::Skip => {
                            new_state.1.shift_remove(&n);
                            for (_, g) in &mut new_state.2 {
                                g.shift_remove(&n);
                            }
                        }
                        NameResolution::Replace => {
                            state.players.insert(n.clone(), PlayerRef::new(p));
                            state.top_group.shift_remove(&n);
                            for (_, g) in &mut state.groups {
                                g.shift_remove(&n);
                            }
                        }
                        NameResolution::Rename(new_name) => {
                            p.name = new_name.clone();
                            if new_state.1.contains(&n) {
                                // puts new_name in same position as old_name was
                                new_state.1.insert(new_name.clone());
                                new_state.1.swap_remove(&n);
                            }
                            for (_, g) in &mut new_state.2 {
                                g.insert(new_name.clone());
                                g.swap_remove(&n);
                            }
                            state.players.insert(new_name, PlayerRef::new(p));
                        }
                    }
                } else {
                    state.players.insert(n, PlayerRef::new(p));
                }
            }
            state.top_group.append(&mut new_state.1);

            for (n, g) in new_state.2 {
                if state.players.contains_key(&n) {
                    match name_conflict_popup
                        .open(("A group".to_string(), n.clone()))
                        .await
                        .clone()
                        .unwrap()
                    {
                        NameResolution::Skip => {}
                        NameResolution::Replace => {
                            state.groups.insert(n, g);
                        }
                        NameResolution::Rename(new_name) => {
                            state.groups.insert(new_name, g);
                        }
                    }
                } else {
                    state.groups.insert(n, g);
                }
            }
        });
    };

    let load_callback = move |mut inner: Box<dyn FnMut()>| {
        spawn(async move {
            if !state.read().saved {
                let res = unsaved_modal.open(()).await;
                let res = res.as_ref().unwrap();
                if *res != UnsavedModalResult::Cancelled {
                    inner()
                }
            } else {
                inner()
            }
        });
    };

    rsx! {
        SplitButton {
            onpress: move |_| load_callback(Box::new(replace_load)),
            options: vec![
                (
                    EventHandler::new(move |_| load_callback(Box::new(merge_load))),
                    rsx! {
                        label { "merge with soundscape" }
                    },
                ),
            ],
            label { "load" }
        }

        if name_conflict_popup.is_open() {
            name_conflict_modal {}
        }
    }
}

#[derive(Clone, Debug)]
enum NameResolution {
    Skip,
    Replace,
    Rename(String),
}

#[component]
fn name_conflict_modal() -> Element {
    let mut name_input = use_signal(String::new);
    let mut popup_answer = use_popup_answer::<(String, String), NameResolution>();
    let (thing, name) = popup_answer.data().clone().unwrap();

    rsx! {
        Popup { close_on_escape_key: false, show_close_button: false,
            PopupTitle {
                label {
                    "{thing} with the name {name} already exists. How do you want to resolve the conflict?"
                }
            }
            PopupContent {
                label { "New Name:" }
                Input {
                    value: name_input,
                    onchange: move |e| {
                        name_input.set(e);
                    },
                }
                Button {
                    onpress: move |_| {
                        popup_answer.answer(NameResolution::Skip);
                    },
                    label { "Skip" }
                }
                Button {
                    onpress: move |_| {
                        popup_answer.answer(NameResolution::Replace);
                    },
                    label { "Replace" }
                }
                Button {
                    onpress: move |_| {
                        popup_answer.answer(NameResolution::Rename(name_input.read().clone()));
                    },
                    label { "Ok" }
                }
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
