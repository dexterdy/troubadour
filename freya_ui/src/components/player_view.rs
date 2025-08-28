use crate::common_actions::{clone, use_polling, ShowError};
use crate::components::ToggleButton;
use crate::{AppState, PlayerId, StateChannel, HAS_SAVED, MASTER_VOLUME, SELECTED_PLAYER};
use anyhow::Error;
use dioxus_radio::hooks::use_radio;
use duration_human::DurationHuman;
use freya::prelude::*;
use std::time::Duration;

#[component]
pub fn PlayerView(player_id: PlayerId) -> Element {
    let mut player_channel =
        use_radio::<AppState, StateChannel>(StateChannel::SpecificPlayer(player_id.clone()));
    let mut player_pause_channel =
        use_radio::<AppState, StateChannel>(StateChannel::SpecificPlayerPaused(player_id.clone()));
    
    let mut show_error_popup = use_context::<UsePopup<Error, ShowError>>();
    let theme = use_get_theme();

    macro_rules! player {
        ($id:expr) => {
            player_channel.read().players.get(&$id).unwrap()
        };
    }
    macro_rules! player_mut {
        ($id:expr) => {
            player_channel.write().players.get_mut(&$id).unwrap()
        };
    }
    macro_rules! pause_player_mut {
        ($id:expr) => {
            player_pause_channel.write().players.get_mut(&$id).unwrap()
        };
    }

    let play = clone!(player_id, move |_| {
        if let Err(e) = pause_player_mut!(player_id).play() {
            spawn(async move {
                show_error_popup.open(Some(e.into())).await;
            });
        }
    });

    let pause = clone!(player_id, move |_| {
        pause_player_mut!(player_id).pause();
    });

    let stop = clone!(player_id, move |_| {
        pause_player_mut!(player_id).stop();
    });

    let set_volume = clone!(player_id, move |new_volume| {
        player_mut!(player_id).volume((new_volume * 0.02) as f32, *MASTER_VOLUME.read());
        *HAS_SAVED.write() = false;
    });

    let toggle_loop = clone!(player_id, move |_| {
        let mut binding = player_channel.write();
        let player = binding.players.get_mut(&player_id).unwrap();
        if let Err(e) = player.toggle_loop(!player.looping, player.loop_gap) {
            spawn(async move {
                show_error_popup.open(Some(e.into())).await;
            });
        }
        *HAS_SAVED.write() = false;
    });
    let remove_player = clone!(player_id, move |_| {
        let mut state = player_channel.write_channel(StateChannel::AddOrRemove);
        state.top_group.shift_remove(&player_id);
        let group_id = state.players.get(&player_id).unwrap().group.clone();
        if let Some(group_id) = group_id {
            state
                .groups
                .get_mut(&group_id)
                .unwrap()
                .shift_remove(&player_id);
        }
        if let Some(id) = SELECTED_PLAYER.peek().clone() {
            if player_id == id {
                *SELECTED_PLAYER.write() = None;
            }
        }
        state.players.remove(&player_id);
        *HAS_SAVED.write() = false;
    });

    let border = if Some(player_id.clone()) == *SELECTED_PLAYER.read() {
        format! {"2 outer {}", theme.colors.highlight_color}
    } else {
        format! {"1 outer {}", theme.colors.solid}
    };

    let mut hovering = use_signal(|| false);

    rsx! {
        rect {
            border,
            corner_radius: "5",
            content: "flex",
            onpointerenter: move |_| hovering.set(true),
            onpointerleave: move |_| hovering.set(false),
            rect {
                width: "flex(1)",
                cross_align: "center",
                rect {
                    width: "35",
                    height: "15",
                    cross_align: "center",
                    onclick: move |_| { *SELECTED_PLAYER.write() = Some(player_id.clone()) },
                    svg {
                        position: "absolute",
                        width: "35",
                        height: "15",
                        fill: theme.colors.secondary_surface.to_string(),
                        layer: "1",
                        svg_data: static_bytes(include_bytes!("../../icons/trapezoid.svg")),
                    }
                    svg {
                        width: "15",
                        height: "15",
                        fill: theme.colors.solid.to_string(),
                        rotate: "90deg",
                        svg_data: static_bytes(include_bytes!("../../icons/list-drag-handle-symbolic.svg")),
                    }
                }
                rect {
                    position: "absolute",
                    position_top: "0",
                    position_right: "0",
                    padding: "3",
                    onclick: remove_player,
                    CrossIcon { fill: theme.colors.solid.to_string() }
                }
            }
            rect { padding: "8", spacing: "6", content: "flex",
                if *hovering.read() {
                    OverflowedContent { width: "flex(1)",
                        label { font_size: "16", font_weight: "bold", "{player_id.clone()}" }
                    }
                } else {
                    label {
                        width: "flex(1)",
                        font_size: "16",
                        font_weight: "bold",
                        max_lines: "1",
                        text_overflow: "ellipsis",
                        "{player_id.clone()}"
                    }
                }
                rect { direction: "horizontal", spacing: "5",
                    ToggleButton {
                        toggled: player!(player_id.clone()).looping,
                        onpress: toggle_loop,
                        width: "20",
                        height: "20",
                        svg_data: include_bytes!("../../icons/loop-arrow-symbolic.svg"),
                    }
                    ToggleButton {
                        toggled: player!(player_id.clone()).get_is_playing(),
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
                        toggled: player!(player_id.clone()).get_is_paused(),
                        onpress: pause,
                        width: "20",
                        height: "20",
                        svg_data: include_bytes!("../../icons/media-playback-pause-symbolic.svg"),
                    }
                }
                rect { width: "flex(1)",
                    label { width: "0", height: "0", a11y_hidden: "true", "volume" }
                    Slider {
                        value: (player!(player_id.clone()).volume * 50.0) as f64,
                        onmoved: set_volume,
                    }
                }
            }
        }
    }
}

