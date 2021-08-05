use std::{
    char,
    os::raw::c_int,
    ptr,
    sync::atomic::{AtomicBool, AtomicPtr, Ordering},
};

use crate::event::{ModifiersState, ScanCode, VirtualKeyCode};

use winapi::{
    shared::minwindef::{HKL, HKL__, LPARAM, UINT, WPARAM},
    um::winuser,
};

fn key_pressed(vkey: c_int) -> bool {
    unsafe { (winuser::GetKeyState(vkey) & (1 << 15)) == (1 << 15) }
}

pub fn get_key_mods() -> ModifiersState {
    let filter_out_altgr = layout_uses_altgr() && key_pressed(winuser::VK_RMENU);

    let mut mods = ModifiersState::empty();
    mods.set(ModifiersState::SHIFT, key_pressed(winuser::VK_SHIFT));
    mods.set(
        ModifiersState::CTRL,
        key_pressed(winuser::VK_CONTROL) && !filter_out_altgr,
    );
    mods.set(
        ModifiersState::ALT,
        key_pressed(winuser::VK_MENU) && !filter_out_altgr,
    );
    mods.set(
        ModifiersState::LOGO,
        key_pressed(winuser::VK_LWIN) || key_pressed(winuser::VK_RWIN),
    );
    mods
}

bitflags! {
    #[derive(Default)]
    pub struct ModifiersStateSide: u32 {
        const LSHIFT = 0b010 << 0;
        const RSHIFT = 0b001 << 0;

        const LCTRL = 0b010 << 3;
        const RCTRL = 0b001 << 3;

        const LALT = 0b010 << 6;
        const RALT = 0b001 << 6;

        const LLOGO = 0b010 << 9;
        const RLOGO = 0b001 << 9;
    }
}

impl ModifiersStateSide {
    pub fn filter_out_altgr(&self) -> ModifiersStateSide {
        match layout_uses_altgr() && self.contains(Self::RALT) {
            false => *self,
            true => *self & !(Self::LCTRL | Self::RCTRL | Self::LALT | Self::RALT),
        }
    }
}

impl From<ModifiersStateSide> for ModifiersState {
    fn from(side: ModifiersStateSide) -> Self {
        let mut state = ModifiersState::default();
        state.set(
            Self::SHIFT,
            side.intersects(ModifiersStateSide::LSHIFT | ModifiersStateSide::RSHIFT),
        );
        state.set(
            Self::CTRL,
            side.intersects(ModifiersStateSide::LCTRL | ModifiersStateSide::RCTRL),
        );
        state.set(
            Self::ALT,
            side.intersects(ModifiersStateSide::LALT | ModifiersStateSide::RALT),
        );
        state.set(
            Self::LOGO,
            side.intersects(ModifiersStateSide::LLOGO | ModifiersStateSide::RLOGO),
        );
        state
    }
}

pub fn get_pressed_keys() -> impl Iterator<Item = c_int> {
    let mut keyboard_state = vec![0u8; 256];
    unsafe { winuser::GetKeyboardState(keyboard_state.as_mut_ptr()) };
    keyboard_state
        .into_iter()
        .enumerate()
        .filter(|(_, p)| (*p & (1 << 7)) != 0) // whether or not a key is pressed is communicated via the high-order bit
        .map(|(i, _)| i as c_int)
}

unsafe fn get_char(keyboard_state: &[u8; 256], v_key: u32, hkl: HKL) -> Option<char> {
    let mut unicode_bytes = [0u16; 5];
    let len = winuser::ToUnicodeEx(
        v_key,
        0,
        keyboard_state.as_ptr(),
        unicode_bytes.as_mut_ptr(),
        unicode_bytes.len() as _,
        0,
        hkl,
    );
    if len >= 1 {
        char::decode_utf16(unicode_bytes.iter().cloned())
            .next()
            .and_then(|c| c.ok())
    } else {
        None
    }
}

