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
use types::{cef_cursor_handle_t, cef_cursor_type_t, cef_rect_t, CefScreenInfo};
use wrappers::CefWrap;

use compositing::compositor_thread::EventLoopWaker;
use compositing::windowing::{WindowEvent, WindowMethods};
use euclid::{Point2D, TypedPoint2D, Size2D, TypedSize2D, TypedScale};
use gleam::gl;
use msg::constellation_msg::{Key, KeyModifiers};
use net_traits::net_error_list::NetError;
use script_traits::LoadData;
use servo::BrowserId;
use servo::ipc_channel::ipc::IpcSender;
use servo_geometry::DeviceIndependentPixel;
use std::cell::RefCell;
use std::ffi::CString;
use std::os::raw::{c_char, c_void};
use std::ptr;
use std::rc::Rc;
use servo_url::ServoUrl;
use style_traits::cursor::Cursor;
use style_traits::DevicePixel;
#[cfg(target_os="linux")]
extern crate x11;
#[cfg(target_os="linux")]
use self::x11::xlib::{XInitThreads,XOpenDisplay};
use webrender_api::{DeviceUintSize, DeviceUintRect};

#[cfg(target_os="linux")]
pub static mut DISPLAY: *mut c_void = 0 as *mut c_void;

/// The type of an off-screen window.
#[derive(Clone)]
pub struct Window {
    cef_browser: RefCell<Option<CefBrowser>>,
    size: DeviceUintSize,
    gl: Rc<gl::Gl>,
}

#[cfg(target_os="macos")]
fn load_gl() -> Rc<gl::Gl> {
    const RTLD_DEFAULT: *mut c_void = (-2isize) as usize as *mut c_void;

    extern {
        fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
    }

    unsafe {
        gl::GlFns::load_with(|s| {
            let c_str = CString::new(s).unwrap();
            dlsym(RTLD_DEFAULT, c_str.as_ptr()) as *const c_void
        })
    }
}

#[cfg(target_os="linux")]
fn load_gl() -> Rc<gl::Gl> {
    extern {
        fn glXGetProcAddress(symbol: *const c_char) -> *mut c_void;
    }

    unsafe {
        gl::GlFns::load_with(|s| {
            let c_str = CString::new(s).unwrap();
            glXGetProcAddress(c_str.as_ptr()) as *const c_void
        })
    }
}

