/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A windowing implementation using glutin.

use compositing::compositor_task::{mod, CompositorProxy, CompositorReceiver};
use compositing::windowing::{WindowEvent, WindowMethods};
use compositing::windowing::{IdleWindowEvent, ResizeWindowEvent};
use compositing::windowing::{MouseWindowEventClass,  MouseWindowMoveEventClass, ScrollWindowEvent};
use compositing::windowing::{ZoomWindowEvent, PinchZoomWindowEvent, NavigationWindowEvent};
use compositing::windowing::{QuitWindowEvent, MouseWindowClickEvent};
use compositing::windowing::{MouseWindowMouseDownEvent, MouseWindowMouseUpEvent};
use compositing::windowing::{Forward, Back};
use geom::point::{Point2D, TypedPoint2D};
use geom::scale_factor::ScaleFactor;
use geom::size::TypedSize2D;
use gleam::gl;
use layers::geometry::DevicePixel;
use layers::platform::surface::NativeGraphicsMetadata;
use msg::compositor_msg::{IdleRenderState, RenderState, RenderingRenderState};
use msg::compositor_msg::{FinishedLoading, Blank, Loading, PerformingLayout, ReadyState};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use time::{mod, Timespec};
use util::geometry::ScreenPx;
use util::opts::{RenderApi, Mesa, OpenGL};
use glutin;
use NestedEventLoopListener;

#[cfg(target_os="linux")]
use std::ptr;

struct HeadlessContext {
    // Although currently unused, this context needs to be stored.
    // Otherwise, its drop() is called, deleting the mesa context
    // before it can be used.
    _context: glutin::HeadlessContext,
    size: TypedSize2D<DevicePixel, uint>,
}

enum WindowHandle {
    Windowed(glutin::Window),
    Headless(HeadlessContext),
}

bitflags!(
    #[deriving(Show)]
    flags KeyModifiers: u8 {
        const LEFT_CONTROL = 1,
        const RIGHT_CONTROL = 2,
        const LEFT_SHIFT = 4,
        const RIGHT_SHIFT = 8,
        const LEFT_ALT = 16,
        const RIGHT_ALT = 32,
    }
)

/// The type of a window.
pub struct Window {
    glutin: WindowHandle,

    mouse_down_button: Cell<Option<glutin::MouseButton>>,
    mouse_down_point: Cell<Point2D<int>>,
    event_queue: RefCell<Vec<WindowEvent>>,

    mouse_pos: Cell<Point2D<int>>,
    ready_state: Cell<ReadyState>,
    render_state: Cell<RenderState>,
    key_modifiers: Cell<KeyModifiers>,

    last_title_set_time: Cell<Timespec>,
}

impl Window {
    /// Creates a new window.
    pub fn new(is_foreground: bool, size: TypedSize2D<DevicePixel, uint>, render_api: RenderApi)
               -> Rc<Window> {

        // Create the glutin window.
        let window_size = size.to_untyped();

        let glutin = match render_api {
            OpenGL => {
                let glutin_window = glutin::WindowBuilder::new()
                                    .with_title("Servo [glutin]".to_string())
                                    .with_dimensions(window_size.width, window_size.height)
                                    .with_gl_version((3, 0))
                                    .with_visibility(is_foreground)
                                    .build()
                                    .unwrap();
                unsafe { glutin_window.make_current() };

                gl::load_with(|s| glutin_window.get_proc_address(s));

                Windowed(glutin_window)
            }
            Mesa => {
                let headless_builder = glutin::HeadlessRendererBuilder::new(window_size.width,
                                                                            window_size.height);
                let headless_context = headless_builder.build().unwrap();
                unsafe { headless_context.make_current() };

                gl::load_with(|s| headless_context.get_proc_address(s));

                Headless(HeadlessContext {
                    _context: headless_context,
                    size: size,
                })
            }
        };

        let window = Window {
            glutin: glutin,
            event_queue: RefCell::new(vec!()),
            mouse_down_button: Cell::new(None),
            mouse_down_point: Cell::new(Point2D(0, 0)),

            mouse_pos: Cell::new(Point2D(0, 0)),
            ready_state: Cell::new(Blank),
            render_state: Cell::new(IdleRenderState),
            key_modifiers: Cell::new(KeyModifiers::empty()),

            last_title_set_time: Cell::new(Timespec::new(0, 0)),
        };

        Rc::new(window)
    }
}

