use std::thread::sleep;
use std::time::Duration;
use freya::prelude::*;

#[component]
pub fn SplitButton(
    onpress: Option<EventHandler<()>>,
    children: Element,
    options: Vec<(EventHandler<()>, Element)>,
) -> Element {
    let theme = use_get_theme();
    let ColorsSheet { color, .. } = theme.colors;
    let menu_open = use_signal(|| false);

    rsx! {
        rect { direction: "vertical",
            rect {
                overflow: "clip",
                color: "{color}",
                corner_radius: "6",
                text_height: "disable-least-ascent",
                direction: "horizontal",
                main_align: "center",
                cross_align: "center",
                SplitLeftInnerButton { onpress, {children} }
                Separator { orientation: Orientation::Vertical }
                SplitRightInnerButton { menu_open }
            }
            if *menu_open.read() {
                SplitInnerModal { menu_open, options }
            }
        }
    }
}

#[component]
fn SplitLeftInnerButton(onpress: Option<EventHandler<()>>, children: Element) -> Element {
    let theme = use_get_theme();
    let ColorsSheet {
        surface,
        neutral_surface,
        focused_surface,
        focused_border,
        ..
    } = theme.colors;

    let focussed = use_focus();
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

    let onkeydown = move |ev: KeyboardEvent| {
        if focussed.validate_keydown(&ev) {
            if let Some(onpress) = &onpress {
                onpress.call(())
            }
        }
    };

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
            corner_radius: "6 0 0 6",
            border,
            padding: "6 12",
            onmouseenter,
            onmouseleave,
            onclick: move |_| {
                onpress.map(|c| (c)(()));
            },
            onkeydown,
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
        solid,
        ..
    } = theme.colors;

    let focussed = use_focus();
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

    let onkeydown = move |ev: KeyboardEvent| {
        if focussed.validate_keydown(&ev) {
            menu_open.toggle();
        }
    };

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
            corner_radius: "0 6 6 0",
            border,
            height: "100%",
            onmouseenter,
            onmouseleave,
            onclick: move |_| {
                menu_open.toggle();
            },
            onkeydown,
            TickIcon { fill: "{solid}" }
        }
    }
}

#[component]
fn SplitInnerModal(menu_open: Signal<bool>, options: Vec<(EventHandler<()>, Element)>) -> Element {
    let theme = use_get_theme();
    let ColorsSheet {
        surface,
        background,
        ..
    } = theme.colors;

    let mut prev_inner_focused = use_signal(|| 0);
    let inner_focused = use_signal(|| 0);

    use_effect(move || {
        let prev = prev_inner_focused.peek().clone();
        let cur = inner_focused.read().clone();
        if cur == 0 && prev > 0 {
            menu_open.set(false);
        }
        prev_inner_focused.set(cur);
    });

    rsx! {
        rect { width: "0", height: "0",
            rect { width: "100v",
                rect {
                    a11y_modal: true,
                    margin: "5 0 0 0",
                    border: "1 inner {surface}",
                    corner_radius: "8",
                    shadow: "0 0 8 0 rgb(0, 0, 0, 0.15)",
                    background: "{background}",
                    padding: "6",
                    for (onpress , children) in options {
                        SplitOptionInnerButton { onpress, menu_open, inner_focused, children }
                    }
                }
            }
        }
    }
}

#[component]
fn SplitOptionInnerButton(
    onpress: EventHandler<()>,
    menu_open: Signal<bool>,
    inner_focused: Signal<i32>,
    children: Element,
) -> Element {
    let theme = use_get_theme();
    let ColorsSheet {
        focused_surface,
        focused_border,
        ..
    } = theme.colors;

    let mut focused = use_focus();
    let mut status = use_signal(ButtonStatus::default);
    let platform = use_platform();
    let mut first_render = use_signal(|| true);

    use_effect(move || {
        let foc = focused.is_focused();
        let cur_inn_foc = inner_focused.peek().clone();
        if *first_render.peek() {
            first_render.set(false);
            return;
        }
        if foc {
            inner_focused.set(cur_inn_foc + 1);
        } else {
            inner_focused.set(cur_inn_foc - 1);
        }
    });

    let onkeydown = move |ev: KeyboardEvent| {
        if focused.validate_keydown(&ev) {
            menu_open.set(false);
            focused.request_unfocus();
            onpress.call(());
        }
    };

    let onmouseenter = move |_| {
        platform.set_cursor(CursorIcon::Pointer);
        status.set(ButtonStatus::Hovering);
    };

    let onmouseleave = move |_| {
        platform.set_cursor(CursorIcon::default());
        status.set(ButtonStatus::default());
    };

    use_drop(move || {
        if *status.read() == ButtonStatus::Hovering {
            platform.set_cursor(CursorIcon::default());
        }
    });

    let background = match *status.read() {
        ButtonStatus::Hovering => focused_surface.to_string(),
        ButtonStatus::Idle => "none".to_string(),
    };

    let border = if focused.is_focused_with_keyboard() {
        format!("2 inner {focused_border}")
    } else {
        "".to_string()
    };

    rsx! {
        rect {
            a11y_id: focused.attribute(),
            a11y_focusable: true,
            a11y_role: "button",
            padding: "6 12",
            corner_radius: "6",
            onkeydown,
            onmouseenter,
            onmouseleave,
            background,
            border,
            onclick: move |_| {
                menu_open.set(false);
                focused.request_unfocus();
                onpress.call(())
            },
            {children}
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