#[component]
pub fn EditPlayerPanel(player_id: PlayerId) -> Element {
    let player_no_update_channel = use_radio::<AppState, StateChannel>(StateChannel::NoUpdate);
    let mut player_channel =
        use_radio::<AppState, StateChannel>(StateChannel::SpecificPlayer(player_id.clone()));
    let theme = use_get_theme();
    let mut show_error_popup = use_context::<UsePopup<Error, ShowError>>();

    macro_rules! player {
        ($id:expr) => {
            player_channel.read().players.get(&$id).unwrap()
        };
    }
    macro_rules! player_peek {
        ($id:expr) => {
            player_no_update_channel.read().players.get(&$id).unwrap()
        };
    }
    macro_rules! player_mut {
        ($id:expr) => {
            player_channel.write().players.get_mut(&$id).unwrap()
        };
    }

    let mut cut_start_input =
        use_signal(|| duration_to_string(player!(player_id.clone()).cut_start, false));
    let cut_start = clone!(player_id, move |new_cut: String| {
        cut_start_input.set(new_cut.clone());
        if let Ok(cut) = duration_str::parse(new_cut) {
            if let Err(e) = player_mut!(player_id).cut_start(cut) {
                spawn(async move {
                    show_error_popup.open(Some(e.into())).await;
                });
            }
        }
        *HAS_SAVED.write() = false;
    });

    let mut cut_end_input =
        use_signal(|| duration_to_string(player!(player_id.clone()).cut_end, false));
    let cut_end = clone!(player_id, move |new_cut: String| {
        cut_end_input.set(new_cut.clone());
        if let Ok(cut) = duration_str::parse(new_cut) {
            if let Err(e) = player_mut!(player_id).cut_end(cut) {
                spawn(async move {
                    show_error_popup.open(Some(e.into())).await;
                });
            }
        }
        *HAS_SAVED.write() = false;
    });

    let mut loop_gap_input =
        use_signal(|| duration_to_string(player!(player_id.clone()).loop_gap, false));
    let set_loop_gap = clone!(player_id, move |new_loop_gap: String| {
        loop_gap_input.set(new_loop_gap.clone());
        if let Ok(gap) = duration_str::parse(new_loop_gap) {
            let mut binding = player_channel.write();
            let player = binding.players.get_mut(&player_id).unwrap();
            if let Err(e) = player.toggle_loop(player.looping, gap) {
                spawn(async move {
                    show_error_popup.open(Some(e.into())).await;
                });
            }
        }
        *HAS_SAVED.write() = false;
    });

    let mut delay_input =
        use_signal(|| duration_to_string(player!(player_id.clone()).delay_length, false));
    let set_delay = clone!(player_id, move |new_delay: String| {
        delay_input.set(new_delay.clone());
        if let Ok(delay) = duration_str::parse(new_delay) {
            if let Err(e) = player_mut!(player_id).set_delay(delay) {
                spawn(async move {
                    show_error_popup.open(Some(e.into())).await;
                });
            }
        }
        *HAS_SAVED.write() = false;
    });

    use_effect(use_reactive(
        &player_id,
        clone!(player_id, move |_| {
            cut_start_input.set(duration_to_string(player_peek!(player_id).cut_start, false));
            cut_end_input.set(duration_to_string(player_peek!(player_id).cut_end, false));
            loop_gap_input.set(duration_to_string(player_peek!(player_id).loop_gap, true));
            delay_input.set(duration_to_string(
                player_peek!(player_id).delay_length,
                true,
            ));
        }),
    ));

    rsx! {
        rect {
            direction: "horizontal",
            spacing: "10",
            width: "fill",
            padding: "9",
            background: "{theme.colors.neutral_surface}",
            rect {
                label { "cut start" }
                Input {
                    value: cut_start_input,
                    onchange: cut_start,
                    onfocuschange: clone!(
                        player_id, move | focus : bool | { if ! focus { cut_start_input
                        .set(duration_to_string(player!(player_id) .cut_start, false)); } }
                    ),
                }
                label { "cut end" }
                Input {
                    value: cut_end_input,
                    onchange: cut_end,
                    onfocuschange: clone!(
                        player_id, move | focus : bool | { if ! focus { cut_end_input
                        .set(duration_to_string(player!(player_id) .cut_end, false)); } }
                    ),
                }
            }
            rect {
                label { "loop gap" }
                Input {
                    value: loop_gap_input,
                    onchange: set_loop_gap,
                    onfocuschange: clone!(
                        player_id, move | focus : bool | { if ! focus { loop_gap_input
                        .set(duration_to_string(player!(player_id) .loop_gap, false)); } }
                    ),
                }
                label { "delay" }
                Input {
                    value: delay_input,
                    onchange: set_delay,
                    onfocuschange: clone!(
                        player_id, move | focus : bool | { if ! focus { delay_input
                        .set(duration_to_string(player!(player_id) .delay_length, false)); } }
                    ),
                }
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
