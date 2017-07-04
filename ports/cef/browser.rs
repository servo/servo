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
use servo::servo_url::ServoUrl;
use types::{cef_browser_settings_t, cef_string_t, cef_window_info_t, cef_window_handle_t};
use window;
use wrappers::CefWrap;

use compositing::windowing::{WindowNavigateMsg, WindowEvent};
use glutin_app;
use libc::c_int;
use std::cell::{Cell, RefCell};
use std::ptr;
use std::rc::Rc;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::thread;

thread_local!(pub static ID_COUNTER: AtomicIsize = AtomicIsize::new(0));
thread_local!(pub static BROWSERS: RefCell<Vec<CefBrowser>> = RefCell::new(vec!()));

pub enum ServoBrowser {
    Invalid,
    OnScreen(Browser<glutin_app::window::Window>),
    OffScreen(Browser<window::Window>),
}

impl ServoBrowser {
    fn handle_event(&mut self, event: WindowEvent) {
        match *self {
            ServoBrowser::OnScreen(ref mut browser) => { browser.handle_events(vec![event]); }
            ServoBrowser::OffScreen(ref mut browser) => { browser.handle_events(vec![event]); }
            ServoBrowser::Invalid => {}
        }
    }

    pub fn request_title_for_main_frame(&self) {
        match *self {
            ServoBrowser::OnScreen(ref browser) => browser.request_title_for_main_frame(),
            ServoBrowser::OffScreen(ref browser) => browser.request_title_for_main_frame(),
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

        fn can_go_back(&this,) -> c_int {{
            this.downcast().back.get() as c_int
        }}

        fn can_go_forward(&this,) -> c_int {{
            this.downcast().forward.get() as c_int
        }}

        fn is_loading(&this,) -> c_int {{
            this.downcast().loading.get() as c_int
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

        fn get_identifier(&this,) -> c_int {{
            this.downcast().id as c_int
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
    /// the glutin window when using windowed rendering
    pub window: Option<Rc<glutin_app::window::Window>>,
    /// Whether the on-created callback has fired yet.
    pub callback_executed: Cell<bool>,
    /// whether the browser can navigate back
    pub back: Cell<bool>,
    /// whether the browser can navigate forward
    pub forward: Cell<bool>,
    /// whether the browser is loading
    pub loading: Cell<bool>,
    /// a vec of favicon urls for the current page
    pub favicons: RefCell<Vec<String>>,
    /// the display system window handle: only to be used with host.get_window_handle()
    window_handle: cef_window_handle_t,

    id: isize,
    servo_browser: RefCell<ServoBrowser>,
    message_queue: RefCell<Vec<WindowEvent>>,
}

impl ServoCefBrowser {
    pub fn new(window_info: &cef_window_info_t, client: CefClient, target_url: ServoUrl) -> ServoCefBrowser {
        let frame = ServoCefFrame::new().as_cef_interface();
        let host = ServoCefBrowserHost::new(client.clone()).as_cef_interface();
        let mut window_handle: cef_window_handle_t = get_null_window_handle();

        let (glutin_window, servo_browser) = if window_info.windowless_rendering_enabled == 0 {
            let parent_window = glutin_app::WindowID::new(window_info.parent_window as *mut _);
            let glutin_window = glutin_app::create_window(Some(parent_window));
            let servo_browser = Browser::new(glutin_window.clone(), target_url);
            window_handle = glutin_window.platform_window().window as cef_window_handle_t;
            (Some(glutin_window), ServoBrowser::OnScreen(servo_browser))
        } else {
            (None, ServoBrowser::Invalid)
        };

        let id = ID_COUNTER.with(|counter| {
            counter.fetch_add(1, Ordering::SeqCst)
        });

        ServoCefBrowser {
            frame: frame,
            host: host,
            client: client,
            window: glutin_window,
            callback_executed: Cell::new(false),
            servo_browser: RefCell::new(servo_browser),
            message_queue: RefCell::new(vec!()),
            id: id,
            back: Cell::new(false),
            forward: Cell::new(false),
            loading: Cell::new(false),
            favicons: RefCell::new(vec!()),
            window_handle: window_handle,
        }
    }
}

pub trait ServoCefBrowserExtensions {
    fn init(&self, window_info: &cef_window_info_t);
    fn send_window_event(&self, event: WindowEvent);
    fn request_title_for_main_frame(&self);
    fn pinch_zoom_level(&self) -> f32;
}

impl ServoCefBrowserExtensions for CefBrowser {
    fn init(&self, window_info: &cef_window_info_t) {
        if window_info.windowless_rendering_enabled != 0 {
            let window = window::Window::new(window_info.width, window_info.height);
            window.set_browser(self.clone());
            let home_url = ServoUrl::parse("about:blank").unwrap();
            let servo_browser = Browser::new(window.clone(), home_url);
            *self.downcast().servo_browser.borrow_mut() = ServoBrowser::OffScreen(servo_browser);
        }

        self.downcast().host.set_browser((*self).clone());
        self.downcast().frame.set_browser((*self).clone());
        if window_info.windowless_rendering_enabled == 0 {
            self.downcast().host.initialize_compositing();
        }
    }

    fn send_window_event(&self, event: WindowEvent) {
        let browser = self.downcast();

        if let Ok(mut servo_browser) = browser.servo_browser.try_borrow_mut() {
            servo_browser.handle_event(event);
            while let Some(event) = browser.message_queue.borrow_mut().pop() {
                servo_browser.handle_event(event);
            }
        } else {
            // If we fail to borrow mutably, this means we're trying to send an
            // event while processing another one. This will cause general badness,
            // we just queue up that event instead of immediately processing it.
            browser.message_queue.borrow_mut().push(event);
        }
    }

    fn request_title_for_main_frame(&self) {
        self.downcast().servo_browser.borrow().request_title_for_main_frame()
    }

    fn pinch_zoom_level(&self) -> f32 {
        self.downcast().servo_browser.borrow().pinch_zoom_level()
    }
}

#[cfg(target_os="macos")]
pub fn get_null_window_handle() -> cef_window_handle_t {
    ptr::null_mut()
}
#[cfg(target_os="linux")]
pub fn get_null_window_handle() -> cef_window_handle_t {
    0
}

pub fn update() {
    BROWSERS.with(|browsers| {
        for browser in &*browsers.borrow() {
            if browser.downcast().callback_executed.get() == false {
                browser_callback_after_created(browser.clone());
            }
            let mut events = match browser.downcast().window {
                Some(ref win) => win.wait_events(),
                None => vec![WindowEvent::Idle]
            };
            loop {
               match events.pop() {
                   Some(event) => browser.send_window_event(event),
                   None => break
               }
            }
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

pub fn get_window(browser: &CefBrowser) -> cef_window_handle_t {
    browser.downcast().window_handle
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
    browser.downcast().frame.load();
    browser.downcast().host.was_resized();
}

fn browser_host_create(window_info: &cef_window_info_t,
                       client: CefClient,
                       url: *const cef_string_t,
                       callback_executed: bool)
                       -> CefBrowser {
    let url_string = if url != ptr::null() {
        unsafe { String::from_utf16(CefWrap::to_rust(url)).ok() }
    } else {
        None
    };
    let target_url = url_string
        .and_then(|val| ServoUrl::parse(&val).ok())
        .or(ServoUrl::parse("about:blank").ok())
        .unwrap();
    let browser = ServoCefBrowser::new(window_info, client, target_url).as_cef_interface();
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
                                       url: *const cef_string_t,
                                       _browser_settings: *const cef_browser_settings_t,
                                       _request_context: *mut cef_request_context_t)
                                       -> c_int {{
        let client: CefClient = client;
        let _browser_settings: &cef_browser_settings_t = _browser_settings;
        let _request_context: CefRequestContext = _request_context;
        browser_host_create(window_info, client, url, false);
        thread::Builder::new().name("async_browser_creation".to_owned()).spawn(move || {
            window::app_wakeup();
        }).expect("Thread spawning failed");
        1i32
    }}
    fn cef_browser_host_create_browser_sync(window_info: *const cef_window_info_t,
                                            client: *mut cef_client_t,
                                            url: *const cef_string_t,
                                            _browser_settings: *const cef_browser_settings_t,
                                            _request_context: *mut cef_request_context_t)
                                            -> *mut cef_browser_t {{
        let client: CefClient = client;
        let _browser_settings: &cef_browser_settings_t = _browser_settings;
        let _request_context: CefRequestContext = _request_context;
        browser_host_create(window_info, client, url, true)
    }}
}