impl WindowMethods for Window {
    /// Returns the size of the window in hardware pixels.
    fn framebuffer_size(&self) -> TypedSize2D<DevicePixel, uint> {
        let (width, height) = match self.glutin {
            Windowed(ref window) => window.get_inner_size(),
            Headless(ref context) => Some((context.size.to_untyped().width,
                                           context.size.to_untyped().height)),
        }.unwrap();
        TypedSize2D(width as uint, height as uint)
    }

    /// Returns the size of the window in density-independent "px" units.
    fn size(&self) -> TypedSize2D<ScreenPx, f32> {
        // TODO: Handle hidpi
        let (width, height) = match self.glutin {
            Windowed(ref window) => window.get_inner_size(),
            Headless(ref context) => Some((context.size.to_untyped().width,
                                           context.size.to_untyped().height)),
        }.unwrap();
        TypedSize2D(width as f32, height as f32)
    }

    /// Presents the window to the screen (perhaps by page flipping).
    fn present(&self) {
        match self.glutin {
            Windowed(ref window) => window.swap_buffers(),
            Headless(_) => {},
        }
    }

    fn create_compositor_channel(_: &Option<Rc<Window>>)
                                 -> (Box<CompositorProxy+Send>, Box<CompositorReceiver>) {
        let (sender, receiver) = channel();
        (box GlutinCompositorProxy {
             sender: sender,
         } as Box<CompositorProxy+Send>,
         box receiver as Box<CompositorReceiver>)
    }

    /// Sets the ready state.
    fn set_ready_state(&self, ready_state: ReadyState) {
        self.ready_state.set(ready_state);
        self.update_window_title()
    }

    /// Sets the render state.
    fn set_render_state(&self, render_state: RenderState) {
        self.render_state.set(render_state);
        self.update_window_title()
    }

    fn hidpi_factor(&self) -> ScaleFactor<ScreenPx, DevicePixel, f32> {
        // TODO - handle hidpi
        ScaleFactor(1.0)
    }

    #[cfg(target_os="linux")]
    fn native_metadata(&self) -> NativeGraphicsMetadata {
        match self.glutin {
            Windowed(ref window) => {
                NativeGraphicsMetadata {
                    display: unsafe { window.platform_display() }
                }
            }
            Headless(_) => {
                NativeGraphicsMetadata {
                    display: ptr::null_mut()
                }
            },
        }
    }

    #[cfg(target_os="macos")]
    fn native_metadata(&self) -> NativeGraphicsMetadata {
        use cgl::{CGLGetCurrentContext, CGLGetPixelFormat};
        unsafe {
            NativeGraphicsMetadata {
                pixel_format: CGLGetPixelFormat(CGLGetCurrentContext()),
            }
        }
    }
}

impl Window {
    /// Helper function to set the window title in accordance with the ready state.
    fn update_window_title(&self) {
        match self.glutin {
            Windowed(ref window) => {
                let now = time::get_time();
                if now.sec == self.last_title_set_time.get().sec {
                    return
                }
                self.last_title_set_time.set(now);

                match self.ready_state.get() {
                    Blank => {
                        window.set_title("blank - Servo [glutin]")
                    }
                    Loading => {
                        window.set_title("Loading - Servo [glutin]")
                    }
                    PerformingLayout => {
                        window.set_title("Performing Layout - Servo [glutin]")
                    }
                    FinishedLoading => {
                        match self.render_state.get() {
                            RenderingRenderState => {
                                window.set_title("Rendering - Servo [glutin]")
                            }
                            IdleRenderState => {
                                window.set_title("Servo [glutin]")
                            }
                        }
                    }
                }
            }
            Headless(_) => {},
        }
    }
}

