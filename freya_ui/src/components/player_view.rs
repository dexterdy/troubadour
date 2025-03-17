use crate::player_ref::PlayerRef;
use freya::prelude::*;

#[component]
pub fn PlayerView(player: PlayerRef) -> Element {
    let player_borrow = player.borrow();
    rsx! {
        label { "{player_borrow.name}" }
    }
}
