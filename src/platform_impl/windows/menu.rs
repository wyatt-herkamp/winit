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

    fn parse(&self) -> String{
        let mut string = String::new();
        if self.modifiers.logo() {
            string.push_str("Windows+");
        }
        if self.modifiers.ctrl() {
            string.push_str("Ctrl+");
        }
        if self.modifiers.shift() {
            string.push_str("Shift+");
        }
        if self.modifiers.alt() {
            string.push_str("Alt+");
        }
        
        let converted = match self.key {
            VirtualKeyCode::Key1 => "1",
            VirtualKeyCode::Key2 => "2",
            VirtualKeyCode::Key3 => "3",
            VirtualKeyCode::Key4 => "4",
            VirtualKeyCode::Key5 => "5",
            VirtualKeyCode::Key6 => "6",
            VirtualKeyCode::Key7 => "7",
            VirtualKeyCode::Key8 => "8",
            VirtualKeyCode::Key9 => "9",
            VirtualKeyCode::Key0 => "0",
        
            VirtualKeyCode::A => "A",
            VirtualKeyCode::B => "B",
            VirtualKeyCode::C => "C",
            VirtualKeyCode::D => "D",
            VirtualKeyCode::E => "E",
            VirtualKeyCode::F => "F",
            VirtualKeyCode::G => "G",
            VirtualKeyCode::H => "H",
            VirtualKeyCode::I => "I",
            VirtualKeyCode::J => "J",
            VirtualKeyCode::K => "K",
            VirtualKeyCode::L => "L",
            VirtualKeyCode::M => "M",
            VirtualKeyCode::N => "N",
            VirtualKeyCode::O => "O",
            VirtualKeyCode::P => "P",
            VirtualKeyCode::Q => "Q",
            VirtualKeyCode::R => "R",
            VirtualKeyCode::S => "S",
            VirtualKeyCode::T => "T",
            VirtualKeyCode::U => "U",
            VirtualKeyCode::V => "V",
            VirtualKeyCode::W => "W",
            VirtualKeyCode::X => "X",
            VirtualKeyCode::Y => "Y",
            VirtualKeyCode::Z => "Z",
        
            VirtualKeyCode::Escape => "Esc",
        
            VirtualKeyCode::F1 => "F1",
            VirtualKeyCode::F2 => "F2",
            VirtualKeyCode::F3 => "F3",
            VirtualKeyCode::F4 => "F4",
            VirtualKeyCode::F5 => "F5",
            VirtualKeyCode::F6 => "F6",
            VirtualKeyCode::F7 => "F7",
            VirtualKeyCode::F8 => "F8",
            VirtualKeyCode::F9 => "F9",
            VirtualKeyCode::F10 => "F10",
            VirtualKeyCode::F11 => "F11",
            VirtualKeyCode::F12 => "F12",
            VirtualKeyCode::F13 => "F13",
            VirtualKeyCode::F14 => "F14",
            VirtualKeyCode::F15 => "F15",
            VirtualKeyCode::F16 => "F16",
            VirtualKeyCode::F17 => "F17",
            VirtualKeyCode::F18 => "F18",
            VirtualKeyCode::F19 => "F19",
            VirtualKeyCode::F20 => "F20",
            VirtualKeyCode::F21 => "F21",
            VirtualKeyCode::F22 => "F22",
            VirtualKeyCode::F23 => "F23",
            VirtualKeyCode::F24 => "F24",
        
            VirtualKeyCode::Snapshot => "PrtScn",
            VirtualKeyCode::Scroll => "ScrLk",
            VirtualKeyCode::Pause => "Pause",
        
            VirtualKeyCode::Insert => "Ins",
            VirtualKeyCode::Home => "Home",
            VirtualKeyCode::Delete => "Del",
            VirtualKeyCode::End => "End",
            VirtualKeyCode::PageDown => "PgDn",
            VirtualKeyCode::PageUp => "PgUp",
        
            VirtualKeyCode::Left => "Left",
            VirtualKeyCode::Up => "Up",
            VirtualKeyCode::Right => "Right",
            VirtualKeyCode::Down => "Down",
        
            VirtualKeyCode::Back => "Backspace",
            VirtualKeyCode::Return => "Enter",
            VirtualKeyCode::Space => "Space",
        
            VirtualKeyCode::Compose => "Compose",
        
            VirtualKeyCode::Caret => "^",
        
            VirtualKeyCode::Numlock => "NumLk",
            VirtualKeyCode::Numpad0 => "Numpad0",
            VirtualKeyCode::Numpad1 => "Numpad1",
            VirtualKeyCode::Numpad2 => "Numpad2",
            VirtualKeyCode::Numpad3 => "Numpad3",
            VirtualKeyCode::Numpad4 => "Numpad4",
            VirtualKeyCode::Numpad5 => "Numpad5",
            VirtualKeyCode::Numpad6 => "Numpad6",
            VirtualKeyCode::Numpad7 => "Numpad7",
            VirtualKeyCode::Numpad8 => "Numpad8",
            VirtualKeyCode::Numpad9 => "Numpad9",
            VirtualKeyCode::NumpadAdd => "+",
            VirtualKeyCode::NumpadDivide => "/",
            VirtualKeyCode::NumpadDecimal => ".",
            VirtualKeyCode::NumpadComma => ",",
            VirtualKeyCode::NumpadEnter => "Enter",
            VirtualKeyCode::NumpadEquals => "=",
            VirtualKeyCode::NumpadMultiply => "*",
            VirtualKeyCode::NumpadSubtract => "-",
        
            VirtualKeyCode::AbntC1 => "AbntC1",
            VirtualKeyCode::AbntC2 => "AbntC2",
            VirtualKeyCode::Apostrophe => "'",
            VirtualKeyCode::Apps => "Apps",
            VirtualKeyCode::Asterisk => "*",
            VirtualKeyCode::At => "At",
            VirtualKeyCode::Ax => "Ax",
            VirtualKeyCode::Backslash => "\\",
            VirtualKeyCode::Calculator => "Calculator",
            VirtualKeyCode::Capital => "CapsLock",
            VirtualKeyCode::Colon => ";",
            VirtualKeyCode::Comma => ",",
            VirtualKeyCode::Convert => "Convert",
            VirtualKeyCode::Equals => "=",
            VirtualKeyCode::Grave => "Grave",
            VirtualKeyCode::Kana => "Kana",
            VirtualKeyCode::Kanji => "Kanji",
            VirtualKeyCode::LAlt => "LAlt",
            VirtualKeyCode::LBracket => "LBracket",
            VirtualKeyCode::LControl => "LControl",
            VirtualKeyCode::LShift => "LShift",
            VirtualKeyCode::LWin => "LWin",
            VirtualKeyCode::Mail => "Mail",
            VirtualKeyCode::MediaSelect => "MediaSelect",
            VirtualKeyCode::MediaStop => "MediaStop",
            VirtualKeyCode::Minus => "Minus",
            VirtualKeyCode::Mute => "Mute",
            VirtualKeyCode::MyComputer => "MyComputer",
        
            VirtualKeyCode::NavigateForward => "Next",
            VirtualKeyCode::NavigateBackward => "Prior",
        
            VirtualKeyCode::NextTrack => "NextTrack",
            VirtualKeyCode::NoConvert => "NoConvert",
            VirtualKeyCode::OEM102 => "OEM102",
            VirtualKeyCode::Period => ".",
            VirtualKeyCode::PlayPause => "PlayPause",
            VirtualKeyCode::Plus => "Plus",
            VirtualKeyCode::Power => "Power",
            VirtualKeyCode::PrevTrack => "PrevTrack",
            VirtualKeyCode::RAlt => "RAlt",
            VirtualKeyCode::RBracket => "RBracket",
            VirtualKeyCode::RControl => "RControl",
            VirtualKeyCode::RShift => "RShift",
            VirtualKeyCode::RWin => "RWin",
            VirtualKeyCode::Semicolon => "Semicolon",
            VirtualKeyCode::Slash => "Slash",
            VirtualKeyCode::Sleep => "Sleep",
            VirtualKeyCode::Stop => "Stop",
            VirtualKeyCode::Sysrq => "Sysrq",
            VirtualKeyCode::Tab => "Tab",
            VirtualKeyCode::Underline => "_",
            VirtualKeyCode::Unlabeled => "Unlabeled",
            VirtualKeyCode::VolumeDown => "VolDown",
            VirtualKeyCode::VolumeUp => "VolUp",
            VirtualKeyCode::Wake => "Wake",
            VirtualKeyCode::WebBack => "WebBack",
            VirtualKeyCode::WebFavorites => "WebFavorites",
            VirtualKeyCode::WebForward => "WebForward",
            VirtualKeyCode::WebHome => "WebHome",
            VirtualKeyCode::WebRefresh => "WebRefresh",
            VirtualKeyCode::WebSearch => "WebSearch",
            VirtualKeyCode::WebStop => "WebStop",
            VirtualKeyCode::Yen => "Yen",
            VirtualKeyCode::Copy => "Copy",
            VirtualKeyCode::Paste => "Paste",
            VirtualKeyCode::Cut => "Cut",
        };
        
        string.push_str(converted);
        
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
            format!("{}\t{}", name.into(), key.parse())
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
