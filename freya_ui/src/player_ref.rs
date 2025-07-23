use std::{fmt::Debug, panic::Location, rc::Rc};

use freya::prelude::{Readable, ReadableRef, Signal, Writable};
use troubadour_lib::player::{Player, Serializable};

struct InnerPlayerRef {
    player_signal: Signal<Player>,
}

impl Drop for InnerPlayerRef {
    fn drop(&mut self) {
        self.player_signal.manually_drop();
    }
}

#[derive(Clone)]
pub struct PlayerRef {
    inner: Rc<InnerPlayerRef>,
}

impl Debug for PlayerRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.player_signal.fmt(f)
    }
}

impl PlayerRef {
    pub fn new(player: Player) -> Self {
        PlayerRef {
            inner: Rc::new(InnerPlayerRef {
                player_signal: Signal::leak_with_caller(player, Location::caller()),
            }),
        }
    }
    pub fn read(&self) -> ReadableRef<Signal<Player>> {
        self.inner.player_signal.read()
    }

    pub fn with_mut<O, F: Fn(&mut Player) -> O>(&self, f: F) -> O {
        self.inner.player_signal.clone().with_mut(f)
    }
}

impl PartialEq for PlayerRef {
    fn eq(&self, other: &Self) -> bool {
        self.inner.player_signal == other.inner.player_signal
    }
}

impl Into<Serializable> for PlayerRef {
    fn into(self) -> Serializable {
        (&*self.read()).into()
    }
}
