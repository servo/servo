/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use interfaces::{CefBrowser, CefClient, CefRequestContext, cef_browser_t, cef_client_t};
use interfaces::{cef_request_context_t};
use types::{cef_browser_settings_t, cef_string_t, cef_window_info_t};

use eutil::Downcast;
use glfw_app;
use libc::c_int;
use servo::Browser;
use servo_util::opts;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

pub struct ServoCefBrowser {
    pub client: CefClient,
    pub servo_browser: RefCell<Option<Browser<glfw_app::window::Window>>>,
    pub window: RefCell<Option<Rc<glfw_app::window::Window>>>,
    pub callback_executed: Cell<bool>,
}

impl ServoCefBrowser {
    pub fn new(client: CefClient) -> ServoCefBrowser {
        ServoCefBrowser {
            client: client,
            servo_browser: RefCell::new(None),
            window: RefCell::new(None),
            callback_executed: Cell::new(false),
        }
    }
}

cef_class_impl! {
    ServoCefBrowser : CefBrowser, cef_browser_t {}
}

local_data_key!(pub GLOBAL_BROWSERS: RefCell<Vec<CefBrowser>>)

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

fn browser_host_create(client: CefClient, callback_executed: bool) -> CefBrowser {
    let mut urls = Vec::new();
    urls.push("http://s27.postimg.org/vqbtrolyr/servo.jpg".to_string());
    let mut opts = opts::default_opts();
    opts.urls = urls;
    let browser = ServoCefBrowser::new(client).as_cef_interface();
    if callback_executed {
        browser_callback_after_created(browser.clone());
    }
    match GLOBAL_BROWSERS.replace(None) {
        Some(brs) => {
            brs.borrow_mut().push(browser.clone());
            GLOBAL_BROWSERS.replace(Some(brs));
        },
        None => {
            let brs = RefCell::new(vec!(browser.clone()));
            GLOBAL_BROWSERS.replace(Some(brs));
        }
    }
    browser
}

cef_static_method_impls! {
    fn cef_browser_host_create_browser(_window_info: *const cef_window_info_t,
                                       client: *mut cef_client_t,
                                       _url: *const cef_string_t,
                                       _browser_settings: *const cef_browser_settings_t,
                                       _request_context: *mut cef_request_context_t)
                                       -> c_int {
        let _window_info: &cef_window_info_t = _window_info;
        let client: CefClient = client;
        let _url: &[u16] = _url;
        let _browser_settings: &cef_browser_settings_t = _browser_settings;
        let _request_context: CefRequestContext = _request_context;
        browser_host_create(client, false);
        1i32
    }

    fn cef_browser_host_create_browser_sync(_window_info: *const cef_window_info_t,
                                            client: *mut cef_client_t,
                                            _url: *const cef_string_t,
                                            _browser_settings: *const cef_browser_settings_t,
                                            _request_context: *mut cef_request_context_t)
                                            -> *mut cef_browser_t {
        let _window_info: &cef_window_info_t = _window_info;
        let client: CefClient = client;
        let _url: &[u16] = _url;
        let _browser_settings: &cef_browser_settings_t = _browser_settings;
        let _request_context: CefRequestContext = _request_context;
        browser_host_create(client, true)
    }
}
