use std::{
    cell::{Ref, RefCell, RefMut},
    ops::{Deref, DerefMut},
    rc::Rc,
};

use freya::prelude::{Readable, Signal, Writable};
use troubadour_lib::player::Player;

pub struct InnerPlayerRef {
    player: Player,
    generation: i32,
    subscribers: Vec<Signal<i32>>,
}

impl Deref for InnerPlayerRef {
    type Target = Player;

    fn deref(&self) -> &Self::Target {
        &self.player
    }
}

impl DerefMut for InnerPlayerRef {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.player
    }
}

#[derive(Clone)]
pub struct PlayerRef {
    inner: Rc<RefCell<InnerPlayerRef>>,
}

impl PlayerRef {
    pub fn new(player: Player) -> Self {
        PlayerRef {
            inner: Rc::new(RefCell::new(InnerPlayerRef {
                player: player,
                generation: 0,
                subscribers: vec![],
            })),
        }
    }

    pub fn subscribe(&mut self, signal: Signal<i32>) {
        let mut borrow = self.inner.borrow_mut();
        borrow.subscribers.push(signal);
    }

    pub fn read(&self) -> Ref<'_, InnerPlayerRef> {
        self.inner.borrow()
    }

    pub fn with_mut<F: Fn(RefMut<'_, InnerPlayerRef>)>(&self, f: F) {
        (f)(self.inner.borrow_mut());
        let mut borrow_mut = self.inner.borrow_mut();
        borrow_mut.generation = borrow_mut.generation + 1;
        let mut to_remove = vec![];
        for (i, sub) in borrow_mut.subscribers.iter_mut().enumerate() {
            let e = sub.try_peek();
            if let Err(_) = e {
                to_remove.push(i);
            } else {
                drop(e);
                let a = sub.read().clone();
                sub.set(a + 1);
            }
        }
        for r in to_remove.iter().rev() {
            let s = borrow_mut.subscribers.remove(*r);
            s.manually_drop();
        }
    }
}

impl PartialEq for PlayerRef {
    fn eq(&self, other: &Self) -> bool {
        self.inner.borrow().generation == other.inner.borrow().generation
    }
}
