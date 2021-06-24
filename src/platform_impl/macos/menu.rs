use crate::event::{ModifiersState, VirtualKeyCode};
use super::util::IdRef;
use cocoa::appkit::{NSEventModifierFlags, NSMenu, NSMenuItem};
use cocoa::base::{nil, selector};
use cocoa::foundation::{NSProcessInfo, NSString};
use objc::{
    rc::autoreleasepool,
    runtime::{Object, Sel},
};

pub struct Hotkey{
    modifiers: ModifiersState,
    key: VirtualKeyCode
}

impl Hotkey{
    pub fn new(modifiers: ModifiersState, key: VirtualKeyCode) -> Self{
        Self{
            modifiers,
            key,
        }
    }

    fn parse(&self) -> (String, Option<NSEventModifierFlags>){
        let key = match self.key{
            VirtualKeyCode::Key1 => '1',
            VirtualKeyCode::Key2 => '2',
            VirtualKeyCode::Key3 => '3',
            VirtualKeyCode::Key4 => '4',
            VirtualKeyCode::Key5 => '5',
            VirtualKeyCode::Key6 => '6',
            VirtualKeyCode::Key7 => '7',
            VirtualKeyCode::Key8 => '8',
            VirtualKeyCode::Key9 => '9',
            VirtualKeyCode::Key0 => '0',

            VirtualKeyCode::A => 'a',
            VirtualKeyCode::B => 'b',
            VirtualKeyCode::C => 'c',
            VirtualKeyCode::D => 'd',
            VirtualKeyCode::E => 'e',
            VirtualKeyCode::F => 'f',
            VirtualKeyCode::G => 'g',
            VirtualKeyCode::H => 'h',
            VirtualKeyCode::I => 'i',
            VirtualKeyCode::J => 'j',
            VirtualKeyCode::K => 'k',
            VirtualKeyCode::L => 'l',
            VirtualKeyCode::M => 'm',
            VirtualKeyCode::N => 'n',
            VirtualKeyCode::O => 'o',
            VirtualKeyCode::P => 'p',
            VirtualKeyCode::Q => 'q',
            VirtualKeyCode::R => 'r',
            VirtualKeyCode::S => 's',
            VirtualKeyCode::T => 't',
            VirtualKeyCode::U => 'u',
            VirtualKeyCode::V => 'v',
            VirtualKeyCode::W => 'w',
            VirtualKeyCode::X => 'x',
            VirtualKeyCode::Y => 'y',
            VirtualKeyCode::Z => 'z',

            VirtualKeyCode::F1 => '\u{f704}',
            VirtualKeyCode::F2 => '\u{f705}',
            VirtualKeyCode::F3 => '\u{f706}',
            VirtualKeyCode::F4 => '\u{f707}',
            VirtualKeyCode::F5 => '\u{f708}',
            VirtualKeyCode::F6 => '\u{f709}',
            VirtualKeyCode::F7 => '\u{f70A}',
            VirtualKeyCode::F8 => '\u{f70B}',
            VirtualKeyCode::F9 => '\u{f70C}',
            VirtualKeyCode::F10 => '\u{f70D}',
            VirtualKeyCode::F11 => '\u{f70E}',
            VirtualKeyCode::F12 => '\u{f70F}',
            VirtualKeyCode::F13 => '\u{f710}',
            VirtualKeyCode::F14 => '\u{f711}',
            VirtualKeyCode::F15 => '\u{f712}',
            VirtualKeyCode::F16 => '\u{f713}',
            VirtualKeyCode::F17 => '\u{f714}',
            VirtualKeyCode::F18 => '\u{f715}',
            VirtualKeyCode::F19 => '\u{f716}',
            VirtualKeyCode::F20 => '\u{f717}',
            VirtualKeyCode::F21 => '\u{f718}',
            VirtualKeyCode::F22 => '\u{f719}',
            VirtualKeyCode::F23 => '\u{f71A}',
            VirtualKeyCode::F24 => '\u{f71B}',

            VirtualKeyCode::Snapshot => '\u{f738}',
            VirtualKeyCode::Scroll => '\u{f72F}',
            VirtualKeyCode::Pause => '\u{f730}',
            VirtualKeyCode::Sysrq => '\u{f731}',
            VirtualKeyCode::Stop => '\u{f734}',

            VirtualKeyCode::Insert => '\u{f727}',
            VirtualKeyCode::Home => '\u{f729}',
            VirtualKeyCode::Delete => '\u{f728}',
            VirtualKeyCode::End => '\u{f72B}',
            VirtualKeyCode::PageDown => '\u{f72D}',
            VirtualKeyCode::PageUp => '\u{f72C}',

            VirtualKeyCode::Up => '\u{f700}',
            VirtualKeyCode::Down => '\u{f701}',
            VirtualKeyCode::Left => '\u{f702}',
            VirtualKeyCode::Right => '\u{f703}',

            VirtualKeyCode::Back => '\u{0008}',
            VirtualKeyCode::Return => '\u{000d}',
            VirtualKeyCode::Space => '\u{0020}',
            VirtualKeyCode::Escape => '\u{001b}',
            _ => ' ',
        };

        let mut mask = NSEventModifierFlags::empty();
        if self.modifiers.logo(){
            mask.set(NSEventModifierFlags::NSCommandKeyMask, true);
        }
        
        if self.modifiers.ctrl(){
            mask.set(NSEventModifierFlags::NSControlKeyMask, true);
        }
        
        if self.modifiers.shift(){
            mask.set(NSEventModifierFlags::NSShiftKeyMask, true);
        }
        
        if self.modifiers.alt(){
            mask.set(NSEventModifierFlags::NSAlternateKeyMask, true);
        }

        (String::from(key), Some(mask))
    }
}

