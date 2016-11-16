/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The `servo` test application.
//!
//! Creates a `Browser` instance with a simple implementation of
//! the compositor's `WindowMethods` to create a working web browser.
//!
//! This browser's implementation of `WindowMethods` is built on top
//! of [glutin], the cross-platform OpenGL utility and windowing
//! library.
//!
//! For the engine itself look next door in `components/servo/lib.rs`.
//!
//! [glutin]: https://github.com/tomaka/glutin

#![feature(start, core_intrinsics)]

#[cfg(target_os = "android")]
#[macro_use]
extern crate android_glue;
extern crate backtrace;
// The window backed by glutin
extern crate glutin_app as app;
#[cfg(target_os = "android")]
extern crate libc;
#[macro_use]
extern crate log;
// The Servo engine
extern crate servo;
#[cfg(not(target_os = "android"))]
#[macro_use]
extern crate sig;

use backtrace::Backtrace;
use servo::Browser;
use servo::compositing::windowing::WindowEvent;
use servo::util::opts::{self, ArgumentParsingResult};
use servo::util::servo_version;
use std::env;
use std::panic;
use std::process;
use std::rc::Rc;
use std::thread;

pub mod platform {
    #[cfg(target_os = "macos")]
    pub use platform::macos::deinit;

    #[cfg(target_os = "macos")]
    pub mod macos;

    #[cfg(not(target_os = "macos"))]
    pub fn deinit() {}
}

#[cfg(not(target_os = "android"))]
fn install_crash_handler() {
    use backtrace::Backtrace;
    use sig::ffi::Sig;
    use std::intrinsics::abort;
    use std::thread;

    fn handler(_sig: i32) {
        let name = thread::current().name()
                                    .map(|n| format!(" for thread \"{}\"", n))
                                    .unwrap_or("".to_owned());
        println!("Stack trace{}\n{:?}", name, Backtrace::new());
        unsafe {
            abort();
        }
    }

    signal!(Sig::SEGV, handler); // handle segfaults
    signal!(Sig::ILL, handler); // handle stack overflow and unsupported CPUs
    signal!(Sig::IOT, handler); // handle double panics
    signal!(Sig::BUS, handler); // handle invalid memory access
}

#[cfg(target_os = "android")]
fn install_crash_handler() {
}

fn main() {
    install_crash_handler();

    // Parse the command line options and store them globally
    let opts_result = opts::from_cmdline_args(&*args());

    let content_process_token = if let ArgumentParsingResult::ContentProcess(token) = opts_result {
        Some(token)
    } else {
        if opts::get().is_running_problem_test && ::std::env::var("RUST_LOG").is_err() {
            ::std::env::set_var("RUST_LOG", "compositing::constellation");
        }

        None
    };

    // TODO: once log-panics is released, can this be replaced by
    // log_panics::init()?
    panic::set_hook(Box::new(|info| {
        warn!("Panic hook called.");
        let msg = match info.payload().downcast_ref::<&'static str>() {
            Some(s) => *s,
            None => match info.payload().downcast_ref::<String>() {
                Some(s) => &**s,
                None => "Box<Any>",
            },
        };
        let current_thread = thread::current();
        let name = current_thread.name().unwrap_or("<unnamed>");
        if let Some(location) = info.location() {
            println!("{} (thread {}, at {}:{})", msg, name, location.file(), location.line());
        } else {
            println!("{} (thread {})", msg, name);
        }
        if env::var("RUST_BACKTRACE").is_ok() {
            println!("{:?}", Backtrace::new());
        }

        error!("{}", msg);
    }));

    setup_logging();

    if let Some(token) = content_process_token {
        return servo::run_content_process(token)
    }

    if opts::get().is_printing_version {
        println!("{}", servo_version());
        process::exit(0);
    }

    let window = app::create_window(None);

    // Our wrapper around `Browser` that also implements some
    // callbacks required by the glutin window implementation.
    let mut browser = BrowserWrapper {
        browser: Browser::new(window.clone()),
    };

    browser.browser.setup_logging();

    register_glutin_resize_handler(&window, &mut browser);

    browser.browser.handle_events(vec![WindowEvent::InitializeCompositing]);

    // Feed events from the window to the browser until the browser
    // says to stop.
    loop {
        let should_continue = browser.browser.handle_events(window.wait_events());
        if !should_continue {
            break
        }
    };

    unregister_glutin_resize_handler(&window);

    platform::deinit()
}

