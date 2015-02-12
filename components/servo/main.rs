/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(env, os)]

#[cfg(target_os="android")]
extern crate libc;

extern crate servo;
extern crate time;
extern crate util;
extern crate net;

#[cfg(not(test))]
extern crate "glutin_app" as app;

#[cfg(not(test))]
extern crate compositing;

#[cfg(target_os="android")]
#[macro_use]
extern crate android_glue;

#[cfg(target_os="android")]
use libc::c_int;

#[cfg(not(test))]
use util::opts;

#[cfg(not(test))]
use net::resource_task;

#[cfg(not(test))]
use servo::Browser;
#[cfg(not(test))]
use compositing::windowing::WindowEvent;

#[cfg(target_os="android")]
use std::borrow::ToOwned;

#[cfg(not(test))]
struct BrowserWrapper {
    browser: Browser<app::window::Window>,
}

#[cfg(target_os="android")]
android_start!(main);

#[cfg(target_os="android")]
fn get_args() -> Vec<String> {
    vec![
        "servo".to_owned(),
        "http://en.wikipedia.org/wiki/Rust".to_owned()
    ]
}

#[cfg(not(target_os="android"))]
fn get_args() -> Vec<String> {
    use std::env;
    env::args().map(|s| s.into_string().unwrap()).collect()
}

#[cfg(target_os="android")]
struct FilePtr(*mut libc::types::common::c95::FILE);

#[cfg(target_os="android")]
unsafe impl Send for FilePtr {}

#[cfg(target_os="android")]
fn redirect_output(file_no: c_int) {
    use libc::funcs::posix88::unistd::{pipe, dup2};
    use libc::funcs::posix88::stdio::fdopen;
    use libc::funcs::c95::stdio::fgets;
    use util::task::spawn_named;
    use std::mem;
    use std::ffi::CString;
    use std::str::from_utf8;

    unsafe {
        let mut pipes: [c_int; 2] = [ 0, 0 ];
        pipe(pipes.as_mut_ptr());
        dup2(pipes[1], file_no);
        let mode = CString::from_slice("r".as_bytes());
        let input_file = FilePtr(fdopen(pipes[0], mode.as_ptr()));
        spawn_named("android-logger".to_owned(), move || {
            loop {
                let mut read_buffer: [u8; 1024] = mem::zeroed();
                let FilePtr(input_file) = input_file;
                fgets(read_buffer.as_mut_ptr() as *mut i8, read_buffer.len() as i32, input_file);
                let cs = CString::from_slice(&read_buffer);
                match from_utf8(cs.as_bytes()) {
                    Ok(s) => android_glue::write_log(s),
                    _ => {}
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
    if opts::from_cmdline_args(&*get_args()) {
        setup_logging();
        resource_task::global_init();

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

