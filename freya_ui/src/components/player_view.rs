use crate::common_actions::ShowError;
use crate::components::{use_polling, ToggleButton};
use crate::{player_ref::PlayerRef, AppState};
use anyhow::Error;
use duration_human::DurationHuman;
use freya::prelude::*;
use std::time::Duration;

#[component]
pub fn PlayerView(player: PlayerRef, state: Signal<AppState>) -> Element {
    let player_clone = player.clone();
    let player_borrow = player_clone.read();
    let mut show_error_popup = use_context::<UsePopup<Error, ShowError>>();
    let theme = use_get_theme();

    let player_clone = player.clone();
    let is_playing = use_polling(
        move || player_clone.read().get_is_playing(),
        player_borrow.get_is_playing(),
    );

    let player_clone = player.clone();
    let is_paused = use_polling(
        move || player_clone.read().get_is_paused(),
        player_borrow.get_is_paused(),
    );

    let player_clone = player.clone();
    let play = move |_| {
        player_clone.with_mut(|p| {
            if let Err(e) = p.play() {
                spawn(async move {
                    show_error_popup.open(Some(e.into())).await;
                });
            }
        });
    };

    let player_clone = player.clone();
    let pause = move |_| {
        player_clone.with_mut(|p| {
            p.pause();
        });
    };

    let player_clone = player.clone();
    let stop = move |_| {
        player_clone.with_mut(|p| {
            p.stop();
        });
    };

    let player_clone = player.clone();
    let set_volume = move |new_volume| {
        player_clone.with_mut(|p| {
            p.volume((new_volume * 0.02) as f32, state.read().master_volume);
        });
        state.write().saved = false;
    };

    let player_clone = player.clone();
    let toggle_loop = move |_| {
        player_clone.with_mut(|p| {
            if let Err(e) = p.toggle_loop(!p.looping, p.loop_gap) {
                spawn(async move {
                    show_error_popup.open(Some(e.into())).await;
                });
            }
        });
        state.write().saved = false;
    };

    let border = if state.read().selected_player.is_some()
        && state.read().selected_player.clone().unwrap().read().name == player.read().name
    {
        format! {"2 outer {}", theme.colors.highlight_color}
    } else {
        format! {"1 outer {}", theme.colors.solid}
    };

    rsx! {
        rect { border, corner_radius: "5", content: "flex",
            rect { width: "flex(1)", cross_align: "center",
                rect {
                    width: "35",
                    height: "15",
                    cross_align: "center",
                    onclick: move |_| { state.write().selected_player = Some(player.clone()) },
                    svg {
                        position: "absolute",
                        width: "35",
                        height: "15",
                        fill: "{theme.colors.secondary_surface}",
                        layer: "1",
                        svg_data: static_bytes(include_bytes!("../../icons/trapezoid.svg")),
                    }
                    svg {
                        width: "15",
                        height: "15",
                        fill: "{theme.colors.solid}",
                        rotate: "90deg",
                        svg_data: static_bytes(include_bytes!("../../icons/list-drag-handle-symbolic.svg")),
                    }
                }
            }
            rect { padding: "8", spacing: "6", content: "flex",
                label {
                    width: "flex(1)",
                    font_size: "16",
                    font_weight: "bold",
                    max_lines: "1",
                    text_overflow: "ellipsis",
                    "{player_borrow.name}"
                }
                rect { direction: "horizontal", spacing: "5",
                    ToggleButton {
                        toggled: player_borrow.looping,
                        onpress: toggle_loop,
                        width: "20",
                        height: "20",
                        svg_data: include_bytes!("../../icons/loop-arrow-symbolic.svg"),
                    }
                    ToggleButton {
                        toggled: *is_playing.read(),
                        onpress: play,
                        width: "20",
                        height: "20",
                        svg_data: include_bytes!("../../icons/media-playback-start-symbolic.svg"),
                    }
                    Button {
                        onpress: stop,
                        theme: theme_with!(ButtonTheme { padding : "4 8".into() }),
                        svg {
                            width: "20",
                            height: "20",
                            fill: "{theme.colors.solid}",
                            svg_data: static_bytes(include_bytes!("../../icons/media-playback-stop-symbolic.svg")),
                        }
                    }
                    ToggleButton {
                        toggled: *is_paused.read(),
                        onpress: pause,
                        width: "20",
                        height: "20",
                        svg_data: include_bytes!("../../icons/media-playback-pause-symbolic.svg"),
                    }
                }
                rect { width: "flex(1)",
                    label { width: "0", height: "0", a11y_hidden: "true", "volume" }
                    Slider {
                        value: (player_borrow.volume * 50.0) as f64,
                        onmoved: set_volume,
                    }
                }
            }
        }
    }
}