/// Figures out if the keyboard layout has an AltGr key instead of an Alt key.
///
/// Unfortunately, the Windows API doesn't give a way for us to conveniently figure that out. So,
/// we use a technique blatantly stolen from [the Firefox source code][source]: iterate over every
/// possible virtual key and compare the `char` output when AltGr is pressed vs when it isn't. If
/// pressing AltGr outputs characters that are different from the standard characters, the layout
/// uses AltGr. Otherwise, it doesn't.
///
/// [source]: https://github.com/mozilla/gecko-dev/blob/265e6721798a455604328ed5262f430cfcc37c2f/widget/windows/KeyboardLayout.cpp#L4356-L4416
fn layout_uses_altgr() -> bool {
    unsafe {
        static ACTIVE_LAYOUT: AtomicPtr<HKL__> = AtomicPtr::new(ptr::null_mut());
        static USES_ALTGR: AtomicBool = AtomicBool::new(false);

        let hkl = winuser::GetKeyboardLayout(0);
        let old_hkl = ACTIVE_LAYOUT.swap(hkl, Ordering::SeqCst);

        if hkl == old_hkl {
            return USES_ALTGR.load(Ordering::SeqCst);
        }

        let mut keyboard_state_altgr = [0u8; 256];
        // AltGr is an alias for Ctrl+Alt for... some reason. Whatever it is, those are the keypresses
        // we have to emulate to do an AltGr test.
        keyboard_state_altgr[winuser::VK_MENU as usize] = 0x80;
        keyboard_state_altgr[winuser::VK_CONTROL as usize] = 0x80;

        let keyboard_state_empty = [0u8; 256];

        for v_key in 0..=255 {
            let key_noaltgr = get_char(&keyboard_state_empty, v_key, hkl);
            let key_altgr = get_char(&keyboard_state_altgr, v_key, hkl);
            if let (Some(noaltgr), Some(altgr)) = (key_noaltgr, key_altgr) {
                if noaltgr != altgr {
                    USES_ALTGR.store(true, Ordering::SeqCst);
                    return true;
                }
            }
        }

        USES_ALTGR.store(false, Ordering::SeqCst);
        false
    }
}

