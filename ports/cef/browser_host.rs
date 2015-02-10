/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use eutil::Downcast;
use interfaces::{CefBrowser, CefBrowserHost, CefClient, cef_browser_host_t, cef_client_t};
use types::{cef_mouse_button_type_t, cef_mouse_event, cef_rect_t, cef_key_event};
use types::cef_key_event_type_t::{KEYEVENT_CHAR, KEYEVENT_KEYDOWN, KEYEVENT_KEYUP, KEYEVENT_RAWKEYDOWN};
use browser::{self, ServoCefBrowserExtensions};

use compositing::windowing::{WindowEvent, MouseWindowEvent};
use geom::point::TypedPoint2D;
use geom::size::TypedSize2D;
use libc::{c_double, c_int};
use msg::constellation_msg::{self, KeyModifiers, KeyState};
use std::cell::RefCell;

pub struct ServoCefBrowserHost {
    /// A reference to the browser.
    pub browser: RefCell<Option<CefBrowser>>,
    /// A reference to the client.
    pub client: CefClient,
}

full_cef_class_impl! {
    ServoCefBrowserHost : CefBrowserHost, cef_browser_host_t {
        fn get_client(&this,) -> *mut cef_client_t {{
            this.downcast().client.clone()
        }}

        fn was_resized(&this,) -> () {{
            let mut rect = cef_rect_t::zero();
            this.get_client()
                .get_render_handler()
                .get_backing_rect(this.downcast().browser.borrow().clone().unwrap(), &mut rect);
            let size = TypedSize2D(rect.width as u32, rect.height as u32);
            this.downcast().send_window_event(WindowEvent::Resize(size));
        }}

        fn close_browser(&this, _force: c_int [c_int],) -> () {{
            browser::close(this.downcast().browser.borrow_mut().take().unwrap());
        }}

        fn send_focus_event(&this, focus: c_int [c_int],) -> () {{
            let focus: c_int = focus;
            if focus != 0 {
                this.downcast().send_window_event(WindowEvent::Refresh);
            }
        }}

        fn send_key_event(&this, event: *const cef_key_event [&cef_key_event],) -> () {{
            // FIXME(pcwalton): So awful. But it's nearly midnight here and I have to get
            // Google working.
            let event: &cef_key_event = event;
            let key = match (*event).character as u8 {
                b'a' | b'A' => constellation_msg::Key::A,
                b'b' | b'B' => constellation_msg::Key::B,
                b'c' | b'C' => constellation_msg::Key::C,
                b'd' | b'D' => constellation_msg::Key::D,
                b'e' | b'E' => constellation_msg::Key::E,
                b'f' | b'F' => constellation_msg::Key::F,
                b'g' | b'G' => constellation_msg::Key::G,
                b'h' | b'H' => constellation_msg::Key::H,
                b'i' | b'I' => constellation_msg::Key::I,
                b'j' | b'J' => constellation_msg::Key::J,
                b'k' | b'K' => constellation_msg::Key::K,
                b'l' | b'L' => constellation_msg::Key::L,
                b'm' | b'M' => constellation_msg::Key::M,
                b'n' | b'N' => constellation_msg::Key::N,
                b'o' | b'O' => constellation_msg::Key::O,
                b'p' | b'P' => constellation_msg::Key::P,
                b'q' | b'Q' => constellation_msg::Key::Q,
                b'r' | b'R' => constellation_msg::Key::R,
                b's' | b'S' => constellation_msg::Key::S,
                b't' | b'T' => constellation_msg::Key::T,
                b'u' | b'U' => constellation_msg::Key::U,
                b'v' | b'V' => constellation_msg::Key::V,
                b'w' | b'W' => constellation_msg::Key::W,
                b'x' | b'X' => constellation_msg::Key::X,
                b'y' | b'Y' => constellation_msg::Key::Y,
                b'z' | b'Z' => constellation_msg::Key::Z,
                b'0' => constellation_msg::Key::Num0,
                b'1' => constellation_msg::Key::Num1,
                b'2' => constellation_msg::Key::Num2,
                b'3' => constellation_msg::Key::Num3,
                b'4' => constellation_msg::Key::Num4,
                b'5' => constellation_msg::Key::Num5,
                b'6' => constellation_msg::Key::Num6,
                b'7' => constellation_msg::Key::Num7,
                b'8' => constellation_msg::Key::Num8,
                b'9' => constellation_msg::Key::Num9,
                b'\n' | b'\r' => constellation_msg::Key::Enter,
                _ => constellation_msg::Key::Space,
            };
            let key_state = match (*event).t {
                KEYEVENT_RAWKEYDOWN => KeyState::Pressed,
                KEYEVENT_KEYDOWN | KEYEVENT_CHAR => KeyState::Repeated,
                KEYEVENT_KEYUP => KeyState::Released,
            };
            let key_modifiers = KeyModifiers::empty();  // TODO(pcwalton)
            this.downcast().send_window_event(WindowEvent::KeyEvent(key, key_state, key_modifiers))
        }}

        fn send_mouse_click_event(&this,
                                  event: *const cef_mouse_event [&cef_mouse_event],
                                  mouse_button_type: cef_mouse_button_type_t [cef_mouse_button_type_t],
                                  mouse_up: c_int [c_int],
                                  _click_count: c_int [c_int],)
                                  -> () {{
            let event: &cef_mouse_event = event;
            let mouse_button_type: cef_mouse_button_type_t = mouse_button_type;
            let mouse_up: c_int = mouse_up;
            let button_type = mouse_button_type as uint;
            let point = TypedPoint2D((*event).x as f32, (*event).y as f32);
            if mouse_up != 0 {
                this.downcast().send_window_event(WindowEvent::MouseWindowEventClass(
                    MouseWindowEvent::Click(button_type, point)))
            } else {
                this.downcast().send_window_event(WindowEvent::MouseWindowEventClass(
                    MouseWindowEvent::MouseUp(button_type, point)))
            }
        }}

        fn send_mouse_move_event(&this, event: *const cef_mouse_event [&cef_mouse_event],
                                 _mouse_exited: c_int [c_int],)
                                 -> () {{
            let event: &cef_mouse_event = event;
            let point = TypedPoint2D((*event).x as f32, (*event).y as f32);
            this.downcast().send_window_event(WindowEvent::MouseWindowMoveEventClass(point))
        }}

        fn send_mouse_wheel_event(&this,
                                  event: *const cef_mouse_event [&cef_mouse_event],
                                  delta_x: c_int [c_int],
                                  delta_y: c_int [c_int],)
                                  -> () {{
            let event: &cef_mouse_event = event;
            let delta_x: c_int = delta_x;
            let delta_y: c_int = delta_y;
            let delta = TypedPoint2D(delta_x as f32, delta_y as f32);
            let origin = TypedPoint2D((*event).x as i32, (*event).y as i32);
            this.downcast().send_window_event(WindowEvent::Scroll(delta, origin))
        }}

        fn get_zoom_level(&this,) -> c_double {{
            this.downcast().pinch_zoom_level() as c_double
        }}

        fn set_zoom_level(&this, new_zoom_level: c_double [c_double],) -> () {{
            let new_zoom_level: c_double = new_zoom_level;
            let old_zoom_level = this.get_zoom_level();
            this.downcast().send_window_event(WindowEvent::PinchZoom((new_zoom_level / old_zoom_level) as f32))
        }}

        fn initialize_compositing(&this,) -> () {{
            this.downcast().send_window_event(WindowEvent::InitializeCompositing);
        }}
    }
}

impl ServoCefBrowserHost {
    pub fn new(client: CefClient) -> ServoCefBrowserHost {
        ServoCefBrowserHost {
            browser: RefCell::new(None),
            client: client,
        }
    }

    fn send_window_event(&self, event: WindowEvent) {
        self.browser.borrow_mut().as_mut().unwrap().send_window_event(event);
    }

    fn pinch_zoom_level(&self) -> f32 {
        self.browser.borrow_mut().as_mut().unwrap().pinch_zoom_level()
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

