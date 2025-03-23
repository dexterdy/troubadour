use freya::prelude::*;

#[component]
pub fn SplitButton(
    onclick: Option<EventHandler<()>>,
    children: Element,
    options: Vec<(EventHandler<()>, Element)>,
) -> Element {
    // let theme = use_theme();
    let mut menu_open = use_signal(|| false);
    let mut focussed = use_focus();

    if !focussed.is_focused() {
        menu_open.set(false);
    }

    rsx! {
        rect {
            a11y_id: focussed.attribute(),
            a11y_focusable: true,
            onclick: move |_| focussed.focus(),
            rect {
                rect {
                    onclick: move |_| {
                        menu_open.set(false);
                        onclick.map(|c| (c)(()));
                    },
                    {children}
                }
                rect { onclick: move |_| menu_open.toggle(),
                    TickIcon { fill: "black" }
                }
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
