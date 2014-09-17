/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A windowing implementation using GLFW.

use windowing::{ApplicationMethods, WindowEvent, WindowMethods};
use windowing::{IdleWindowEvent, ResizeWindowEvent, LoadUrlWindowEvent, MouseWindowEventClass,  MouseWindowMoveEventClass};
use windowing::{ScrollWindowEvent, ZoomWindowEvent, PinchZoomWindowEvent, NavigationWindowEvent, FinishedWindowEvent};
use windowing::{QuitWindowEvent, MouseWindowClickEvent, MouseWindowMouseDownEvent, MouseWindowMouseUpEvent};
use windowing::RefreshWindowEvent;
use windowing::{Forward, Back};

use alert::{Alert, AlertMethods};
use libc::{exit, c_int};
use time;
use time::Timespec;
use std::cell::{Cell, RefCell};
use std::comm::Receiver;
use std::rc::Rc;

use geom::point::{Point2D, TypedPoint2D};
use geom::scale_factor::ScaleFactor;
use geom::size::TypedSize2D;
use layers::geometry::DevicePixel;
use servo_msg::compositor_msg::{IdleRenderState, RenderState, RenderingRenderState};
use servo_msg::compositor_msg::{FinishedLoading, Blank, Loading, PerformingLayout, ReadyState};
use servo_util::geometry::ScreenPx;

use glfw;
use glfw::Context;

/// A structure responsible for setting up and tearing down the entire windowing system.
pub struct Application {
    pub glfw: glfw::Glfw,
}

impl ApplicationMethods for Application {
    fn new() -> Application {
        let app = glfw::init(glfw::LOG_ERRORS);
        match app {
            Err(_) => {
                // handles things like inability to connect to X
                // cannot simply fail, since the runtime isn't up yet (causes a nasty abort)
                println!("GLFW initialization failed");
                unsafe { exit(1); }
            }
            Ok(app) => {
                Application { glfw: app }
            }
        }
    }
}

macro_rules! glfw_callback(
    (
        $callback:path ($($arg:ident: $arg_ty:ty),*) $block:expr
    ) => ({
        struct GlfwCallback;
        impl $callback for GlfwCallback {
            fn call(&self $(, $arg: $arg_ty)*) {
                $block
            }
        }
        ~GlfwCallback
    });

    (
        [$($state:ident: $state_ty:ty),*],
        $callback:path ($($arg:ident: $arg_ty:ty),*) $block:expr
    ) => ({
        struct GlfwCallback {
            $($state: $state_ty,)*
        }
        impl $callback for GlfwCallback {
            fn call(&self $(, $arg: $arg_ty)*) {
                $block
            }
        }
        ~GlfwCallback {
            $($state: $state,)*
        }
    });
)


/// The type of a window.
pub struct Window {
    glfw: glfw::Glfw,

    glfw_window: glfw::Window,
    events: Receiver<(f64, glfw::WindowEvent)>,

    event_queue: RefCell<Vec<WindowEvent>>,

    mouse_down_button: Cell<Option<glfw::MouseButton>>,
    mouse_down_point: Cell<Point2D<c_int>>,

    ready_state: Cell<ReadyState>,
    render_state: Cell<RenderState>,

    last_title_set_time: Cell<Timespec>,
}

impl WindowMethods<Application> for Window {
    /// Creates a new window.
    fn new(app: &Application, is_foreground: bool, size: TypedSize2D<DevicePixel, uint>) -> Rc<Window> {
        // Create the GLFW window.
        let window_size = size.to_untyped();
        app.glfw.window_hint(glfw::Visible(is_foreground));
        let (glfw_window, events) = app.glfw.create_window(window_size.width as u32,
                                                            window_size.height as u32,
                                                            "Servo", glfw::Windowed)
            .expect("Failed to create GLFW window");
        glfw_window.make_current();

        // Create our window object.
        let window = Window {
            glfw: app.glfw,

            glfw_window: glfw_window,
            events: events,

            event_queue: RefCell::new(vec!()),

            mouse_down_button: Cell::new(None),
            mouse_down_point: Cell::new(Point2D(0 as c_int, 0)),

            ready_state: Cell::new(Blank),
            render_state: Cell::new(IdleRenderState),

            last_title_set_time: Cell::new(Timespec::new(0, 0)),
        };

        // Register event handlers.
        window.glfw_window.set_framebuffer_size_polling(true);
        window.glfw_window.set_refresh_polling(true);
        window.glfw_window.set_key_polling(true);
        window.glfw_window.set_mouse_button_polling(true);
        window.glfw_window.set_cursor_pos_polling(true);
        window.glfw_window.set_scroll_polling(true);

        let wrapped_window = Rc::new(window);

        wrapped_window
    }

