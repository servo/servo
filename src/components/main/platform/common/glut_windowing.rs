/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A windowing implementation using GLUT.

use windowing::{ApplicationMethods, WindowEvent, WindowMethods};
use windowing::{IdleWindowEvent, ResizeWindowEvent, LoadUrlWindowEvent, MouseWindowEventClass};
use windowing::{ScrollWindowEvent, ZoomWindowEvent, NavigationWindowEvent, FinishedWindowEvent};
use windowing::{QuitWindowEvent, MouseWindowClickEvent, MouseWindowMouseDownEvent, MouseWindowMouseUpEvent};
use windowing::{Forward, Back};

use alert::{Alert, AlertMethods};
use std::libc::c_int;
use geom::point::Point2D;
use geom::size::Size2D;
use servo_msg::compositor_msg::{IdleRenderState, RenderState, RenderingRenderState};
use servo_msg::compositor_msg::{FinishedLoading, Blank, Loading, PerformingLayout, ReadyState};

use glut::glut::{ACTIVE_CTRL, ACTIVE_SHIFT, DOUBLE, HAVE_PRECISE_MOUSE_WHEEL, WindowHeight};
use glut::glut::WindowWidth;
use glut::glut;

static THROBBER: [char, ..8] = [ '⣾', '⣽', '⣻', '⢿', '⡿', '⣟', '⣯', '⣷' ];

/// A structure responsible for setting up and tearing down the entire windowing system.
pub struct Application;

impl ApplicationMethods for Application {
    fn new() -> Application {
        glut::init();
        glut::init_display_mode(DOUBLE);
        Application
    }
}

impl Drop for Application {
    fn drop(&self) {
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
        glut::init_window_size(800, 600);
        let glut_window = glut::create_window(~"Servo");

        // Create our window object.
        let window = @mut Window {
            glut_window: glut_window,

            event_queue: @mut ~[],

            drag_origin: Point2D(0 as c_int, 0),

            mouse_down_button: @mut 0,
            mouse_down_point: @mut Point2D(0 as c_int, 0),

            ready_state: Blank,
            render_state: IdleRenderState,
            throbber_frame: 0,
        };

        let event_queue = window.event_queue;

        // Register event handlers.
        do glut::reshape_func(window.glut_window) |width, height| {
            event_queue.push(ResizeWindowEvent(width as uint, height as uint))
        }
        do glut::keyboard_func |key, _, _| {
            window.handle_key(key)
        }
        do glut::mouse_func |button, state, x, y| {
            if button < 3 {
                window.handle_mouse(button, state, x, y);
            }
            else {
                match button {
                    3 => {
                        event_queue.push(ScrollWindowEvent(Point2D(0.0, 5.0 as f32), Point2D(0.0 as i32, 5.0 as i32)));
                    },
                    4 => {
                        event_queue.push(ScrollWindowEvent(Point2D(0.0, -5.0 as f32), Point2D(0.0 as i32, -5.0 as i32)));
                    },
                    _ => {}
                }
            }
        }

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
    
    fn recv(@mut self) -> WindowEvent {
        if !self.event_queue.is_empty() {
            return self.event_queue.shift()
        }
        glut::check_loop();
        if !self.event_queue.is_empty() {
            self.event_queue.shift()
        } else {
            IdleWindowEvent
        }
    }

    /// Sets the ready state.
    fn set_ready_state(@mut self, ready_state: ReadyState) {
        self.ready_state = ready_state;
        //FIXME: set_window_title causes crash with Android version of freeGLUT. Temporarily blocked.
        //self.update_window_title()
    }

    /// Sets the render state.
    fn set_render_state(@mut self, render_state: RenderState) {
        if self.ready_state == FinishedLoading &&
            self.render_state == RenderingRenderState &&
            render_state == IdleRenderState {
            // page loaded
            self.event_queue.push(FinishedWindowEvent);
        }

        self.render_state = render_state;
        //FIXME: set_window_title causes crash with Android version of freeGLUT. Temporarily blocked.
        //self.update_window_title()
    }

    fn hidpi_factor(@mut self) -> f32 {
        //FIXME: Do nothing in GLUT now.
    0f32
    }
}

impl Window {
    /// Helper function to set the window title in accordance with the ready state.
    fn update_window_title(&self) {
        let throbber = THROBBER[self.throbber_frame];
        match self.ready_state {
            Blank => {
                glut::set_window_title(self.glut_window, fmt!("Blank"))
            }
            Loading => {
                glut::set_window_title(self.glut_window, fmt!("%c Loading . Servo", throbber))
            }
            PerformingLayout => {
                glut::set_window_title(self.glut_window, fmt!("%c Performing Layout . Servo", throbber))
            }
            FinishedLoading => {
                match self.render_state {
                    RenderingRenderState => {
                        glut::set_window_title(self.glut_window, fmt!("%c Rendering . Servo", throbber))
                    }
                    IdleRenderState => glut::set_window_title(self.glut_window, "Servo"),
                }
            }
        }
    }

