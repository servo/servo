/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A windowing implementation using GLUT.

use windowing::{ApplicationMethods, WindowEvent, WindowMethods};
use windowing::{IdleWindowEvent, ResizeWindowEvent, LoadUrlWindowEvent, MouseWindowEventClass};
use windowing::{ScrollWindowEvent, ZoomWindowEvent, NavigationWindowEvent, FinishedWindowEvent};
use windowing::{MouseWindowClickEvent, MouseWindowMouseDownEvent, MouseWindowMouseUpEvent};
use windowing::{Forward, Back};

use alert::{Alert, AlertMethods};
use libc::{c_int, c_uchar};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use geom::point::{Point2D, TypedPoint2D};
use geom::scale_factor::ScaleFactor;
use geom::size::TypedSize2D;
use layers::geometry::DevicePixel;
use servo_msg::compositor_msg::{IdleRenderState, RenderState, RenderingRenderState};
use servo_msg::compositor_msg::{FinishedLoading, Blank, ReadyState};
use servo_util::geometry::ScreenPx;

use glut::glut::{ACTIVE_SHIFT, DOUBLE, WindowHeight};
use glut::glut::WindowWidth;
use glut::glut;

// static THROBBER: [char, ..8] = [ '⣾', '⣽', '⣻', '⢿', '⡿', '⣟', '⣯', '⣷' ];

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
    fn drop(&mut self) {
        drop_local_window();
    }
}

/// The type of a window.
pub struct Window {
    pub glut_window: glut::Window,

    pub event_queue: RefCell<Vec<WindowEvent>>,

    pub drag_origin: Point2D<c_int>,

    pub mouse_down_button: Cell<c_int>,
    pub mouse_down_point: Cell<Point2D<c_int>>,

    pub ready_state: Cell<ReadyState>,
    pub render_state: Cell<RenderState>,
    pub throbber_frame: Cell<u8>,
}

impl WindowMethods<Application> for Window {
    /// Creates a new window.
    fn new(_: &Application, _: bool, size: TypedSize2D<DevicePixel, uint>) -> Rc<Window> {
        // Create the GLUT window.
        let window_size = size.to_untyped();
        glut::init_window_size(window_size.width, window_size.height);
        let glut_window = glut::create_window("Servo".to_string());

        // Create our window object.
        let window = Window {
            glut_window: glut_window,

            event_queue: RefCell::new(vec!()),

            drag_origin: Point2D(0 as c_int, 0),

            mouse_down_button: Cell::new(0),
            mouse_down_point: Cell::new(Point2D(0 as c_int, 0)),

            ready_state: Cell::new(Blank),
            render_state: Cell::new(IdleRenderState),
            throbber_frame: Cell::new(0),
        };

        // Register event handlers.

        //Added dummy display callback to freeglut. According to freeglut ref, we should register some kind of display callback after freeglut 3.0.

        struct DisplayCallbackState;
        impl glut::DisplayCallback for DisplayCallbackState {
            fn call(&self) {
                debug!("GLUT display func registgered");
            }
        }
        glut::display_func(box DisplayCallbackState);
        struct ReshapeCallbackState;
        impl glut::ReshapeCallback for ReshapeCallbackState {
            fn call(&self, width: c_int, height: c_int) {
                let tmp = local_window();
                tmp.event_queue.borrow_mut().push(ResizeWindowEvent(TypedSize2D(width as uint, height as uint)))
            }
        }
        glut::reshape_func(glut_window, box ReshapeCallbackState);
        struct KeyboardCallbackState;
        impl glut::KeyboardCallback for KeyboardCallbackState {
            fn call(&self, key: c_uchar, _x: c_int, _y: c_int) {
                let tmp = local_window();
                tmp.handle_key(key)
            }
        }
        glut::keyboard_func(box KeyboardCallbackState);
        struct MouseCallbackState;
        impl glut::MouseCallback for MouseCallbackState {
            fn call(&self, button: c_int, state: c_int, x: c_int, y: c_int) {
                if button < 3 {
                    let tmp = local_window();
                    tmp.handle_mouse(button, state, x, y);
                } else {
                    match button {
                        3 => {
                            let tmp = local_window();
                            tmp.event_queue.borrow_mut().push(ScrollWindowEvent(
                                    TypedPoint2D(0.0f32, 5.0f32),
                                    TypedPoint2D(0i32, 5i32)));
                        },
                        4 => {
                            let tmp = local_window();
                            tmp.event_queue.borrow_mut().push(ScrollWindowEvent(
                                    TypedPoint2D(0.0f32, -5.0f32),
                                    TypedPoint2D(0i32, -5i32)));
                        },
                        _ => {}
                    }
                }
            }
        }
        glut::mouse_func(box MouseCallbackState);

        let wrapped_window = Rc::new(window);

        install_local_window(wrapped_window.clone());

        wrapped_window
    }

