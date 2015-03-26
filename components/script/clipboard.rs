/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::trace::JSTraceable;

// X11 clipboard support
#[cfg(target_os="linux")]
mod x11_clipboard {
    use xlib::{Display, Window};
    use xlib::{XOpenDisplay, XCloseDisplay};
    use xlib::{XCreateSimpleWindow, XDefaultRootWindow};
    use libc::*;

    #[jstraceable]
    pub struct ClipboardContext {
        display: *mut Display,
        window: Window,
    }
    no_jsmanaged_fields!(ClipboardContext);

    impl ClipboardContext {
        pub fn new() -> Result<ClipboardContext, &'static str> {
            // http://sourceforge.net/p/xclip/code/HEAD/tree/trunk/xclip.c
            let dpy = XOpenDisplay(0 as *mut c_char);
            if dpy == 0 {
                return Err("XOpenDisplay")
            }
            let win = XCreateSimpleWindow(dpy, XDefaultRootWindow(dpy), 0, 0, 1, 1, 0, 0, 0);
            if win == 0 {
                return Err("XCreateSimpleWindow")
            }
            Ok(ClipboardContext {
                display: dpy,
                window: win,
            })
        }
        pub fn get_contents(&self) -> String {
            "dummy string".to_owned()
        }
    }

    impl Drop for ClipboardContext {
        fn drop(&mut self) {
            // TODO: error checking of some sort
            if XCloseDisplay(self.display) == 0 {
                panic!("XCloseDisplay failed.");
            }
        }
    }
}

// catch-all "not-yet-implemented" clipboard
#[cfg(not(target_os="linux"))]
mod notyetimplemented_clipboard {
    #[jstraceable]
    pub struct ClipboardContext;
    impl ClipBoardContext {
        pub fn new() -> Result<ClipboardContext, &'static str> {
            Ok(ClipboardContext)
        }
        pub fn get_contents(&self) -> String {
            "Clipboard not yet implemented on $PLATFORM".to_owned()
        }
    }
}

#[cfg(target_os="linux")]
pub use self::x11_clipboard::*;
#[cfg(not(target_os="linux"))]
pub use self::notyetimplemented_clipboard::*;

