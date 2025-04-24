use std::time::Duration;

use crate::{player_ref::PlayerRef, AppState};
use duration_human::DurationHuman;
use freya::prelude::{
    dioxus_elements::attributes::{visible_height, visible_width},
    *,
};

#[component]
pub fn PlayerView(player: PlayerRef, state: Signal<AppState>) -> Element {
    let player_clone = player.clone();
    let player_borrow = player_clone.read();

    let player_clone = player.clone();
    let play = move |_| {
        player_clone.with_mut(|p| {
            let _ = p.play();
        });
    };

    let player_clone = player.clone();
    let pause = move |_| {
        player_clone.with_mut(|p| {
            let _ = p.pause();
        });
    };

    let player_clone = player.clone();
    let stop = move |_| {
        player_clone.with_mut(|p| {
            let _ = p.stop();
        });
    };

    let player_clone = player.clone();
    let set_volume = move |new_volume| {
        player_clone.with_mut(|p| {
            p.volume((new_volume * 0.02) as f32, state.read().master_volume);
        });
        state.write().saved = false;
    };

    let mut cut_start_input = use_signal(|| duration_to_string(player_borrow.cut_start, false));
    let player_clone = player.clone();
    let cut_start = move |new_cut: String| {
        cut_start_input.set(new_cut.clone());
        if let Ok(cut) = duration_str::parse(new_cut) {
            player_clone.with_mut(|p| {
                let _ = p.cut_start(cut);
            });
        }
        state.write().saved = false;
    };

    let mut cut_end_input = use_signal(|| duration_to_string(player_borrow.cut_end, false));
    let player_clone = player.clone();
    let cut_end = move |new_cut: String| {
        cut_end_input.set(new_cut.clone());
        if let Ok(cut) = duration_str::parse(new_cut) {
            player_clone.with_mut(|p| {
                let _ = p.cut_end(cut);
            });
        }
        state.write().saved = false;
    };

    let player_clone = player.clone();
    let toggle_loop = move |_| {
        player_clone.with_mut(|p| {
            let _ = p.toggle_loop(!p.looping, p.loop_gap);
        });
        state.write().saved = false;
    };

    let mut loop_gap_input = use_signal(|| duration_to_string(player_borrow.loop_gap, false));
    let player_clone = player.clone();
    let set_loop_gap = move |new_loop_gap: String| {
        loop_gap_input.set(new_loop_gap.clone());
        if let Ok(gap) = duration_str::parse(new_loop_gap) {
            player_clone.with_mut(|p| {
                let _ = p.toggle_loop(p.looping, gap);
            });
        }
        state.write().saved = false;
    };

    let mut delay_input = use_signal(|| duration_to_string(player_borrow.delay_length, false));
    let player_clone = player.clone();
    let set_delay = move |new_delay: String| {
        delay_input.set(new_delay.clone());
        if let Ok(delay) = duration_str::parse(new_delay) {
            player_clone.with_mut(|p| {
                let _ = p.set_delay(delay);
            });
        }
        state.write().saved = false;
    };

    rsx! {
        rect {
            main_align: "space-between",
            direction: "horizontal",
            width: "fill",
            rect {
                label {
                    height: "40",
                    main_align: "center",
                    font_size: "20",
                    font_weight: "bold",
                    "{player_borrow.name}"
                }
                rect {
                    height: "40",
                    cross_align: "center",
                    direction: "horizontal",
                    spacing: "5",
                    label { "loop:" }
                    Switch {
                        enabled: player_borrow.looping,
                        ontoggled: toggle_loop,
                    }
                }
            }
            rect {
                content: "fit",
                rect {
                    height: "40",
                    width: "fill-min",
                    cross_align: "center",
                    direction: "horizontal",
                    spacing: "5",
                    Button { onclick: play,
                        label { "Play" }
                    }
                    Button { onclick: stop,
                        label { "Stop" }
                    }
                    Button { onclick: pause,
                        label { "Pause" }
                    }
                }
                rect { height: "40", main_align: "center", width: "fill-min",
                    label { width: "0", height: "0", a11y_hidden: true, "volume" }
                    Slider {
                        value: (player_borrow.volume * 50.0) as f64,
                        onmoved: set_volume,
                    }
                }
            }
        }
    }
    // label { "cut start" }
    // Input { value: cut_start_input, onchange: cut_start }
    // label { "cut end" }
    // Input { value: cut_end_input, onchange: cut_end }
    // label { "loop gap" }
    // Input { value: loop_gap_input, onchange: set_loop_gap }
    // label { "delay" }
    // Input { value: delay_input, onchange: set_delay }
}

pub fn duration_to_string(dur: Duration, no_smaller_than_secs: bool) -> String {
    let nanos = if no_smaller_than_secs {
        dur.as_secs() * 1_000_000_000
    } else {
        dur.as_nanos() as u64
    };
    if nanos == 0 {
        "0s".to_string()
    } else {
        format!("{:#}", DurationHuman::from(nanos))
    }
}