pub fn vkey_to_winit_vkey(vkey: c_int) -> Option<VirtualKeyCode> {
    // VK_* codes are documented here https://msdn.microsoft.com/en-us/library/windows/desktop/dd375731(v=vs.85).aspx
    match vkey {
        //winuser::VK_LBUTTON => Some(VirtualKeyCode::Lbutton),
        //winuser::VK_RBUTTON => Some(VirtualKeyCode::Rbutton),
        //winuser::VK_CANCEL => Some(VirtualKeyCode::Cancel),
        //winuser::VK_MBUTTON => Some(VirtualKeyCode::Mbutton),
        //winuser::VK_XBUTTON1 => Some(VirtualKeyCode::Xbutton1),
        //winuser::VK_XBUTTON2 => Some(VirtualKeyCode::Xbutton2),
        winuser::VK_BACK => Some(VirtualKeyCode::Back),
        winuser::VK_TAB => Some(VirtualKeyCode::Tab),
        //winuser::VK_CLEAR => Some(VirtualKeyCode::Clear),
        winuser::VK_RETURN => Some(VirtualKeyCode::Return),
        winuser::VK_LSHIFT => Some(VirtualKeyCode::LShift),
        winuser::VK_RSHIFT => Some(VirtualKeyCode::RShift),
        winuser::VK_LCONTROL => Some(VirtualKeyCode::LControl),
        winuser::VK_RCONTROL => Some(VirtualKeyCode::RControl),
        winuser::VK_LMENU => Some(VirtualKeyCode::LAlt),
        winuser::VK_RMENU => Some(VirtualKeyCode::RAlt),
        winuser::VK_PAUSE => Some(VirtualKeyCode::Pause),
        winuser::VK_CAPITAL => Some(VirtualKeyCode::Capital),
        winuser::VK_KANA => Some(VirtualKeyCode::Kana),
        //winuser::VK_HANGUEL => Some(VirtualKeyCode::Hanguel),
        //winuser::VK_HANGUL => Some(VirtualKeyCode::Hangul),
        //winuser::VK_JUNJA => Some(VirtualKeyCode::Junja),
        //winuser::VK_FINAL => Some(VirtualKeyCode::Final),
        //winuser::VK_HANJA => Some(VirtualKeyCode::Hanja),
        winuser::VK_KANJI => Some(VirtualKeyCode::Kanji),
        winuser::VK_ESCAPE => Some(VirtualKeyCode::Escape),
        winuser::VK_CONVERT => Some(VirtualKeyCode::Convert),
        winuser::VK_NONCONVERT => Some(VirtualKeyCode::NoConvert),
        //winuser::VK_ACCEPT => Some(VirtualKeyCode::Accept),
        //winuser::VK_MODECHANGE => Some(VirtualKeyCode::Modechange),
        winuser::VK_SPACE => Some(VirtualKeyCode::Space),
        winuser::VK_PRIOR => Some(VirtualKeyCode::PageUp),
        winuser::VK_NEXT => Some(VirtualKeyCode::PageDown),
        winuser::VK_END => Some(VirtualKeyCode::End),
        winuser::VK_HOME => Some(VirtualKeyCode::Home),
        winuser::VK_LEFT => Some(VirtualKeyCode::Left),
        winuser::VK_UP => Some(VirtualKeyCode::Up),
        winuser::VK_RIGHT => Some(VirtualKeyCode::Right),
        winuser::VK_DOWN => Some(VirtualKeyCode::Down),
        //winuser::VK_SELECT => Some(VirtualKeyCode::Select),
        //winuser::VK_PRINT => Some(VirtualKeyCode::Print),
        //winuser::VK_EXECUTE => Some(VirtualKeyCode::Execute),
        winuser::VK_SNAPSHOT => Some(VirtualKeyCode::Snapshot),
        winuser::VK_INSERT => Some(VirtualKeyCode::Insert),
        winuser::VK_DELETE => Some(VirtualKeyCode::Delete),
        //winuser::VK_HELP => Some(VirtualKeyCode::Help),
        0x30 => Some(VirtualKeyCode::Key0),
        0x31 => Some(VirtualKeyCode::Key1),
        0x32 => Some(VirtualKeyCode::Key2),
        0x33 => Some(VirtualKeyCode::Key3),
        0x34 => Some(VirtualKeyCode::Key4),
        0x35 => Some(VirtualKeyCode::Key5),
        0x36 => Some(VirtualKeyCode::Key6),
        0x37 => Some(VirtualKeyCode::Key7),
        0x38 => Some(VirtualKeyCode::Key8),
        0x39 => Some(VirtualKeyCode::Key9),
        0x41 => Some(VirtualKeyCode::A),
        0x42 => Some(VirtualKeyCode::B),
        0x43 => Some(VirtualKeyCode::C),
        0x44 => Some(VirtualKeyCode::D),
        0x45 => Some(VirtualKeyCode::E),
        0x46 => Some(VirtualKeyCode::F),
        0x47 => Some(VirtualKeyCode::G),
        0x48 => Some(VirtualKeyCode::H),
        0x49 => Some(VirtualKeyCode::I),
        0x4A => Some(VirtualKeyCode::J),
        0x4B => Some(VirtualKeyCode::K),
        0x4C => Some(VirtualKeyCode::L),
        0x4D => Some(VirtualKeyCode::M),
        0x4E => Some(VirtualKeyCode::N),
        0x4F => Some(VirtualKeyCode::O),
        0x50 => Some(VirtualKeyCode::P),
        0x51 => Some(VirtualKeyCode::Q),
        0x52 => Some(VirtualKeyCode::R),
        0x53 => Some(VirtualKeyCode::S),
        0x54 => Some(VirtualKeyCode::T),
        0x55 => Some(VirtualKeyCode::U),
        0x56 => Some(VirtualKeyCode::V),
        0x57 => Some(VirtualKeyCode::W),
        0x58 => Some(VirtualKeyCode::X),
        0x59 => Some(VirtualKeyCode::Y),
        0x5A => Some(VirtualKeyCode::Z),
        winuser::VK_LWIN => Some(VirtualKeyCode::LWin),
        winuser::VK_RWIN => Some(VirtualKeyCode::RWin),
        winuser::VK_APPS => Some(VirtualKeyCode::Apps),
        winuser::VK_SLEEP => Some(VirtualKeyCode::Sleep),
        winuser::VK_NUMPAD0 => Some(VirtualKeyCode::Numpad0),
        winuser::VK_NUMPAD1 => Some(VirtualKeyCode::Numpad1),
        winuser::VK_NUMPAD2 => Some(VirtualKeyCode::Numpad2),
        winuser::VK_NUMPAD3 => Some(VirtualKeyCode::Numpad3),
        winuser::VK_NUMPAD4 => Some(VirtualKeyCode::Numpad4),
        winuser::VK_NUMPAD5 => Some(VirtualKeyCode::Numpad5),
        winuser::VK_NUMPAD6 => Some(VirtualKeyCode::Numpad6),
        winuser::VK_NUMPAD7 => Some(VirtualKeyCode::Numpad7),
        winuser::VK_NUMPAD8 => Some(VirtualKeyCode::Numpad8),
        winuser::VK_NUMPAD9 => Some(VirtualKeyCode::Numpad9),
        winuser::VK_MULTIPLY => Some(VirtualKeyCode::NumpadMultiply),
        winuser::VK_ADD => Some(VirtualKeyCode::NumpadAdd),
        //winuser::VK_SEPARATOR => Some(VirtualKeyCode::Separator),
        winuser::VK_SUBTRACT => Some(VirtualKeyCode::NumpadSubtract),
        winuser::VK_DECIMAL => Some(VirtualKeyCode::NumpadDecimal),
        winuser::VK_DIVIDE => Some(VirtualKeyCode::NumpadDivide),
        winuser::VK_F1 => Some(VirtualKeyCode::F1),
        winuser::VK_F2 => Some(VirtualKeyCode::F2),
        winuser::VK_F3 => Some(VirtualKeyCode::F3),
        winuser::VK_F4 => Some(VirtualKeyCode::F4),
        winuser::VK_F5 => Some(VirtualKeyCode::F5),
        winuser::VK_F6 => Some(VirtualKeyCode::F6),
        winuser::VK_F7 => Some(VirtualKeyCode::F7),
        winuser::VK_F8 => Some(VirtualKeyCode::F8),
        winuser::VK_F9 => Some(VirtualKeyCode::F9),
        winuser::VK_F10 => Some(VirtualKeyCode::F10),
        winuser::VK_F11 => Some(VirtualKeyCode::F11),
        winuser::VK_F12 => Some(VirtualKeyCode::F12),
        winuser::VK_F13 => Some(VirtualKeyCode::F13),
        winuser::VK_F14 => Some(VirtualKeyCode::F14),
        winuser::VK_F15 => Some(VirtualKeyCode::F15),
        winuser::VK_F16 => Some(VirtualKeyCode::F16),
        winuser::VK_F17 => Some(VirtualKeyCode::F17),
        winuser::VK_F18 => Some(VirtualKeyCode::F18),
        winuser::VK_F19 => Some(VirtualKeyCode::F19),
        winuser::VK_F20 => Some(VirtualKeyCode::F20),
        winuser::VK_F21 => Some(VirtualKeyCode::F21),
        winuser::VK_F22 => Some(VirtualKeyCode::F22),
        winuser::VK_F23 => Some(VirtualKeyCode::F23),
        winuser::VK_F24 => Some(VirtualKeyCode::F24),
        winuser::VK_NUMLOCK => Some(VirtualKeyCode::Numlock),
        winuser::VK_SCROLL => Some(VirtualKeyCode::Scroll),
        winuser::VK_BROWSER_BACK => Some(VirtualKeyCode::NavigateBackward),
        winuser::VK_BROWSER_FORWARD => Some(VirtualKeyCode::NavigateForward),
        winuser::VK_BROWSER_REFRESH => Some(VirtualKeyCode::WebRefresh),
        winuser::VK_BROWSER_STOP => Some(VirtualKeyCode::WebStop),
        winuser::VK_BROWSER_SEARCH => Some(VirtualKeyCode::WebSearch),
        winuser::VK_BROWSER_FAVORITES => Some(VirtualKeyCode::WebFavorites),
        winuser::VK_BROWSER_HOME => Some(VirtualKeyCode::WebHome),
        winuser::VK_VOLUME_MUTE => Some(VirtualKeyCode::Mute),
        winuser::VK_VOLUME_DOWN => Some(VirtualKeyCode::VolumeDown),
        winuser::VK_VOLUME_UP => Some(VirtualKeyCode::VolumeUp),
        winuser::VK_MEDIA_NEXT_TRACK => Some(VirtualKeyCode::NextTrack),
        winuser::VK_MEDIA_PREV_TRACK => Some(VirtualKeyCode::PrevTrack),
        winuser::VK_MEDIA_STOP => Some(VirtualKeyCode::MediaStop),
        winuser::VK_MEDIA_PLAY_PAUSE => Some(VirtualKeyCode::PlayPause),
        winuser::VK_LAUNCH_MAIL => Some(VirtualKeyCode::Mail),
        winuser::VK_LAUNCH_MEDIA_SELECT => Some(VirtualKeyCode::MediaSelect),
        /*winuser::VK_LAUNCH_APP1 => Some(VirtualKeyCode::Launch_app1),
        winuser::VK_LAUNCH_APP2 => Some(VirtualKeyCode::Launch_app2),*/
        winuser::VK_OEM_PLUS => Some(VirtualKeyCode::Equals),
        winuser::VK_OEM_COMMA => Some(VirtualKeyCode::Comma),
        winuser::VK_OEM_MINUS => Some(VirtualKeyCode::Minus),
        winuser::VK_OEM_PERIOD => Some(VirtualKeyCode::Period),
        winuser::VK_OEM_1 => map_text_keys(vkey),
        winuser::VK_OEM_2 => map_text_keys(vkey),
        winuser::VK_OEM_3 => map_text_keys(vkey),
        winuser::VK_OEM_4 => map_text_keys(vkey),
        winuser::VK_OEM_5 => map_text_keys(vkey),
        winuser::VK_OEM_6 => map_text_keys(vkey),
        winuser::VK_OEM_7 => map_text_keys(vkey),
        /* winuser::VK_OEM_8 => Some(VirtualKeyCode::Oem_8), */
        winuser::VK_OEM_102 => Some(VirtualKeyCode::OEM102),
        /*winuser::VK_PROCESSKEY => Some(VirtualKeyCode::Processkey),
        winuser::VK_PACKET => Some(VirtualKeyCode::Packet),
        winuser::VK_ATTN => Some(VirtualKeyCode::Attn),
        winuser::VK_CRSEL => Some(VirtualKeyCode::Crsel),
        winuser::VK_EXSEL => Some(VirtualKeyCode::Exsel),
        winuser::VK_EREOF => Some(VirtualKeyCode::Ereof),
        winuser::VK_PLAY => Some(VirtualKeyCode::Play),
        winuser::VK_ZOOM => Some(VirtualKeyCode::Zoom),
        winuser::VK_NONAME => Some(VirtualKeyCode::Noname),
        winuser::VK_PA1 => Some(VirtualKeyCode::Pa1),
        winuser::VK_OEM_CLEAR => Some(VirtualKeyCode::Oem_clear),*/
        _ => None,
    }
}

