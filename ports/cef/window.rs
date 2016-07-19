/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Off-screen windows.
//!
//! This is used for off-screen rendering mode only; on-screen windows (the default embedding mode)
//! are managed by a platform toolkit (Glutin).

#[cfg(target_os="linux")]
use core::CEF_APP;
use eutil::Downcast;
#[cfg(target_os="linux")]
use interfaces::CefApp;
use interfaces::CefBrowser;
use render_handler::CefRenderHandlerExtensions;
use rustc_unicode::str::Utf16Encoder;
use types::{cef_cursor_handle_t, cef_cursor_type_t, cef_rect_t};
use wrappers::CefWrap;

use compositing::compositor_thread::{self, CompositorProxy, CompositorReceiver};
use compositing::windowing::{WindowEvent, WindowMethods};
use euclid::point::Point2D;
use euclid::scale_factor::ScaleFactor;
use euclid::size::{Size2D, TypedSize2D};
use gleam::gl;
use layers::geometry::DevicePixel;
use layers::platform::surface::NativeDisplay;
use msg::constellation_msg::{Key, KeyModifiers};
use net_traits::net_error_list::NetError;
use std::cell::RefCell;
use std::ffi::CString;
use std::os::raw::{c_char, c_void};
use std::ptr;
use std::rc::Rc;
use std::sync::mpsc::{Sender, channel};
use std_url::Url;
use style_traits::cursor::Cursor;
use util::geometry::ScreenPx;
#[cfg(target_os="linux")]
extern crate x11;
#[cfg(target_os="linux")]
use self::x11::xlib::{XInitThreads,XOpenDisplay};

#[cfg(target_os="linux")]
pub static mut DISPLAY: *mut c_void = 0 as *mut c_void;

/// The type of an off-screen window.
#[derive(Clone)]
pub struct Window {
    cef_browser: RefCell<Option<CefBrowser>>,
    size: TypedSize2D<DevicePixel,u32>
}

#[cfg(target_os="macos")]
fn load_gl() {
    const RTLD_DEFAULT: *mut c_void = (-2isize) as usize as *mut c_void;

    extern {
        fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
    }

    gl::load_with(|s| {
        unsafe {
            let c_str = CString::new(s).unwrap();
            dlsym(RTLD_DEFAULT, c_str.as_ptr()) as *const c_void
        }
    });
}

#[cfg(target_os="linux")]
fn load_gl() {
    extern {
        fn glXGetProcAddress(symbol: *const c_char) -> *mut c_void;
    }

    gl::load_with(|s| {
        unsafe {
            let c_str = CString::new(s).unwrap();
            glXGetProcAddress(c_str.as_ptr()) as *const c_void
        }
    });
}

impl Window {
    /// Creates a new window.
    pub fn new(width: u32, height: u32) -> Rc<Window> {
        load_gl();

        Rc::new(Window {
            cef_browser: RefCell::new(None),
            size: Size2D::typed(width, height)
        })
    }

    /// Sets the current browser.
    pub fn set_browser(&self, browser: CefBrowser) {
        *self.cef_browser.borrow_mut() = Some(browser)
    }

    /// Currently unimplemented.
    pub fn wait_events(&self) -> Vec<WindowEvent> {
        vec![WindowEvent::Idle]
    }

    fn cursor_type_for_cursor(&self, cursor: Cursor) -> cef_cursor_type_t {
        match cursor {
            Cursor::None => return cef_cursor_type_t::CT_NONE,
            Cursor::ContextMenu => return cef_cursor_type_t::CT_CONTEXTMENU,
            Cursor::Grabbing => return cef_cursor_type_t::CT_GRABBING,
            Cursor::Crosshair => return cef_cursor_type_t::CT_CROSS,
            Cursor::Copy => return cef_cursor_type_t::CT_COPY,
            Cursor::Alias => return cef_cursor_type_t::CT_ALIAS,
            Cursor::Text => return cef_cursor_type_t::CT_IBEAM,
            Cursor::Grab | Cursor::AllScroll =>
                return cef_cursor_type_t::CT_GRAB,
            Cursor::NoDrop => return cef_cursor_type_t::CT_NODROP,
            Cursor::NotAllowed => return cef_cursor_type_t::CT_NOTALLOWED,
            Cursor::Pointer => return cef_cursor_type_t::CT_POINTER,
            Cursor::SResize => return cef_cursor_type_t::CT_SOUTHRESIZE,
            Cursor::WResize => return cef_cursor_type_t::CT_WESTRESIZE,
            Cursor::EwResize => return cef_cursor_type_t::CT_EASTWESTRESIZE,
            Cursor::ColResize => return cef_cursor_type_t::CT_COLUMNRESIZE,
            Cursor::EResize => return cef_cursor_type_t::CT_EASTRESIZE,
            Cursor::NResize => return cef_cursor_type_t::CT_NORTHRESIZE,
            Cursor::NsResize => return cef_cursor_type_t::CT_NORTHSOUTHRESIZE,
            Cursor::RowResize => return cef_cursor_type_t::CT_ROWRESIZE,
            Cursor::VerticalText => return cef_cursor_type_t::CT_VERTICALTEXT,
            _ => return cef_cursor_type_t::CT_POINTER,
        }
    }

