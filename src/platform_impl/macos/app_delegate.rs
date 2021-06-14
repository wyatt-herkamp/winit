use crate::event::{Event, MacOS, PlatformSpecific};
use crate::platform::macos::ActivationPolicy;
use crate::platform_impl::platform::{app_state::AppState, event::EventWrapper};
use cocoa::base::id;
use objc::{
    declare::ClassDecl,
    runtime::{Class, Object, Sel},
};
use std::{
    cell::{RefCell, RefMut},
    os::raw::c_void,
};

static AUX_DELEGATE_STATE_NAME: &str = "auxState";

pub struct AuxDelegateState {
    /// We store this value in order to be able to defer setting the activation policy until
    /// after the app has finished launching. If the activation policy is set earlier, the
    /// menubar is initially unresponsive on macOS 10.15 for example.
    pub activation_policy: ActivationPolicy,

    pub create_default_menu: bool,
}

/// Apple constants
#[allow(non_upper_case_globals)]
pub const kInternetEventClass: u32 = 0x4755524c;
#[allow(non_upper_case_globals)]
pub const kAEGetURL: u32 = 0x4755524c;
#[allow(non_upper_case_globals)]
pub const keyDirectObject: u32 = 0x2d2d2d2d;

pub struct AppDelegateClass(pub *const Class);
unsafe impl Send for AppDelegateClass {}
unsafe impl Sync for AppDelegateClass {}

lazy_static! {
    pub static ref APP_DELEGATE_CLASS: AppDelegateClass = unsafe {
        let superclass = class!(NSResponder);
        let mut decl = ClassDecl::new("WinitAppDelegate", superclass).unwrap();

        decl.add_class_method(sel!(new), new as extern "C" fn(&Class, Sel) -> id);
        decl.add_method(sel!(dealloc), dealloc as extern "C" fn(&Object, Sel));

        decl.add_method(
            sel!(applicationWillFinishLaunching:),
            will_finish_launching as extern "C" fn(&Object, Sel, id),
        );
        decl.add_method(
            sel!(applicationDidFinishLaunching:),
            did_finish_launching as extern "C" fn(&Object, Sel, id),
        );
        decl.add_ivar::<*mut c_void>(AUX_DELEGATE_STATE_NAME);
        decl.add_method(
            sel!(handleEvent:withReplyEvent:),
            handle_url
                as extern "C" fn(
                    &objc::runtime::Object,
                    _cmd: objc::runtime::Sel,
                    event: *mut Object,
                    _reply: u64,
                ),
        );

        AppDelegateClass(decl.register())
    };
}

/// Safety: Assumes that Object is an instance of APP_DELEGATE_CLASS
pub unsafe fn get_aux_state_mut(this: &Object) -> RefMut<'_, AuxDelegateState> {
    let ptr: *mut c_void = *this.get_ivar(AUX_DELEGATE_STATE_NAME);
    // Watch out that this needs to be the correct type
    (*(ptr as *mut RefCell<AuxDelegateState>)).borrow_mut()
}

extern "C" fn new(class: &Class, _: Sel) -> id {
    unsafe {
        let this: id = msg_send![class, alloc];
        let this: id = msg_send![this, init];
        (*this).set_ivar(
            AUX_DELEGATE_STATE_NAME,
            Box::into_raw(Box::new(RefCell::new(AuxDelegateState {
                activation_policy: ActivationPolicy::Regular,
                create_default_menu: true,
            }))) as *mut c_void,
        );
        this
    }
}

extern "C" fn dealloc(this: &Object, _: Sel) {
    unsafe {
        let state_ptr: *mut c_void = *(this.get_ivar(AUX_DELEGATE_STATE_NAME));
        // As soon as the box is constructed it is immediately dropped, releasing the underlying
        // memory
        Box::from_raw(state_ptr as *mut RefCell<AuxDelegateState>);
    }
}

fn parse_url(event: *mut Object) -> Option<String> {
    unsafe {
        let class: u32 = msg_send![event, eventClass];
        let id: u32 = msg_send![event, eventID];
        if class != kInternetEventClass || id != kAEGetURL {
            return None;
        }
        let subevent: *mut Object = msg_send![event, paramDescriptorForKeyword: keyDirectObject];
        let nsstring: *mut Object = msg_send![subevent, stringValue];

        let cstr: *const i8 = msg_send![nsstring, UTF8String];
        if cstr != std::ptr::null() {
            Some(
                std::ffi::CStr::from_ptr(cstr)
                    .to_string_lossy()
                    .into_owned(),
            )
        } else {
            None
        }
    }
}

extern "C" fn handle_url(
    _this: &objc::runtime::Object,
    _cmd: objc::runtime::Sel,
    event: *mut Object,
    _reply: u64,
) {
    if let Some(string) = parse_url(event) {
        AppState::queue_event(EventWrapper::StaticEvent(Event::PlatformSpecific(
            PlatformSpecific::MacOS(MacOS::ReceivedUrl(string)),
        )));
    }
}

extern "C" fn will_finish_launching(this: &Object, _: Sel, _: id) {
    trace!("Triggered `applicationWillFinishLaunching`");
    unsafe {
        let event_manager = class!(NSAppleEventManager);
        let shared_manager: *mut Object = msg_send![event_manager, sharedAppleEventManager];
        let () = msg_send![shared_manager,
                    setEventHandler: this
                    andSelector: sel!(handleEvent:withReplyEvent:)
                    forEventClass: kInternetEventClass
                    andEventID: kAEGetURL
        ];
    }
    trace!("Completed `applicationWillFinishLaunching`");
}

extern "C" fn did_finish_launching(this: &Object, _: Sel, _: id) {
    trace!("Triggered `applicationDidFinishLaunching`");
    AppState::launched(this);
    trace!("Completed `applicationDidFinishLaunching`");
}
