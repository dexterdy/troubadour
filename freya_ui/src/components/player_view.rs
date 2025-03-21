use crate::{player_ref::PlayerRef, AppState};
use freya::prelude::*;

#[component]
pub fn PlayerView(player: PlayerRef, state: Signal<AppState>) -> Element {
    let update_signal = use_signal(|| 0);
    let player = player.subscribe(update_signal);

    let player_clone = player.clone();
    let player_borrow = player_clone.read();

    let player_clone = player.clone();
    let play = move |_| {
        player_clone.with_mut(|mut p| {
            let _ = p.play();
        });
    };

    let player_clone = player.clone();
    let pause = move |_| {
        player_clone.with_mut(|mut p| {
            let _ = p.pause();
        });
    };

    let player_clone = player.clone();
    let stop = move |_| {
        player_clone.with_mut(|mut p| {
            let _ = p.stop();
        });
    };

    let player_clone = player.clone();
    let set_volume = move |new_volume| {
        player_clone.with_mut(|mut p| {
            let _ = p.volume((new_volume * 0.02) as f32, state.read().master_volume);
        });
    };
    // clip (start, end)
    // loop
    // loop gap
    // delay
    rsx! {
        "{update_signal}"
        Button { onclick: play,
            label { "Play" }
        }
        Button { onclick: stop,
            label { "Stop" }
        }
        Button { onclick: pause,
            label { "Pause" }
        }
        Slider { value: (player_borrow.volume * 50.0) as f64, onmoved: set_volume }
    }
}
