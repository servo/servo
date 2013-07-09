/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A windowing implementation using GLFW.

use windowing::{ApplicationMethods, LoadUrlCallback, MouseCallback};
use windowing::{ResizeCallback, ScrollCallback, WindowMethods, WindowMouseEvent, WindowClickEvent};
use windowing::{WindowMouseDownEvent, WindowMouseUpEvent, ZoomCallback, Forward, Back, NavigationCallback};

use alert::{Alert, AlertMethods};
use std::libc::c_int;
use geom::point::Point2D;
use geom::size::Size2D;
use servo_msg::compositor_msg::{IdleRenderState, RenderState, RenderingRenderState};
use servo_msg::compositor_msg::{FinishedLoading, Loading, PerformingLayout, ReadyState};

use glfw;

static THROBBER: [char, ..8] = [ '⣾', '⣽', '⣻', '⢿', '⡿', '⣟', '⣯', '⣷' ];

/// A structure responsible for setting up and tearing down the entire windowing system.
pub struct Application;

impl ApplicationMethods for Application {
    pub fn new() -> Application {
        glfw::init();
        Application
    }
}

impl Drop for Application {
    fn drop(&self) {
        glfw::terminate();
    }
}

/// The type of a window.
pub struct Window {
    glfw_window: glfw::Window,

    resize_callback: Option<ResizeCallback>,
    load_url_callback: Option<LoadUrlCallback>,
    mouse_callback: Option<MouseCallback>,
    scroll_callback: Option<ScrollCallback>,
    zoom_callback: Option<ZoomCallback>,
    navigation_callback: Option<NavigationCallback>,

    drag_origin: Point2D<c_int>,

    mouse_down_button: @mut c_int,
    mouse_down_point: @mut Point2D<c_int>,

    ready_state: ReadyState,
    render_state: RenderState,
    throbber_frame: u8,
}

impl WindowMethods<Application> for Window {
    /// Creates a new window.
    pub fn new(_: &Application) -> @mut Window {
        // Create the GLFW window.
        let glfw_window = glfw::Window::create(800, 600, "Servo", glfw::Windowed).unwrap();
        glfw_window.make_context_current();

        // Create our window object.
        let window = @mut Window {
            glfw_window: glfw_window,

            resize_callback: None,
            load_url_callback: None,
            mouse_callback: None,
            scroll_callback: None,
            zoom_callback: None,
            navigation_callback: None,

            drag_origin: Point2D(0 as c_int, 0),

            mouse_down_button: @mut 0,
            mouse_down_point: @mut Point2D(0 as c_int, 0),

            ready_state: FinishedLoading,
            render_state: IdleRenderState,
            throbber_frame: 0,
        };

        // Register event handlers.
        do window.glfw_window.set_framebuffer_size_callback |_win, width, height| {
            match window.resize_callback {
                None => {}
                Some(callback) => callback(width as uint, height as uint),
            }
        }
        do window.glfw_window.set_key_callback |_win, key, _scancode, action, mods| {
            if action == glfw::PRESS {
                window.handle_key(key, mods)
            }
        }
        do window.glfw_window.set_mouse_button_callback |win, button, action, _mods| {
            let (x, y) = win.get_cursor_pos();
            if button < 3 {
                window.handle_mouse(button, action, x as i32, y as i32);
            }
        }
        do window.glfw_window.set_scroll_callback |_win, x_offset, y_offset| {
            let dx = (x_offset as f32) * 30.0;
            let dy = (y_offset as f32) * 30.0;
            
            window.handle_scroll(Point2D(dx, dy));
        }

        window
    }

    /// Returns the size of the window.
    pub fn size(&self) -> Size2D<f32> {
        let (width, height) = self.glfw_window.get_framebuffer_size();
        Size2D(width as f32, height as f32)
    }

    /// Presents the window to the screen (perhaps by page flipping).
    pub fn present(&mut self) {
        self.glfw_window.swap_buffers();
    }

    /// Registers a callback to run when a resize event occurs.
    pub fn set_resize_callback(&mut self, new_resize_callback: ResizeCallback) {
        self.resize_callback = Some(new_resize_callback)
    }

    /// Registers a callback to be run when a new URL is to be loaded.
    pub fn set_load_url_callback(&mut self, new_load_url_callback: LoadUrlCallback) {
        self.load_url_callback = Some(new_load_url_callback)
    }

    /// Registers a callback to be run when a mouse event occurs.
    pub fn set_mouse_callback(&mut self, new_mouse_callback: MouseCallback) {
        self.mouse_callback = Some(new_mouse_callback)
    }

    /// Registers a callback to be run when the user scrolls.
    pub fn set_scroll_callback(&mut self, new_scroll_callback: ScrollCallback) {
        self.scroll_callback = Some(new_scroll_callback)
    }

    /// Registers a zoom to be run when the user zooms.
    pub fn set_zoom_callback(&mut self, new_zoom_callback: ZoomCallback) {
        self.zoom_callback = Some(new_zoom_callback)
    }

