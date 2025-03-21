use std::{
    cell::{Cell, Ref, RefCell, RefMut},
    rc::Rc,
};

use freya::prelude::{Readable, Signal, Writable};
use troubadour_lib::player::Player;

#[derive(Clone)]
pub struct PlayerRef {
    inner: Rc<RefCell<Player>>,
    generation: Cell<i32>,
    subscriber: Option<RefCell<Signal<i32>>>,
}

impl PlayerRef {
    pub fn new(player: Player) -> Self {
        PlayerRef {
            inner: Rc::new(RefCell::new(player)),
            generation: Cell::new(0),
            subscriber: None,
        }
    }

    pub fn subscribe(&mut self, signal: Signal<i32>) -> Self {
        let mut cloned = self.clone();
        cloned.subscriber = Some(RefCell::new(signal));
        cloned
    }

    pub fn read(&self) -> Ref<'_, Player> {
        self.inner.borrow()
    }

    pub fn with_mut<F: Fn(RefMut<'_, Player>)>(&self, f: F) {
        self.generation.set(self.generation.get() + 1);
        (f)(self.inner.borrow_mut());
        if let Some(sub) = &self.subscriber {
            let mut sub = sub.borrow_mut();
            let a = sub.read().clone();
            sub.set(a + 1);
        }
    }
}

impl PartialEq for PlayerRef {
    fn eq(&self, other: &Self) -> bool {
        self.generation == other.generation
    }
}
