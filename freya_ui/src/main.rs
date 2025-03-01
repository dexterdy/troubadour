use std::path::PathBuf;

use freya::prelude::*;
use rfd::AsyncFileDialog;
use troubadour_lib::AppState;

fn main() {
    launch(app);
}

fn app() -> Element {
    let state = use_signal(|| AppState::new());

    let state_lock = state.read();
    let names_rendered = state_lock.players.iter().map(|(name, _)| {
        rsx! {
            label { "{name}" }
        }
    });

    rsx! {
        AddPlayer { state }
        {names_rendered}
    }
}

#[component]
fn AddPlayer(state: Signal<AppState>) -> Element {
    let mut path = use_signal::<Option<PathBuf>>(|| None);
    let mut show_name_dialogue = use_signal(|| false);
    let mut name = use_signal(|| "".to_string());

    let pick_file = move |_| {
        spawn(async move {
            let file = AsyncFileDialog::new().pick_file().await;
            path.set(file.map(|f| f.path().to_path_buf()));
            if path.read().is_some() {
                show_name_dialogue.set(true);
            }
        });
    };

    let done = move |_| {
        show_name_dialogue.set(false);
        state.with_mut(|s| {
            s.add(path.read().clone().unwrap(), name.read().clone())
                .unwrap()
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