pub fn winit_vkey_to_vkey(vkey: VirtualKeyCode) -> Option<c_int> {
    match vkey {
        VirtualKeyCode::Back => Some(winuser::VK_BACK),
        VirtualKeyCode::Tab => Some(winuser::VK_TAB),
        VirtualKeyCode::Return => Some(winuser::VK_RETURN),
        VirtualKeyCode::LShift => Some(winuser::VK_LSHIFT),
        VirtualKeyCode::RShift => Some(winuser::VK_RSHIFT),
        VirtualKeyCode::LControl => Some(winuser::VK_LCONTROL),
        VirtualKeyCode::RControl => Some(winuser::VK_RCONTROL),
        VirtualKeyCode::LAlt => Some(winuser::VK_LMENU),
        VirtualKeyCode::RAlt => Some(winuser::VK_RMENU),
        VirtualKeyCode::Pause => Some(winuser::VK_PAUSE),
        VirtualKeyCode::Capital => Some(winuser::VK_CAPITAL),
        VirtualKeyCode::Kana => Some(winuser::VK_KANA),
        VirtualKeyCode::Kanji => Some(winuser::VK_KANJI),
        VirtualKeyCode::Escape => Some(winuser::VK_ESCAPE),
        VirtualKeyCode::Convert => Some(winuser::VK_CONVERT),
        VirtualKeyCode::NoConvert => Some(winuser::VK_NONCONVERT),
        VirtualKeyCode::Space => Some(winuser::VK_SPACE),
        VirtualKeyCode::PageUp => Some(winuser::VK_PRIOR),
        VirtualKeyCode::PageDown => Some(winuser::VK_NEXT),
        VirtualKeyCode::End => Some(winuser::VK_END),
        VirtualKeyCode::Home => Some(winuser::VK_HOME),
        VirtualKeyCode::Left => Some(winuser::VK_LEFT),
        VirtualKeyCode::Up => Some(winuser::VK_UP),
        VirtualKeyCode::Right => Some(winuser::VK_RIGHT),
        VirtualKeyCode::Down => Some(winuser::VK_DOWN),
        VirtualKeyCode::Snapshot => Some(winuser::VK_SNAPSHOT),
        VirtualKeyCode::Insert => Some(winuser::VK_INSERT),
        VirtualKeyCode::Delete => Some(winuser::VK_DELETE),
        VirtualKeyCode::Key0 => Some(0x30),
        VirtualKeyCode::Key1 => Some(0x31),
        VirtualKeyCode::Key2 => Some(0x32),
        VirtualKeyCode::Key3 => Some(0x33),
        VirtualKeyCode::Key4 => Some(0x34),
        VirtualKeyCode::Key5 => Some(0x35),
        VirtualKeyCode::Key6 => Some(0x36),
        VirtualKeyCode::Key7 => Some(0x37),
        VirtualKeyCode::Key8 => Some(0x38),
        VirtualKeyCode::Key9 => Some(0x39),
        VirtualKeyCode::A => Some(0x41),
        VirtualKeyCode::B => Some(0x42),
        VirtualKeyCode::C => Some(0x43),
        VirtualKeyCode::D => Some(0x44),
        VirtualKeyCode::E => Some(0x45),
        VirtualKeyCode::F => Some(0x46),
        VirtualKeyCode::G => Some(0x47),
        VirtualKeyCode::H => Some(0x48),
        VirtualKeyCode::I => Some(0x49),
        VirtualKeyCode::J => Some(0x4A),
        VirtualKeyCode::K => Some(0x4B),
        VirtualKeyCode::L => Some(0x4C),
        VirtualKeyCode::M => Some(0x4D),
        VirtualKeyCode::N => Some(0x4E),
        VirtualKeyCode::O => Some(0x4F),
        VirtualKeyCode::P => Some(0x50),
        VirtualKeyCode::Q => Some(0x51),
        VirtualKeyCode::R => Some(0x52),
        VirtualKeyCode::S => Some(0x53),
        VirtualKeyCode::T => Some(0x54),
        VirtualKeyCode::U => Some(0x55),
        VirtualKeyCode::V => Some(0x56),
        VirtualKeyCode::W => Some(0x57),
        VirtualKeyCode::X => Some(0x58),
        VirtualKeyCode::Y => Some(0x59),
        VirtualKeyCode::Z => Some(0x5A),
        VirtualKeyCode::LWin => Some(winuser::VK_LWIN),
        VirtualKeyCode::RWin => Some(winuser::VK_RWIN),
        VirtualKeyCode::Apps => Some(winuser::VK_APPS),
        VirtualKeyCode::Sleep => Some(winuser::VK_SLEEP),
        VirtualKeyCode::Numpad0 => Some(winuser::VK_NUMPAD0),
        VirtualKeyCode::Numpad1 => Some(winuser::VK_NUMPAD1),
        VirtualKeyCode::Numpad2 => Some(winuser::VK_NUMPAD2),
        VirtualKeyCode::Numpad3 => Some(winuser::VK_NUMPAD3),
        VirtualKeyCode::Numpad4 => Some(winuser::VK_NUMPAD4),
        VirtualKeyCode::Numpad5 => Some(winuser::VK_NUMPAD5),
        VirtualKeyCode::Numpad6 => Some(winuser::VK_NUMPAD6),
        VirtualKeyCode::Numpad7 => Some(winuser::VK_NUMPAD7),
        VirtualKeyCode::Numpad8 => Some(winuser::VK_NUMPAD8),
        VirtualKeyCode::Numpad9 => Some(winuser::VK_NUMPAD9),
        VirtualKeyCode::NumpadMultiply => Some(winuser::VK_MULTIPLY),
        VirtualKeyCode::NumpadAdd => Some(winuser::VK_ADD),
        VirtualKeyCode::NumpadSubtract => Some(winuser::VK_SUBTRACT),
        VirtualKeyCode::NumpadDecimal => Some(winuser::VK_DECIMAL),
        VirtualKeyCode::NumpadDivide => Some(winuser::VK_DIVIDE),
        VirtualKeyCode::F1 => Some(winuser::VK_F1),
        VirtualKeyCode::F2 => Some(winuser::VK_F2),
        VirtualKeyCode::F3 => Some(winuser::VK_F3),
        VirtualKeyCode::F4 => Some(winuser::VK_F4),
        VirtualKeyCode::F5 => Some(winuser::VK_F5),
        VirtualKeyCode::F6 => Some(winuser::VK_F6),
        VirtualKeyCode::F7 => Some(winuser::VK_F7),
        VirtualKeyCode::F8 => Some(winuser::VK_F8),
        VirtualKeyCode::F9 => Some(winuser::VK_F9),
        VirtualKeyCode::F10 => Some(winuser::VK_F10),
        VirtualKeyCode::F11 => Some(winuser::VK_F11),
        VirtualKeyCode::F12 => Some(winuser::VK_F12),
        VirtualKeyCode::F13 => Some(winuser::VK_F13),
        VirtualKeyCode::F14 => Some(winuser::VK_F14),
        VirtualKeyCode::F15 => Some(winuser::VK_F15),
        VirtualKeyCode::F16 => Some(winuser::VK_F16),
        VirtualKeyCode::F17 => Some(winuser::VK_F17),
        VirtualKeyCode::F18 => Some(winuser::VK_F18),
        VirtualKeyCode::F19 => Some(winuser::VK_F19),
        VirtualKeyCode::F20 => Some(winuser::VK_F20),
        VirtualKeyCode::F21 => Some(winuser::VK_F21),
        VirtualKeyCode::F22 => Some(winuser::VK_F22),
        VirtualKeyCode::F23 => Some(winuser::VK_F23),
        VirtualKeyCode::F24 => Some(winuser::VK_F24),
        VirtualKeyCode::Numlock => Some(winuser::VK_NUMLOCK),
        VirtualKeyCode::Scroll => Some(winuser::VK_SCROLL),
        VirtualKeyCode::NavigateBackward => Some(winuser::VK_BROWSER_BACK),
        VirtualKeyCode::NavigateForward => Some(winuser::VK_BROWSER_FORWARD),
        VirtualKeyCode::WebRefresh => Some(winuser::VK_BROWSER_REFRESH),
        VirtualKeyCode::WebStop => Some(winuser::VK_BROWSER_STOP),
        VirtualKeyCode::WebSearch => Some(winuser::VK_BROWSER_SEARCH),
        VirtualKeyCode::WebFavorites => Some(winuser::VK_BROWSER_FAVORITES),
        VirtualKeyCode::WebHome => Some(winuser::VK_BROWSER_HOME),
        VirtualKeyCode::Mute => Some(winuser::VK_VOLUME_MUTE),
        VirtualKeyCode::VolumeDown => Some(winuser::VK_VOLUME_DOWN),
        VirtualKeyCode::VolumeUp => Some(winuser::VK_VOLUME_UP),
        VirtualKeyCode::NextTrack => Some(winuser::VK_MEDIA_NEXT_TRACK),
        VirtualKeyCode::PrevTrack => Some(winuser::VK_MEDIA_PREV_TRACK),
        VirtualKeyCode::MediaStop => Some(winuser::VK_MEDIA_STOP),
        VirtualKeyCode::PlayPause => Some(winuser::VK_MEDIA_PLAY_PAUSE),
        VirtualKeyCode::Mail => Some(winuser::VK_LAUNCH_MAIL),
        VirtualKeyCode::MediaSelect => Some(winuser::VK_LAUNCH_MEDIA_SELECT),
        VirtualKeyCode::Equals => Some(winuser::VK_OEM_PLUS),
        VirtualKeyCode::Comma => Some(winuser::VK_OEM_COMMA),
        VirtualKeyCode::Minus => Some(winuser::VK_OEM_MINUS),
        VirtualKeyCode::Period => Some(winuser::VK_OEM_PERIOD),
        VirtualKeyCode::OEM102 => Some(winuser::VK_OEM_102),
        _ => None,
    }
}