struct KeyEquivalent<'a> {
    key: &'a str,
    masks: Option<NSEventModifierFlags>,
}

impl Default for Menu{
    fn default() -> Self {
        autoreleasepool(|| unsafe {
            let menubar = IdRef::new(NSMenu::new(nil));
            let app_menu_item = IdRef::new(NSMenuItem::new(nil));
            menubar.addItem_(*app_menu_item);

            let app_menu = NSMenu::new(nil);
            let process_name = NSProcessInfo::processInfo(nil).processName();

            // About menu item
            let about_item_prefix = NSString::alloc(nil).init_str("About ");
            let about_item_title = about_item_prefix.stringByAppendingString_(process_name);
            let about_item = menu_item(
                about_item_title,
                selector("orderFrontStandardAboutPanel:"),
                None,
            );

            // Seperator menu item
            let sep_first = NSMenuItem::separatorItem(nil);

            // Hide application menu item
            let hide_item_prefix = NSString::alloc(nil).init_str("Hide ");
            let hide_item_title = hide_item_prefix.stringByAppendingString_(process_name);
            let hide_item = menu_item(
                hide_item_title,
                selector("hide:"),
                Some(KeyEquivalent {
                    key: "h",
                    masks: None,
                }),
            );

            // Hide other applications menu item
            let hide_others_item_title = NSString::alloc(nil).init_str("Hide Others");
            let hide_others_item = menu_item(
                hide_others_item_title,
                selector("hideOtherApplications:"),
                Some(KeyEquivalent {
                    key: "h",
                    masks: Some(
                        NSEventModifierFlags::NSAlternateKeyMask
                            | NSEventModifierFlags::NSCommandKeyMask,
                    ),
                }),
            );

            // Show applications menu item
            let show_all_item_title = NSString::alloc(nil).init_str("Show All");
            let show_all_item = menu_item(
                show_all_item_title,
                selector("unhideAllApplications:"),
                None,
            );

            // Seperator menu item
            let sep = NSMenuItem::separatorItem(nil);

            // Quit application menu item
            let quit_item_prefix = NSString::alloc(nil).init_str("Quit ");
            let quit_item_title = quit_item_prefix.stringByAppendingString_(process_name);
            let quit_item = menu_item(
                quit_item_title,
                selector("terminate:"),
                Some(KeyEquivalent {
                    key: "q",
                    masks: None,
                }),
            );

            app_menu.addItem_(about_item);
            app_menu.addItem_(sep_first);
            app_menu.addItem_(hide_item);
            app_menu.addItem_(hide_others_item);
            app_menu.addItem_(show_all_item);
            app_menu.addItem_(sep);
            app_menu.addItem_(quit_item);
            app_menu_item.setSubmenu_(app_menu);

            Menu{
                raw: menubar,
            }
        })
    }
}

fn menu_item(
    title: *mut Object,
    selector: Sel,
    key_equivalent: Option<KeyEquivalent<'_>>,
) -> *mut Object {
    unsafe {
        let (key, masks) = match key_equivalent {
            Some(ke) => (NSString::alloc(nil).init_str(ke.key), ke.masks),
            None => (NSString::alloc(nil).init_str(""), None),
        };
        let item = NSMenuItem::alloc(nil).initWithTitle_action_keyEquivalent_(title, selector, key);
        if let Some(masks) = masks {
            item.setKeyEquivalentModifierMask_(masks)
        }

        item
    }
}

#[derive(Debug, Clone)]
pub struct Menu {
    pub (crate) raw: IdRef,
}

impl Menu {
    pub fn new() -> Self {
        unsafe{
            Self{
                raw: IdRef::new(NSMenu::new(nil)),
            }
        }
    }

    pub fn add_item<S: Into<String>, H: Into<Option<Hotkey>>>(&mut self, id: usize, name: S, key: H) {
        autoreleasepool(||unsafe{
            let title = NSString::alloc(nil).init_str(&name.into());
            let (key, mask) = match key.into(){
                Some(hotkey) => hotkey.parse(),
                None => (String::new(), None),
            };

            let key = NSString::alloc(nil).init_str(&key);
            let menu_item = NSMenuItem::alloc(nil).initWithTitle_action_keyEquivalent_(title, selector("handle_menu:"), key);
            let () = msg_send![menu_item, setTag: id as isize];
            if let Some(mask) = mask{
                menu_item.setKeyEquivalentModifierMask_(mask);
            }

            self.raw.addItem_(menu_item);
        });
    }

    pub fn add_dropdown<S: Into<String>>(&mut self, name: S, menu: Menu) {
        autoreleasepool(||unsafe{
            let title = NSString::alloc(nil).init_str(&name.into());
            let menu_item = NSMenuItem::alloc(nil);
            let () = msg_send![*menu.raw, setTitle: title];
            menu_item.setSubmenu_(*menu.raw);

            self.raw.addItem_(menu_item);
        });
    }

    pub fn add_separator(&mut self) {
        autoreleasepool(||unsafe{
            let menu_item = NSMenuItem::separatorItem(nil);
            self.raw.addItem_(menu_item);
        });
    }
}