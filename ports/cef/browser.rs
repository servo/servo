/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use browser_host::{ServoCefBrowserHost, ServoCefBrowserHostExtensions};
use eutil::Downcast;
use frame::{ServoCefFrame, ServoCefFrameExtensions};
use interfaces::{CefBrowser, CefBrowserHost, CefClient, CefFrame, CefRequestContext};
use interfaces::{cef_browser_t, cef_browser_host_t, cef_client_t, cef_frame_t};
use interfaces::{cef_request_context_t};
use servo::Browser;
use types::{cef_browser_settings_t, cef_string_t, cef_window_info_t};
use window;

use compositing::windowing::{WindowNavigateMsg, WindowEvent};
use glutin_app;
use libc::c_int;
use util::opts;
use std::borrow::ToOwned;
use std::cell::{Cell, RefCell, BorrowState};
use std::sync::atomic::{AtomicInt, Ordering};

thread_local!(pub static ID_COUNTER: AtomicInt = AtomicInt::new(0));
thread_local!(pub static BROWSERS: RefCell<Vec<CefBrowser>> = RefCell::new(vec!()));

pub enum ServoBrowser {
    Invalid,
    OnScreen(Browser<glutin_app::window::Window>),
    OffScreen(Browser<window::Window>),
}

impl ServoBrowser {
    fn handle_event(&mut self, event: WindowEvent) {
        match *self {
            ServoBrowser::OnScreen(ref mut browser) => { browser.handle_event(event); }
            ServoBrowser::OffScreen(ref mut browser) => { browser.handle_event(event); }
            ServoBrowser::Invalid => {}
        }
    }

    pub fn get_title_for_main_frame(&self) {
        match *self {
            ServoBrowser::OnScreen(ref browser) => browser.get_title_for_main_frame(),
            ServoBrowser::OffScreen(ref browser) => browser.get_title_for_main_frame(),
            ServoBrowser::Invalid => {}
        }
    }

    pub fn pinch_zoom_level(&self) -> f32 {
        match *self {
            ServoBrowser::OnScreen(ref browser) => browser.pinch_zoom_level(),
            ServoBrowser::OffScreen(ref browser) => browser.pinch_zoom_level(),
            ServoBrowser::Invalid => 1.0,
        }
    }
}