impl Window {
    /// Creates a new window.
    pub fn new(width: u32, height: u32) -> Rc<Window> {
        let gl = load_gl();

        Rc::new(Window {
            cef_browser: RefCell::new(None),
            size: TypedSize2D::new(width, height),
            gl: gl,
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

    fn screen_info(&self) -> CefScreenInfo {
        let mut screen_info = CefScreenInfo::default();
        let browser = self.cef_browser.borrow();
        if let Some(ref browser) = *browser {
            browser.get_host()
                   .get_client()
                   .get_render_handler()
                   .get_screen_info(browser.clone(), &mut screen_info);
        }
        screen_info
    }

}

impl WindowMethods for Window {
    fn gl(&self) -> Rc<gl::Gl> {
        self.gl.clone()
    }

    fn framebuffer_size(&self) -> DeviceUintSize {
        let browser = self.cef_browser.borrow();
        match *browser {
            None => self.size,
            Some(ref browser) => {
                if browser.downcast().callback_executed.get() != true {
                    self.size
                } else {
                    let mut rect = cef_rect_t::zero();
                    rect.width = self.size.width as i32;
                    rect.height = self.size.height as i32;
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

                    DeviceUintSize::new(rect.width as u32, rect.height as u32)
                }
            }
        }
    }

    fn window_rect(&self) -> DeviceUintRect {
        let size = self.framebuffer_size();
        let origin = TypedPoint2D::zero();
        DeviceUintRect::new(origin, size)
    }

    fn size(&self) -> TypedSize2D<f32, DeviceIndependentPixel> {
        let browser = self.cef_browser.borrow();
        match *browser {
            None => TypedSize2D::new(400.0, 300.0),
            Some(ref browser) => {
                let mut rect = cef_rect_t::zero();
                browser.get_host()
                       .get_client()
                       .get_render_handler()
                       .get_view_rect((*browser).clone(), &mut rect);
                TypedSize2D::new(rect.width as f32, rect.height as f32)
            }
        }
    }

    fn client_window(&self, _: BrowserId) -> (Size2D<u32>, Point2D<i32>) {
        let size = self.size().to_untyped();
        let width = size.width as u32;
        let height = size.height as u32;
        //TODO get real window position
        (Size2D::new(width, height), Point2D::zero())
    }

    fn set_inner_size(&self, _: BrowserId, _size: Size2D<u32>) {

    }

    fn set_position(&self, _: BrowserId, _point: Point2D<i32>) {

    }

    fn set_fullscreen_state(&self, _: BrowserId, _state: bool) {
    }

    fn present(&self) {
        let browser = self.cef_browser.borrow();
        if let Some(ref browser) = *browser {
            if check_ptr_exist!(browser.get_host().get_client(), get_render_handler) &&
               check_ptr_exist!(browser.get_host().get_client().get_render_handler(), on_present) {
                browser.get_host().get_client().get_render_handler().on_present(browser.clone());
            }
        }
    }

    fn hidpi_factor(&self) -> TypedScale<f32, DeviceIndependentPixel, DevicePixel> {
        if cfg!(target_os="macos") {
            let browser = self.cef_browser.borrow();
            match *browser {
                None => TypedScale::new(1.0),
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
                    TypedScale::new(backing_rect.width as f32 / view_rect.width as f32)
                }
            }
        } else {
            // FIXME(zmike)
            // need to figure out a method for actually getting the scale factor instead of this nonsense
            TypedScale::new(1.0 as f32)
        }
    }

    fn create_event_loop_waker(&self) -> Box<EventLoopWaker> {
        struct CefEventLoopWaker;
        impl EventLoopWaker for CefEventLoopWaker {
            fn wake(&self) {
                app_wakeup();
            }
            fn clone(&self) -> Box<EventLoopWaker + Send> {
                Box::new(CefEventLoopWaker)
            }
        }
        Box::new(CefEventLoopWaker)
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

    fn set_favicon(&self, _: BrowserId, url: ServoUrl) {
        let browser = self.cef_browser.borrow();
        let browser = match *browser {
            None => return,
            Some(ref browser) => browser,
        };
        browser.downcast().favicons.borrow_mut().push(url.into_string());
    }

    fn status(&self, _: BrowserId, info: Option<String>) {
        let browser = self.cef_browser.borrow();
        let browser = match *browser {
            None => return,
            Some(ref browser) => browser,
        };
        let str = match info {
            Some(s) => s.encode_utf16().collect::<Vec<u16>>(),
            None => vec![]
        };

        if check_ptr_exist!(browser.get_host().get_client(), get_display_handler) &&
           check_ptr_exist!(browser.get_host().get_client().get_display_handler(), on_status_message) {
            browser.get_host().get_client().get_display_handler().on_status_message((*browser).clone(), str.as_slice());
        }
    }

    fn load_start(&self, _: BrowserId) {
        let browser = self.cef_browser.borrow();
        let browser = match *browser {
            None => return,
            Some(ref browser) => browser,
        };
        let back = browser.downcast().back.get();
        let forward = browser.downcast().forward.get();
        browser.downcast().loading.set(true);
        browser.downcast().favicons.borrow_mut().clear();
        if check_ptr_exist!(browser.get_host().get_client(), get_load_handler) &&
           check_ptr_exist!(browser.get_host().get_client().get_load_handler(), on_loading_state_change) {
            browser.get_host()
                   .get_client()
                   .get_load_handler()
                   .on_loading_state_change((*browser).clone(), 1i32, back as i32, forward as i32);
        }
    }

    fn load_end(&self, _: BrowserId) {
        // FIXME(pcwalton): The status code 200 is a lie.
        let browser = self.cef_browser.borrow();
        let browser = match *browser {
            None => return,
            Some(ref browser) => browser,
        };
        let back = browser.downcast().back.get();
        let forward = browser.downcast().forward.get();
        browser.downcast().loading.set(false);
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

    fn load_error(&self, _: BrowserId, code: NetError, url: String) {
        let browser = self.cef_browser.borrow();
        let browser = match *browser {
            None => return,
            Some(ref browser) => browser,
        };
        if check_ptr_exist!(browser.get_host().get_client(), get_load_handler) &&
           check_ptr_exist!(browser.get_host().get_client().get_load_handler(), on_load_error) {
            let utf16_chars: Vec<u16> = url.encode_utf16().collect();
            browser.get_host()
                   .get_client()
                   .get_load_handler()
                   .on_load_error((*browser).clone(), browser.get_main_frame(),
                   code, &[], utf16_chars.as_slice());
        }
    }

    fn head_parsed(&self, _: BrowserId) {
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

    fn set_page_title(&self, _: BrowserId, string: Option<String>) {
        let browser = self.cef_browser.borrow();
        let browser = match *browser {
            None => return,
            Some(ref browser) => browser,
        };
        let frame = browser.get_main_frame();
        let frame = frame.downcast();
        let str = match string {
            Some(s) => s.encode_utf16().collect(),
            None => vec![]
        };

        if check_ptr_exist!(browser.get_host().get_client(), get_display_handler) &&
           check_ptr_exist!(browser.get_host().get_client().get_display_handler(), on_title_change) {
            browser.get_host().get_client().get_display_handler().on_title_change((*browser).clone(), str.as_slice());
        }

        *frame.title.borrow_mut() = str;
    }

    fn history_changed(&self, _: BrowserId, history: Vec<LoadData>, current: usize) {
        let browser = self.cef_browser.borrow();
        let browser = match *browser {
            None => return,
            Some(ref browser) => browser,
        };

        let can_go_back = current > 0;
        let can_go_forward = current < history.len() - 1;

        browser.downcast().back.set(can_go_back);
        browser.downcast().forward.set(can_go_forward);
        let frame = browser.get_main_frame();
        let mut frame_url = frame.downcast().url.borrow_mut();
        *frame_url = history[current].url.to_string();
        let utf16_chars: Vec<u16> = frame_url.encode_utf16().collect();
        if check_ptr_exist!(browser.get_host().get_client(), get_display_handler) &&
           check_ptr_exist!(browser.get_host().get_client().get_display_handler(), on_address_change) {
            browser.get_host().get_client().get_display_handler().on_address_change((*browser).clone(), frame.clone(), utf16_chars.as_slice());
        }
    }

    fn handle_key(&self, _: Option<BrowserId>, _: Option<char>, _: Key, _: KeyModifiers) {
        // TODO(negge)
    }

    fn set_cursor(&self, cursor: Cursor) {
        use types::{CefCursorInfo,cef_point_t,cef_size_t};
        let browser = self.cef_browser.borrow();
        if let Some(ref browser) = *browser {
            let cursor_handle = self.cursor_handle_for_cursor(cursor);
            let info = CefCursorInfo { hotspot: cef_point_t {x: 0, y: 0}, image_scale_factor: 0.0, buffer: 0 as *mut isize, size: cef_size_t { width: 0, height: 0 } };
            if check_ptr_exist!(browser.get_host().get_client(), get_render_handler) &&
               check_ptr_exist!(browser.get_host().get_client().get_render_handler(), on_cursor_change) {
                browser.get_host()
                       .get_client()
                       .get_render_handler()
                       .on_cursor_change(browser.clone(), cursor_handle,
                                         self.cursor_type_for_cursor(cursor), &info);
            }
        }
    }

    fn allow_navigation(&self, _: BrowserId, _: ServoUrl, response_chan: IpcSender<bool>) {
        if let Err(e) = response_chan.send(true) {
            warn!("Failed to send allow_navigation() response: {}", e);
        };
    }

    fn supports_clipboard(&self) -> bool {
        false
    }

    fn screen_size(&self, _: BrowserId) -> Size2D<u32> {
        let screen_info = self.screen_info();
        Size2D::new(screen_info.rect.width as u32, screen_info.rect.height as u32)
    }

    fn screen_avail_size(&self, _: BrowserId) -> Size2D<u32> {
        let screen_info = self.screen_info();
        Size2D::new(screen_info.available_rect.width as u32, screen_info.available_rect.height as u32)
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