    /// Returns the size of the window in hardware pixels.
    fn framebuffer_size(&self) -> TypedSize2D<DevicePixel, uint> {
        let (width, height) = self.glfw_window.get_framebuffer_size();
        TypedSize2D(width as uint, height as uint)
    }

    /// Returns the size of the window in density-independent "px" units.
    fn size(&self) -> TypedSize2D<ScreenPx, f32> {
        let (width, height) = self.glfw_window.get_size();
        TypedSize2D(width as f32, height as f32)
    }

    /// Presents the window to the screen (perhaps by page flipping).
    fn present(&self) {
        self.glfw_window.swap_buffers();
    }

    fn recv(&self) -> WindowEvent {
        {
            let mut event_queue = self.event_queue.borrow_mut();
            if !event_queue.is_empty() {
                return event_queue.remove(0).unwrap();
            }
        }

        self.glfw.poll_events();
        for (_, event) in glfw::flush_messages(&self.events) {
            self.handle_window_event(&self.glfw_window, event);
        }

        if self.glfw_window.should_close() {
            QuitWindowEvent
        } else {
            self.event_queue.borrow_mut().remove(0).unwrap_or(IdleWindowEvent)
        }
    }

    /// Sets the ready state.
    fn set_ready_state(&self, ready_state: ReadyState) {
        self.ready_state.set(ready_state);
        self.update_window_title()
    }

    /// Sets the render state.
    fn set_render_state(&self, render_state: RenderState) {
        if self.ready_state.get() == FinishedLoading &&
            self.render_state.get() == RenderingRenderState &&
            render_state == IdleRenderState {
            // page loaded
            self.event_queue.borrow_mut().push(FinishedWindowEvent);
        }

        self.render_state.set(render_state);
        self.update_window_title()
    }

    fn hidpi_factor(&self) -> ScaleFactor<ScreenPx, DevicePixel, f32> {
        let backing_size = self.framebuffer_size().width.get();
        let window_size = self.size().width.get();
        ScaleFactor((backing_size as f32) / window_size)
    }
}

impl Window {
    fn handle_window_event(&self, window: &glfw::Window, event: glfw::WindowEvent) {
        match event {
            glfw::KeyEvent(key, _, action, mods) => {
                if action == glfw::Press {
                    self.handle_key(key, mods)
                }
            },
            glfw::FramebufferSizeEvent(width, height) => {
                self.event_queue.borrow_mut().push(
                    ResizeWindowEvent(TypedSize2D(width as uint, height as uint)));
            },
            glfw::RefreshEvent => {
                self.event_queue.borrow_mut().push(RefreshWindowEvent);
            },
            glfw::MouseButtonEvent(button, action, _mods) => {
                let (x, y) = window.get_cursor_pos();
                //handle hidpi displays, since GLFW returns non-hi-def coordinates.
                let (backing_size, _) = window.get_framebuffer_size();
                let (window_size, _) = window.get_size();
                let hidpi = (backing_size as f32) / (window_size as f32);
                let x = x as f32 * hidpi;
                let y = y as f32 * hidpi;
                if button == glfw::MouseButtonLeft || button == glfw::MouseButtonRight {
                    self.handle_mouse(button, action, x as i32, y as i32);
                }
            },
            glfw::CursorPosEvent(xpos, ypos) => {
                self.event_queue.borrow_mut().push(
                    MouseWindowMoveEventClass(TypedPoint2D(xpos as f32, ypos as f32)));
            },
            glfw::ScrollEvent(xpos, ypos) => {
                match (window.get_key(glfw::KeyLeftControl),
                       window.get_key(glfw::KeyRightControl)) {
                    (glfw::Press, _) | (_, glfw::Press) => {
                        // Ctrl-Scrollwheel simulates a "pinch zoom" gesture.
                        if ypos < 0.0 {
                            self.event_queue.borrow_mut().push(PinchZoomWindowEvent(1.0/1.1));
                        } else if ypos > 0.0 {
                            self.event_queue.borrow_mut().push(PinchZoomWindowEvent(1.1));
                        }
                    },
                    _ => {
                        let dx = (xpos as f32) * 30.0;
                        let dy = (ypos as f32) * 30.0;
                        self.scroll_window(dx, dy);
                    }
                }

            },
            _ => {}
        }
    }