    /// Returns the Cocoa cursor for a CSS cursor. These match Firefox, except where Firefox
    /// bundles custom resources (which we don't yet do).
    #[cfg(target_os="macos")]
    fn cursor_handle_for_cursor(&self, cursor: Cursor) -> cef_cursor_handle_t {
        use cocoa::base::class;

        unsafe {
            match cursor {
                Cursor::None => return 0 as cef_cursor_handle_t,
                Cursor::ContextMenu => msg_send![class("NSCursor"), contextualMenuCursor],
                Cursor::Grabbing => msg_send![class("NSCursor"), closedHandCursor],
                Cursor::Crosshair => msg_send![class("NSCursor"), crosshairCursor],
                Cursor::Copy => msg_send![class("NSCursor"), dragCopyCursor],
                Cursor::Alias => msg_send![class("NSCursor"), dragLinkCursor],
                Cursor::Text => msg_send![class("NSCursor"), IBeamCursor],
                Cursor::Grab | Cursor::AllScroll =>
                    msg_send![class("NSCursor"), openHandCursor],
                Cursor::NoDrop | Cursor::NotAllowed =>
                    msg_send![class("NSCursor"), operationNotAllowedCursor],
                Cursor::Pointer => msg_send![class("NSCursor"), pointingHandCursor],
                Cursor::SResize => msg_send![class("NSCursor"), resizeDownCursor],
                Cursor::WResize => msg_send![class("NSCursor"), resizeLeftCursor],
                Cursor::EwResize | Cursor::ColResize =>
                    msg_send![class("NSCursor"), resizeLeftRightCursor],
                Cursor::EResize => msg_send![class("NSCursor"), resizeRightCursor],
                Cursor::NResize => msg_send![class("NSCursor"), resizeUpCursor],
                Cursor::NsResize | Cursor::RowResize =>
                    msg_send![class("NSCursor"), resizeUpDownCursor],
                Cursor::VerticalText => msg_send![class("NSCursor"), IBeamCursorForVerticalLayout],
                _ => msg_send![class("NSCursor"), arrowCursor],
            }
        }
    }

    #[cfg(not(target_os="macos"))]
    fn cursor_handle_for_cursor(&self, _: Cursor) -> cef_cursor_handle_t {
        0
    }
}

impl WindowMethods for Window {
    fn framebuffer_size(&self) -> TypedSize2D<DevicePixel,u32> {
        let browser = self.cef_browser.borrow();
        match *browser {
            None => self.size,
            Some(ref browser) => {
                if browser.downcast().callback_executed.get() != true {
                    self.size
                } else {
                    let mut rect = cef_rect_t::zero();
                    rect.width = self.size.width.get() as i32;
                    rect.height = self.size.height.get() as i32;
                    if cfg!(target_os="macos") {
                        // osx relies on virtual pixel scaling to provide sizes different from actual
                        // pixel size on screen. other platforms are just 1.0 unless the desktop/toolkit says otherwise
                        if check_ptr_exist!(browser.get_host().get_client(), get_render_handler) &&
                           check_ptr_exist!(browser.get_host().get_client().get_render_handler(), get_backing_rect) {
                            browser.get_host()
                                   .get_client()
                                   .get_render_handler()
                                   .get_backing_rect((*browser).clone(), &mut rect);
                        }
                    } else {
                        if check_ptr_exist!(browser.get_host().get_client(), get_render_handler) &&
                           check_ptr_exist!(browser.get_host().get_client().get_render_handler(), get_view_rect) {
                            browser.get_host()
                                   .get_client()
                                   .get_render_handler()
                                   .get_view_rect((*browser).clone(), &mut rect);
                        }
                    }

                    Size2D::typed(rect.width as u32, rect.height as u32)
                }
            }
        }
    }