impl Window {
    fn handle_window_event(&self, event: glutin::Event) -> bool {
        match event {
            glutin::KeyboardInput(element_state, _scan_code, virtual_key_code) => {
                if virtual_key_code.is_some() {
                    let virtual_key_code = virtual_key_code.unwrap();

                    match (element_state, virtual_key_code) {
                        (_, glutin::LControl) => self.toggle_modifier(LEFT_CONTROL),
                        (_, glutin::RControl) => self.toggle_modifier(RIGHT_CONTROL),
                        (_, glutin::LShift) => self.toggle_modifier(LEFT_SHIFT),
                        (_, glutin::RShift) => self.toggle_modifier(RIGHT_SHIFT),
                        (_, glutin::LAlt) => self.toggle_modifier(LEFT_ALT),
                        (_, glutin::RAlt) => self.toggle_modifier(RIGHT_ALT),
                        (glutin::Pressed, key_code) => return self.handle_key(key_code),
                        (_, _) => {}
                    }
                }
            }
            glutin::Resized(width, height) => {
                self.event_queue.borrow_mut().push(ResizeWindowEvent(TypedSize2D(width, height)));
            }
            glutin::MouseInput(element_state, mouse_button) => {
                if mouse_button == glutin::LeftMouseButton ||
                                    mouse_button == glutin::RightMouseButton {
                        let mouse_pos = self.mouse_pos.get();
                        self.handle_mouse(mouse_button, element_state, mouse_pos.x, mouse_pos.y);
                   }
            }
            glutin::MouseMoved((x, y)) => {
                self.mouse_pos.set(Point2D(x, y));
                self.event_queue.borrow_mut().push(
                    MouseWindowMoveEventClass(TypedPoint2D(x as f32, y as f32)));
            }
            glutin::MouseWheel(delta) => {
                if self.ctrl_pressed() {
                    // Ctrl-Scrollwheel simulates a "pinch zoom" gesture.
                    if delta < 0 {
                        self.event_queue.borrow_mut().push(PinchZoomWindowEvent(1.0/1.1));
                    } else if delta > 0 {
                        self.event_queue.borrow_mut().push(PinchZoomWindowEvent(1.1));
                    }
                } else {
                    let dx = 0.0;
                    let dy = (delta as f32) * 30.0;
                    self.scroll_window(dx, dy);
                }
            },
            _ => {}
        }

        false
    }

    #[inline]
    fn ctrl_pressed(&self) -> bool {
        self.key_modifiers.get().intersects(LEFT_CONTROL | RIGHT_CONTROL)
    }

    #[inline]
    fn shift_pressed(&self) -> bool {
        self.key_modifiers.get().intersects(LEFT_SHIFT | RIGHT_SHIFT)
    }

    fn toggle_modifier(&self, modifier: KeyModifiers) {
        let mut modifiers = self.key_modifiers.get();
        modifiers.toggle(modifier);
        self.key_modifiers.set(modifiers);
    }

    /// Helper function to send a scroll event.
    fn scroll_window(&self, dx: f32, dy: f32) {
        let mouse_pos = self.mouse_pos.get();
        let event = ScrollWindowEvent(TypedPoint2D(dx as f32, dy as f32),
                                      TypedPoint2D(mouse_pos.x as i32, mouse_pos.y as i32));
        self.event_queue.borrow_mut().push(event);
    }

    /// Helper function to handle keyboard events.
    fn handle_key(&self, key: glutin::VirtualKeyCode) -> bool {
        match key {
            glutin::Escape => return true,
            glutin::Equals if self.ctrl_pressed() => { // Ctrl-+
                self.event_queue.borrow_mut().push(ZoomWindowEvent(1.1));
            }
            glutin::Minus if self.ctrl_pressed() => { // Ctrl--
                self.event_queue.borrow_mut().push(ZoomWindowEvent(1.0/1.1));
            }
            glutin::Back if self.shift_pressed() => { // Shift-Backspace
                self.event_queue.borrow_mut().push(NavigationWindowEvent(Forward));
            }
            glutin::Back => { // Backspace
                self.event_queue.borrow_mut().push(NavigationWindowEvent(Back));
            }
            glutin::PageDown => {
                self.scroll_window(0.0, -self.framebuffer_size().as_f32().to_untyped().height);
            }
            glutin::PageUp => {
                self.scroll_window(0.0, self.framebuffer_size().as_f32().to_untyped().height);
            }
            _ => {}
        }

        false
    }

