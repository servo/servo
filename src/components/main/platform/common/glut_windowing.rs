/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A windowing implementation using GLUT.
///
/// GLUT is a very old and bare-bones toolkit. However, it has good cross-platform support, at
/// least on desktops. It is designed for testing Servo without the need of a UI.

use windowing::{ApplicationMethods, CompositeWindowEvent, IdleWindowEvent, MouseWindowClickEvent};
use windowing::{MouseWindowEventClass, MouseWindowMouseDownEvent, MouseWindowMouseUpEvent};
use windowing::{NavigateWindowEvent, ResizeWindowEvent, ScrollWindowEvent, WindowEvent};
use windowing::{WindowMethods, ZoomWindowEvent};

use alert::{Alert, AlertMethods};
use core::cell::Cell;
use core::libc::c_int;
use core::util;
use geom::point::Point2D;
use geom::size::Size2D;
use gfx::compositor::{IdleRenderState, RenderState, RenderingRenderState};
use glut::glut::{ACTIVE_CTRL, DOUBLE, HAVE_PRECISE_MOUSE_WHEEL, WindowHeight, WindowWidth};
use glut::glut;
use glut::machack;
use script::compositor_interface::{FinishedLoading, Loading, PerformingLayout, ReadyState};

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

    event_queue: @mut ~[WindowEvent],

    drag_origin: Point2D<c_int>,

    mouse_down_button: @mut c_int,
    mouse_down_point: @mut Point2D<c_int>,

    ready_state: ReadyState,
    render_state: RenderState,
    throbber_frame: u8,
}

impl WindowMethods<Application> for Window {
    /// Creates a new window.
    fn new(_: &Application) -> @mut Window {
        // Create the GLUT window.
        unsafe { glut::bindgen::glutInitWindowSize(800, 600); }
        let glut_window = glut::create_window(~"Servo");

        // Create our window object.
        let window = @mut Window {
            glut_window: glut_window,

            event_queue: @mut ~[],

            drag_origin: Point2D(0, 0),

            mouse_down_button: @mut 0,
            mouse_down_point: @mut Point2D(0, 0),

            ready_state: FinishedLoading,
            render_state: IdleRenderState,
            throbber_frame: 0,
        };

        let event_queue = window.event_queue;

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
            event_queue.push(ResizeWindowEvent(Size2D(width as uint, height as uint)))
        }
        do glut::display_func {
            // FIXME(pcwalton): This will not work with multiple windows.
            event_queue.push(CompositeWindowEvent)
        }
        do glut::keyboard_func |key, _, _| {
            window.handle_key(key)
        }
        do glut::mouse_func |button, state, x, y| {
            if button < 3 {
                window.handle_mouse(button, state, x, y);
            }
        }
        do glut::mouse_wheel_func |wheel, direction, x, y| {
            let delta = if HAVE_PRECISE_MOUSE_WHEEL {
                (direction as f32) / 10000.0
            } else {
                (direction as f32) * 30.0
            };

            match wheel {
                1 => event_queue.push(ScrollWindowEvent(Point2D(delta, 0.0))),
                2 => event_queue.push(ZoomWindowEvent(delta)),
                _ => event_queue.push(ScrollWindowEvent(Point2D(0.0, delta))),
            }
        }
        (*register_timer_callback)();

        machack::perform_scroll_wheel_hack();

        window
    }

    /// Returns the size of the window.
    fn size(&self) -> Size2D<f32> {
        Size2D(glut::get(WindowWidth) as f32, glut::get(WindowHeight) as f32)
    }

    /// Presents the window to the screen (perhaps by page flipping).
    fn present(&mut self) {
        glut::swap_buffers();
    }

    /// Spins the event loop and returns the next event.
    fn recv(@mut self) -> WindowEvent {
        if self.event_queue.len() > 0 {
            return self.event_queue.shift()
        }

        glut::check_loop();

        if self.event_queue.len() > 0 {
            self.event_queue.shift()
        } else {
            IdleWindowEvent
        }
    }

    /// Schedules a redisplay.
    fn set_needs_display(@mut self) {
        glut::post_redisplay()
    }

    /// Sets the ready state.
    fn set_ready_state(@mut self, ready_state: ReadyState) {
        self.ready_state = ready_state;
        self.update_window_title()
    }

    /// Sets the render state.
    fn set_render_state(@mut self, render_state: RenderState) {
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
                self.event_queue.push(ZoomWindowEvent(0.1))
            }
            k if k == 31 && (glut::get_modifiers() & ACTIVE_CTRL) != 0 => {             // Ctrl+-
                self.event_queue.push(ZoomWindowEvent(-0.1))
            }
            _ => {}
        }
    }

    /// Helper function to handle a click
    fn handle_mouse(&self, button: c_int, state: c_int, x: c_int, y: c_int) {
        // FIXME(tkuehn): max pixel dist should be based on pixel density
        let max_pixel_dist = 10f;
        match state {
            glut::MOUSE_DOWN => {
                let mouse_event = MouseWindowMouseDownEvent(button as uint,
                                                            Point2D(x as f32, y as f32));
                self.event_queue.push(MouseWindowEventClass(mouse_event));

                *self.mouse_down_point = Point2D(x, y);
                *self.mouse_down_button = button;
            }
            glut::MOUSE_UP => {
                let mouse_event = MouseWindowMouseUpEvent(button as uint,
                                                          Point2D(x as f32, y as f32));
                self.event_queue.push(MouseWindowEventClass(mouse_event));

                if *self.mouse_down_button == button {
                    let pixel_dist = *self.mouse_down_point - Point2D(x, y);
                    let pixel_dist = ((pixel_dist.x * pixel_dist.x +
                                      pixel_dist.y * pixel_dist.y) as float).sqrt();
                    if pixel_dist < max_pixel_dist {
                        let mouse_event = MouseWindowClickEvent(button as uint,
                                                                Point2D(x as f32, y as f32));
                        self.event_queue.push(MouseWindowEventClass(mouse_event));
                    }
                }
            }
            _ => fail!("I cannot recognize the type of mouse action that occured. :-(")
        }
    }

    /// Helper function to pop up an alert box prompting the user to load a URL.
    fn load_url(&self) {
        let mut alert: Alert = AlertMethods::new("Navigate to:");
        alert.add_prompt();
        alert.run();

        let value = alert.prompt_value();
        if "" == value {    // To avoid crashing on Linux.
            self.event_queue.push(NavigateWindowEvent(~"http://purple.com/"))
        } else {
            // FIXME(pcwalton): Do we need this `.to_str()` copy?
            self.event_queue.push(NavigateWindowEvent(value.to_str()))
        }
    }
}

