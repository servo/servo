/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(phase)]

#![deny(unused_imports)]
#![deny(unused_variables)]

#[cfg(target_os="android")]
extern crate libc;

extern crate servo;
extern crate time;
extern crate "util" as servo_util;

#[cfg(all(feature = "glutin_app",not(test)))]
extern crate "glutin_app" as app;
#[cfg(all(feature = "glfw",not(test)))]
extern crate "glfw_app" as app;

#[cfg(not(test))]
extern crate compositing;

#[cfg(target_os="android")]
#[phase(plugin, link)]
extern crate android_glue;

#[cfg(target_os="android")]
use libc::c_int;

#[cfg(not(test))]
use servo_util::opts;

// FIXME: Find replacement for this post-runtime removal
//#[cfg(not(test))]
//use servo_util::rtinstrument;

#[cfg(not(test))]
use servo::Browser;
#[cfg(not(test))]
use compositing::windowing::WindowEvent;

#[cfg(not(any(test,target_os="android")))]
use std::os;

#[cfg(not(test))]
struct BrowserWrapper {
    browser: Browser<app::window::Window>,
}

#[cfg(target_os="android")]
android_start!(main)

#[cfg(target_os="android")]
fn get_args() -> Vec<String> {
    vec![
        "servo".into_string(),
        "http://en.wikipedia.org/wiki/Rust".into_string()
    ]
}

#[cfg(not(target_os="android"))]
fn get_args() -> Vec<String> {
    os::args()
}

#[cfg(target_os="android")]
fn redirect_output(file_no: c_int) {
    use libc::funcs::posix88::unistd::{pipe, dup2};
    use libc::funcs::posix88::stdio::fdopen;
    use libc::c_char;
    use libc::funcs::c95::stdio::fgets;
    use std::mem;
    use std::c_str::CString;

    unsafe {
        let mut pipes: [c_int, ..2] = [ 0, 0 ];
        pipe(pipes.as_mut_ptr());
        dup2(pipes[1], file_no);
        let input_file = "r".with_c_str(|mode| {
            fdopen(pipes[0], mode)
        });
        spawn(proc() {
            loop {
                let mut read_buffer: [c_char, ..1024] = mem::zeroed();
                fgets(read_buffer.as_mut_ptr(), read_buffer.len() as i32, input_file);
                let cs = CString::new(read_buffer.as_ptr(), false);
                match cs.as_str() {
                    Some(s) => android_glue::write_log(s),
                    None => {},
                }
            }
        });
    }
}

#[cfg(target_os="android")]
fn setup_logging() {
    use libc::consts::os::posix88::{STDERR_FILENO, STDOUT_FILENO};
    //os::setenv("RUST_LOG", "servo,gfx,msg,util,layers,js,std,rt,extra");
    redirect_output(STDERR_FILENO);
    redirect_output(STDOUT_FILENO);
}

#[cfg(not(target_os="android"))]
fn setup_logging() {
}

fn main() {
    if opts::from_cmdline_args(get_args().as_slice()) {
        setup_logging();

        let window = if opts::get().headless {
            None
        } else {
            Some(app::create_window())
        };

        let mut browser = BrowserWrapper {
            browser: Browser::new(window.clone()),
        };

        match window {
            None => {}
            Some(ref window) => {
                unsafe {
                    window.set_nested_event_loop_listener(&mut browser);
                }
            }
        }

        browser.browser.handle_event(WindowEvent::InitializeCompositing);

        loop {
            let should_continue = match window {
                None => browser.browser.handle_event(WindowEvent::Idle),
                Some(ref window) => {
                    let event = window.wait_events();
                    browser.browser.handle_event(event)
                }
            };
            if !should_continue {
                break
            }
        };

        match window {
            None => {}
            Some(ref window) => {
                unsafe {
                    window.remove_nested_event_loop_listener();
                }
            }
        }

        let BrowserWrapper {
            browser
        } = browser;
        browser.shutdown();

        //rtinstrument::teardown();
    }
}

impl app::NestedEventLoopListener for BrowserWrapper {
    fn handle_event_from_nested_event_loop(&mut self, event: WindowEvent) -> bool {
        let is_resize = match event {
            WindowEvent::Resize(..) => true,
            _ => false,
        };
        if !self.browser.handle_event(event) {
            return false
        }
        if is_resize {
            self.browser.repaint_synchronously()
        }
        true
    }
}

