use freya::prelude::*;

use crate::components::{use_hover, Orientation, Separator};

// TODO: refactor to make use of freya menu's

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
                SplitInnerLeftButton { onpress, {children} }
                Separator { orientation: Orientation::Vertical }
                SplitInnerRightButton { menu_open }
            }
            if *menu_open.read() {
                SplitInnerModal { menu_open, options }
            }
        }
    }
}

#[component]
fn SplitInnerLeftButton(onpress: Option<EventHandler<()>>, children: Element) -> Element {
    let theme = use_get_theme();
    let ColorsSheet {
        surface,
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
        format!("1 0 1 1 inner {surface}")
    };

    rsx! {
        rect {
            a11y_id: focused.attribute(),
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

import_svg!(DownCaretIcon, "../../icons/down-small-symbolic.svg", { fill: "", rotate: "", width: "16", height: "16" });

#[component]
fn SplitInnerRightButton(menu_open: Signal<bool>) -> Element {
    let theme = use_get_theme();
    let ColorsSheet {
        surface,
        neutral_surface,
        focused_surface,
        focused_border,
        solid,
        ..
    } = theme.colors;

    let mut focused = use_focus();
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
        format!("1 1 1 0 inner {surface}")
    };

    rsx! {
        rect {
            a11y_id: focused.attribute(),
            a11y_focusable: true,
            a11y_role: "button",
            background: "{background}",
            main_align: "center",
            cross_align: "center",
            padding: "0 4",
            corner_radius: "0 6 6 0",
            border,
            height: "100%",
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

    let onglobalkeydown = move |ev: Event<KeyboardData>| {
        if ev.key == Key::Escape {
            menu_open.set(false);
        }
    };

    rsx! {
        rect { width: "0", height: "0",
            rect { width: "100v",
                rect {
                    layer: "-1000",
                    a11y_modal: true,
                    margin: "5 0 0 0",
                    border: "1 inner {surface}",
                    corner_radius: "8",
                    shadow: "0 0 8 0 rgb(0, 0, 0, 0.15)",
                    background: "{background}",
                    padding: "6",
                    onglobalkeydown,
                    for (i , (onpress , children)) in options.into_iter().enumerate() {
                        SplitInnerOptionButton {
                            is_first: i == 0,
                            onpress,
                            menu_open,
                            inner_focused,
                            children,
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn SplitInnerOptionButton(
    is_first: bool,
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
    let mut first_render = use_signal(|| true);
    let (onmouseenter, onmouseleave, status) = use_hover(CursorIcon::Pointer);

    use_hook(|| {
        if is_first {
            focused.request_focus();
        }
    });

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