pub fn handle_extended_keys(
    vkey: c_int,
    mut scancode: UINT,
    extended: bool,
) -> Option<(c_int, UINT)> {
    // Welcome to hell https://blog.molecular-matters.com/2011/09/05/properly-handling-keyboard-input/
    scancode = if extended { 0xE000 } else { 0x0000 } | scancode;
    let vkey = match vkey {
        winuser::VK_SHIFT => unsafe {
            winuser::MapVirtualKeyA(scancode, winuser::MAPVK_VSC_TO_VK_EX) as _
        },
        winuser::VK_CONTROL => {
            if extended {
                winuser::VK_RCONTROL
            } else {
                winuser::VK_LCONTROL
            }
        }
        winuser::VK_MENU => {
            if extended {
                winuser::VK_RMENU
            } else {
                winuser::VK_LMENU
            }
        }
        _ => {
            match scancode {
                // When VK_PAUSE is pressed it emits a LeftControl + NumLock scancode event sequence, but reports VK_PAUSE
                // as the virtual key on both events, or VK_PAUSE on the first event or 0xFF when using raw input.
                // Don't emit anything for the LeftControl event in the pair...
                0xE01D if vkey == winuser::VK_PAUSE => return None,
                // ...and emit the Pause event for the second event in the pair.
                0x45 if vkey == winuser::VK_PAUSE || vkey == 0xFF as _ => {
                    scancode = 0xE059;
                    winuser::VK_PAUSE
                }
                // VK_PAUSE has an incorrect vkey value when used with modifiers. VK_PAUSE also reports a different
                // scancode when used with modifiers than when used without
                0xE046 => {
                    scancode = 0xE059;
                    winuser::VK_PAUSE
                }
                // VK_SCROLL has an incorrect vkey value when used with modifiers.
                0x46 => winuser::VK_SCROLL,
                _ => vkey,
            }
        }
    };
    Some((vkey, scancode))
}

