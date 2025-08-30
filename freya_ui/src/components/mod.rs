pub mod player_view;
pub mod save_load;
pub mod split_button;
pub mod top_controls;

pub use crate::components::split_button::SplitButton;
use freya::prelude::*;

pub fn use_hover(
    cursor: CursorIcon,
) -> (
    Box<dyn FnMut(Event<MouseData>)>,
    Box<dyn FnMut(Event<MouseData>)>,
    Signal<ButtonStatus>,
) {
    let platform = use_platform();
    let mut status = use_signal(ButtonStatus::default);
    let onmouseenter = move |_: Event<MouseData>| {
        platform.set_cursor(cursor);
        status.set(ButtonStatus::Hovering);
    };

    let onmouseleave = move |_: Event<MouseData>| {
        platform.set_cursor(CursorIcon::default());
        status.set(ButtonStatus::default());
    };

    use_drop(move || {
        if *status.read() == ButtonStatus::Hovering {
            platform.set_cursor(CursorIcon::default());
        }
    });

    (Box::new(onmouseenter), Box::new(onmouseleave), status)
}

#[component]
pub fn HoverCursor(
    cursor_icon: CursorIcon,
    children: Element,
    width: Option<&'static str>,
    height: Option<&'static str>,
) -> Element {
    let (onmouseenter, onmouseleave, _) = use_hover(cursor_icon);

    rsx! {
        rect {
            onmouseenter,
            onmouseleave,
            width,
            height,
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
pub fn Separator(orientation: Orientation, size: Option<String>) -> Element {
    let (width, height) = match orientation {
        Orientation::Horizontal => (size.unwrap_or("fill".to_string()), "1".to_string()),
        Orientation::Vertical => ("1".to_string(), size.unwrap_or("fill".to_string())),
    };

    let theme = use_get_theme();

    rsx! {
        rect {
            width,
            height,
            background: theme.colors.primary_surface.to_string(),
        }
    }
}

#[component]
pub fn ToggleButton(
    toggled: Option<bool>,
    onpress: Option<EventHandler<()>>,
    width: Option<String>,
    height: Option<String>,
    svg_data: &'static [u8],
) -> Element {
    let theme = use_get_theme();
    let toggled = toggled.unwrap_or(false);

    rsx! {
        Button {
            onpress: move |_| {
                onpress.map(|c| c(()));
            },
            theme: theme_with!(
                ButtonTheme { background : if toggled { theme.button.hover_background } else {
                theme.button.background }, padding : "4 8".into() }
            ),
            svg {
                fill: if toggled { theme.colors.primary_accent.to_string() } else { theme.colors.solid.to_string() },
                width,
                height,
                svg_data: static_bytes(svg_data),
            }
        }
    }
}
