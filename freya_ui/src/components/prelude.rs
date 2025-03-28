use freya::prelude::{
    dioxus_elements::attributes::{height, width},
    *,
};

#[component]
pub fn SplitButton(
    onclick: Option<EventHandler<()>>,
    children: Element,
    options: Vec<(EventHandler<()>, Element)>,
) -> Element {
    let mut theme = use_get_theme();
    theme.button.apply_colors(&theme.colors);
    let ButtonTheme {
        border_fill,
        margin,
        corner_radius,
        font_theme,
        shadow,
        width,
        height,
        ..
    } = theme.button.clone();

    let mut menu_open = use_signal(|| false);

    rsx! {
        rect { direction: "vertical",
            rect {
                margin: "{margin}",
                overflow: "clip",
                color: "{font_theme.color}",
                shadow: "{shadow}",
                border: "1 inner {border_fill}",
                corner_radius: "{corner_radius}",
                text_height: "disable-least-ascent",
                direction: "horizontal",
                main_align: "center",
                cross_align: "center",
                width: "{width}",
                height: "{height}",
                padding: "1",
                SplitLeftInnerButton { onclick, theme: theme.button.clone(), {children} }
                Separator { orientation: Orientation::Vertical }
                SplitRightInnerButton { theme: theme.button, menu_open }
            }
            if *menu_open.read() {
                rect {
                    for (onclick , children) in options {
                        rect {
                            onclick: move |_| {
                                menu_open.set(false);
                                (onclick)(());
                            },
                            {children}
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn SplitLeftInnerButton(
    onclick: Option<EventHandler<()>>,
    theme: ButtonTheme,
    children: Element,
) -> Element {
    let ButtonTheme {
        hover_background,
        background,
        padding,
        focus_border_fill,
        ..
    } = theme;

    let mut focussed = use_focus();
    let mut status = use_signal(ButtonStatus::default);
    let platform = use_platform();

    use_drop(move || {
        if *status.read() == ButtonStatus::Hovering {
            platform.set_cursor(CursorIcon::default());
        }
    });

    let onmouseenter = move |_| {
        platform.set_cursor(CursorIcon::Pointer);
        status.set(ButtonStatus::Hovering);
    };

    let onmouseleave = move |_| {
        platform.set_cursor(CursorIcon::default());
        status.set(ButtonStatus::default());
    };

    // let onkeydown = move |ev: KeyboardEvent| {
    //     if focussed.validate_keydown(&ev) {
    //         if let Some(onpress) = &onpress {
    //             onpress.call(PressEvent::Key(ev))
    //         }
    //     }
    // };

    let background = match *status.read() {
        ButtonStatus::Hovering => hover_background,
        ButtonStatus::Idle => background,
    };

    let border = if focussed.is_focused_with_keyboard() {
        format!("2 inner {focus_border_fill}")
    } else {
        "".to_string()
    };

    rsx! {
        rect {
            a11y_id: focussed.attribute(),
            a11y_focusable: true,
            a11y_role: "button",
            background: "{background}",
            direction: "horizontal",
            main_align: "center",
            cross_align: "center",
            border,
            padding: parse_and_subtract_from_padding(padding),
            onmouseenter,
            onmouseleave,
            onclick: move |_| {
                focussed.focus();
                onclick.map(|c| (c)(()));
            },
            {children}
        }
    }
}

#[component]
fn SplitRightInnerButton(theme: ButtonTheme, menu_open: Signal<bool>) -> Element {
    let ButtonTheme {
        hover_background,
        background,
        padding,
        focus_border_fill,
        ..
    } = theme;

    let mut focussed = use_focus();
    let mut status = use_signal(ButtonStatus::default);
    let platform = use_platform();

    use_effect(move || {
        if !focussed.is_focused() && *menu_open.read() {
            menu_open.set(false);
        }
    });

    use_drop(move || {
        if *status.read() == ButtonStatus::Hovering {
            platform.set_cursor(CursorIcon::default());
        }
    });

    let onmouseenter = move |_| {
        platform.set_cursor(CursorIcon::Pointer);
        status.set(ButtonStatus::Hovering);
    };

    let onmouseleave = move |_| {
        platform.set_cursor(CursorIcon::default());
        status.set(ButtonStatus::default());
    };

    // let onkeydown = move |ev: KeyboardEvent| {
    //     if focussed.validate_keydown(&ev) {
    //         if let Some(onpress) = &onpress {
    //             onpress.call(PressEvent::Key(ev))
    //         }
    //     }
    // };

    let background = match *status.read() {
        ButtonStatus::Hovering => hover_background,
        ButtonStatus::Idle => background,
    };

    let border = if focussed.is_focused_with_keyboard() {
        format!("2 inner {focus_border_fill}")
    } else {
        "".to_string()
    };

    rsx! {
        rect {
            a11y_id: focussed.attribute(),
            a11y_focusable: true,
            a11y_role: "button",
            background: "{background}",
            main_align: "center",
            cross_align: "center",
            padding: parse_and_subtract_from_padding(padding),
            border,
            height: "100%",
            onmouseenter,
            onmouseleave,
            onclick: move |_| {
                menu_open.toggle();
                focussed.focus();
            },
            TickIcon { fill: "black" }
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

#[component]
pub fn Separator(orientation: Orientation) -> Element {
    let (width, height) = match orientation {
        Orientation::Horizontal => ("100%", "1"),
        Orientation::Vertical => ("1", "100%"),
    };

    let theme = use_get_theme();

    rsx! {
        rect {
            width,
            height,
            background: "{theme.colors.opposite_surface}",
        }
    }
}

fn parse_and_subtract_from_padding(padding: Cow<'static, str>) -> String {
    padding
        .split(" ")
        .map(|n| (n.parse::<f32>().unwrap_or(1.0) - 1.0).to_string() + " ")
        .collect::<String>()
}
