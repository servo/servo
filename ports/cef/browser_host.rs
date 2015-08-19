/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use browser::{self, ServoCefBrowserExtensions};
use eutil::Downcast;
use interfaces::{CefBrowser, CefBrowserHost, CefClient, cef_browser_t, cef_browser_host_t, cef_client_t};
use types::cef_event_flags_t::{EVENTFLAG_ALT_DOWN, EVENTFLAG_CONTROL_DOWN, EVENTFLAG_SHIFT_DOWN};
use types::cef_key_event_type_t::{KEYEVENT_CHAR, KEYEVENT_KEYDOWN, KEYEVENT_KEYUP, KEYEVENT_RAWKEYDOWN};
use types::{cef_mouse_button_type_t, cef_mouse_event, cef_rect_t, cef_key_event, cef_window_handle_t};
use wrappers::CefWrap;

use compositing::windowing::{WindowEvent, MouseWindowEvent};
use euclid::point::Point2D;
use euclid::size::Size2D;
use libc::{c_double, c_int};
use msg::constellation_msg::{self, KeyModifiers, KeyState};
use script_traits::MouseButton;
use std::cell::{Cell, RefCell};
use std::intrinsics;
use std::mem::transmute;

pub struct ServoCefBrowserHost {
    /// A reference to the browser.
    pub browser: RefCell<Option<CefBrowser>>,
    /// A reference to the client.
    pub client: CefClient,
    /// flag for return value of prepare_for_composite
    pub composite_ok: Cell<bool>,
}

