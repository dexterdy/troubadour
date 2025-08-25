use anyhow::Error;
use dioxus_radio::prelude::use_radio_station;
use freya::prelude::*;
use rfd::AsyncFileDialog;
use troubadour_lib::SaveState;

use crate::{AppState, StateChannel, HAS_SAVED};

#[derive(PartialEq)]
pub enum Unsaved {
    Saved,
    NotSaved,
    Cancelled,
}

#[derive(PartialEq)]
pub struct ShowError();

#[component]
pub fn GlobalModals(children: Element) -> Element {
    let unsaved_popup = use_popup::<(), Unsaved>();
    use_context_provider(|| unsaved_popup);

    let show_error_popup = use_popup::<Error, ShowError>();
    use_context_provider(|| show_error_popup);

    rsx! {
        if unsaved_popup.is_open() {
            UnsavedModal {}
        }
        if show_error_popup.is_open() {
            ShowErrorModal {}
        }
        {children}
    }
}

#[component]
fn UnsavedModal() -> Element {
    let mut unsaved_answer = use_popup_answer::<(), Unsaved>();
    let mut show_file_pick = use_signal(|| false);
    let state = use_radio_station::<AppState, StateChannel>();
    let theme = use_get_theme();

    let save = move |_| {
        if !*show_file_pick.read() {
            show_file_pick.set(true);
            spawn(async move {
                let _ = save(&*state.read()).await;
                show_file_pick.set(false);
                unsaved_answer.answer(Unsaved::Saved);
            });
        }
    };

    let cancel = move |_| {
        unsaved_answer.answer(Unsaved::Cancelled);
    };

    let not_save = move |_| {
        unsaved_answer.answer(Unsaved::NotSaved);
    };

    rsx! {
        Popup { show_close_button: false, close_on_escape_key: false,
            rect { padding: "20",
                label {
                    font_size: "18",
                    font_weight: "bold",
                    margin: "0 0 15 0",
                    "You have unsaved changes. Do you want to save?"
                }
                rect {
                    width: "fill",
                    direction: "horizontal",
                    main_align: "space-between",
                    Button { onpress: cancel,
                        label { "Cancel" }
                    }
                    Button { onpress: not_save,
                        label { "Don't save" }
                    }
                    Button {
                        theme: theme_with!(
                            ButtonTheme { background : theme.colors.primary_accent, hover_background : theme
                            .colors.tertiary_accent }
                        ),
                        onpress: save,
                        label { "Save" }
                    }
                }
            }
        }
    }
}

pub async fn save(state: &AppState) -> Result<(), Error> {
    let file = AsyncFileDialog::new().save_file().await;
    if let Some(path) = file {
        let save_state = SaveState {
            players: state.players.iter().map(|(n, p)| (n.clone(), p)).collect(),
            top_group: state.top_group.clone(),
            groups: state.groups.clone(),
        };

        troubadour_lib::save(save_state, path.path())?;
        *HAS_SAVED.write() = true;
    }
    Ok(())
}

#[component]
fn ShowErrorModal() -> Element {
    let mut show_error_answer: UsePopupAnswer<Error, ShowError> =
        use_popup_answer::<Error, ShowError>();

    rsx! {
        Popup { show_close_button: false, close_on_escape_key: false,
            rect { padding: "20",
                label {
                    font_size: "18",
                    font_weight: "bold",
                    margin: "0 0 15 0",
                    "An error occurred"
                }
                label { margin: "0 0 10 0", {format!("{}", show_error_answer.data().read())} }
                rect { width: "100%", cross_align: "end",
                    Button { onpress: move |_| show_error_answer.answer(ShowError()),
                        label { "OK" }
                    }
                }
            }
        }
    }
}

macro_rules! clone {
    ($x:ident, $rest:expr) => {{
        let $x = $x.clone();
        $rest
    }};
}

pub(crate) use clone;
