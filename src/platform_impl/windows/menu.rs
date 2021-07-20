use crate::{
    event::{ModifiersState, VirtualKeyCode},
    platform_impl::platform::{event, util},
};
use parking_lot::Mutex;
use std::{collections::HashMap, sync::Arc};
use winapi::shared::windef::{HACCEL__, HMENU__};
use winapi::um::winuser;
use winapi::um::winuser::ACCEL;
use winapi::{
    ctypes::c_int,
    shared::basetsd::UINT_PTR,
    um::winuser::{CreateAcceleratorTableW, FALT, FCONTROL, FSHIFT, FVIRTKEY},
};

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

#[derive(Clone)]
pub struct Menu {
    pub(crate) raw: *mut HMENU__,
    pub(crate) accelerators: HashMap<Hotkey, Accelerator>,
}

unsafe impl Send for Menu {}
unsafe impl Sync for Menu {}

impl Menu {
    pub fn new() -> Self {
        unsafe {
            Self {
                raw: winuser::CreateMenu(),
                accelerators: HashMap::new(),
            }
        }
    }

    pub fn add_item<S: Into<String>, H: Into<Option<Hotkey>>>(
        &mut self,
        id: u16,
        name: S,
        key: H,
    ) {
        let content = if let Some(key) = key.into() {
            if let Some(accel) = Accelerator::parse(&key, id) {
                self.accelerators.insert(key, accel);
            }
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

    pub fn add_dropdown<S: Into<String>>(&mut self, name: S, mut menu: Menu) {
        self.accelerators
            .extend(std::mem::take(&mut menu.accelerators));

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

#[derive(Clone)]
pub(crate) struct Accelerator {
    pub raw: ACCEL,
}
unsafe impl Send for Accelerator {}
unsafe impl Sync for Accelerator {}

impl Accelerator {
    pub(crate) fn parse(hotkey: &Hotkey, id: u16) -> Option<Self> {
        let mut v_key = FVIRTKEY;
        if hotkey.modifiers.ctrl() {
            v_key |= FCONTROL;
        }
        if hotkey.modifiers.alt() {
            v_key |= FALT;
        }
        if hotkey.modifiers.shift() {
            v_key |= FSHIFT;
        }

        let key = (event::winit_vkey_to_vkey(hotkey.key)? & 0x00ff) as u16;

        let raw = ACCEL {
            fVirt: v_key,
            key,
            cmd: id,
        };

        Some(Accelerator { raw })
    }
}

pub(crate) struct AcceleratorTable {
    pub raw: *mut HACCEL__,
}
unsafe impl Send for AcceleratorTable {}
unsafe impl Sync for AcceleratorTable {}

impl AcceleratorTable {
    pub fn new(list: &[ACCEL]) -> Self {
        let raw =
            unsafe { CreateAcceleratorTableW(list as *const _ as *mut _, list.len() as c_int) };

        Self { raw }
    }
}

lazy_static! {
    // TODO: Possibly map this to each window instead of only one table (like `druid` does)?
    pub(crate) static ref ACCELS: Arc<Mutex<Option<AcceleratorTable>>> = Arc::new(Mutex::new(None));
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

impl std::fmt::Debug for Menu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Menu")
            .field("raw", &self.raw)
            .field("accelerators", &self.accelerators.keys())
            .finish()
    }
}

impl Drop for AcceleratorTable {
    fn drop(&mut self) {
        unsafe {
            winuser::DestroyAcceleratorTable(self.raw);
        }
    }
}
