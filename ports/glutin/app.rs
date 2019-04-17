/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::{browser, window};
use servo::{Servo, BrowserId};
use servo::config::opts::{self, parse_url_or_filename};
use servo::compositing::windowing::WindowEvent;
use servo::servo_config::pref;
use servo::servo_url::ServoUrl;
use std::env;

pub struct App;

impl App {
    pub fn run() {
        let opts = opts::get();

        let foreground = opts.output_file.is_none() && !opts.headless;
        let window = window::Window::new(foreground, opts.initial_window_size);

        let mut browser = browser::Browser::new(window.clone());

        let mut servo = Servo::new(window.clone());
        let browser_id = BrowserId::new();
        servo.handle_events(vec![WindowEvent::NewBrowser(get_default_url(), browser_id)]);

        servo.setup_logging();

        window.run(|| {
            let win_events = window.get_events();

            // FIXME: this could be handled by Servo. We don't need
            // a repaint_synchronously function exposed.
            let need_resize = win_events.iter().any(|e| match *e {
                WindowEvent::Resize => true,
                _ => false,
            });

            browser.handle_window_events(win_events);

            let mut servo_events = servo.get_events();
            loop {
                browser.handle_servo_events(servo_events);
                servo.handle_events(browser.get_events());
                if browser.shutdown_requested() {
                    return true;
                }
                servo_events = servo.get_events();
                if servo_events.is_empty() {
                    break;
                }
            }

            if need_resize {
                servo.repaint_synchronously();
            }
            false
        });

        servo.deinit();
    }
}

fn get_default_url() -> ServoUrl {
    // If the url is not provided, we fallback to the homepage in prefs,
    // or a blank page in case the homepage is not set either.
    let cwd = env::current_dir().unwrap();
    let cmdline_url = opts::get().url.clone();
    let pref_url = {
        let homepage_url = pref!(shell.homepage);
        parse_url_or_filename(&cwd, &homepage_url).ok()
    };
    let blank_url = ServoUrl::parse("about:blank").ok();

    cmdline_url.or(pref_url).or(blank_url).unwrap()
}
