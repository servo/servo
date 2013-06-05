/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A windowing implementation using GLUT.
///
/// GLUT is a very old and bare-bones toolkit. However, it has good cross-platform support, at
/// least on desktops. It is designed for testing Servo without the need of a UI.

use windowing::{ApplicationMethods, CompositeCallback, LoadUrlCallback, ClickCallback};
use windowing::{ResizeCallback, ScrollCallback, ZoomCallback, WindowMethods};

use alert::{Alert, AlertMethods};
use core::libc::c_int;
use geom::point::Point2D;
use geom::size::Size2D;
use glut::glut::{DOUBLE, WindowHeight, WindowWidth};
use glut::glut;

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
    click_callback: Option<ClickCallback>,
    scroll_callback: Option<ScrollCallback>,
    zoom_callback: Option<ZoomCallback>,

    drag_origin: Point2D<c_int>,
    down_button: c_int
}

impl WindowMethods<Application> for Window {
    /// Creates a new window.
    pub fn new(_: &Application) -> @mut Window {
        // Create the GLUT window.
        let glut_window = glut::create_window(~"Servo");
        glut::reshape_window(glut_window, 800, 600);

        // Create our window object.
        let window = @mut Window {
            glut_window: glut_window,

            composite_callback: None,
            resize_callback: None,
            load_url_callback: None,
            click_callback: None,
            scroll_callback: None,
            zoom_callback: None,

            drag_origin: Point2D(0, 0),
            down_button: 0 // FIXME: Hacky solution to 2 button mouse. 
                           // Replace with tkuehn's code.
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
        do glut::mouse_func |button, _, x, y| {
            window.handle_click(x, y);
            window.down_button = button;
            window.start_drag(x, y)
        }
        do glut::motion_func |x, y| {
            window.continue_drag(x, y)
        }

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

    /// Registers a callback to be run when a click event occurs.
    pub fn set_click_callback(&mut self, new_click_callback: ClickCallback) {
        self.click_callback = Some(new_click_callback)
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
}

impl Window {
    /// Helper function to handle keyboard events.
    fn handle_key(&self, key: u8) {
        debug!("got key: %d", key as int);
        if key == 12 {  // ^L
            self.load_url()
        }
    }

    /// Helper function to handle a click
    fn handle_click(&self, x: c_int, y: c_int) {
        match self.click_callback {
            None => {}
            Some(callback) => callback(Point2D(x as f32, y as f32)),
        }
    }

    /// Helper function to start a drag.
    fn start_drag(&mut self, x: c_int, y: c_int) {
        self.drag_origin = Point2D(x, y)
    }

    /// Helper function to continue a drag.
    fn continue_drag(&mut self, x: c_int, y: c_int) {
        let new_point = Point2D(x, y);
        let delta = new_point - self.drag_origin;
        self.drag_origin = new_point;

        if self.down_button == 2 { 
            match self.zoom_callback {
                None => {}
                Some(callback) => callback(Point2D(delta.x as f32, delta.y as f32)),
            }
        } else {
            match self.scroll_callback {
                None => {}
                Some(callback) => callback(Point2D(delta.x as f32, delta.y as f32)),
            }
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