fn register_glutin_resize_handler(window: &Rc<app::window::Window>,
                                        browser: &mut BrowserWrapper) {
    unsafe {
        window.set_nested_event_loop_listener(browser);
    }
}

fn unregister_glutin_resize_handler(window: &Rc<app::window::Window>) {
    unsafe {
        window.remove_nested_event_loop_listener();
    }
}

struct BrowserWrapper {
    browser: Browser<app::window::Window>,
}

impl app::NestedEventLoopListener for BrowserWrapper {
    fn handle_event_from_nested_event_loop(&mut self, event: WindowEvent) -> bool {
        let is_resize = match event {
            WindowEvent::Resize(..) => true,
            _ => false,
        };
        if !self.browser.handle_events(vec![event]) {
            return false
        }
        if is_resize {
            self.browser.repaint_synchronously()
        }
        true
    }
}

#[cfg(target_os = "android")]
fn setup_logging() {
    android::setup_logging();
}

#[cfg(not(target_os = "android"))]
fn setup_logging() {
}

#[cfg(target_os = "android")]
/// Attempt to read parameters from a file since they are not passed to us in Android environments.
/// The first line should be the "servo" argument and the last should be the URL to load.
/// Blank lines and those beginning with a '#' are ignored.
/// Each line should be a separate parameter as would be parsed by the shell.
/// For example, "servo -p 10 http://en.wikipedia.org/wiki/Rust" would take 4 lines.
fn args() -> Vec<String> {
    use std::error::Error;
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    const PARAMS_FILE: &'static str = "/sdcard/servo/android_params";
    match File::open(PARAMS_FILE) {
        Ok(f) => {
            let mut vec = Vec::new();
            let file = BufReader::new(&f);
            for line in file.lines() {
                let l = line.unwrap().trim().to_owned();
                // ignore blank lines and those that start with a '#'
                match l.is_empty() || l.as_bytes()[0] == b'#' {
                    true => (),
                    false => vec.push(l),
                }
            }
            vec
        },
        Err(e) => {
            debug!("Failed to open params file '{}': {}", PARAMS_FILE, Error::description(&e));
            vec![
                "servo".to_owned(),
                "http://en.wikipedia.org/wiki/Rust".to_owned()
            ]
        },
    }
}

#[cfg(not(target_os = "android"))]
fn args() -> Vec<String> {
    use std::env;
    env::args().collect()
}


// This extern definition ensures that the linker will not discard
// the static native lib bits, which are brought in from the NDK libraries
// we link in from build.rs.
#[cfg(target_os = "android")]
extern {
    fn app_dummy() -> libc::c_void;
}


#[cfg(target_os = "android")]
mod android {
    extern crate android_glue;
    extern crate libc;

    use self::libc::c_int;
    use std::borrow::ToOwned;

    pub fn setup_logging() {
        use self::libc::{STDERR_FILENO, STDOUT_FILENO};
        //use std::env;

        //env::set_var("RUST_LOG", "servo,gfx,msg,util,layers,js,std,rt,extra");
        redirect_output(STDERR_FILENO);
        redirect_output(STDOUT_FILENO);

        unsafe { super::app_dummy(); }
    }

    struct FilePtr(*mut self::libc::FILE);

    unsafe impl Send for FilePtr {}

    fn redirect_output(file_no: c_int) {
        use self::libc::{pipe, dup2};
        use self::libc::fdopen;
        use self::libc::fgets;
        use servo::util::thread::spawn_named;
        use std::ffi::CStr;
        use std::ffi::CString;
        use std::str::from_utf8;

        unsafe {
            let mut pipes: [c_int; 2] = [ 0, 0 ];
            pipe(pipes.as_mut_ptr());
            dup2(pipes[1], file_no);
            let mode = CString::new("r").unwrap();
            let input_file = FilePtr(fdopen(pipes[0], mode.as_ptr()));
            spawn_named("android-logger".to_owned(), move || {
                static READ_SIZE: usize = 1024;
                let mut read_buffer = vec![0; READ_SIZE];
                let FilePtr(input_file) = input_file;
                loop {
                    fgets(read_buffer.as_mut_ptr(), (read_buffer.len() as i32)-1, input_file);
                    let c_str = CStr::from_ptr(read_buffer.as_ptr());
                    let slice = from_utf8(c_str.to_bytes()).unwrap();
                    android_glue::write_log(slice);
                }
            });
        }
    }

}
