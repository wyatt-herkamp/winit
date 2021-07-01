use crate::{
    event::{ModifiersState, VirtualKeyCode},
    platform_impl::platform::util,
};
use winapi::shared::basetsd::UINT_PTR;
use winapi::shared::windef::HMENU__;
use winapi::um::winuser;

pub struct Hotkey {
    modifiers: ModifiersState,
    key: VirtualKeyCode,
}

impl Hotkey {
    pub fn new(modifiers: ModifiersState, key: VirtualKeyCode) -> Self {
        Self { modifiers, key }
    }
}

impl From<Hotkey> for String {
    fn from(hotkey: Hotkey) -> Self {
        let mut string = String::new();
        if hotkey.modifiers.logo() {
            string.push_str("Windows+");
        }
        if hotkey.modifiers.ctrl() {
            string.push_str("Ctrl+");
        }
        if hotkey.modifiers.shift() {
            string.push_str("Shift+");
        }
        if hotkey.modifiers.alt() {
            string.push_str("Alt+");
        }

        string.push_str(&String::from(hotkey.key));

        string
    }
}

#[derive(Debug, Clone)]
pub struct Menu {
    pub(crate) raw: *mut HMENU__,
}

impl Menu {
    pub fn new() -> Self {
        unsafe {
            Self {
                raw: winuser::CreateMenu(),
            }
        }
    }

    pub fn add_item<S: Into<String>, H: Into<Option<Hotkey>>>(
        &mut self,
        id: usize,
        name: S,
        key: H,
    ) {
        let content = if let Some(key) = key.into() {
            format!("{}\t{}", name.into(), String::from(key))
        } else {
            format!("{}", name.into())
        };

        unsafe {
            winuser::AppendMenuW(
                self.raw,
                winuser::MF_STRING,
                id as UINT_PTR,
                util::string_to_wchar(&content).as_mut_ptr(),
            );
        }
    }

    pub fn add_dropdown<S: Into<String>>(&mut self, name: S, menu: Menu) {
        unsafe {
            winuser::AppendMenuW(
                self.raw,
                winuser::MF_POPUP,
                menu.raw as UINT_PTR,
                util::string_to_wchar(&name.into()).as_mut_ptr(),
            );
        }
    }

    pub fn add_separator(&mut self) {
        unsafe {
            winuser::AppendMenuW(self.raw, winuser::MF_SEPARATOR, 0, std::ptr::null());
        }
    }
}

impl Default for Menu {
    fn default() -> Self {
        Menu::new()
    }
}

impl Drop for Menu {
    fn drop(&mut self) {
        unsafe {
            winuser::DestroyMenu(self.raw);
        }
    }
}
