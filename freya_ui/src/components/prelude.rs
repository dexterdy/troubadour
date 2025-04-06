use freya::prelude::*;

#[component]
pub fn SplitButton(
    onclick: Option<EventHandler<()>>,
    children: Element,
    options: Vec<(EventHandler<()>, Element)>,
) -> Element {
    let theme = use_get_theme();
    let mut menu_open = use_signal(|| false);

    rsx! {
        rect { direction: "vertical",
            rect {
                overflow: "clip",
                color: "{theme.colors.color}",
                corner_radius: "6",
                text_height: "disable-least-ascent",
                direction: "horizontal",
                main_align: "center",
                cross_align: "center",
                SplitLeftInnerButton { onclick, {children} }
                Separator { orientation: Orientation::Vertical }
                SplitRightInnerButton { menu_open }
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
fn SplitLeftInnerButton(onclick: Option<EventHandler<()>>, children: Element) -> Element {
    let theme = use_get_theme();
    let ColorsSheet {
        surface,
        neutral_surface,
        focused_surface,
        focused_border,
        ..
    } = theme.colors;

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
        ButtonStatus::Hovering => focused_surface,
        ButtonStatus::Idle => neutral_surface,
    };

    let border = if focussed.is_focused_with_keyboard() {
        format!("2 inner {focused_border}")
    } else {
        format!("1 0 1 1 inner {surface}")
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
            corner_radius: "6 0 6 0",
            border,
            padding: "6 12",
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
fn SplitRightInnerButton(menu_open: Signal<bool>) -> Element {
    let theme = use_get_theme();
    let ColorsSheet {
        surface,
        neutral_surface,
        focused_surface,
        focused_border,
        ..
    } = theme.colors;

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
        ButtonStatus::Hovering => focused_surface,
        ButtonStatus::Idle => neutral_surface,
    };

    let border = if focussed.is_focused_with_keyboard() {
        format!("2 inner {focused_border}")
    } else {
        format!("1 1 1 0 inner {surface}")
    };

    rsx! {
        rect {
            a11y_id: focussed.attribute(),
            a11y_focusable: true,
            a11y_role: "button",
            background: "{background}",
            main_align: "center",
            cross_align: "center",
            padding: "6 12",
            corner_radius: "0 6 0 6",
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
