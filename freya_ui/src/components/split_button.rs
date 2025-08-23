use freya::prelude::*;

use crate::components::{use_hover, Orientation, Separator};

#[component]
pub fn SplitButton(
    onpress: Option<EventHandler<()>>,
    left_button: Element,
    children: Element,
) -> Element {
    let mut menu_open = use_signal(|| false);

    rsx! {
        rect { direction: "vertical",
            rect {
                corner_radius: "6",
                direction: "horizontal",
                content: "flex",
                SplitInnerLeftButton { onpress, {left_button} }
                Separator { orientation: Orientation::Vertical, size: "flex(1)" }
                SplitInnerRightButton { menu_open }
            }
            if *menu_open.read() {
                rect { width: "0", height: "0",
                    rect { width: "100v",
                        Menu { onclose: move |_| menu_open.set(false), {children} }
                    }
                }
            }
        }
    }
}

#[component]
fn SplitInnerLeftButton(onpress: Option<EventHandler<()>>, children: Element) -> Element {
    let theme = use_get_theme();
    let ColorsSheet {
        primary_surface,
        neutral_surface,
        focused_surface,
        focused_border,
        ..
    } = theme.colors;

    let focused = use_focus();
    let (onmouseenter, onmouseleave, status) = use_hover(CursorIcon::Pointer);

    let onkeydown = move |ev: KeyboardEvent| {
        if focused.validate_keydown(&ev) {
            if let Some(onpress) = &onpress {
                onpress.call(())
            }
        }
    };

    let background = match *status.read() {
        ButtonStatus::Hovering => focused_surface,
        ButtonStatus::Idle => neutral_surface,
    };

    let border = if focused.is_focused_with_keyboard() {
        format!("2 inner {focused_border}")
    } else {
        format!("1 0 1 1 inner {primary_surface}")
    };

    rsx! {
        rect {
            a11y_id: focused.attribute(),
            a11y_focusable: "true",
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

import_svg!(DownCaretIcon, "../../icons/down-small-symbolic.svg", { fill: "", rotate: "", width: "16", height: "16" });

#[component]
fn SplitInnerRightButton(menu_open: Signal<bool>) -> Element {
    let theme = use_get_theme();
    let ColorsSheet {
        primary_surface,
        neutral_surface,
        focused_surface,
        focused_border,
        solid,
        ..
    } = theme.colors;

    let focused = use_focus();
    let (onmouseenter, onmouseleave, status) = use_hover(CursorIcon::Pointer);

    let onkeydown = move |ev: KeyboardEvent| {
        if focused.validate_keydown(&ev) {
            menu_open.toggle();
        }
    };

    let background = match *status.read() {
        ButtonStatus::Hovering => focused_surface,
        ButtonStatus::Idle => neutral_surface,
    };

    let border = if focused.is_focused_with_keyboard() {
        format!("2 inner {focused_border}")
    } else {
        format!("1 1 1 0 inner {primary_surface}")
    };

    rsx! {
        rect {
            a11y_id: focused.attribute(),
            a11y_focusable: "true",
            a11y_role: "button",
            background: "{background}",
            main_align: "center",
            cross_align: "center",
            padding: "0 4",
            corner_radius: "0 6 6 0",
            border,
            height: "flex(1)",
            onmouseenter,
            onmouseleave,
            onclick: move |_| {
                menu_open.toggle();
            },
            onkeydown,
            DownCaretIcon { fill: "{solid}", rotate: if *menu_open.read() { "180deg" } else { "" } }
        }
    }
}