    fn size(&self) -> TypedSize2D<ScreenPx,f32> {
        let browser = self.cef_browser.borrow();
        match *browser {
            None => Size2D::typed(400.0, 300.0),
            Some(ref browser) => {
                let mut rect = cef_rect_t::zero();
                browser.get_host()
                       .get_client()
                       .get_render_handler()
                       .get_view_rect((*browser).clone(), &mut rect);
                Size2D::typed(rect.width as f32, rect.height as f32)
            }
        }
    }

    fn client_window(&self) -> (Size2D<u32>, Point2D<i32>) {
        let size = self.size().to_untyped();
        let width = size.width as u32;
        let height = size.height as u32;
        //TODO get real window position
        (Size2D::new(width, height), Point2D::zero())
    }

    fn set_inner_size(&self, _size: Size2D<u32>) {

    }

    fn set_position(&self, _point: Point2D<i32>) {

    }

    fn present(&self) {
        let browser = self.cef_browser.borrow();
        match *browser {
            None => {}
            Some(ref browser) => {
                if check_ptr_exist!(browser.get_host().get_client(), get_render_handler) &&
                   check_ptr_exist!(browser.get_host().get_client().get_render_handler(), on_present) {
                    browser.get_host().get_client().get_render_handler().on_present(browser.clone());
                   }
            }
        }
    }

    fn scale_factor(&self) -> ScaleFactor<ScreenPx,DevicePixel,f32> {
        if cfg!(target_os="macos") {
            let browser = self.cef_browser.borrow();
            match *browser {
                None => ScaleFactor::new(1.0),
                Some(ref browser) => {
                    let mut view_rect = cef_rect_t::zero();
                    if check_ptr_exist!(browser.get_host().get_client(), get_render_handler) &&
                       check_ptr_exist!(browser.get_host().get_client().get_render_handler(), get_view_rect) {
                        browser.get_host()
                               .get_client()
                               .get_render_handler()
                               .get_view_rect((*browser).clone(), &mut view_rect);
                    }
                    let mut backing_rect = cef_rect_t::zero();
                    if check_ptr_exist!(browser.get_host().get_client(), get_render_handler) &&
                       check_ptr_exist!(browser.get_host().get_client().get_render_handler(), get_backing_rect) {
                        browser.get_host()
                               .get_client()
                               .get_render_handler()
                               .get_backing_rect((*browser).clone(), &mut backing_rect);
                    }
                    ScaleFactor::new(backing_rect.width as f32 / view_rect.width as f32)
                }
            }
        } else {
            // FIXME(zmike)
            // need to figure out a method for actually getting the scale factor instead of this nonsense
            ScaleFactor::new(1.0 as f32)
        }
    }

    #[cfg(target_os="linux")]
    fn native_display(&self) -> NativeDisplay {
        use x11::xlib;
        unsafe {
            NativeDisplay::new(DISPLAY as *mut xlib::Display)
        }
    }

    #[cfg(not(target_os="linux"))]
    fn native_display(&self) -> NativeDisplay {
        NativeDisplay::new()
    }

    fn create_compositor_channel(&self)
                                 -> (Box<CompositorProxy+Send>, Box<CompositorReceiver>) {
        let (sender, receiver) = channel();
        (box CefCompositorProxy {
             sender: sender,
         } as Box<CompositorProxy+Send>,
         box receiver as Box<CompositorReceiver>)
    }

    fn prepare_for_composite(&self, width: usize, height: usize) -> bool {
        let browser = self.cef_browser.borrow();
        match *browser {
            None => {
                panic!("No browser?!?");
            }
            Some(ref browser) => {
                if browser.downcast().host.downcast().composite_ok.get() == true {
                    true
                } else {
                    if check_ptr_exist!(browser.get_host().get_client(), get_render_handler) &&
                       check_ptr_exist!(browser.get_host().get_client().get_render_handler(), on_paint) {
                        browser.get_host().get_client().get_render_handler().paint(browser.clone(), width, height);
                    }
                    false
                }
            }
        }
    }

    fn set_favicon(&self, url: Url) {
        let browser = self.cef_browser.borrow();
        let browser = match *browser {
            None => return,
            Some(ref browser) => browser,
        };
        browser.downcast().favicons.borrow_mut().push(url.to_string().clone());
    }

    fn status(&self, info: Option<String>) {
        let browser = self.cef_browser.borrow();
        let browser = match *browser {
            None => return,
            Some(ref browser) => browser,
        };
        let str = match info {
            Some(s) => {
                let utf16_chars: Vec<u16> = Utf16Encoder::new(s.chars()).collect();
                utf16_chars
            }
            None => vec![]
        };

        if check_ptr_exist!(browser.get_host().get_client(), get_display_handler) &&
           check_ptr_exist!(browser.get_host().get_client().get_display_handler(), on_status_message) {
            browser.get_host().get_client().get_display_handler().on_status_message((*browser).clone(), str.as_slice());
        }
    }