    /// Helper function to send a scroll event.
    fn scroll_window(&self, dx: f32, dy: f32) {
        let (x, y) = self.glfw_window.get_cursor_pos();
        //handle hidpi displays, since GLFW returns non-hi-def coordinates.
        let (backing_size, _) = self.glfw_window.get_framebuffer_size();
        let (window_size, _) = self.glfw_window.get_size();
        let hidpi = (backing_size as f32) / (window_size as f32);
        let x = x as f32 * hidpi;
        let y = y as f32 * hidpi;

        self.event_queue.borrow_mut().push(ScrollWindowEvent(TypedPoint2D(dx, dy),
        TypedPoint2D(x as i32, y as i32)));
    }

    /// Helper function to set the window title in accordance with the ready state.
    fn update_window_title(&self) {
        let now = time::get_time();
        if now.sec == self.last_title_set_time.get().sec {
            return
        }
        self.last_title_set_time.set(now);

        match self.ready_state.get() {
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
                match self.render_state.get() {
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
                self.event_queue.borrow_mut().push(ZoomWindowEvent(1.1));
            }
            glfw::KeyMinus if mods.contains(glfw::Control) => { // Ctrl--
                self.event_queue.borrow_mut().push(ZoomWindowEvent(1.0/1.1));
            }
            glfw::KeyBackspace if mods.contains(glfw::Shift) => { // Shift-Backspace
                self.event_queue.borrow_mut().push(NavigationWindowEvent(Forward));
            }
            glfw::KeyBackspace => { // Backspace
                self.event_queue.borrow_mut().push(NavigationWindowEvent(Back));
            }
            glfw::KeyPageDown => {
                let (_, height) = self.glfw_window.get_size();
                self.scroll_window(0.0, -height as f32);
            }
            glfw::KeyPageUp => {
                let (_, height) = self.glfw_window.get_size();
                self.scroll_window(0.0, height as f32);
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
                self.mouse_down_point.set(Point2D(x, y));
                self.mouse_down_button.set(Some(button));
                MouseWindowMouseDownEvent(button as uint, TypedPoint2D(x as f32, y as f32))
            }
            glfw::Release => {
                match self.mouse_down_button.get() {
                    None => (),
                    Some(but) if button == but => {
                        let pixel_dist = self.mouse_down_point.get() - Point2D(x, y);
                        let pixel_dist = ((pixel_dist.x * pixel_dist.x +
                                           pixel_dist.y * pixel_dist.y) as f64).sqrt();
                        if pixel_dist < max_pixel_dist {
                            let click_event = MouseWindowClickEvent(button as uint,
                                                                    TypedPoint2D(x as f32, y as f32));
                            self.event_queue.borrow_mut().push(MouseWindowEventClass(click_event));
                        }
                    }
                    Some(_) => (),
                }
                MouseWindowMouseUpEvent(button as uint, TypedPoint2D(x as f32, y as f32))
            }
            _ => fail!("I cannot recognize the type of mouse action that occured. :-(")
        };
        self.event_queue.borrow_mut().push(MouseWindowEventClass(event));
    }

    /// Helper function to pop up an alert box prompting the user to load a URL.
    fn load_url(&self) {
        let mut alert: Alert = AlertMethods::new("Navigate to:");
        alert.add_prompt();
        alert.run();
        let value = alert.prompt_value();
        if "" == value.as_slice() {    // To avoid crashing on Linux.
            self.event_queue.borrow_mut().push(LoadUrlWindowEvent("http://purple.com/".to_string()))
        } else {
            self.event_queue.borrow_mut().push(LoadUrlWindowEvent(value.clone()))
        }
    }
}