// From blink ui/events/keycodes/keyboard_codes_posix.h.
#[allow(dead_code)]
enum KeyboardCode {
  VKEY_BACK = 0x08,
  VKEY_TAB = 0x09,
  VKEY_BACKTAB = 0x0A,
  VKEY_CLEAR = 0x0C,
  VKEY_RETURN = 0x0D,
  VKEY_SHIFT = 0x10,
  VKEY_CONTROL = 0x11,
  VKEY_MENU = 0x12,
  VKEY_PAUSE = 0x13,
  VKEY_CAPITAL = 0x14,
  VKEY_KANA = 0x15,
  //VKEY_HANGUL = 0x15,
  VKEY_JUNJA = 0x17,
  VKEY_FINAL = 0x18,
  VKEY_HANJA = 0x19,
  //VKEY_KANJI = 0x19,
  VKEY_ESCAPE = 0x1B,
  VKEY_CONVERT = 0x1C,
  VKEY_NONCONVERT = 0x1D,
  VKEY_ACCEPT = 0x1E,
  VKEY_MODECHANGE = 0x1F,
  VKEY_SPACE = 0x20,
  VKEY_PRIOR = 0x21,
  VKEY_NEXT = 0x22,
  VKEY_END = 0x23,
  VKEY_HOME = 0x24,
  VKEY_LEFT = 0x25,
  VKEY_UP = 0x26,
  VKEY_RIGHT = 0x27,
  VKEY_DOWN = 0x28,
  VKEY_SELECT = 0x29,
  VKEY_PRINT = 0x2A,
  VKEY_EXECUTE = 0x2B,
  VKEY_SNAPSHOT = 0x2C,
  VKEY_INSERT = 0x2D,
  VKEY_DELETE = 0x2E,
  VKEY_HELP = 0x2F,
  VKEY_0 = 0x30,
  VKEY_1 = 0x31,
  VKEY_2 = 0x32,
  VKEY_3 = 0x33,
  VKEY_4 = 0x34,
  VKEY_5 = 0x35,
  VKEY_6 = 0x36,
  VKEY_7 = 0x37,
  VKEY_8 = 0x38,
  VKEY_9 = 0x39,
  VKEY_A = 0x41,
  VKEY_B = 0x42,
  VKEY_C = 0x43,
  VKEY_D = 0x44,
  VKEY_E = 0x45,
  VKEY_F = 0x46,
  VKEY_G = 0x47,
  VKEY_H = 0x48,
  VKEY_I = 0x49,
  VKEY_J = 0x4A,
  VKEY_K = 0x4B,
  VKEY_L = 0x4C,
  VKEY_M = 0x4D,
  VKEY_N = 0x4E,
  VKEY_O = 0x4F,
  VKEY_P = 0x50,
  VKEY_Q = 0x51,
  VKEY_R = 0x52,
  VKEY_S = 0x53,
  VKEY_T = 0x54,
  VKEY_U = 0x55,
  VKEY_V = 0x56,
  VKEY_W = 0x57,
  VKEY_X = 0x58,
  VKEY_Y = 0x59,
  VKEY_Z = 0x5A,
  VKEY_LWIN = 0x5B,
  VKEY_RWIN = 0x5C,
  VKEY_APPS = 0x5D,
  VKEY_SLEEP = 0x5F,
  VKEY_NUMPAD0 = 0x60,
  VKEY_NUMPAD1 = 0x61,
  VKEY_NUMPAD2 = 0x62,
  VKEY_NUMPAD3 = 0x63,
  VKEY_NUMPAD4 = 0x64,
  VKEY_NUMPAD5 = 0x65,
  VKEY_NUMPAD6 = 0x66,
  VKEY_NUMPAD7 = 0x67,
  VKEY_NUMPAD8 = 0x68,
  VKEY_NUMPAD9 = 0x69,
  VKEY_MULTIPLY = 0x6A,
  VKEY_ADD = 0x6B,
  VKEY_SEPARATOR = 0x6C,
  VKEY_SUBTRACT = 0x6D,
  VKEY_DECIMAL = 0x6E,
  VKEY_DIVIDE = 0x6F,
  VKEY_F1 = 0x70,
  VKEY_F2 = 0x71,
  VKEY_F3 = 0x72,
  VKEY_F4 = 0x73,
  VKEY_F5 = 0x74,
  VKEY_F6 = 0x75,
  VKEY_F7 = 0x76,
  VKEY_F8 = 0x77,
  VKEY_F9 = 0x78,
  VKEY_F10 = 0x79,
  VKEY_F11 = 0x7A,
  VKEY_F12 = 0x7B,
  VKEY_F13 = 0x7C,
  VKEY_F14 = 0x7D,
  VKEY_F15 = 0x7E,
  VKEY_F16 = 0x7F,
  VKEY_F17 = 0x80,
  VKEY_F18 = 0x81,
  VKEY_F19 = 0x82,
  VKEY_F20 = 0x83,
  VKEY_F21 = 0x84,
  VKEY_F22 = 0x85,
  VKEY_F23 = 0x86,
  VKEY_F24 = 0x87,
  VKEY_NUMLOCK = 0x90,
  VKEY_SCROLL = 0x91,
  VKEY_LSHIFT = 0xA0,
  VKEY_RSHIFT = 0xA1,
  VKEY_LCONTROL = 0xA2,
  VKEY_RCONTROL = 0xA3,
  VKEY_LMENU = 0xA4,
  VKEY_RMENU = 0xA5,
  VKEY_BROWSER_BACK = 0xA6,
  VKEY_BROWSER_FORWARD = 0xA7,
  VKEY_BROWSER_REFRESH = 0xA8,
  VKEY_BROWSER_STOP = 0xA9,
  VKEY_BROWSER_SEARCH = 0xAA,
  VKEY_BROWSER_FAVORITES = 0xAB,
  VKEY_BROWSER_HOME = 0xAC,
  VKEY_VOLUME_MUTE = 0xAD,
  VKEY_VOLUME_DOWN = 0xAE,
  VKEY_VOLUME_UP = 0xAF,
  VKEY_MEDIA_NEXT_TRACK = 0xB0,
  VKEY_MEDIA_PREV_TRACK = 0xB1,
  VKEY_MEDIA_STOP = 0xB2,
  VKEY_MEDIA_PLAY_PAUSE = 0xB3,
  VKEY_MEDIA_LAUNCH_MAIL = 0xB4,
  VKEY_MEDIA_LAUNCH_MEDIA_SELECT = 0xB5,
  VKEY_MEDIA_LAUNCH_APP1 = 0xB6,
  VKEY_MEDIA_LAUNCH_APP2 = 0xB7,
  VKEY_OEM_1 = 0xBA,
  VKEY_OEM_PLUS = 0xBB,
  VKEY_OEM_COMMA = 0xBC,
  VKEY_OEM_MINUS = 0xBD,
  VKEY_OEM_PERIOD = 0xBE,
  VKEY_OEM_2 = 0xBF,
  VKEY_OEM_3 = 0xC0,
  VKEY_OEM_4 = 0xDB,
  VKEY_OEM_5 = 0xDC,
  VKEY_OEM_6 = 0xDD,
  VKEY_OEM_7 = 0xDE,
  VKEY_OEM_8 = 0xDF,
  VKEY_OEM_102 = 0xE2,
  VKEY_OEM_103 = 0xE3,  // GTV KEYCODE_MEDIA_REWIND
  VKEY_OEM_104 = 0xE4,  // GTV KEYCODE_MEDIA_FAST_FORWARD
  VKEY_PROCESSKEY = 0xE5,
  VKEY_PACKET = 0xE7,
  VKEY_DBE_SBCSCHAR = 0xF3,
  VKEY_DBE_DBCSCHAR = 0xF4,
  VKEY_ATTN = 0xF6,
  VKEY_CRSEL = 0xF7,
  VKEY_EXSEL = 0xF8,
  VKEY_EREOF = 0xF9,
  VKEY_PLAY = 0xFA,
  VKEY_ZOOM = 0xFB,
  VKEY_NONAME = 0xFC,
  VKEY_PA1 = 0xFD,
  VKEY_OEM_CLEAR = 0xFE,
  VKEY_UNKNOWN = 0,

