/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use core;
use eutil::Downcast;
use interfaces::{CefBrowser, CefBrowserHost, CefClient, cef_browser_host_t, cef_client_t};
use types::{KEYEVENT_CHAR, KEYEVENT_KEYDOWN, KEYEVENT_KEYUP, KEYEVENT_RAWKEYDOWN, cef_key_event};
use types::{cef_mouse_button_type_t, cef_mouse_event, cef_rect_t};

use compositing::windowing::{InitializeCompositingWindowEvent, KeyEvent, MouseWindowClickEvent};
use compositing::windowing::{MouseWindowEventClass, MouseWindowMouseUpEvent, PinchZoomWindowEvent};
use compositing::windowing::{ResizeWindowEvent, ScrollWindowEvent};
use geom::point::TypedPoint2D;
use geom::size::TypedSize2D;
use libc::{c_double, c_int};
use servo_msg::constellation_msg::{mod, KeyModifiers, Pressed, Released, Repeated};
use std::cell::RefCell;

pub struct ServoCefBrowserHost {
    /// A reference to the browser.
    pub browser: RefCell<Option<CefBrowser>>,
    /// A reference to the client.
    pub client: CefClient,
}

cef_class_impl! {
    ServoCefBrowserHost : CefBrowserHost, cef_browser_host_t {
        fn get_client(&this) -> *mut cef_client_t {
            this.downcast().client.clone()
        }

        fn was_resized(&this) -> () {
            let mut rect = cef_rect_t::zero();
            this.get_client()
                .get_render_handler()
                .get_backing_rect(this.downcast().browser.borrow().clone().unwrap(), &mut rect);
            let size = TypedSize2D(rect.width as uint, rect.height as uint);
            core::send_window_event(ResizeWindowEvent(size));
            core::repaint_synchronously();
        }

        fn send_key_event(&_this, event: *const cef_key_event) -> () {
            // FIXME(pcwalton): So awful. But it's nearly midnight here and I have to get
            // Google working.
            let event: &cef_key_event = event;
            let key = match (*event).character as u8 {
                b'a' | b'A' => constellation_msg::KeyA,
                b'b' | b'B' => constellation_msg::KeyB,
                b'c' | b'C' => constellation_msg::KeyC,
                b'd' | b'D' => constellation_msg::KeyD,
                b'e' | b'E' => constellation_msg::KeyE,
                b'f' | b'F' => constellation_msg::KeyF,
                b'g' | b'G' => constellation_msg::KeyG,
                b'h' | b'H' => constellation_msg::KeyH,
                b'i' | b'I' => constellation_msg::KeyI,
                b'j' | b'J' => constellation_msg::KeyJ,
                b'k' | b'K' => constellation_msg::KeyK,
                b'l' | b'L' => constellation_msg::KeyL,
                b'm' | b'M' => constellation_msg::KeyM,
                b'n' | b'N' => constellation_msg::KeyN,
                b'o' | b'O' => constellation_msg::KeyO,
                b'p' | b'P' => constellation_msg::KeyP,
                b'q' | b'Q' => constellation_msg::KeyQ,
                b'r' | b'R' => constellation_msg::KeyR,
                b's' | b'S' => constellation_msg::KeyS,
                b't' | b'T' => constellation_msg::KeyT,
                b'u' | b'U' => constellation_msg::KeyU,
                b'v' | b'V' => constellation_msg::KeyV,
                b'w' | b'W' => constellation_msg::KeyW,
                b'x' | b'X' => constellation_msg::KeyX,
                b'y' | b'Y' => constellation_msg::KeyY,
                b'z' | b'Z' => constellation_msg::KeyZ,
                b'0' => constellation_msg::Key0,
                b'1' => constellation_msg::Key1,
                b'2' => constellation_msg::Key2,
                b'3' => constellation_msg::Key3,
                b'4' => constellation_msg::Key4,
                b'5' => constellation_msg::Key5,
                b'6' => constellation_msg::Key6,
                b'7' => constellation_msg::Key7,
                b'8' => constellation_msg::Key8,
                b'9' => constellation_msg::Key9,
                b'\n' | b'\r' => constellation_msg::KeyEnter,
                _ => constellation_msg::KeySpace,
            };
            let key_state = match (*event).t {
                KEYEVENT_RAWKEYDOWN => Pressed,
                KEYEVENT_KEYDOWN | KEYEVENT_CHAR => Repeated,
                KEYEVENT_KEYUP => Released,
            };
            let key_modifiers = KeyModifiers::empty();  // TODO(pcwalton)
            core::send_window_event(KeyEvent(key, key_state, key_modifiers))
        }

        fn send_mouse_click_event(&_this,
                                  event: *const cef_mouse_event,
                                  mouse_button_type: cef_mouse_button_type_t,
                                  mouse_up: c_int,
                                  _click_count: c_int)
                                  -> () {
            let event: &cef_mouse_event = event;
            let button_type = mouse_button_type as uint;
            let point = TypedPoint2D((*event).x as f32, (*event).y as f32);
            if mouse_up != 0 {
                core::send_window_event(MouseWindowEventClass(MouseWindowClickEvent(button_type,
                                                                                    point)))
            } else {
                core::send_window_event(MouseWindowEventClass(MouseWindowMouseUpEvent(button_type,
                                                                                      point)))
            }
        }

        fn send_mouse_wheel_event(&_this,
                                  event: *const cef_mouse_event,
                                  delta_x: c_int,
                                  delta_y: c_int)
                                  -> () {
            let event: &cef_mouse_event = event;
            let delta = TypedPoint2D(delta_x as f32, delta_y as f32);
            let origin = TypedPoint2D((*event).x as i32, (*event).y as i32);
            core::send_window_event(ScrollWindowEvent(delta, origin))
        }

        fn get_zoom_level(&_this) -> c_double {
            core::pinch_zoom_level() as c_double
        }

        fn set_zoom_level(&this, new_zoom_level: c_double) -> () {
            let old_zoom_level = this.get_zoom_level();
            core::send_window_event(PinchZoomWindowEvent((new_zoom_level / old_zoom_level) as f32))
        }

        fn initialize_compositing(&_this) -> () {
            core::send_window_event(InitializeCompositingWindowEvent);
        }
    }
}

impl ServoCefBrowserHost {
    pub fn new(client: CefClient) -> ServoCefBrowserHost {
        ServoCefBrowserHost {
            browser: RefCell::new(None),
            client: client,
        }
    }
}

pub trait ServoCefBrowserHostExtensions {
    fn set_browser(&self, browser: CefBrowser);
}

impl ServoCefBrowserHostExtensions for CefBrowserHost {
    fn set_browser(&self, browser: CefBrowser) {
        *self.downcast().browser.borrow_mut() = Some(browser)
    }
}

