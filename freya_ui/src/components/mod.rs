pub mod player_view;
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
        rect { width, height, background: "{theme.colors.surface}" }
    }
}
