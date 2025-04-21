use anyhow::Error;
use freya::prelude::*;
use rfd::AsyncFileDialog;

use crate::AppState;

#[derive(PartialEq)]
pub enum UnsavedModalResult {
    Saved,
    NotSaved,
    Cancelled,
}

#[component]
pub fn GlobalModals(state: Signal<AppState>, children: Element) -> Element {
    let unsaved_popup = use_popup::<(), UnsavedModalResult>();
    use_context_provider(|| unsaved_popup);

    rsx! {
        if unsaved_popup.is_open() {
            UnsavedModal { state }
        }
        {children}
    }
}

#[component]
pub fn UnsavedModal(state: Signal<AppState>) -> Element {
    let mut unsaved_answer = use_popup_answer::<(), UnsavedModalResult>();
    let mut show_file_pick = use_signal(|| false);

    let save = move |_| {
        if !*show_file_pick.read() {
            show_file_pick.set(true);
            spawn(async move {
                let _ = save(state).await;
                show_file_pick.set(false);
                unsaved_answer.answer(UnsavedModalResult::Saved);
            });
        }
    };

    let cancel = move |_| {
        unsaved_answer.answer(UnsavedModalResult::Cancelled);
    };

    let not_save = move |_| {
        unsaved_answer.answer(UnsavedModalResult::NotSaved);
    };

    rsx! {
        Popup { show_close_button: false, close_on_escape_key: false,
            PopupTitle {
                label { "You have unsaved changes. Do you want to save?" }
            }
            PopupContent {
                label { "Unsaved changes will be lost." }
                Button { onclick: save,
                    label { "Save" }
                }
                Button { onclick: not_save,
                    label { "Don't Save" }
                }
                Button { onclick: cancel,
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