    /// Returns the size of the window in hardware pixels.
    fn framebuffer_size(&self) -> TypedSize2D<DevicePixel, uint> {
        TypedSize2D(glut::get(WindowWidth) as uint, glut::get(WindowHeight) as uint)
    }

    /// Returns the size of the window in density-independent "px" units.
    fn size(&self) -> TypedSize2D<ScreenPx, f32> {
        self.framebuffer_size().as_f32() / self.hidpi_factor()
    }

    /// Presents the window to the screen (perhaps by page flipping).
    fn present(&self) {
        glut::swap_buffers();
    }

    fn recv(&self) -> WindowEvent {
        if !self.event_queue.borrow_mut().is_empty() {
            return self.event_queue.borrow_mut().remove(0).unwrap();
        }

        glut::check_loop();

        self.event_queue.borrow_mut().remove(0).unwrap_or(IdleWindowEvent)
    }

    /// Sets the ready state.
    fn set_ready_state(&self, ready_state: ReadyState) {
        self.ready_state.set(ready_state);
        //FIXME: set_window_title causes crash with Android version of freeGLUT. Temporarily blocked.
        //self.update_window_title()
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
        //FIXME: set_window_title causes crash with Android version of freeGLUT. Temporarily blocked.
        //self.update_window_title()
    }

    fn hidpi_factor(&self) -> ScaleFactor<ScreenPx, DevicePixel, f32> {
        //FIXME: Do nothing in GLUT now.
        ScaleFactor(1.0)
    }
}

impl Window {
    /// Helper function to set the window title in accordance with the ready state.
    // fn update_window_title(&self) {
    //     let throbber = THROBBER[self.throbber_frame];
    //     match self.ready_state {
    //         Blank => {
    //             glut::set_window_title(self.glut_window, "Blank")
    //         }
    //         Loading => {
    //             glut::set_window_title(self.glut_window, format!("{:c} Loading . Servo", throbber))
    //         }
    //         PerformingLayout => {
    //             glut::set_window_title(self.glut_window, format!("{:c} Performing Layout . Servo", throbber))
    //         }
    //         FinishedLoading => {
    //             match self.render_state {
    //                 RenderingRenderState => {
    //                     glut::set_window_title(self.glut_window, format!("{:c} Rendering . Servo", throbber))
    //                 }
    //                 IdleRenderState => glut::set_window_title(self.glut_window, "Servo"),
    //             }
    //         }
    //     }
    // }

    /// Helper function to handle keyboard events.
    fn handle_key(&self, key: u8) {
        debug!("got key: {}", key);
        let modifiers = glut::get_modifiers();
        match key {
            42 => self.load_url(),
            43 => self.event_queue.borrow_mut().push(ZoomWindowEvent(1.1)),
            45 => self.event_queue.borrow_mut().push(ZoomWindowEvent(0.909090909)),
            56 => self.event_queue.borrow_mut().push(ScrollWindowEvent(TypedPoint2D(0.0f32, 5.0f32),
                                                                       TypedPoint2D(0i32, 5i32))),
            50 => self.event_queue.borrow_mut().push(ScrollWindowEvent(TypedPoint2D(0.0f32, -5.0f32),
                                                                       TypedPoint2D(0i32, -5i32))),
            127 => {
                if (modifiers & ACTIVE_SHIFT) != 0 {
                    self.event_queue.borrow_mut().push(NavigationWindowEvent(Forward));
                }
                else {
                    self.event_queue.borrow_mut().push(NavigationWindowEvent(Back));
                }
            }
            _ => {}
        }
    }

    /// Helper function to handle a click
    fn handle_mouse(&self, button: c_int, state: c_int, x: c_int, y: c_int) {
        // FIXME(tkuehn): max pixel dist should be based on pixel density
        let max_pixel_dist = 10f32;
        let event = match state {
            glut::MOUSE_DOWN => {
                self.mouse_down_point.set(Point2D(x, y));
                self.mouse_down_button.set(button);
                MouseWindowMouseDownEvent(button as uint, TypedPoint2D(x as f32, y as f32))
            }
            glut::MOUSE_UP => {
                if self.mouse_down_button.get() == button {
                    let pixel_dist = self.mouse_down_point.get() - Point2D(x, y);
                    let pixel_dist = ((pixel_dist.x * pixel_dist.x +
                                       pixel_dist.y * pixel_dist.y) as f32).sqrt();
                    if pixel_dist < max_pixel_dist {
                        let click_event = MouseWindowClickEvent(button as uint,
                                                           TypedPoint2D(x as f32, y as f32));
                        self.event_queue.borrow_mut().push(MouseWindowEventClass(click_event));
                    }
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

local_data_key!(TLS_KEY: Rc<Window>)

fn install_local_window(window: Rc<Window>) {
    TLS_KEY.replace(Some(window));
}

fn drop_local_window() {
    TLS_KEY.replace(None);
}

fn local_window() -> Rc<Window> {
    TLS_KEY.get().unwrap().clone()
}
