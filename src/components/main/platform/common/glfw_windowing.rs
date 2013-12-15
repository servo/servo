/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A windowing implementation using GLFW.

use windowing::{ApplicationMethods, WindowEvent, WindowMethods};
use windowing::{IdleWindowEvent, ResizeWindowEvent, LoadUrlWindowEvent, MouseWindowEventClass};
use windowing::{ScrollWindowEvent, ZoomWindowEvent, NavigationWindowEvent, FinishedWindowEvent};
use windowing::{QuitWindowEvent, MouseWindowClickEvent, MouseWindowMouseDownEvent, MouseWindowMouseUpEvent};
use windowing::RefreshWindowEvent;
use windowing::{Forward, Back};

use alert::{Alert, AlertMethods};
use extra::time::Timespec;
use extra::time;
use std::libc::c_int;
use std::local_data;
use geom::point::Point2D;
use geom::size::Size2D;
use servo_msg::compositor_msg::{IdleRenderState, RenderState, RenderingRenderState};
use servo_msg::compositor_msg::{FinishedLoading, Blank, Loading, PerformingLayout, ReadyState};

use glfw;

/// A structure responsible for setting up and tearing down the entire windowing system.
pub struct Application;

impl ApplicationMethods for Application {
    fn new() -> Application {
        // Per GLFW docs it's safe to set the error callback before calling
        // glfwInit(), and this way we notice errors from init too.
        do glfw::set_error_callback |_error_code, description| {
            error!("GLFW error: {:s}", description);
        };
        glfw::init();
        Application
    }
}

impl Drop for Application {
    fn drop(&mut self) {
        drop_local_window();
        glfw::terminate();
    }
}

/// The type of a window.
pub struct Window {
    glfw_window: glfw::Window,

    event_queue: @mut ~[WindowEvent],

    drag_origin: Point2D<c_int>,

    mouse_down_button: @mut Option<glfw::MouseButton>,
    mouse_down_point: @mut Point2D<c_int>,

    ready_state: ReadyState,
    render_state: RenderState,

    last_title_set_time: Timespec,
}

