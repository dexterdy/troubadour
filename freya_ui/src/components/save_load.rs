use std::mem;
use crate::{common_actions::{save, ShowError, Unsaved}, components::SplitButton, AppState, StateChannel, GLOBAL_PAUSED, HAS_SAVED, MASTER_VOLUME, SELECTED_PLAYER};
use anyhow::Error;
use dioxus_radio::hooks::{use_radio, use_radio_station};
use freya::prelude::*;
use rfd::AsyncFileDialog;
use troubadour_lib::{load, player::Player, SaveState};

#[component]
pub fn Save() -> Element {
    let theme = use_get_theme();
    let state =use_radio_station::<AppState, StateChannel>();
    let mut show_error_popup = use_context::<UsePopup<Error, ShowError>>();

    let save = move |_| {
        spawn(async move {
            if let Err(e) = save(&*state.read()).await {
                show_error_popup.open(Some(e)).await;
            }
        });
    };

    rsx! {
        Button {
            onpress: save,
            theme: theme_with!(ButtonTheme { padding : "4 8".into() }),
            svg {
                width: "20",
                height: "20",
                fill: "{theme.colors.solid}",
                svg_data: static_bytes(include_bytes!("../../icons/save-symbolic.svg")),
            }
        }
    }
}

#[component]
pub fn Load() -> Element {
    let unsaved_modal = use_context::<UsePopup<(), Unsaved>>();
    let name_conflict_popup = use_popup::<(String, String), NameResolution>();
    let show_error_popup = use_context::<UsePopup<Error, ShowError>>();
    let mut state = use_radio::<AppState, StateChannel>(StateChannel::AddOrRemove);

    let replace_load = move || {
        spawn(async move {
            if let Some(new_state) = load_file_and_handle_errors(show_error_popup).await {
                replace_load(&mut *state.write(), new_state);
            }
        });
    };

    let merge_load = move || {
        spawn(async move {
            if let Some(new_state) = load_file_and_handle_errors(show_error_popup).await {
                merge_load(&mut *state.write(), new_state, name_conflict_popup).await;
            }
        });
    };

    rsx! {
        SplitButton {
            onpress: move |_| handle_unsaved_changes(unsaved_modal, Box::new(replace_load)),
            left_button: rsx! {
                label { "load" }
            },
            MenuButton { onpress: move |_| handle_unsaved_changes(unsaved_modal, Box::new(merge_load)),
                label { "merge with soundscape" }
            }
        }

        if name_conflict_popup.is_open() {
            name_conflict_modal {}
        }
    }
}

/// Opens a file dialog, loads state from the selected file, and handles potential errors.
async fn load_file_and_handle_errors(
    mut show_error_popup: UsePopup<Error, ShowError>,
) -> Option<SaveState<Player>> {
    let load_result = async {
        let file = AsyncFileDialog::new().pick_file().await;
        file.map(|f| load(f.path())).transpose()
    }
    .await;

    match load_result {
        Ok(Some(data)) => Some(data),
        Ok(None) => None, // User cancelled the file dialog.
        Err(e) => {
            // Show error popup on failure.
            show_error_popup.open(Some(e.into())).await;
            None
        }
    }
}

/// Replaces current state with saved state
fn replace_load(state: &mut AppState, new_state: SaveState<Player>) {
    state.players = new_state
        .players
        .into_iter()
        .map(|(n, p)| (n, p))
        .collect();
    state.top_group = new_state.top_group;
    state.groups = new_state.groups;
    
    *HAS_SAVED.write() = false;
    *MASTER_VOLUME.write() = 1.0;
    *GLOBAL_PAUSED.write() = false;
    *SELECTED_PLAYER.write() = None;
}

/// Merges new players and groups into the existing state, handling any name conflicts.
async fn merge_load(
    state: &mut AppState,
    mut new_state: SaveState<Player>,
    mut name_conflict_popup: UsePopup<(String, String), NameResolution>,
) {
    // Take ownership of players to iterate over them while mutably borrowing the rest of new_state.
    let players_to_merge = mem::take(&mut new_state.players);

    for (n, mut p) in players_to_merge {
        if state.players.contains_key(&n) {
            let resolution = name_conflict_popup
                .open(("A player".to_string(), n.clone()))
                .await
                .clone()
                .unwrap();

            match resolution {
                NameResolution::Skip => {
                    new_state.top_group.shift_remove(&n);
                    for (_, g) in new_state.groups.iter_mut() {
                        g.shift_remove(&n);
                    }
                }
                NameResolution::Replace => {
                    state.players.insert(n.clone(), p);
                    state.top_group.shift_remove(&n);
                    for (_, g) in state.groups.iter_mut() {
                        g.shift_remove(&n);
                    }
                }
                NameResolution::Rename(new_name) => {
                    p.name = new_name.clone();

                    if new_state.top_group.contains(&n) {
                        new_state.top_group.insert(new_name.clone());
                        new_state.top_group.swap_remove(&n);
                    }

                    for (_, g) in new_state.groups.iter_mut() {
                        g.insert(new_name.clone());
                        g.swap_remove(&n);
                    }

                    state.players.insert(new_name, p);
                }
            }
        } else {
            state.players.insert(n, p);
        }
    }

    state.top_group.append(&mut new_state.top_group);

    for (n, g) in new_state.groups {
        if state.groups.contains_key(&n) {
            let resolution = name_conflict_popup
                .open(("A group".to_string(), n.clone()))
                .await
                .clone()
                .unwrap();

            match resolution {
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
    
    *HAS_SAVED.write() = false;
}

/// Checks for unsaved changes before executing a given action.
fn handle_unsaved_changes<F>(
    mut unsaved_modal: UsePopup<(), Unsaved>,
    mut inner: F,
) where
    F: FnMut() + 'static,
{
    spawn(async move {
        if !*HAS_SAVED.read() {
            let res = unsaved_modal.open(()).await;
            // Replicating original logic, which proceeds unless explicitly cancelled.
            if *res.as_ref().unwrap() != Unsaved::Cancelled {
                inner();
            }
        } else {
            inner();
        }
    });
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
    let (thing, name) = popup_answer.data().read().clone();

    rsx! {
        Popup { close_on_escape_key: false, show_close_button: false,
            PopupTitle { text: "{thing} with the name {name} already exists. How do you want to resolve the conflict?" }
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
