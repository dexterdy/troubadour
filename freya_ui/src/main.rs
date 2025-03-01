use freya::prelude::*;

fn main() {
    launch(app);
}

fn app() -> Element {
    let mut state = use_signal(|| 0);
    let onclick = move |_| {
        state += 1;
    };

    rsx!(
        label {
            onclick,
            "State is {state}"
         }
    )
}
