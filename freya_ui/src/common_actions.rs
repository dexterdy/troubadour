use anyhow::Error;
use freya::prelude::*;
use rfd::AsyncFileDialog;

use crate::AppState;

#[derive(PartialEq)]
pub enum Unsaved {
    Saved,
    NotSaved,
    Cancelled,
}

#[derive(PartialEq)]
pub struct ShowError();

#[component]
pub fn GlobalModals(state: Signal<AppState>, children: Element) -> Element {
    let unsaved_popup = use_popup::<(), Unsaved>();
    use_context_provider(|| unsaved_popup);

    let show_error_popup = use_popup::<Error, ShowError>();
    use_context_provider(|| show_error_popup);

    rsx! {
        if unsaved_popup.is_open() {
            UnsavedModal { state }
        }
        if show_error_popup.is_open() {
            UnsavedModal { state }
        }
        {children}
    }
}

#[component]
pub fn UnsavedModal(state: Signal<AppState>) -> Element {
    let mut unsaved_answer = use_popup_answer::<(), Unsaved>();
    let mut show_file_pick = use_signal(|| false);

    let save = move |_| {
        if !*show_file_pick.read() {
            show_file_pick.set(true);
            spawn(async move {
                let _ = save(state).await;
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
            PopupTitle {
                label { "You have unsaved changes. Do you want to save?" }
            }
            PopupContent {
                label { "Unsaved changes will be lost." }
                Button { onpress: save,
                    label { "Save" }
                }
                Button { onpress: not_save,
                    label { "Don't Save" }
                }
                Button { onpress: cancel,
                    label { "Cancel" }
                }
            }
        }
    }
}

pub async fn save(mut state: Signal<AppState>) -> Result<(), Error> {
    let file = AsyncFileDialog::new().save_file().await;
    if let Some(path) = file {
        let s = state.peek();
        troubadour_lib::save(
            s.players
                .iter()
                .map(|(n, p)| (n.clone(), p.clone()))
                .collect(),
            &s.top_group,
            &s.groups,
            path.path(),
        )?;
        drop(s);
        state.write().saved = true;
    }
    Ok(())
}

#[component]
pub fn ShowErrorModal() -> Element {
    let mut show_error_popup = use_popup_answer::<Error, ShowError>();

    rsx! {
        Popup { show_close_button: false, close_on_escape_key: false,
            PopupTitle {
                label { "An error occurred" }
            }
            PopupContent {
                label { {format!("{}", show_error_popup.data().unwrap())} }
                Button { onpress: move |_| show_error_popup.answer(ShowError()),
                    label { "OK" }
                }
            }
        }
    }
}