    fn load_start(&self, back: bool, forward: bool) {
        let browser = self.cef_browser.borrow();
        let browser = match *browser {
            None => return,
            Some(ref browser) => browser,
        };
        browser.downcast().loading.set(true);
        browser.downcast().back.set(back);
        browser.downcast().forward.set(forward);
        browser.downcast().favicons.borrow_mut().clear();
        if check_ptr_exist!(browser.get_host().get_client(), get_load_handler) &&
           check_ptr_exist!(browser.get_host().get_client().get_load_handler(), on_loading_state_change) {
            browser.get_host()
                   .get_client()
                   .get_load_handler()
                   .on_loading_state_change((*browser).clone(), 1i32, back as i32, forward as i32);
        }
    }

    fn load_end(&self, back: bool, forward: bool, _: bool) {
        // FIXME(pcwalton): The status code 200 is a lie.
        let browser = self.cef_browser.borrow();
        let browser = match *browser {
            None => return,
            Some(ref browser) => browser,
        };
        browser.downcast().loading.set(false);
        browser.downcast().back.set(back);
        browser.downcast().forward.set(forward);
        if check_ptr_exist!(browser.get_host().get_client(), get_load_handler) &&
           check_ptr_exist!(browser.get_host().get_client().get_load_handler(), on_loading_state_change) {
            browser.get_host()
                   .get_client()
                   .get_load_handler()
                   .on_loading_state_change((*browser).clone(), 0i32, back as i32, forward as i32);
        }
        if check_ptr_exist!(browser.get_host().get_client(), get_load_handler) &&
           check_ptr_exist!(browser.get_host().get_client().get_load_handler(), on_load_end) {
            browser.get_host()
                   .get_client()
                   .get_load_handler()
                   .on_load_end((*browser).clone(), browser.get_main_frame(), 200);
        }
    }

    fn load_error(&self, code: NetError, url: String) {
        let browser = self.cef_browser.borrow();
        let browser = match *browser {
            None => return,
            Some(ref browser) => browser,
        };
        if check_ptr_exist!(browser.get_host().get_client(), get_load_handler) &&
           check_ptr_exist!(browser.get_host().get_client().get_load_handler(), on_load_error) {
            let utf16_chars: Vec<u16> = Utf16Encoder::new((url).chars()).collect();
            browser.get_host()
                   .get_client()
                   .get_load_handler()
                   .on_load_error((*browser).clone(), browser.get_main_frame(),
                   code, &[], utf16_chars.as_slice());
        }
    }

    fn head_parsed(&self) {
        let browser = self.cef_browser.borrow();
        let browser = match *browser {
            None => return,
            Some(ref browser) => browser,
        };
        if check_ptr_exist!(browser.get_host().get_client(), get_display_handler) &&
           check_ptr_exist!(browser.get_host().get_client().get_display_handler(), on_favicon_urlchange) {
            browser.get_host().get_client().get_display_handler().on_favicon_urlchange((*browser).clone(), &browser.downcast().favicons.borrow());
        }
    }

    fn set_page_title(&self, string: Option<String>) {
        let browser = self.cef_browser.borrow();
        let browser = match *browser {
            None => return,
            Some(ref browser) => browser,
        };
        let frame = browser.get_main_frame();
        let frame = frame.downcast();
        let mut title_visitor = frame.title_visitor.borrow_mut();
        let str = match string {
            Some(s) => {
                let utf16_chars: Vec<u16> = Utf16Encoder::new(s.chars()).collect();
                utf16_chars
            }
            None => vec![]
        };

        if check_ptr_exist!(browser.get_host().get_client(), get_display_handler) &&
           check_ptr_exist!(browser.get_host().get_client().get_display_handler(), on_title_change) {
            browser.get_host().get_client().get_display_handler().on_title_change((*browser).clone(), str.as_slice());
        }
        match &mut *title_visitor {
            &mut None => {},
            &mut Some(ref mut visitor) => {
                visitor.visit(&str);
            }
        };
    }

    fn set_page_url(&self, url: Url) {
        // it seems to be the case that load start is always called
        // IMMEDIATELY before address change, so just stick it here
        on_load_start(self);
        let browser = self.cef_browser.borrow();
        let browser = match *browser {
            None => return,
            Some(ref browser) => browser,
        };
        let frame = browser.get_main_frame();
        let servoframe = frame.downcast();
        // FIXME(https://github.com/rust-lang/rust/issues/23338)
        let mut frame_url = servoframe.url.borrow_mut();
        *frame_url = url.to_string();
        let utf16_chars: Vec<u16> = Utf16Encoder::new((*frame_url).chars()).collect();
        if check_ptr_exist!(browser.get_host().get_client(), get_display_handler) &&
           check_ptr_exist!(browser.get_host().get_client().get_display_handler(), on_address_change) {
            browser.get_host().get_client().get_display_handler().on_address_change((*browser).clone(), frame.clone(), utf16_chars.as_slice());
        }
    }

