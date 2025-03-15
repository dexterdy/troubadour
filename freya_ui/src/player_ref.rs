use std::{
    cell::{Cell, Ref, RefCell, RefMut},
    rc::Rc,
};

use troubadour_lib::player::Player;

#[derive(Clone)]
pub struct PlayerRef {
    inner: Rc<RefCell<Player>>,
    generation: Cell<i32>,
}

impl PlayerRef {
    pub fn new(player: Player) -> Self {
        PlayerRef {
            inner: Rc::new(RefCell::new(player)),
            generation: Cell::new(0),
        }
    }

    pub fn borrow(&self) -> Ref<'_, Player> {
        self.inner.borrow()
    }

    pub fn borrow_mut(&self) -> RefMut<'_, Player> {
        self.generation.set(self.generation.get() + 1);
        self.inner.borrow_mut()
    }
}

impl PartialEq for PlayerRef {
    fn eq(&self, other: &Self) -> bool {
        self.generation == other.generation
    }
}
