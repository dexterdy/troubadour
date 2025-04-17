use anyhow::Error;
use freya::prelude::{Readable, Signal};
use rfd::AsyncFileDialog;

use crate::AppState;

pub struct ModalConfig<T> {
    pub(super) continuations: Vec<Box<dyn FnMut(T)>>,
    pub(super) shown: bool,
}

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

pub async fn save(state: Signal<AppState>) -> Result<(), Error> {
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
    }
    Ok(())
}
