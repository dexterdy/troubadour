use anyhow::Error;
use freya::prelude::*;
use rfd::AsyncFileDialog;

use crate::AppState;

pub struct ModalConfig<T> {
    pub(super) continuations: Vec<Box<dyn FnMut(T)>>,
    pub(super) shown: bool,
}

#[derive(PartialEq)]
pub enum UnsavedModalResults {
    Saved,
    NotSaved,
    Cancelled,
}

pub struct GlobalModals {
    pub(super) unsaved_modal: ModalConfig<UnsavedModalResults>,
}

impl GlobalModals {
    pub fn show_unsaved_modal<F: FnMut(UnsavedModalResults) + 'static>(&mut self, f: F) {
        self.unsaved_modal.continuations.push(Box::new(f));
        self.unsaved_modal.shown = true;
    }
}

#[component]
pub fn UnsavedModal(state: Signal<AppState>) -> Element {
    let mut show_file_pick = use_signal(|| false);

    let save = move |_| {
        if !*show_file_pick.read() {
            show_file_pick.set(true);
            spawn(async move {
                let _ = save(state).await;
                let modals = &mut state.write().global_modals;
                for cont in &mut modals.unsaved_modal.continuations {
                    (cont)(UnsavedModalResults::Saved)
                }
                modals.unsaved_modal.continuations.clear();
                modals.unsaved_modal.shown = false;
                show_file_pick.set(false);
            });
        }
    };

    let cancel = move |_| {
        spawn(async move {
            let modals = &mut state.write().global_modals;
            for cont in &mut modals.unsaved_modal.continuations {
                (cont)(UnsavedModalResults::Cancelled)
            }
            modals.unsaved_modal.continuations.clear();
            modals.unsaved_modal.shown = false;
        });
    };

    let not_save = move |_| {
        spawn(async move {
            let modals = &mut state.write().global_modals;
            for cont in &mut modals.unsaved_modal.continuations {
                (cont)(UnsavedModalResults::NotSaved)
            }
            modals.unsaved_modal.continuations.clear();
            modals.unsaved_modal.shown = false;
        });
    };

    rsx! {
        if state.read().global_modals.unsaved_modal.shown {
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
