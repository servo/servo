/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use browser::{self, ServoCefBrowserExtensions};
use eutil::Downcast;
use interfaces::{CefBrowser, CefBrowserHost, CefClient, cef_browser_t, cef_browser_host_t, cef_client_t};
use types::cef_event_flags_t::{EVENTFLAG_ALT_DOWN, EVENTFLAG_CONTROL_DOWN, EVENTFLAG_SHIFT_DOWN};
use types::cef_key_event_type_t::{KEYEVENT_CHAR, KEYEVENT_KEYDOWN, KEYEVENT_KEYUP, KEYEVENT_RAWKEYDOWN};
use types::{cef_mouse_button_type_t, cef_mouse_event, cef_rect_t, cef_key_event, cef_window_handle_t};
use webrender_api::ScrollLocation;
use wrappers::CefWrap;

use compositing::windowing::{WindowEvent, MouseWindowEvent};
use euclid::{TypedPoint2D, TypedVector2D, TypedSize2D};
use libc::{c_double, c_int};
use msg::constellation_msg::{self, KeyModifiers, KeyState};
use script_traits::{MouseButton, TouchEventType};
use std::cell::{Cell, RefCell};
use std::char;

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
#[allow(non_snake_case)]
mod KeyboardCode {
    pub const VKEY_BACK : u8 = 0x08;
    pub const VKEY_TAB : u8 = 0x09;
    pub const VKEY_BACKTAB : u8 = 0x0A;
    pub const VKEY_CLEAR : u8 = 0x0C;
    pub const VKEY_RETURN : u8 = 0x0D;
    pub const VKEY_SHIFT : u8 = 0x10;
    pub const VKEY_CONTROL : u8 = 0x11;
    pub const VKEY_MENU : u8 = 0x12;
    pub const VKEY_PAUSE : u8 = 0x13;
    pub const VKEY_CAPITAL : u8 = 0x14;
    pub const VKEY_KANA : u8 = 0x15;
    //VKEY_HANGUL = 0x15,
    pub const VKEY_JUNJA : u8 = 0x17;
    pub const VKEY_FINAL : u8 = 0x18;
    pub const VKEY_HANJA : u8 = 0x19;
    //VKEY_KANJI = 0x19,
    pub const VKEY_ESCAPE : u8 = 0x1B;
    pub const VKEY_CONVERT : u8 = 0x1C;
    pub const VKEY_NONCONVERT : u8 = 0x1D;
    pub const VKEY_ACCEPT : u8 = 0x1E;
    pub const VKEY_MODECHANGE : u8 = 0x1F;
    pub const VKEY_SPACE : u8 = 0x20;
    pub const VKEY_PRIOR : u8 = 0x21;
    pub const VKEY_NEXT : u8 = 0x22;
    pub const VKEY_END : u8 = 0x23;
    pub const VKEY_HOME : u8 = 0x24;
    pub const VKEY_LEFT : u8 = 0x25;
    pub const VKEY_UP : u8 = 0x26;
    pub const VKEY_RIGHT : u8 = 0x27;
    pub const VKEY_DOWN : u8 = 0x28;
    pub const VKEY_SELECT : u8 = 0x29;
    pub const VKEY_PRINT : u8 = 0x2A;
    pub const VKEY_EXECUTE : u8 = 0x2B;
    pub const VKEY_SNAPSHOT : u8 = 0x2C;
    pub const VKEY_INSERT : u8 = 0x2D;
    pub const VKEY_DELETE : u8 = 0x2E;
    pub const VKEY_HELP : u8 = 0x2F;
    pub const VKEY_0 : u8 = 0x30;
    pub const VKEY_1 : u8 = 0x31;
    pub const VKEY_2 : u8 = 0x32;
    pub const VKEY_3 : u8 = 0x33;
    pub const VKEY_4 : u8 = 0x34;
    pub const VKEY_5 : u8 = 0x35;
    pub const VKEY_6 : u8 = 0x36;
    pub const VKEY_7 : u8 = 0x37;
    pub const VKEY_8 : u8 = 0x38;
    pub const VKEY_9 : u8 = 0x39;
    pub const VKEY_A : u8 = 0x41;
    pub const VKEY_B : u8 = 0x42;
    pub const VKEY_C : u8 = 0x43;
    pub const VKEY_D : u8 = 0x44;
    pub const VKEY_E : u8 = 0x45;
    pub const VKEY_F : u8 = 0x46;
    pub const VKEY_G : u8 = 0x47;
    pub const VKEY_H : u8 = 0x48;
    pub const VKEY_I : u8 = 0x49;
    pub const VKEY_J : u8 = 0x4A;
    pub const VKEY_K : u8 = 0x4B;
    pub const VKEY_L : u8 = 0x4C;
    pub const VKEY_M : u8 = 0x4D;
    pub const VKEY_N : u8 = 0x4E;
    pub const VKEY_O : u8 = 0x4F;
    pub const VKEY_P : u8 = 0x50;
    pub const VKEY_Q : u8 = 0x51;
    pub const VKEY_R : u8 = 0x52;
    pub const VKEY_S : u8 = 0x53;
    pub const VKEY_T : u8 = 0x54;
    pub const VKEY_U : u8 = 0x55;
    pub const VKEY_V : u8 = 0x56;
    pub const VKEY_W : u8 = 0x57;
    pub const VKEY_X : u8 = 0x58;
    pub const VKEY_Y : u8 = 0x59;
    pub const VKEY_Z : u8 = 0x5A;
    pub const VKEY_LWIN : u8 = 0x5B;
    pub const VKEY_RWIN : u8 = 0x5C;
    pub const VKEY_APPS : u8 = 0x5D;
    pub const VKEY_SLEEP : u8 = 0x5F;
    pub const VKEY_NUMPAD0 : u8 = 0x60;
    pub const VKEY_NUMPAD1 : u8 = 0x61;
    pub const VKEY_NUMPAD2 : u8 = 0x62;
    pub const VKEY_NUMPAD3 : u8 = 0x63;
    pub const VKEY_NUMPAD4 : u8 = 0x64;
    pub const VKEY_NUMPAD5 : u8 = 0x65;
    pub const VKEY_NUMPAD6 : u8 = 0x66;
    pub const VKEY_NUMPAD7 : u8 = 0x67;
    pub const VKEY_NUMPAD8 : u8 = 0x68;
    pub const VKEY_NUMPAD9 : u8 = 0x69;
    pub const VKEY_MULTIPLY : u8 = 0x6A;
    pub const VKEY_ADD : u8 = 0x6B;
    pub const VKEY_SEPARATOR : u8 = 0x6C;
    pub const VKEY_SUBTRACT : u8 = 0x6D;
    pub const VKEY_DECIMAL : u8 = 0x6E;
    pub const VKEY_DIVIDE : u8 = 0x6F;
    pub const VKEY_F1 : u8 = 0x70;
    pub const VKEY_F2 : u8 = 0x71;
    pub const VKEY_F3 : u8 = 0x72;
    pub const VKEY_F4 : u8 = 0x73;
    pub const VKEY_F5 : u8 = 0x74;
    pub const VKEY_F6 : u8 = 0x75;
    pub const VKEY_F7 : u8 = 0x76;
    pub const VKEY_F8 : u8 = 0x77;
    pub const VKEY_F9 : u8 = 0x78;
    pub const VKEY_F10 : u8 = 0x79;
    pub const VKEY_F11 : u8 = 0x7A;
    pub const VKEY_F12 : u8 = 0x7B;
    pub const VKEY_F13 : u8 = 0x7C;
    pub const VKEY_F14 : u8 = 0x7D;
    pub const VKEY_F15 : u8 = 0x7E;
    pub const VKEY_F16 : u8 = 0x7F;
    pub const VKEY_F17 : u8 = 0x80;
    pub const VKEY_F18 : u8 = 0x81;
    pub const VKEY_F19 : u8 = 0x82;
    pub const VKEY_F20 : u8 = 0x83;
    pub const VKEY_F21 : u8 = 0x84;
    pub const VKEY_F22 : u8 = 0x85;
    pub const VKEY_F23 : u8 = 0x86;
    pub const VKEY_F24 : u8 = 0x87;
    pub const VKEY_NUMLOCK : u8 = 0x90;
    pub const VKEY_SCROLL : u8 = 0x91;
    pub const VKEY_LSHIFT : u8 = 0xA0;
    pub const VKEY_RSHIFT : u8 = 0xA1;
    pub const VKEY_LCONTROL : u8 = 0xA2;
    pub const VKEY_RCONTROL : u8 = 0xA3;
    pub const VKEY_LMENU : u8 = 0xA4;
    pub const VKEY_RMENU : u8 = 0xA5;
    pub const VKEY_BROWSER_BACK : u8 = 0xA6;
    pub const VKEY_BROWSER_FORWARD : u8 = 0xA7;
    pub const VKEY_BROWSER_REFRESH : u8 = 0xA8;
    pub const VKEY_BROWSER_STOP : u8 = 0xA9;
    pub const VKEY_BROWSER_SEARCH : u8 = 0xAA;
    pub const VKEY_BROWSER_FAVORITES : u8 = 0xAB;
    pub const VKEY_BROWSER_HOME : u8 = 0xAC;
    pub const VKEY_VOLUME_MUTE : u8 = 0xAD;
    pub const VKEY_VOLUME_DOWN : u8 = 0xAE;
    pub const VKEY_VOLUME_UP : u8 = 0xAF;
    pub const VKEY_MEDIA_NEXT_TRACK : u8 = 0xB0;
    pub const VKEY_MEDIA_PREV_TRACK : u8 = 0xB1;
    pub const VKEY_MEDIA_STOP : u8 = 0xB2;
    pub const VKEY_MEDIA_PLAY_PAUSE : u8 = 0xB3;
    pub const VKEY_MEDIA_LAUNCH_MAIL : u8 = 0xB4;
    pub const VKEY_MEDIA_LAUNCH_MEDIA_SELECT : u8 = 0xB5;
    pub const VKEY_MEDIA_LAUNCH_APP1 : u8 = 0xB6;
    pub const VKEY_MEDIA_LAUNCH_APP2 : u8 = 0xB7;
    pub const VKEY_OEM_1 : u8 = 0xBA;
    pub const VKEY_OEM_PLUS : u8 = 0xBB;
    pub const VKEY_OEM_COMMA : u8 = 0xBC;
    pub const VKEY_OEM_MINUS : u8 = 0xBD;
    pub const VKEY_OEM_PERIOD : u8 = 0xBE;
    pub const VKEY_OEM_2 : u8 = 0xBF;
    pub const VKEY_OEM_3 : u8 = 0xC0;
    pub const VKEY_OEM_4 : u8 = 0xDB;
    pub const VKEY_OEM_5 : u8 = 0xDC;
    pub const VKEY_OEM_6 : u8 = 0xDD;
    pub const VKEY_OEM_7 : u8 = 0xDE;
    pub const VKEY_OEM_8 : u8 = 0xDF;
    pub const VKEY_OEM_102 : u8 = 0xE2;
    pub const VKEY_OEM_103 : u8 = 0xE3;  // GTV KEYCODE_MEDIA_REWIND
    pub const VKEY_OEM_104 : u8 = 0xE4;  // GTV KEYCODE_MEDIA_FAST_FORWARD
    pub const VKEY_PROCESSKEY : u8 = 0xE5;
    pub const VKEY_PACKET : u8 = 0xE7;
    pub const VKEY_DBE_SBCSCHAR : u8 = 0xF3;
    pub const VKEY_DBE_DBCSCHAR : u8 = 0xF4;
    pub const VKEY_ATTN : u8 = 0xF6;
    pub const VKEY_CRSEL : u8 = 0xF7;
    pub const VKEY_EXSEL : u8 = 0xF8;
    pub const VKEY_EREOF : u8 = 0xF9;
    pub const VKEY_PLAY : u8 = 0xFA;
    pub const VKEY_ZOOM : u8 = 0xFB;
    pub const VKEY_NONAME : u8 = 0xFC;
    pub const VKEY_PA1 : u8 = 0xFD;
    pub const VKEY_OEM_CLEAR : u8 = 0xFE;
    pub const VKEY_UNKNOWN : u8 = 0x0;