  // POSIX specific VKEYs. Note that as of Windows SDK 7.1, 0x97-9F, 0xD8-DA,
  // and 0xE8 are unassigned.
  VKEY_WLAN = 0x97,
  VKEY_POWER = 0x98,
  VKEY_BRIGHTNESS_DOWN = 0xD8,
  VKEY_BRIGHTNESS_UP = 0xD9,
  VKEY_KBD_BRIGHTNESS_DOWN = 0xDA,
  VKEY_KBD_BRIGHTNESS_UP = 0xE8,

  // Windows does not have a specific key code for AltGr. We use the unused 0xE1
  // (VK_OEM_AX) code to represent AltGr, matching the behaviour of Firefox on
  // Linux.
  VKEY_ALTGR = 0xE1,
  // Windows does not have a specific key code for Compose. We use the unused
  // 0xE6 (VK_ICO_CLEAR) code to represent Compose.
  VKEY_COMPOSE = 0xE6,
}

// this is way too much work to do 100% correctly right now.
// see xkb_keyboard_layout_engine.cc -> XkbKeyboardLayoutEngine::Lookup in chromium for details
fn get_key_msg(keycode: c_int, character: u16) -> Option<constellation_msg::Key> {
    let code: KeyboardCode = unsafe { transmute(keycode as u8) };
    match code {
        KeyboardCode::VKEY_BACK => Some(constellation_msg::Key::Backspace),
        KeyboardCode::VKEY_RIGHT => Some(constellation_msg::Key::Right),
        KeyboardCode::VKEY_LEFT => Some(constellation_msg::Key::Left),
        KeyboardCode::VKEY_UP => Some(constellation_msg::Key::Up),
        KeyboardCode::VKEY_DOWN => Some(constellation_msg::Key::Down),
        KeyboardCode::VKEY_RSHIFT => Some(constellation_msg::Key::RightShift),
        KeyboardCode::VKEY_SHIFT | KeyboardCode::VKEY_LSHIFT => Some(constellation_msg::Key::LeftShift),
        KeyboardCode::VKEY_RCONTROL => Some(constellation_msg::Key::RightControl),
        KeyboardCode::VKEY_CONTROL | KeyboardCode::VKEY_LCONTROL => Some(constellation_msg::Key::LeftControl),
        KeyboardCode::VKEY_LWIN => Some(constellation_msg::Key::LeftSuper),
        KeyboardCode::VKEY_RWIN => Some(constellation_msg::Key::RightSuper),
        KeyboardCode::VKEY_MENU => Some(constellation_msg::Key::LeftAlt),
        KeyboardCode::VKEY_APPS => Some(constellation_msg::Key::Menu),
        KeyboardCode::VKEY_ALTGR => Some(constellation_msg::Key::RightAlt), //not sure if correct...
        KeyboardCode::VKEY_ESCAPE => Some(constellation_msg::Key::Escape),
        KeyboardCode::VKEY_INSERT => Some(constellation_msg::Key::Insert),
        KeyboardCode::VKEY_DELETE => Some(constellation_msg::Key::Delete),
        KeyboardCode::VKEY_NEXT => Some(constellation_msg::Key::PageUp),
        KeyboardCode::VKEY_PRIOR => Some(constellation_msg::Key::PageDown),
        KeyboardCode::VKEY_HOME => Some(constellation_msg::Key::Home),
        KeyboardCode::VKEY_END => Some(constellation_msg::Key::End),
        KeyboardCode::VKEY_CAPITAL => Some(constellation_msg::Key::CapsLock),
        KeyboardCode::VKEY_F1 => Some(constellation_msg::Key::F1),
        KeyboardCode::VKEY_F2 => Some(constellation_msg::Key::F2),
        KeyboardCode::VKEY_F3 => Some(constellation_msg::Key::F3),
        KeyboardCode::VKEY_F4 => Some(constellation_msg::Key::F4),
        KeyboardCode::VKEY_F5 => Some(constellation_msg::Key::F5),
        KeyboardCode::VKEY_F6 => Some(constellation_msg::Key::F6),
        KeyboardCode::VKEY_F7 => Some(constellation_msg::Key::F7),
        KeyboardCode::VKEY_F8 => Some(constellation_msg::Key::F8),
        KeyboardCode::VKEY_F9 => Some(constellation_msg::Key::F9),
        KeyboardCode::VKEY_F10 => Some(constellation_msg::Key::F10),
        KeyboardCode::VKEY_F11 => Some(constellation_msg::Key::F11),
        KeyboardCode::VKEY_F12 => Some(constellation_msg::Key::F12),
        KeyboardCode::VKEY_F13 => Some(constellation_msg::Key::F13),
        KeyboardCode::VKEY_F14 => Some(constellation_msg::Key::F14),
        KeyboardCode::VKEY_F15 => Some(constellation_msg::Key::F15),
        KeyboardCode::VKEY_F16 => Some(constellation_msg::Key::F16),
        KeyboardCode::VKEY_F17 => Some(constellation_msg::Key::F17),
        KeyboardCode::VKEY_F18 => Some(constellation_msg::Key::F18),
        KeyboardCode::VKEY_F19 => Some(constellation_msg::Key::F19),
        KeyboardCode::VKEY_F20 => Some(constellation_msg::Key::F20),
        KeyboardCode::VKEY_F21 => Some(constellation_msg::Key::F21),
        KeyboardCode::VKEY_F22 => Some(constellation_msg::Key::F22),
        KeyboardCode::VKEY_F23 => Some(constellation_msg::Key::F23),
        KeyboardCode::VKEY_F24 => Some(constellation_msg::Key::F24),
        KeyboardCode::VKEY_NUMPAD0 => Some(constellation_msg::Key::Kp0),
        KeyboardCode::VKEY_NUMPAD1 => Some(constellation_msg::Key::Kp1),
        KeyboardCode::VKEY_NUMPAD2 => Some(constellation_msg::Key::Kp2),
        KeyboardCode::VKEY_NUMPAD3 => Some(constellation_msg::Key::Kp3),
        KeyboardCode::VKEY_NUMPAD4 => Some(constellation_msg::Key::Kp4),
        KeyboardCode::VKEY_NUMPAD5 => Some(constellation_msg::Key::Kp5),
        KeyboardCode::VKEY_NUMPAD6 => Some(constellation_msg::Key::Kp6),
        KeyboardCode::VKEY_NUMPAD7 => Some(constellation_msg::Key::Kp7),
        KeyboardCode::VKEY_NUMPAD8 => Some(constellation_msg::Key::Kp8),
        KeyboardCode::VKEY_NUMPAD9 => Some(constellation_msg::Key::Kp9),
        KeyboardCode::VKEY_DECIMAL => Some(constellation_msg::Key::KpDecimal),
        KeyboardCode::VKEY_DIVIDE => Some(constellation_msg::Key::KpDivide),
        KeyboardCode::VKEY_MULTIPLY => Some(constellation_msg::Key::KpMultiply),
        KeyboardCode::VKEY_SUBTRACT => Some(constellation_msg::Key::KpSubtract),
        KeyboardCode::VKEY_ADD => Some(constellation_msg::Key::KpAdd),
        KeyboardCode::VKEY_NUMLOCK => Some(constellation_msg::Key::NumLock),
        KeyboardCode::VKEY_PRINT => Some(constellation_msg::Key::PrintScreen),
        KeyboardCode::VKEY_PAUSE => Some(constellation_msg::Key::Pause),
        //VKEY_BACK
        _ => { match character as u8 {
                 b'[' => Some(constellation_msg::Key::LeftBracket),
                 b']' => Some(constellation_msg::Key::RightBracket),
                 b'=' => Some(constellation_msg::Key::Equal),
                 b';' => Some(constellation_msg::Key::Semicolon),
                 b'/' => Some(constellation_msg::Key::Slash),
                 b'.' => Some(constellation_msg::Key::Period),
                 b'-' => Some(constellation_msg::Key::Minus),
                 b',' => Some(constellation_msg::Key::Comma),
                 b'\'' => Some(constellation_msg::Key::Apostrophe),
                 b'\\' => Some(constellation_msg::Key::Backslash),
                 b'`' => Some(constellation_msg::Key::GraveAccent),
                 b'\t' => Some(constellation_msg::Key::Tab),
                 b'a' | b'A' => Some(constellation_msg::Key::A),
                 b'b' | b'B' => Some(constellation_msg::Key::B),
                 b'c' | b'C' => Some(constellation_msg::Key::C),
                 b'd' | b'D' => Some(constellation_msg::Key::D),
                 b'e' | b'E' => Some(constellation_msg::Key::E),
                 b'f' | b'F' => Some(constellation_msg::Key::F),
                 b'g' | b'G' => Some(constellation_msg::Key::G),
                 b'h' | b'H' => Some(constellation_msg::Key::H),
                 b'i' | b'I' => Some(constellation_msg::Key::I),
                 b'j' | b'J' => Some(constellation_msg::Key::J),
                 b'k' | b'K' => Some(constellation_msg::Key::K),
                 b'l' | b'L' => Some(constellation_msg::Key::L),
                 b'm' | b'M' => Some(constellation_msg::Key::M),
                 b'n' | b'N' => Some(constellation_msg::Key::N),
                 b'o' | b'O' => Some(constellation_msg::Key::O),
                 b'p' | b'P' => Some(constellation_msg::Key::P),
                 b'q' | b'Q' => Some(constellation_msg::Key::Q),
                 b'r' | b'R' => Some(constellation_msg::Key::R),
                 b's' | b'S' => Some(constellation_msg::Key::S),
                 b't' | b'T' => Some(constellation_msg::Key::T),
                 b'u' | b'U' => Some(constellation_msg::Key::U),
                 b'v' | b'V' => Some(constellation_msg::Key::V),
                 b'w' | b'W' => Some(constellation_msg::Key::W),
                 b'x' | b'X' => Some(constellation_msg::Key::X),
                 b'y' | b'Y' => Some(constellation_msg::Key::Y),
                 b'z' | b'Z' => Some(constellation_msg::Key::Z),
                 b'0' => Some(constellation_msg::Key::Num0),
                 b'1' => Some(constellation_msg::Key::Num1),
                 b'2' => Some(constellation_msg::Key::Num2),
                 b'3' => Some(constellation_msg::Key::Num3),
                 b'4' => Some(constellation_msg::Key::Num4),
                 b'5' => Some(constellation_msg::Key::Num5),
                 b'6' => Some(constellation_msg::Key::Num6),
                 b'7' => Some(constellation_msg::Key::Num7),
                 b'8' => Some(constellation_msg::Key::Num8),
                 b'9' => Some(constellation_msg::Key::Num9),
                 b'\n' | b'\r' => Some(constellation_msg::Key::Enter),
                 b' ' => Some(constellation_msg::Key::Space),
                 _ => None
             }
        }
    }
}

