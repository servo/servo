/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use compositing::windowing::{WindowMethods};
use geom::size::TypedSize2D;
use glfw_app;
use libc::{calloc, size_t,c_int};
use servo::Browser;
use servo_util::opts;
use servo_util::opts::OpenGL;
use std::cell::RefCell;
use std::mem;
use std::rc::Rc;
use std::string;
use types::{cef_browser_settings_t, cef_browser_t, cef_client_t, cef_request_context_t, cef_string_t, cef_window_info_t};

pub type servo_browser_t = servo_browser;
pub struct servo_browser {
    pub browser: cef_browser_t,
    pub servo_browser: Option<Browser<glfw_app::window::Window>>,
    pub window: Rc<glfw_app::window::Window>,
    pub running: bool
}

local_data_key!(pub GLOBAL_BROWSERS: RefCell<Vec<*mut servo_browser_t>>)

#[no_mangle]
pub extern "C" fn cef_browser_host_create_browser(window_info: *const cef_window_info_t,
                                                  client: *mut cef_client_t,
                                                  url: *const cef_string_t,
                                                  settings: *const cef_browser_settings_t,
                                                  request_context: *mut cef_request_context_t)
                                                  -> c_int {
    cef_browser_host_create_browser_sync(window_info, client, url, settings, request_context);
    1
}

#[no_mangle]
pub extern "C" fn cef_browser_host_create_browser_sync(_window_info: *const cef_window_info_t,
                                                       client: *mut cef_client_t,
                                                       url: *const cef_string_t,
                                                       _settings: *const cef_browser_settings_t,
                                                       _request_context: *mut cef_request_context_t)
                                                       -> *mut cef_browser_t {
    unsafe {
        let mut urls = Vec::new();
        if url.is_null() || (*url).str.is_null() {
            urls.push("http://s27.postimg.org/vqbtrolyr/servo.jpg".to_string());
        }
        else {
            urls.push(string::raw::from_buf((*url).str as *const u8));
        }
        opts::set_opts(opts::Opts {
            urls: urls,
            n_render_threads: 1,
            gpu_painting: false,
            tile_size: 512,
            device_pixels_per_px: None,
            time_profiler_period: None,
            memory_profiler_period: None,
            enable_experimental: false,
            layout_threads: 1,
            nonincremental_layout: false,
            //layout_threads: cmp::max(rt::default_sched_threads() * 3 / 4, 1),
            exit_after_load: false,
            output_file: None,
            headless: false,
            hard_fail: false,
            bubble_inline_sizes_separately: false,
            show_debug_borders: false,
            show_debug_fragment_borders: false,
            enable_text_antialiasing: true,
            trace_layout: false,
            devtools_port: None,
            initial_window_size: TypedSize2D(800, 600),
            profile_tasks: false,
            user_agent: None,
            dump_flow_tree: false,
            validate_display_list_geometry: false,
            render_api: OpenGL,
        });
        let browser = calloc(1, mem::size_of::<servo_browser_t>() as size_t) as *mut servo_browser_t;
        (*browser).browser.base.size = mem::size_of::<cef_browser_t>() as size_t;
        if client.is_not_null() {
            (*client).get_life_span_handler.map(|cb| {
                 let handler = cb(client);
                 if handler.is_not_null() {
                     (*handler).on_after_created.map(|createcb| createcb(handler, mem::transmute(browser)));
                 }
            });
        }
        match GLOBAL_BROWSERS.replace(None) {
            Some(brs) => {
                brs.borrow_mut().push(browser);
                GLOBAL_BROWSERS.replace(Some(brs));
            },
            None => {
                let brs = RefCell::new(vec!(browser));
                GLOBAL_BROWSERS.replace(Some(brs));
            }
        }
        mem::transmute(browser)
    }
}