    /// Registers a callback to be run when backspace or shift-backspace is pressed.
    pub fn set_navigation_callback(&mut self, new_navigation_callback: NavigationCallback) {
        self.navigation_callback = Some(new_navigation_callback)
    }

    /// Spins the event loop.
    pub fn check_loop(@mut self) {
        glfw::poll_events();
        self.throbber_frame = (self.throbber_frame + 1) % (THROBBER.len() as u8);
        self.update_window_title();
    }

    /// Sets the ready state.
    pub fn set_ready_state(@mut self, ready_state: ReadyState) {
        self.ready_state = ready_state;
        self.update_window_title()
    }

    /// Sets the render state.
    pub fn set_render_state(@mut self, render_state: RenderState) {
        self.render_state = render_state;
        self.update_window_title()
    }
}

impl Window {
    /// Helper function to set the window title in accordance with the ready state.
    fn update_window_title(&self) {
        let throbber = THROBBER[self.throbber_frame];
        match self.ready_state {
            Loading => {
                self.glfw_window.set_title(fmt!("%c Loading — Servo", throbber))
            }
            PerformingLayout => {
                self.glfw_window.set_title(fmt!("%c Performing Layout — Servo", throbber))
            }
            FinishedLoading => {
                match self.render_state {
                    RenderingRenderState => {
                        self.glfw_window.set_title(fmt!("%c Rendering — Servo", throbber))
                    }
                    IdleRenderState => self.glfw_window.set_title("Servo"),
                }
            }
        }
    }

    /// Helper function to handle keyboard events.
    fn handle_key(&self, key: c_int, mods: c_int) {
        match key {
            glfw::KEY_L if mods & glfw::MOD_CONTROL != 0 => self.load_url(), // Ctrl+L
            glfw::KEY_EQUAL if mods & glfw::MOD_CONTROL != 0 => { // Ctrl-+
                for self.zoom_callback.iter().advance |&callback| {
                    callback(1.1);
                }
            }
            glfw::KEY_MINUS if mods & glfw::MOD_CONTROL != 0 => { // Ctrl--
                for self.zoom_callback.iter().advance |&callback| {
                    callback(0.90909090909);
                }
            }
            glfw::KEY_BACKSPACE if mods & glfw::MOD_SHIFT != 0 => { // Shift-Backspace
                for self.navigation_callback.iter().advance |&callback| {
                    callback(Forward);
                }
            }
            glfw::KEY_BACKSPACE => { // Backspace
                for self.navigation_callback.iter().advance |&callback| {
                    callback(Back);
                }
            }
            _ => {}
        }
    }

    /// Helper function to handle a click
    fn handle_mouse(&self, button: c_int, action: c_int, x: c_int, y: c_int) {
        // FIXME(tkuehn): max pixel dist should be based on pixel density
        let max_pixel_dist = 10f;
        match self.mouse_callback {
            None => {}
            Some(callback) => {
                let event: WindowMouseEvent;
                match action {
                    glfw::PRESS => {
                        event = WindowMouseDownEvent(button as uint, Point2D(x as f32, y as f32));
                        *self.mouse_down_point = Point2D(x, y);
                        *self.mouse_down_button = button;
                    }
                    glfw::RELEASE => {
                        event = WindowMouseUpEvent(button as uint, Point2D(x as f32, y as f32));
                        if *self.mouse_down_button == button {
                            let pixel_dist = *self.mouse_down_point - Point2D(x, y);
                            let pixel_dist = ((pixel_dist.x * pixel_dist.x +
                                              pixel_dist.y * pixel_dist.y) as float).sqrt();
                            if pixel_dist < max_pixel_dist {
                                let click_event = WindowClickEvent(button as uint,
                                                                   Point2D(x as f32, y as f32));
                                callback(click_event);
                            }
                        }
                    }
                    _ => fail!("I cannot recognize the type of mouse action that occured. :-(")
                };
                callback(event);
            }
        }
    }

    /// Helper function to handle a scroll.
    fn handle_scroll(&mut self, delta: Point2D<f32>) {
        match self.scroll_callback {
            None => {}
            Some(callback) => callback(delta),
        }
    }

    /// Helper function to handle a zoom.
    fn handle_zoom(&mut self, magnification: f32) {
        match self.zoom_callback {
            None => {}
            Some(callback) => callback(magnification),
        }
    }

    /// Helper function to pop up an alert box prompting the user to load a URL.
    fn load_url(&self) {
        match self.load_url_callback {
            None => error!("no URL callback registered, doing nothing"),
            Some(callback) => {
                let mut alert: Alert = AlertMethods::new("Navigate to:");
                alert.add_prompt();
                alert.run();
                let value = alert.prompt_value();
                if "" == value {    // To avoid crashing on Linux.
                    callback("http://purple.com/")
                } else {
                    callback(value)
                }
            }
        }
    }
}

