/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use browser_host::{ServoCefBrowserHost, ServoCefBrowserHostExtensions};
use core::{mod, ServoCefGlobals, globals};
use eutil::Downcast;
use frame::ServoCefFrame;
use interfaces::{CefBrowser, CefBrowserHost, CefClient, CefFrame, CefRequestContext};
use interfaces::{cef_browser_t, cef_browser_host_t, cef_client_t, cef_frame_t};
use interfaces::{cef_request_context_t};
use servo::Browser;
use types::{cef_browser_settings_t, cef_string_t, cef_window_info_t};
use window;

use compositing::windowing::{WindowNavigateMsg, WindowEvent};
use glfw_app;
use libc::c_int;
use servo_util::opts;
use std::cell::{Cell, RefCell};

cef_class_impl! {
    ServoCefBrowser : CefBrowser, cef_browser_t {
        fn get_host(&this) -> *mut cef_browser_host_t {
            this.downcast().host.clone()
        }

        fn go_back(&_this) -> () {
            core::send_window_event(WindowEvent::Navigation(WindowNavigateMsg::Back));
        }

        fn go_forward(&_this) -> () {
            core::send_window_event(WindowEvent::Navigation(WindowNavigateMsg::Forward));
        }

        // Returns the main (top-level) frame for the browser window.
        fn get_main_frame(&this) -> *mut cef_frame_t {
            this.downcast().frame.clone()
        }
    }
}

pub struct ServoCefBrowser {
    /// A reference to the browser's primary frame.
    pub frame: CefFrame,
    /// A reference to the browser's host.
    pub host: CefBrowserHost,
    /// A reference to the browser client.
    pub client: CefClient,
    /// Whether the on-created callback has fired yet.
    pub callback_executed: Cell<bool>,
}

impl ServoCefBrowser {
    pub fn new(window_info: &cef_window_info_t, client: CefClient) -> ServoCefBrowser {
        let frame = ServoCefFrame::new().as_cef_interface();
        let host = ServoCefBrowserHost::new(client.clone()).as_cef_interface();
        if window_info.windowless_rendering_enabled == 0 {
            globals.with(|ref r| {
                let glfw_window = glfw_app::create_window();
                *r.borrow_mut() = Some(ServoCefGlobals::OnScreenGlobals(
                    RefCell::new(glfw_window.clone()),
                    RefCell::new(Browser::new(Some(glfw_window)))));
            });
        }

        ServoCefBrowser {
            frame: frame,
            host: host,
            client: client,
            callback_executed: Cell::new(false),
        }
    }
}

trait ServoCefBrowserExtensions {
    fn init(&self, window_info: &cef_window_info_t);
}

impl ServoCefBrowserExtensions for CefBrowser {
    fn init(&self, window_info: &cef_window_info_t) {
        if window_info.windowless_rendering_enabled != 0 {
            globals.with(|ref r| {
                let window = window::Window::new();
                let servo_browser = Browser::new(Some(window.clone()));
                window.set_browser(self.clone());

                *r.borrow_mut() = Some(ServoCefGlobals::OffScreenGlobals(
                    RefCell::new(window),
                    RefCell::new(servo_browser)));
            });
        }

        self.downcast().host.set_browser((*self).clone());
    }
}

thread_local!(pub static GLOBAL_BROWSERS: RefCell<Vec<CefBrowser>> = RefCell::new(vec!()))

pub fn browser_callback_after_created(browser: CefBrowser) {
    if browser.downcast().client.is_null_cef_object() {
        return
    }
    let client = browser.downcast().client.clone();
    let life_span_handler = client.get_life_span_handler();
    if life_span_handler.is_not_null_cef_object() {
        life_span_handler.on_after_created(browser.clone());
    }
    browser.downcast().callback_executed.set(true);
}

fn browser_host_create(window_info: &cef_window_info_t,
                       client: CefClient,
                       callback_executed: bool)
                       -> CefBrowser {
    let mut urls = Vec::new();
    urls.push("http://s27.postimg.org/vqbtrolyr/servo.jpg".into_string());
    let mut opts = opts::default_opts();
    opts.urls = urls;
    let browser = ServoCefBrowser::new(window_info, client).as_cef_interface();
    browser.init(window_info);
    if callback_executed {
        browser_callback_after_created(browser.clone());
    }
    GLOBAL_BROWSERS.with(|ref r| r.borrow_mut().push(browser.clone()));
    browser
}

cef_static_method_impls! {
    fn cef_browser_host_create_browser(window_info: *const cef_window_info_t,
                                       client: *mut cef_client_t,
                                       _url: *const cef_string_t,
                                       _browser_settings: *const cef_browser_settings_t,
                                       _request_context: *mut cef_request_context_t)
                                       -> c_int {
        let client: CefClient = client;
        let _url: &[u16] = _url;
        let _browser_settings: &cef_browser_settings_t = _browser_settings;
        let _request_context: CefRequestContext = _request_context;
        browser_host_create(window_info, client, false);
        1i32
    }
    fn cef_browser_host_create_browser_sync(window_info: *const cef_window_info_t,
                                            client: *mut cef_client_t,
                                            _url: *const cef_string_t,
                                            _browser_settings: *const cef_browser_settings_t,
                                            _request_context: *mut cef_request_context_t)
                                            -> *mut cef_browser_t {
        let client: CefClient = client;
        let _url: &[u16] = _url;
        let _browser_settings: &cef_browser_settings_t = _browser_settings;
        let _request_context: CefRequestContext = _request_context;
        browser_host_create(window_info, client, true)
    }
}

