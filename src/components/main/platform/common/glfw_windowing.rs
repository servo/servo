/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A windowing implementation using GLFW.

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
        // Create the GLFW window.
        let glfw_window = glfw::Window::create(800, 600, "Servo", glfw::Windowed).unwrap();
        glfw_window.make_context_current();

        // Create our window object.
        let window = @mut Window {
            glfw_window: glfw_window,

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
        do window.glfw_window.set_framebuffer_size_callback |_win, width, height| {
            event_queue.push(ResizeWindowEvent(width as uint, height as uint))
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
            
            event_queue.push(ScrollWindowEvent(Point2D(dx, dy)));
        }

        window
    }

    /// Returns the size of the window.
    fn size(&self) -> Size2D<f32> {
        let (width, height) = self.glfw_window.get_framebuffer_size();
        Size2D(width as f32, height as f32)
    }

    /// Presents the window to the screen (perhaps by page flipping).
    fn present(&mut self) {
        self.glfw_window.swap_buffers();
    }
    
    fn recv(@mut self) -> WindowEvent {
        if !self.event_queue.is_empty() {
            return self.event_queue.shift()
        }
        glfw::poll_events();
        self.throbber_frame = (self.throbber_frame + 1) % (THROBBER.len() as u8);
        self.update_window_title();
        if self.glfw_window.should_close() {
            QuitWindowEvent
        } else if !self.event_queue.is_empty() {
            self.event_queue.shift()
        } else {
            IdleWindowEvent
        }
    }

    /// Sets the ready state.
    fn set_ready_state(@mut self, ready_state: ReadyState) {
        self.ready_state = ready_state;
        self.update_window_title()
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
        self.update_window_title()
    }
}

impl Window {
    /// Helper function to set the window title in accordance with the ready state.
    fn update_window_title(&self) {
        let throbber = THROBBER[self.throbber_frame];
        match self.ready_state {
            Blank => {
                self.glfw_window.set_title(fmt!("blank — Servo"));
            }
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
            glfw::KEY_ESCAPE => self.glfw_window.set_should_close(true),
            glfw::KEY_L if mods & glfw::MOD_CONTROL != 0 => self.load_url(), // Ctrl+L
            glfw::KEY_EQUAL if mods & glfw::MOD_CONTROL != 0 => { // Ctrl-+
                self.event_queue.push(ZoomWindowEvent(1.1));
            }
            glfw::KEY_MINUS if mods & glfw::MOD_CONTROL != 0 => { // Ctrl--
                self.event_queue.push(ZoomWindowEvent(0.90909090909));
            }
            glfw::KEY_BACKSPACE if mods & glfw::MOD_SHIFT != 0 => { // Shift-Backspace
                self.event_queue.push(NavigationWindowEvent(Forward));
            }
            glfw::KEY_BACKSPACE => { // Backspace
                self.event_queue.push(NavigationWindowEvent(Back));
            }
            _ => {}
        }
    }

    /// Helper function to handle a click
    fn handle_mouse(&self, button: c_int, action: c_int, x: c_int, y: c_int) {
        // FIXME(tkuehn): max pixel dist should be based on pixel density
        let max_pixel_dist = 10f;
        let event = match action {
            glfw::PRESS => {
                *self.mouse_down_point = Point2D(x, y);
                *self.mouse_down_button = button;
                MouseWindowMouseDownEvent(button as uint, Point2D(x as f32, y as f32))
            }
            glfw::RELEASE => {
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