pub fn process_key_params(
    wparam: WPARAM,
    lparam: LPARAM,
) -> Option<(ScanCode, Option<VirtualKeyCode>)> {
    let scancode = ((lparam >> 16) & 0xff) as UINT;
    let extended = (lparam & 0x01000000) != 0;
    handle_extended_keys(wparam as _, scancode, extended)
        .map(|(vkey, scancode)| (scancode, vkey_to_winit_vkey(vkey)))
}

// This is needed as windows doesn't properly distinguish
// some virtual key codes for different keyboard layouts
fn map_text_keys(win_virtual_key: i32) -> Option<VirtualKeyCode> {
    let char_key =
        unsafe { winuser::MapVirtualKeyA(win_virtual_key as u32, winuser::MAPVK_VK_TO_CHAR) }
            & 0x7FFF;
    match char::from_u32(char_key) {
        Some(';') => Some(VirtualKeyCode::Semicolon),
        Some('/') => Some(VirtualKeyCode::Slash),
        Some('`') => Some(VirtualKeyCode::Grave),
        Some('[') => Some(VirtualKeyCode::LBracket),
        Some(']') => Some(VirtualKeyCode::RBracket),
        Some('\'') => Some(VirtualKeyCode::Apostrophe),
        Some('\\') => Some(VirtualKeyCode::Backslash),
        _ => None,
    }
}
