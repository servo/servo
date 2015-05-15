/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Off-screen windows.
//!
//! This is used for off-screen rendering mode only; on-screen windows (the default embedding mode)
//! are managed by a platform toolkit (Glutin).

use eutil::Downcast;
use interfaces::CefBrowser;
use render_handler::CefRenderHandlerExtensions;
use rustc_unicode::str::Utf16Encoder;
use types::{cef_cursor_handle_t, cef_cursor_type_t, cef_rect_t};

use compositing::compositor_task::{self, CompositorProxy, CompositorReceiver};
use compositing::windowing::{WindowEvent, WindowMethods};
use geom::scale_factor::ScaleFactor;
use geom::size::TypedSize2D;
use gleam::gl;
use layers::geometry::DevicePixel;
use layers::platform::surface::NativeGraphicsMetadata;
use libc::{c_char, c_void};
use msg::constellation_msg::{Key, KeyModifiers};
use std::ptr;
use std_url::Url;
use util::cursor::Cursor;
use util::geometry::ScreenPx;
use std::cell::RefCell;
use std::ffi::CString;
use std::rc::Rc;
use std::sync::mpsc::{Sender, channel};
#[cfg(target_os="linux")]
extern crate x11;
#[cfg(target_os="linux")]
use self::x11::xlib::XOpenDisplay;

#[cfg(target_os="linux")]
pub static mut DISPLAY: *mut c_void = 0 as *mut c_void;

/// The type of an off-screen window.
#[allow(raw_pointer_derive)]
#[derive(Clone)]
pub struct Window {
    cef_browser: RefCell<Option<CefBrowser>>,
}

