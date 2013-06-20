/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A windowing implementation using GLUT.
///
/// GLUT is a very old and bare-bones toolkit. However, it has good cross-platform support, at
/// least on desktops. It is designed for testing Servo without the need of a UI.

use windowing::{ApplicationMethods, CompositeCallback, LoadUrlCallback, MouseCallback};
use windowing::{ResizeCallback, ScrollCallback, WindowMethods, WindowMouseEvent, WindowClickEvent};
use windowing::{WindowMouseDownEvent, WindowMouseUpEvent, ZoomCallback};

use alert::{Alert, AlertMethods};
use core::libc::c_int;
use geom::point::Point2D;
use geom::size::Size2D;
use servo_msg::compositor::{IdleRenderState, RenderState, RenderingRenderState};
use servo_msg::compositor::{FinishedLoading, Loading, PerformingLayout, ReadyState};
use glut::glut::{ACTIVE_CTRL, DOUBLE, HAVE_PRECISE_MOUSE_WHEEL, WindowHeight, WindowWidth};
use glut::glut;
use glut::machack;

static THROBBER: [char, ..8] = [ '⣾', '⣽', '⣻', '⢿', '⡿', '⣟', '⣯', '⣷' ];

/// A structure responsible for setting up and tearing down the entire windowing system.
pub struct Application;

impl ApplicationMethods for Application {
    pub fn new() -> Application {
        glut::init();
        glut::init_display_mode(DOUBLE);
        Application
    }
}

/// The type of a window.
pub struct Window {
    glut_window: glut::Window,

    composite_callback: Option<CompositeCallback>,
    resize_callback: Option<ResizeCallback>,
    load_url_callback: Option<LoadUrlCallback>,
    mouse_callback: Option<MouseCallback>,
    scroll_callback: Option<ScrollCallback>,
    zoom_callback: Option<ZoomCallback>,

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
        // Create the GLUT window.
        // FIXME (Rust #3080): These unsafe blocks are *not* unused!
        /*unsafe { */glut::bindgen::glutInitWindowSize(800, 600);/* }*/
        let glut_window = glut::create_window(~"Servo");

        // Create our window object.
        let window = @mut Window {
            glut_window: glut_window,

            composite_callback: None,
            resize_callback: None,
            load_url_callback: None,
            mouse_callback: None,
            scroll_callback: None,
            zoom_callback: None,

            drag_origin: Point2D(0, 0),

            mouse_down_button: @mut 0,
            mouse_down_point: @mut Point2D(0, 0),

            ready_state: FinishedLoading,
            render_state: IdleRenderState,
            throbber_frame: 0,
        };

        // Spin the event loop every 50 ms to allow the Rust channels to be polled.
        //
        // This requirement is pretty much the nail in the coffin for GLUT's usefulness.
        //
        // FIXME(pcwalton): What a mess.
        let register_timer_callback: @mut @fn() = @mut ||{};
        *register_timer_callback = || {
            glut::timer_func(50, *register_timer_callback);
            window.throbber_frame = (window.throbber_frame + 1) % (THROBBER.len() as u8);
            window.update_window_title()
        };

        // Register event handlers.
        do glut::reshape_func(window.glut_window) |width, height| {
            match window.resize_callback {
                None => {}
                Some(callback) => callback(width as uint, height as uint),
            }
        }
        do glut::display_func {
            // FIXME(pcwalton): This will not work with multiple windows.
            match window.composite_callback {
                None => {}
                Some(callback) => callback(),
            }
        }
        do glut::keyboard_func |key, _, _| {
            window.handle_key(key)
        }
        do glut::mouse_func |button, state, x, y| {
            if button < 3 {
                window.handle_mouse(button, state, x, y);
            }
        }
        do glut::mouse_wheel_func |wheel, direction, _x, _y| {
            let delta = if HAVE_PRECISE_MOUSE_WHEEL {
                (direction as f32) / 10000.0
            } else {
                (direction as f32) * 30.0
            };

            match wheel {
                1 => window.handle_scroll(Point2D(delta, 0.0)),
                2 => window.handle_zoom(delta),
                _ => window.handle_scroll(Point2D(0.0, delta)),
            }
        }
        (*register_timer_callback)();

        machack::perform_scroll_wheel_hack();

        window
    }

    /// Returns the size of the window.
    pub fn size(&self) -> Size2D<f32> {
        Size2D(glut::get(WindowWidth) as f32, glut::get(WindowHeight) as f32)
    }

    /// Presents the window to the screen (perhaps by page flipping).
    pub fn present(&mut self) {
        glut::swap_buffers();
    }

    /// Registers a callback to run when a composite event occurs.
    pub fn set_composite_callback(&mut self, new_composite_callback: CompositeCallback) {
        self.composite_callback = Some(new_composite_callback)
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

    /// Spins the event loop.
    pub fn check_loop(@mut self) {
        glut::check_loop()
    }

    /// Schedules a redisplay.
    pub fn set_needs_display(@mut self) {
        glut::post_redisplay()
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
                glut::set_window_title(self.glut_window, fmt!("%c Loading — Servo", throbber))
            }
            PerformingLayout => {
                glut::set_window_title(self.glut_window,
                                       fmt!("%c Performing Layout — Servo", throbber))
            }
            FinishedLoading => {
                match self.render_state {
                    RenderingRenderState => {
                        glut::set_window_title(self.glut_window,
                                               fmt!("%c Rendering — Servo", throbber))
                    }
                    IdleRenderState => glut::set_window_title(self.glut_window, "Servo"),
                }
            }
        }
    }

    /// Helper function to handle keyboard events.
    fn handle_key(&self, key: u8) {
        debug!("got key: %d", key as int);
        match key {
            12 => self.load_url(),                                                      // Ctrl+L
            k if k == ('=' as u8) && (glut::get_modifiers() & ACTIVE_CTRL) != 0 => {    // Ctrl++
                for self.zoom_callback.each |&callback| {
                    callback(0.1);
                }
            }
            k if k == 31 && (glut::get_modifiers() & ACTIVE_CTRL) != 0 => {             // Ctrl+-
                for self.zoom_callback.each |&callback| {
                    callback(-0.1);
                }
            }
            _ => {}
        }
    }

    /// Helper function to handle a click
    fn handle_mouse(&self, button: c_int, state: c_int, x: c_int, y: c_int) {
        // FIXME(tkuehn): max pixel dist should be based on pixel density
        let max_pixel_dist = 10f;
        match self.mouse_callback {
            None => {}
            Some(callback) => {
                let event: WindowMouseEvent;
                match state {
                    glut::MOUSE_DOWN => {
                        event = WindowMouseDownEvent(button as uint, Point2D(x as f32, y as f32));
                        *self.mouse_down_point = Point2D(x, y);
                        *self.mouse_down_button = button;
                    }
                    glut::MOUSE_UP => {
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