    // POSIX specific VKEYs. Note that as of Windows SDK 7.1, 0x97-9F, 0xD8-DA,
    // and 0xE8 are unassigned.
    pub const VKEY_WLAN : u8 = 0x97;
    pub const VKEY_POWER : u8 = 0x98;
    pub const VKEY_BRIGHTNESS_DOWN : u8 = 0xD8;
    pub const VKEY_BRIGHTNESS_UP : u8 = 0xD9;
    pub const VKEY_KBD_BRIGHTNESS_DOWN : u8 = 0xDA;
    pub const VKEY_KBD_BRIGHTNESS_UP : u8 = 0xE8;

    // Windows does not have a specific key code for AltGr. We use the unused 0xE1
    // (VK_OEM_AX) code to represent AltGr, matching the behaviour of Firefox on
    // Linux.
    pub const VKEY_ALTGR : u8 = 0xE1;
    // Windows does not have a specific key code for Compose. We use the unused
    // 0xE6 (VK_ICO_CLEAR) code to represent Compose.
    pub const VKEY_COMPOSE : u8 = 0xE6;
}

// this is way too much work to do 100% correctly right now.
// see xkb_keyboard_layout_engine.cc -> XkbKeyboardLayoutEngine::Lookup in chromium for details
fn get_key_msg(keycode: c_int, character: u16) -> Option<constellation_msg::Key> {
    match keycode as u8 {
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
            let size = TypedSize2D::new(rect.width as u32, rect.height as u32);
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
            if (*event).modifiers & EVENTFLAG_SHIFT_DOWN as u32 != 0 {
               key_modifiers = key_modifiers | constellation_msg::SHIFT;
            }
            if (*event).modifiers & EVENTFLAG_CONTROL_DOWN as u32 != 0 {
               key_modifiers = key_modifiers | constellation_msg::CONTROL;
            }
            if (*event).modifiers & EVENTFLAG_ALT_DOWN as u32 != 0 {
               key_modifiers = key_modifiers | constellation_msg::ALT;
            }
            let ch = char::from_u32((*event).character as u32);
            this.downcast().send_window_event(WindowEvent::KeyEvent(ch, key, key_state, key_modifiers))
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
            let point = TypedPoint2D::new((*event).x as f32, (*event).y as f32);
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
            let point = TypedPoint2D::new((*event).x as f32, (*event).y as f32);
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
            let delta = TypedVector2D::new(delta_x as f32, delta_y as f32);
            let origin = TypedPoint2D::new((*event).x as i32, (*event).y as i32);
            this.downcast().send_window_event(WindowEvent::Scroll(ScrollLocation::Delta(delta),
                                                                  origin,
                                                                  TouchEventType::Move))
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