#[cfg(target_os="macos")]
fn load_gl() {
    const RTLD_DEFAULT: *mut c_void = (-2) as *mut c_void;

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
    pub fn new() -> Rc<Window> {
        load_gl();

        Rc::new(Window {
            cef_browser: RefCell::new(None),
        })
    }

    /// Sets the current browser.
    pub fn set_browser(&self, browser: CefBrowser) {
        *self.cef_browser.borrow_mut() = Some(browser)
    }

    /// Currently unimplemented.
    pub fn wait_events(&self) -> WindowEvent {
        WindowEvent::Idle
    }

    fn cursor_type_for_cursor(&self, cursor: Cursor) -> cef_cursor_type_t {
        match cursor {
            Cursor::NoCursor => return cef_cursor_type_t::CT_NONE,
            Cursor::ContextMenuCursor => return cef_cursor_type_t::CT_CONTEXTMENU,
            Cursor::GrabbingCursor => return cef_cursor_type_t::CT_GRABBING,
            Cursor::CrosshairCursor => return cef_cursor_type_t::CT_CROSS,
            Cursor::CopyCursor => return cef_cursor_type_t::CT_COPY,
            Cursor::AliasCursor => return cef_cursor_type_t::CT_ALIAS,
            Cursor::TextCursor => return cef_cursor_type_t::CT_IBEAM,
            Cursor::GrabCursor | Cursor::AllScrollCursor =>
                return cef_cursor_type_t::CT_GRAB,
            Cursor::NoDropCursor => return cef_cursor_type_t::CT_NODROP,
            Cursor::NotAllowedCursor => return cef_cursor_type_t::CT_NOTALLOWED,
            Cursor::PointerCursor => return cef_cursor_type_t::CT_POINTER,
            Cursor::SResizeCursor => return cef_cursor_type_t::CT_SOUTHRESIZE,
            Cursor::WResizeCursor => return cef_cursor_type_t::CT_WESTRESIZE,
            Cursor::EwResizeCursor => return cef_cursor_type_t::CT_EASTWESTRESIZE,
            Cursor::ColResizeCursor => return cef_cursor_type_t::CT_COLUMNRESIZE,
            Cursor::EResizeCursor => return cef_cursor_type_t::CT_EASTRESIZE,
            Cursor::NResizeCursor => return cef_cursor_type_t::CT_NORTHRESIZE,
            Cursor::NsResizeCursor => return cef_cursor_type_t::CT_NORTHSOUTHRESIZE,
            Cursor::RowResizeCursor => return cef_cursor_type_t::CT_ROWRESIZE,
            Cursor::VerticalTextCursor => return cef_cursor_type_t::CT_VERTICALTEXT,
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
                Cursor::NoCursor => return 0 as cef_cursor_handle_t,
                Cursor::ContextMenuCursor => msg_send![class("NSCursor"), contextualMenuCursor],
                Cursor::GrabbingCursor => msg_send![class("NSCursor"), closedHandCursor],
                Cursor::CrosshairCursor => msg_send![class("NSCursor"), crosshairCursor],
                Cursor::CopyCursor => msg_send![class("NSCursor"), dragCopyCursor],
                Cursor::AliasCursor => msg_send![class("NSCursor"), dragLinkCursor],
                Cursor::TextCursor => msg_send![class("NSCursor"), IBeamCursor],
                Cursor::GrabCursor | Cursor::AllScrollCursor =>
                    msg_send![class("NSCursor"), openHandCursor],
                Cursor::NoDropCursor | Cursor::NotAllowedCursor => 
                    msg_send![class("NSCursor"), operationNotAllowedCursor],
                Cursor::PointerCursor => msg_send![class("NSCursor"), pointingHandCursor],
                Cursor::SResizeCursor => msg_send![class("NSCursor"), resizeDownCursor],
                Cursor::WResizeCursor => msg_send![class("NSCursor"), resizeLeftCursor],
                Cursor::EwResizeCursor | Cursor::ColResizeCursor => 
                    msg_send![class("NSCursor"), resizeLeftRightCursor],
                Cursor::EResizeCursor => msg_send![class("NSCursor"), resizeRightCursor],
                Cursor::NResizeCursor => msg_send![class("NSCursor"), resizeUpCursor],
                Cursor::NsResizeCursor | Cursor::RowResizeCursor => 
                    msg_send![class("NSCursor"), resizeUpDownCursor],
                Cursor::VerticalTextCursor => msg_send![class("NSCursor"), IBeamCursorForVerticalLayout],
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
            None => TypedSize2D(400, 300),
            Some(ref browser) => {
                let mut rect = cef_rect_t::zero();
                browser.get_host()
                       .get_client()
                       .get_render_handler()
                       .get_backing_rect((*browser).clone(), &mut rect);
                TypedSize2D(rect.width as u32, rect.height as u32)
            }
        }
    }

    fn size(&self) -> TypedSize2D<ScreenPx,f32> {
        let browser = self.cef_browser.borrow();
        match *browser {
            None => TypedSize2D(400.0, 300.0),
            Some(ref browser) => {
                let mut rect = cef_rect_t::zero();
                browser.get_host()
                       .get_client()
                       .get_render_handler()
                       .get_view_rect((*browser).clone(), &mut rect);
                TypedSize2D(rect.width as f32, rect.height as f32)
            }
        }
    }

    fn present(&self) {
        let browser = self.cef_browser.borrow();
        match *browser {
            None => {}
            Some(ref browser) => {
                browser.get_host().get_client().get_render_handler().on_present(browser.clone());
            }
        }
    }

    fn hidpi_factor(&self) -> ScaleFactor<ScreenPx,DevicePixel,f32> {
        let browser = self.cef_browser.borrow();
        match *browser {
            None => ScaleFactor::new(1.0),
            Some(ref browser) => {
                let mut view_rect = cef_rect_t::zero();
                browser.get_host()
                       .get_client()
                       .get_render_handler()
                       .get_view_rect((*browser).clone(), &mut view_rect);
                let mut backing_rect = cef_rect_t::zero();
                browser.get_host()
                       .get_client()
                       .get_render_handler()
                       .get_backing_rect((*browser).clone(), &mut backing_rect);
                ScaleFactor::new(backing_rect.width as f32 / view_rect.width as f32)
            }
        }
    }

    #[cfg(target_os="macos")]
    fn native_metadata(&self) -> NativeGraphicsMetadata {
        use cgl::{CGLGetCurrentContext, CGLGetPixelFormat};

        // FIXME(pcwalton)
        unsafe {
            NativeGraphicsMetadata {
                pixel_format: CGLGetPixelFormat(CGLGetCurrentContext()),
            }
        }
    }

    #[cfg(target_os="linux")]
    fn native_metadata(&self) -> NativeGraphicsMetadata {
       unsafe {
           NativeGraphicsMetadata {
               display: DISPLAY,
           }
       }
    }

    fn create_compositor_channel(_: &Option<Rc<Window>>)
                                 -> (Box<CompositorProxy+Send>, Box<CompositorReceiver>) {
        let (sender, receiver) = channel();
        (box CefCompositorProxy {
             sender: sender,
         } as Box<CompositorProxy+Send>,
         box receiver as Box<CompositorReceiver>)
    }

    fn prepare_for_composite(&self) -> bool {
        let browser = self.cef_browser.borrow();
        match *browser {
            None => {}
            Some(ref browser) => {
                browser.get_host().get_client().get_render_handler().paint(browser.clone());
            }
        }
        true
    }

    fn load_end(&self) {
        // FIXME(pcwalton): The status code 200 is a lie.
        let browser = self.cef_browser.borrow();
        let browser = match *browser {
            None => return,
            Some(ref browser) => browser,
        };
        browser.get_host()
               .get_client()
               .get_load_handler()
               .on_load_end((*browser).clone(), browser.get_main_frame(), 200);
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
        match &mut *title_visitor {
            &mut None => {}
            &mut Some(ref mut visitor) => {
                match string {
                    None => visitor.visit(&[]),
                    Some(string) => {
                        let utf16_chars: Vec<u16> = Utf16Encoder::new(string.chars()).collect();
                        visitor.visit(&utf16_chars)
                    }
                }
            }
        }
    }

    fn set_page_url(&self, url: Url) {
        let browser = self.cef_browser.borrow();
        let browser = match *browser {
            None => return,
            Some(ref browser) => browser,
        };
        let frame = browser.get_main_frame();
        let frame = frame.downcast();
        // FIXME(https://github.com/rust-lang/rust/issues/23338)
        let mut frame_url = frame.url.borrow_mut();
        *frame_url = url.to_string()
    }

    fn handle_key(&self, _: Key, _: KeyModifiers) {
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
                browser.get_host()
                       .get_client()
                       .get_render_handler()
                       .on_cursor_change(browser.clone(), cursor_handle,
                         self.cursor_type_for_cursor(cursor), &info)
            }
        }
    }
}

struct CefCompositorProxy {
    sender: Sender<compositor_task::Msg>,
}

impl CompositorProxy for CefCompositorProxy {
    #[cfg(target_os="macos")]
    fn send(&mut self, msg: compositor_task::Msg) {
        use cocoa::appkit::{NSApp, NSApplication, NSApplicationDefined};
        use cocoa::appkit::{NSEvent, NSEventModifierFlags, NSEventSubtype};
        use cocoa::base::nil;
        use cocoa::foundation::{NSAutoreleasePool, NSPoint};

        // Send a message and kick the OS event loop awake.
        self.sender.send(msg).unwrap();

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
    fn send(&mut self, msg: compositor_task::Msg) {
        // FIXME(pcwalton): Kick the GTK event loop awake?
        self.sender.send(msg).unwrap();
    }

    fn clone_compositor_proxy(&self) -> Box<CompositorProxy+Send> {
        box CefCompositorProxy {
            sender: self.sender.clone(),
        } as Box<CompositorProxy+Send>
    }
}

#[cfg(target_os="linux")]
pub fn init_window() {
    unsafe { DISPLAY = XOpenDisplay(ptr::null()) as *mut c_void ; }
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