impl WindowMethods<Application> for Window {
    /// Creates a new window.
    fn new(_: &Application) -> @mut Window {
        // Create the GLFW window.
        let glfw_window = glfw::Window::create(800, 600, "Servo", glfw::Windowed)
            .expect("Failed to create GLFW window");
        glfw_window.make_context_current();

        // Create our window object.
        let window = @mut Window {
            glfw_window: glfw_window,

            event_queue: @mut ~[],

            drag_origin: Point2D(0 as c_int, 0),

            mouse_down_button: @mut None,
            mouse_down_point: @mut Point2D(0 as c_int, 0),

            ready_state: Blank,
            render_state: IdleRenderState,

            last_title_set_time: Timespec::new(0, 0),
        };

        install_local_window(window);

        // Register event handlers.
        do window.glfw_window.set_framebuffer_size_callback |_win, width, height| {
            local_window().event_queue.push(ResizeWindowEvent(width as uint, height as uint))
        }
        do window.glfw_window.set_refresh_callback |_win| {
            local_window().event_queue.push(RefreshWindowEvent)
        }
        do window.glfw_window.set_key_callback |_win, key, _scancode, action, mods| {
            if action == glfw::Press {
                local_window().handle_key(key, mods)
            }
        }
        do window.glfw_window.set_mouse_button_callback |win, button, action, _mods| {
            let (x, y) = win.get_cursor_pos();
            //handle hidpi displays, since GLFW returns non-hi-def coordinates.
            let (backing_size, _) = win.get_framebuffer_size();
            let (window_size, _) = win.get_size();
            let hidpi = (backing_size as f32) / (window_size as f32);
            let x = x as f32 * hidpi;
            let y = y as f32 * hidpi;
            if button == glfw::MouseButtonLeft || button == glfw::MouseButtonRight {
                local_window().handle_mouse(button, action, x as i32, y as i32);
            }
        }
        do window.glfw_window.set_scroll_callback |win, x_offset, y_offset| {
            let dx = (x_offset as f32) * 30.0;
            let dy = (y_offset as f32) * 30.0;
            
            let (x, y) = win.get_cursor_pos();
            //handle hidpi displays, since GLFW returns non-hi-def coordinates.
            let (backing_size, _) = win.get_framebuffer_size();
            let (window_size, _) = win.get_size();
            let hidpi = (backing_size as f32) / (window_size as f32);
            let x = x as f32 * hidpi;
            let y = y as f32 * hidpi;

            local_window().event_queue.push(ScrollWindowEvent(Point2D(dx, dy), Point2D(x as i32, y as i32)));
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

    fn hidpi_factor(@mut self) -> f32 {
        let (backing_size, _) = self.glfw_window.get_framebuffer_size();
        let (window_size, _) = self.glfw_window.get_size();
        (backing_size as f32) / (window_size as f32)
    }
}

impl Window {
    /// Helper function to set the window title in accordance with the ready state.
    fn update_window_title(&mut self) {
        let now = time::get_time();
        if now.sec == self.last_title_set_time.sec {
            return
        }
        self.last_title_set_time = now;

        match self.ready_state {
            Blank => {
                self.glfw_window.set_title("blank — Servo")
            }
            Loading => {
                self.glfw_window.set_title("Loading — Servo")
            }
            PerformingLayout => {
                self.glfw_window.set_title("Performing Layout — Servo")
            }
            FinishedLoading => {
                match self.render_state {
                    RenderingRenderState => {
                        self.glfw_window.set_title("Rendering — Servo")
                    }
                    IdleRenderState => {
                        self.glfw_window.set_title("Servo")
                    }
                }
            }
        }
    }

    /// Helper function to handle keyboard events.
    fn handle_key(&self, key: glfw::Key, mods: glfw::Modifiers) {
        match key {
            glfw::KeyEscape => self.glfw_window.set_should_close(true),
            glfw::KeyL if mods.contains(glfw::Control) => self.load_url(), // Ctrl+L
            glfw::KeyEqual if mods.contains(glfw::Control) => { // Ctrl-+
                self.event_queue.push(ZoomWindowEvent(1.1));
            }
            glfw::KeyMinus if mods.contains(glfw::Control) => { // Ctrl--
                self.event_queue.push(ZoomWindowEvent(0.90909090909));
            }
            glfw::KeyBackspace if mods.contains(glfw::Shift) => { // Shift-Backspace
                self.event_queue.push(NavigationWindowEvent(Forward));
            }
            glfw::KeyBackspace => { // Backspace
                self.event_queue.push(NavigationWindowEvent(Back));
            }
            _ => {}
        }
    }

    /// Helper function to handle a click
    fn handle_mouse(&self, button: glfw::MouseButton, action: glfw::Action, x: c_int, y: c_int) {
        // FIXME(tkuehn): max pixel dist should be based on pixel density
        let max_pixel_dist = 10f64;
        let event = match action {
            glfw::Press => {
                *self.mouse_down_point = Point2D(x, y);
                *self.mouse_down_button = Some(button);
                MouseWindowMouseDownEvent(button as uint, Point2D(x as f32, y as f32))
            }
            glfw::Release => {
                match *self.mouse_down_button {
                    None => (),
                    Some(but) if button == but => {
                        let pixel_dist = *self.mouse_down_point - Point2D(x, y);
                        let pixel_dist = ((pixel_dist.x * pixel_dist.x +
                                           pixel_dist.y * pixel_dist.y) as f64).sqrt();
                        if pixel_dist < max_pixel_dist {
                            let click_event = MouseWindowClickEvent(button as uint,
                                                                    Point2D(x as f32, y as f32));
                            self.event_queue.push(MouseWindowEventClass(click_event));
                        }
                    }
                    Some(_) => (),
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

static TLS_KEY: local_data::Key<@mut Window> = &local_data::Key;

fn install_local_window(window: @mut Window) {
    local_data::set(TLS_KEY, window);
}

fn drop_local_window() {
    local_data::pop(TLS_KEY);
}

fn local_window() -> @mut Window {
    local_data::get(TLS_KEY, |v| *v.unwrap())
}