    /// Helper function to handle keyboard events.
    fn handle_key(&self, key: u8) {
        debug!("got key: %?", key);
        let modifiers = glut::get_modifiers();
        match key {
            42 => self.load_url(),
            43 => self.event_queue.push(ZoomWindowEvent(1.1)),
            45 => self.event_queue.push(ZoomWindowEvent(0.909090909)),
            56 => self.event_queue.push(ScrollWindowEvent(Point2D(0.0, 5.0 as f32), Point2D(0.0 as i32, 5.0 as i32))),
            50 => self.event_queue.push(ScrollWindowEvent(Point2D(0.0, -5.0 as f32), Point2D(0.0 as i32, -5.0 as i32))),
            127 => {
                if (modifiers & ACTIVE_SHIFT) != 0 {
                    self.event_queue.push(NavigationWindowEvent(Forward));
                }
                else {
                    self.event_queue.push(NavigationWindowEvent(Back));
                }
            }
            _ => {}
        }
    }

    /// Helper function to handle a click
    fn handle_mouse(&self, button: c_int, state: c_int, x: c_int, y: c_int) {
        // FIXME(tkuehn): max pixel dist should be based on pixel density
        let max_pixel_dist = 10f;
        let event = match state {
            glut::MOUSE_DOWN => {
                *self.mouse_down_point = Point2D(x, y);
                *self.mouse_down_button = button;
                MouseWindowMouseDownEvent(button as uint, Point2D(x as f32, y as f32))
            }
            glut::MOUSE_UP => {
                if *self.mouse_down_button == button {
                    let pixel_dist = *self.mouse_down_point - Point2D(x, y);
                    let pixel_dist = ((pixel_dist.x * pixel_dist.x +
                                       pixel_dist.y * pixel_dist.y) as float).sqrt();
                    if pixel_dist < max_pixel_dist {
                        let click_event = MouseWindowClickEvent(button as uint,
                                                           Point2D(x as f32, y as f32));
                        self.event_queue.push(MouseWindowEventClass(click_event));
                    }
                }
                MouseWindowMouseUpEvent(button as uint, Point2D(x as f32, y as f32))
            }
            _ => fail!("I cannot recognize the type of mouse action that occured. :-(")
        };
        self.event_queue.push(MouseWindowEventClass(event));
    }

    /// Helper function to pop up an alert box prompting the user to load a URL.
    fn load_url(&self) {
        let mut alert: Alert = AlertMethods::new("Navigate to:");
        alert.add_prompt();
        alert.run();
        let value = alert.prompt_value();
        if "" == value {    // To avoid crashing on Linux.
            self.event_queue.push(LoadUrlWindowEvent(~"http://purple.com/"))
        } else {
            self.event_queue.push(LoadUrlWindowEvent(value))
        }
    }
}

