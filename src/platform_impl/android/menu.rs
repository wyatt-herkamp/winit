use crate::event::{ModifiersState, VirtualKeyCode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Hotkey {
    modifiers: ModifiersState,
    key: VirtualKeyCode,
}

impl Hotkey {
    pub fn new(modifiers: ModifiersState, key: VirtualKeyCode) -> Self {
        Self { modifiers, key }
    }
}

#[derive(Debug, Clone)]
pub struct Menu;

impl Menu {
    pub fn new() -> Self {
        Self
    }

    pub fn add_item<S: Into<String>, H: Into<Option<Hotkey>>>(
        &mut self,
        _id: usize,
        _name: S,
        _key: H,
    ) {
    }

    pub fn add_dropdown<S: Into<String>>(&mut self, _name: S, _menu: Menu) {}

    pub fn add_separator(&mut self) {}
}

impl Default for Menu {
    fn default() -> Self {
        Menu::new()
    }
}