#[component]
pub fn EditPlayerPanel(player: PlayerRef, state: Signal<AppState>) -> Element {
    let theme = use_get_theme();
    let mut show_error_popup = use_context::<UsePopup<Error, ShowError>>();

    let mut cut_start_input = use_signal(|| duration_to_string(player.read().cut_start, false));
    let player_clone = player.clone();
    let cut_start = move |new_cut: String| {
        cut_start_input.set(new_cut.clone());
        if let Ok(cut) = duration_str::parse(new_cut) {
            player_clone.with_mut(|p| {
                if let Err(e) = p.cut_start(cut) {
                    spawn(async move {
                        show_error_popup.open(Some(e.into())).await;
                    });
                }
            });
        }
        state.write().saved = false;
    };

    let mut cut_end_input = use_signal(|| duration_to_string(player.read().cut_end, false));
    let player_clone = player.clone();
    let cut_end = move |new_cut: String| {
        cut_end_input.set(new_cut.clone());
        if let Ok(cut) = duration_str::parse(new_cut) {
            player_clone.with_mut(|p| {
                if let Err(e) = p.cut_end(cut) {
                    spawn(async move {
                        show_error_popup.open(Some(e.into())).await;
                    });
                }
            });
        }
        state.write().saved = false;
    };

    let mut loop_gap_input = use_signal(|| duration_to_string(player.read().loop_gap, false));
    let player_clone = player.clone();
    let set_loop_gap = move |new_loop_gap: String| {
        loop_gap_input.set(new_loop_gap.clone());
        if let Ok(gap) = duration_str::parse(new_loop_gap) {
            player_clone.with_mut(|p| {
                if let Err(e) = p.toggle_loop(p.looping, gap) {
                    spawn(async move {
                        show_error_popup.open(Some(e.into())).await;
                    });
                }
            });
        }
        state.write().saved = false;
    };

    let mut delay_input = use_signal(|| duration_to_string(player.read().delay_length, false));
    let player_clone = player.clone();
    let set_delay = move |new_delay: String| {
        delay_input.set(new_delay.clone());
        if let Ok(delay) = duration_str::parse(new_delay) {
            player_clone.with_mut(|p| {
                if let Err(e) = p.set_delay(delay) {
                    spawn(async move {
                        show_error_popup.open(Some(e.into())).await;
                    });
                }
            });
        }
        state.write().saved = false;
    };

    rsx! {
        rect {
            direction: "horizontal",
            spacing: "10",
            width: "fill",
            padding: "9",
            background: "{theme.colors.neutral_surface}",
            rect {
                label { "cut start" }
                Input { value: cut_start_input, onchange: cut_start }
                label { "cut end" }
                Input { value: cut_end_input, onchange: cut_end }
            }
            rect {
                label { "loop gap" }
                Input { value: loop_gap_input, onchange: set_loop_gap }
                label { "delay" }
                Input { value: delay_input, onchange: set_delay }
            }
            label { width: "200", max_lines: "1", text_overflow: "ellipsis",
                "Looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooong text"
            }
        }
    }
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