    /// Helper function to handle a click
    fn handle_mouse(&self, button: glutin::MouseButton, action: glutin::ElementState, x: int, y: int) {
        // FIXME(tkuehn): max pixel dist should be based on pixel density
        let max_pixel_dist = 10f64;
        let event = match action {
            glutin::Pressed => {
                self.mouse_down_point.set(Point2D(x, y));
                self.mouse_down_button.set(Some(button));
                MouseWindowMouseDownEvent(0, TypedPoint2D(x as f32, y as f32))
            }
            glutin::Released => {
                match self.mouse_down_button.get() {
                    None => (),
                    Some(but) if button == but => {
                        let pixel_dist = self.mouse_down_point.get() - Point2D(x, y);
                        let pixel_dist = ((pixel_dist.x * pixel_dist.x +
                                           pixel_dist.y * pixel_dist.y) as f64).sqrt();
                        if pixel_dist < max_pixel_dist {
                            let click_event = MouseWindowClickEvent(0,
                                                                    TypedPoint2D(x as f32,
                                                                                 y as f32));
                            self.event_queue.borrow_mut().push(MouseWindowEventClass(click_event));
                        }
                    }
                    Some(_) => (),
                }
                MouseWindowMouseUpEvent(0, TypedPoint2D(x as f32, y as f32))
            }
        };
        self.event_queue.borrow_mut().push(MouseWindowEventClass(event));
    }

    pub unsafe fn set_nested_event_loop_listener(
            &self,
            _listener: *mut NestedEventLoopListener + 'static) {
        // TODO: Support this with glutin
        //self.glfw_window.set_refresh_polling(false);
        //glfw::ffi::glfwSetWindowRefreshCallback(self.glfw_window.ptr, Some(on_refresh));
        //glfw::ffi::glfwSetFramebufferSizeCallback(self.glfw_window.ptr, Some(on_framebuffer_size));
        //g_nested_event_loop_listener = Some(listener)
    }

    pub unsafe fn remove_nested_event_loop_listener(&self) {
        // TODO: Support this with glutin
        //glfw::ffi::glfwSetWindowRefreshCallback(self.glfw_window.ptr, None);
        //glfw::ffi::glfwSetFramebufferSizeCallback(self.glfw_window.ptr, None);
        //self.glfw_window.set_refresh_polling(true);
        //g_nested_event_loop_listener = None
    }

    pub fn wait_events(&self) -> WindowEvent {
        {
            let mut event_queue = self.event_queue.borrow_mut();
            if !event_queue.is_empty() {
                return event_queue.remove(0).unwrap();
            }
        }

        match self.glutin {
            Windowed(ref window) => {
                let mut close_event = false;
                for event in window.poll_events() {
                    close_event = self.handle_window_event(event);
                    if close_event {
                        break;
                    }
                }

                if close_event || window.is_closed() {
                    QuitWindowEvent
                } else {
                    self.event_queue.borrow_mut().remove(0).unwrap_or(IdleWindowEvent)
                }
            }
            Headless(_) => {
                self.event_queue.borrow_mut().remove(0).unwrap_or(IdleWindowEvent)
            }
        }
    }
}

struct GlutinCompositorProxy {
    sender: Sender<compositor_task::Msg>,
}

impl CompositorProxy for GlutinCompositorProxy {
    fn send(&mut self, msg: compositor_task::Msg) {
        // Send a message and kick the OS event loop awake.
        self.sender.send(msg);
        // TODO: Support this with glutin
        //glfw::Glfw::post_empty_event()
    }
    fn clone_compositor_proxy(&self) -> Box<CompositorProxy+Send> {
        box GlutinCompositorProxy {
            sender: self.sender.clone(),
        } as Box<CompositorProxy+Send>
    }
}