    fn handle_key(&self, _: Option<char>, _: Key, _: KeyModifiers) {
        // TODO(negge)
    }

    fn set_cursor(&self, cursor: Cursor) {
        use types::{CefCursorInfo,cef_point_t,cef_size_t};
        let browser = self.cef_browser.borrow();
        match *browser {
            None => {}
            Some(ref browser) => {
                let cursor_handle = self.cursor_handle_for_cursor(cursor);
                let info = CefCursorInfo { hotspot: cef_point_t {x: 0, y: 0}, image_scale_factor: 0.0, buffer: 0 as *mut isize, size: cef_size_t { width: 0, height: 0 } };
                if check_ptr_exist!(browser.get_host().get_client(), get_render_handler) &&
                   check_ptr_exist!(browser.get_host().get_client().get_render_handler(), on_cursor_change) {
                    browser.get_host()
                           .get_client()
                           .get_render_handler()
                           .on_cursor_change(browser.clone(), cursor_handle,
                             self.cursor_type_for_cursor(cursor), &info)
                   }
            }
        }
    }

    fn supports_clipboard(&self) -> bool {
        false
    }
}

struct CefCompositorProxy {
    sender: Sender<compositor_thread::Msg>,
}

impl CompositorProxy for CefCompositorProxy {
    fn send(&self, msg: compositor_thread::Msg) {
        self.sender.send(msg).unwrap();
        app_wakeup();
    }

    fn clone_compositor_proxy(&self) -> Box<CompositorProxy+Send> {
        box CefCompositorProxy {
            sender: self.sender.clone(),
        } as Box<CompositorProxy+Send>
    }
}

fn on_load_start(window: &Window) {
    let browser = window.cef_browser.borrow();
    let browser = match *browser {
        None => return,
        Some(ref browser) => browser,
    };
    if check_ptr_exist!(browser.get_host().get_client(), get_load_handler) &&
       check_ptr_exist!(browser.get_host().get_client().get_load_handler(), on_load_start) {
        browser.get_host()
               .get_client()
               .get_load_handler()
               .on_load_start((*browser).clone(), browser.get_main_frame());
    }
}

#[cfg(target_os="macos")]
pub fn app_wakeup() {
    use cocoa::appkit::{NSApp, NSApplication, NSApplicationDefined};
    use cocoa::appkit::{NSEvent, NSEventModifierFlags, NSEventSubtype};
    use cocoa::base::nil;
    use cocoa::foundation::{NSAutoreleasePool, NSPoint};

    unsafe {
        let pool = NSAutoreleasePool::new(nil);
        let event =
            NSEvent::otherEventWithType_location_modifierFlags_timestamp_windowNumber_context_subtype_data1_data2_(
            nil,
            NSApplicationDefined,
            NSPoint::new(0.0, 0.0),
            NSEventModifierFlags::empty(),
            0.0,
            0,
            nil,
            NSEventSubtype::NSWindowExposedEventType,
            0,
            0);
        NSApp().postEvent_atStart_(event, 0);
        pool.drain();
    }
}

#[cfg(target_os="linux")]
pub fn app_wakeup() {
    unsafe { if CEF_APP.is_null() { return; } }
    let capp = unsafe { CefApp::from_c_object_addref(CEF_APP) };
    if unsafe { (*CEF_APP).get_browser_process_handler.is_some() } &&
       check_ptr_exist!(capp.get_browser_process_handler(), on_work_available) {
        capp.get_browser_process_handler().on_work_available();
    }
}

#[cfg(target_os="linux")]
pub fn init_window() {
    unsafe {
        assert!(XInitThreads() != 0);
        DISPLAY = XOpenDisplay(ptr::null()) as *mut c_void;
    }
}
#[cfg(not(target_os="linux"))]
pub fn init_window() {}

#[cfg(target_os="linux")]
#[no_mangle]
pub extern "C" fn cef_get_xdisplay() -> *mut c_void {
    unsafe { DISPLAY }
}
#[cfg(not(target_os="linux"))]
#[no_mangle]
pub extern "C" fn cef_get_xdisplay() -> *mut c_void {
    ptr::null_mut()
}