// unhandled
//pub enum Key {
    //World1,
    //World2,
    //ScrollLock,
    //KpEnter,
    //KpEqual,
    //RightAlt,
//}

full_cef_class_impl! {
    ServoCefBrowserHost : CefBrowserHost, cef_browser_host_t {
        fn get_client(&this,) -> *mut cef_client_t {{
            this.downcast().client.clone()
        }}
        fn get_browser(&this,) -> *mut cef_browser_t {{
            let browser = this.downcast().browser.borrow_mut();
            browser.clone().unwrap()
        }}

        fn was_resized(&this,) -> () {{
            let mut rect = cef_rect_t::zero();
            if cfg!(target_os="macos") {
                if check_ptr_exist!(this.get_client(), get_render_handler) &&
                   check_ptr_exist!(this.get_client().get_render_handler(), get_backing_rect) {
                    this.get_client()
                        .get_render_handler()
                        .get_backing_rect(this.downcast().browser.borrow().clone().unwrap(), &mut rect);
                }
            } else if check_ptr_exist!(this.get_client(), get_render_handler) &&
               check_ptr_exist!(this.get_client().get_render_handler(), get_view_rect) {
                this.get_client()
                    .get_render_handler()
                    .get_view_rect(this.downcast().browser.borrow().clone().unwrap(), &mut rect);
               }
            let size = Size2D::typed(rect.width as u32, rect.height as u32);
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
            let event: &cef_key_event = event;
            let key = match get_key_msg((*event).windows_key_code, (*event).character) {
                Some(keycode) => keycode,
                None => {
                    error!("Unhandled keycode({}) passed!", (*event).windows_key_code);
                    return;
                }
            };
            let key_state = match (*event).t {
                // in tests with cef-real, this event had no effect
                KEYEVENT_RAWKEYDOWN => return,
                KEYEVENT_KEYDOWN => KeyState::Pressed,
                KEYEVENT_CHAR => KeyState::Repeated,
                KEYEVENT_KEYUP => KeyState::Released,
            };
            let mut key_modifiers = KeyModifiers::empty();
            if (*event).modifiers & unsafe { intrinsics::discriminant_value(&EVENTFLAG_SHIFT_DOWN) as u32 } != 0 {
               key_modifiers = key_modifiers | constellation_msg::SHIFT;
            }
            if (*event).modifiers & unsafe { intrinsics::discriminant_value(&EVENTFLAG_CONTROL_DOWN) as u32 } != 0 {
               key_modifiers = key_modifiers | constellation_msg::CONTROL;
            }
            if (*event).modifiers & unsafe { intrinsics::discriminant_value(&EVENTFLAG_ALT_DOWN) as u32 } != 0 {
               key_modifiers = key_modifiers | constellation_msg::ALT;
            }
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
            let button_type = match mouse_button_type {
                cef_mouse_button_type_t::MBT_LEFT => MouseButton::Left,
                cef_mouse_button_type_t::MBT_MIDDLE => MouseButton::Middle,
                cef_mouse_button_type_t::MBT_RIGHT => MouseButton::Right,
            };
            let point = Point2D::typed((*event).x as f32, (*event).y as f32);
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
            let point = Point2D::typed((*event).x as f32, (*event).y as f32);
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
            let delta = Point2D::typed(delta_x as f32, delta_y as f32);
            let origin = Point2D::typed((*event).x as i32, (*event).y as i32);
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

        fn composite(&this,) -> () {{
            this.downcast().composite_ok.set(true);
            this.downcast().send_window_event(WindowEvent::Refresh);
            this.downcast().composite_ok.set(false);
        }}

        fn get_window_handle(&this,) -> cef_window_handle_t {{
            let t = this.downcast();
            let browser = t.browser.borrow();
            browser::get_window(&browser.as_ref().unwrap()) as cef_window_handle_t
        }}
    }
}

impl ServoCefBrowserHost {
    pub fn new(client: CefClient) -> ServoCefBrowserHost {
        ServoCefBrowserHost {
            browser: RefCell::new(None),
            client: client,
            composite_ok: Cell::new(false),
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