cef_class_impl! {
    ServoCefBrowser : CefBrowser, cef_browser_t {
        fn get_host(&this,) -> *mut cef_browser_host_t {{
            this.downcast().host.clone()
        }}

        fn go_back(&this,) -> () {{
            this.send_window_event(WindowEvent::Navigation(WindowNavigateMsg::Back));
        }}

        fn go_forward(&this,) -> () {{
            this.send_window_event(WindowEvent::Navigation(WindowNavigateMsg::Forward));
        }}

        // Returns the main (top-level) frame for the browser window.
        fn get_main_frame(&this,) -> *mut cef_frame_t {{
            this.downcast().frame.clone()
        }}
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

    id: int,
    servo_browser: RefCell<ServoBrowser>,
    message_queue: RefCell<Vec<WindowEvent>>,
}

impl ServoCefBrowser {
    pub fn new(window_info: &cef_window_info_t, client: CefClient) -> ServoCefBrowser {
        let frame = ServoCefFrame::new().as_cef_interface();
        let host = ServoCefBrowserHost::new(client.clone()).as_cef_interface();

        let servo_browser = if window_info.windowless_rendering_enabled == 0 {
            let glutin_window = glutin_app::create_window();
            let servo_browser = Browser::new(Some(glutin_window.clone()));
            ServoBrowser::OnScreen(servo_browser)
        } else {
            ServoBrowser::Invalid
        };

        let id = ID_COUNTER.with(|counter| {
            counter.fetch_add(1, Ordering::SeqCst)
        });

        ServoCefBrowser {
            frame: frame,
            host: host,
            client: client,
            callback_executed: Cell::new(false),
            servo_browser: RefCell::new(servo_browser),
            message_queue: RefCell::new(vec!()),
            id: id,
        }
    }
}

pub trait ServoCefBrowserExtensions {
    fn init(&self, window_info: &cef_window_info_t);
    fn send_window_event(&self, event: WindowEvent);
    fn get_title_for_main_frame(&self);
    fn pinch_zoom_level(&self) -> f32;
}

impl ServoCefBrowserExtensions for CefBrowser {
    fn init(&self, window_info: &cef_window_info_t) {
        if window_info.windowless_rendering_enabled != 0 {
            let window = window::Window::new();
            let servo_browser = Browser::new(Some(window.clone()));
            window.set_browser(self.clone());
            *self.downcast().servo_browser.borrow_mut() = ServoBrowser::OffScreen(servo_browser);
        }

        self.downcast().host.set_browser((*self).clone());
        self.downcast().frame.set_browser((*self).clone());
    }

    fn send_window_event(&self, event: WindowEvent) {
        self.downcast().message_queue.borrow_mut().push(event);

        loop {
            match self.downcast().servo_browser.borrow_state() {
                BorrowState::Unused => {
                    let event = match self.downcast().message_queue.borrow_mut().pop() {
                        None => return,
                        Some(event) => event,
                    };
                    self.downcast().servo_browser.borrow_mut().handle_event(event);
                }
                _ => {
                    // We're trying to send an event while processing another one. This will
                    // cause general badness, so queue up that event instead of immediately
                    // processing it.
                    break
                }
            }
        }
    }

    fn get_title_for_main_frame(&self) {
        self.downcast().servo_browser.borrow().get_title_for_main_frame()
    }

    fn pinch_zoom_level(&self) -> f32 {
        self.downcast().servo_browser.borrow().pinch_zoom_level()
    }
}

pub fn update() {
    BROWSERS.with(|browsers| {
        for browser in browsers.borrow().iter() {
            browser.send_window_event(WindowEvent::Idle);
        }
    });
}

pub fn close(browser: CefBrowser) {
    BROWSERS.with(|browsers| {
        let mut browsers = browsers.borrow_mut();
        browsers.iter()
                .position(|&ref n| n.downcast().id == browser.downcast().id)
                .map(|e| browsers.remove(e));
    });
}

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
    urls.push("http://s27.postimg.org/vqbtrolyr/servo.jpg".to_owned());
    let mut opts = opts::default_opts();
    opts.urls = urls;
    let browser = ServoCefBrowser::new(window_info, client).as_cef_interface();
    browser.init(window_info);
    if callback_executed {
        browser_callback_after_created(browser.clone());
    }
    BROWSERS.with(|browsers| {
        browsers.borrow_mut().push(browser.clone());
    });
    browser
}

cef_static_method_impls! {
    fn cef_browser_host_create_browser(window_info: *const cef_window_info_t,
                                       client: *mut cef_client_t,
                                       _url: *const cef_string_t,
                                       _browser_settings: *const cef_browser_settings_t,
                                       _request_context: *mut cef_request_context_t)
                                       -> c_int {{
        let client: CefClient = client;
        let _url: &[u16] = _url;
        let _browser_settings: &cef_browser_settings_t = _browser_settings;
        let _request_context: CefRequestContext = _request_context;
        browser_host_create(window_info, client, false);
        1i32
    }}
    fn cef_browser_host_create_browser_sync(window_info: *const cef_window_info_t,
                                            client: *mut cef_client_t,
                                            _url: *const cef_string_t,
                                            _browser_settings: *const cef_browser_settings_t,
                                            _request_context: *mut cef_request_context_t)
                                            -> *mut cef_browser_t {{
        let client: CefClient = client;
        let _url: &[u16] = _url;
        let _browser_settings: &cef_browser_settings_t = _browser_settings;
        let _request_context: CefRequestContext = _request_context;
        browser_host_create(window_info, client, true)
    }}
}

