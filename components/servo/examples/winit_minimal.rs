/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, RefCell};
use std::error::Error;
use std::rc::Rc;

use compositing::windowing::{AnimationState, EmbedderMethods, WindowMethods};
use euclid::{Point2D, Scale, Size2D};
use servo::{RenderingContext, Servo, TouchEventType, WebView, WindowRenderingContext};
use servo_geometry::DeviceIndependentPixel;
use tracing::warn;
use url::Url;
use webrender_api::units::{DeviceIntPoint, DeviceIntRect, DevicePixel, LayoutVector2D};
use webrender_api::ScrollLocation;
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{MouseScrollDelta, WindowEvent};
use winit::event_loop::EventLoop;
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::window::Window;

fn main() -> Result<(), Box<dyn Error>> {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to install crypto provider");

    let event_loop = EventLoop::with_user_event()
        .build()
        .expect("Failed to create EventLoop");
    let mut app = App::new(&event_loop);
    event_loop.run_app(&mut app)?;

    if let App::Running(state) = app {
        if let Some(state) = Rc::into_inner(state) {
            state.servo.deinit();
        }
    }

    Ok(())
}

struct AppState {
    window_delegate: Rc<WindowDelegate>,
    servo: Servo,
    rendering_context: Rc<WindowRenderingContext>,
    webviews: RefCell<Vec<WebView>>,
}

impl ::servo::WebViewDelegate for AppState {
    fn notify_ready_to_show(&self, webview: WebView) {
        let rect = self
            .window_delegate
            .get_coordinates()
            .get_viewport()
            .to_f32();
        webview.focus();
        webview.move_resize(rect);
        webview.raise_to_top(true);
    }

    fn notify_new_frame_ready(&self, _: WebView) {
        self.window_delegate.window.request_redraw();
    }

    fn request_open_auxiliary_webview(&self, parent_webview: WebView) -> Option<WebView> {
        let webview = self.servo.new_auxiliary_webview();
        webview.set_delegate(parent_webview.delegate());
        self.webviews.borrow_mut().push(webview.clone());
        Some(webview)
    }
}

enum App {
    Initial(Waker),
    Running(Rc<AppState>),
}

impl App {
    fn new(event_loop: &EventLoop<WakerEvent>) -> Self {
        Self::Initial(Waker::new(event_loop))
    }
}

impl ApplicationHandler<WakerEvent> for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Self::Initial(waker) = self {
            let display_handle = event_loop
                .display_handle()
                .expect("Failed to get display handle");
            let window = event_loop
                .create_window(Window::default_attributes())
                .expect("Failed to create winit Window");
            let window_handle = window.window_handle().expect("Failed to get window handle");

            let rendering_context = Rc::new(
                WindowRenderingContext::new(display_handle, window_handle, &window.inner_size())
                    .expect("Could not create RenderingContext for window."),
            );
            let window_delegate = Rc::new(WindowDelegate::new(window));

            let _ = rendering_context.make_current();

            let servo = Servo::new(
                Default::default(),
                Default::default(),
                rendering_context.clone(),
                Box::new(EmbedderDelegate {
                    waker: waker.clone(),
                }),
                window_delegate.clone(),
                Default::default(),
                compositing::CompositeTarget::ContextFbo,
            );
            servo.setup_logging();

            let app_state = Rc::new(AppState {
                window_delegate,
                servo,
                rendering_context,
                webviews: Default::default(),
            });

            // Make a new WebView and assign the `AppState` as the delegate.
            let url = Url::parse("https://demo.servo.org/experiments/twgl-tunnel/")
                .expect("Guaranteed by argument");
            let webview = app_state.servo.new_webview(url);
            webview.set_delegate(app_state.clone());
            app_state.webviews.borrow_mut().push(webview);

            *self = Self::Running(app_state);
        }
    }

    fn user_event(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, _event: WakerEvent) {
        if let Self::Running(state) = self {
            state.servo.spin_event_loop();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if let Self::Running(state) = self {
            state.servo.spin_event_loop();
        }

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            },
            WindowEvent::RedrawRequested => {
                if let Self::Running(state) = self {
                    state.webviews.borrow().last().unwrap().paint();
                    state.rendering_context.present();
                }
            },
            WindowEvent::MouseWheel { delta, .. } => {
                if let Self::Running(state) = self {
                    if let Some(webview) = state.webviews.borrow().last() {
                        let moved_by = match delta {
                            MouseScrollDelta::LineDelta(horizontal, vertical) => {
                                LayoutVector2D::new(20. * horizontal, 20. * vertical)
                            },
                            MouseScrollDelta::PixelDelta(pos) => {
                                LayoutVector2D::new(pos.x as f32, pos.y as f32)
                            },
                        };
                        webview.notify_scroll_event(
                            ScrollLocation::Delta(moved_by),
                            DeviceIntPoint::new(10, 10),
                            TouchEventType::Down,
                        );
                    }
                }
            },
            WindowEvent::KeyboardInput { event, .. } => {
                // When pressing 'q' close the latest WebView, then show the next most recently
                // opened view or quit when none are left.
                if event.logical_key.to_text() == Some("q") {
                    if let Self::Running(state) = self {
                        let _ = state.webviews.borrow_mut().pop();
                        match state.webviews.borrow().last() {
                            Some(last) => last.show(true),
                            None => event_loop.exit(),
                        }
                    }
                }
            },
            _ => (),
        }
    }
}

struct EmbedderDelegate {
    waker: Waker,
}

impl EmbedderMethods for EmbedderDelegate {
    // FIXME: rust-analyzer “Implement missing members” autocompletes this as
    // webxr_api::MainThreadWaker, which is not available when building without
    // libservo/webxr, and even if it was, it would fail to compile with E0053.
    fn create_event_loop_waker(&mut self) -> Box<dyn embedder_traits::EventLoopWaker> {
        Box::new(self.waker.clone())
    }
}

#[derive(Clone)]
struct Waker(winit::event_loop::EventLoopProxy<WakerEvent>);
#[derive(Debug)]
struct WakerEvent;

impl Waker {
    fn new(event_loop: &EventLoop<WakerEvent>) -> Self {
        Self(event_loop.create_proxy())
    }
}

impl embedder_traits::EventLoopWaker for Waker {
    fn clone_box(&self) -> Box<dyn embedder_traits::EventLoopWaker> {
        Box::new(Self(self.0.clone()))
    }

    fn wake(&self) {
        if let Err(error) = self.0.send_event(WakerEvent) {
            warn!(?error, "Failed to wake event loop");
        }
    }
}

struct WindowDelegate {
    window: Window,
    animation_state: Cell<AnimationState>,
}

impl WindowDelegate {
    fn new(window: Window) -> Self {
        Self {
            window,
            animation_state: Cell::new(AnimationState::Idle),
        }
    }
}

impl WindowMethods for WindowDelegate {
    fn get_coordinates(&self) -> compositing::windowing::EmbedderCoordinates {
        let monitor = self
            .window
            .current_monitor()
            .or_else(|| self.window.available_monitors().nth(0))
            .expect("Failed to get winit monitor");
        let scale =
            Scale::<f64, DeviceIndependentPixel, DevicePixel>::new(self.window.scale_factor());
        let window_size = winit_size_to_euclid_size(self.window.outer_size()).to_i32();
        let window_origin = self.window.outer_position().unwrap_or_default();
        let window_origin = winit_position_to_euclid_point(window_origin).to_i32();
        let window_rect = DeviceIntRect::from_origin_and_size(window_origin, window_size);
        let viewport_origin = DeviceIntPoint::zero(); // bottom left
        let viewport_size = winit_size_to_euclid_size(self.window.inner_size()).to_f32();
        let viewport = DeviceIntRect::from_origin_and_size(viewport_origin, viewport_size.to_i32());

        compositing::windowing::EmbedderCoordinates {
            hidpi_factor: Scale::new(self.window.scale_factor() as f32),
            screen_size: (winit_size_to_euclid_size(monitor.size()).to_f64() / scale).to_i32(),
            available_screen_size: (winit_size_to_euclid_size(monitor.size()).to_f64() / scale)
                .to_i32(),
            window_rect: (window_rect.to_f64() / scale).to_i32(),
            framebuffer: viewport.size(),
            viewport,
        }
    }

    fn set_animation_state(&self, state: compositing::windowing::AnimationState) {
        self.animation_state.set(state);
    }
}

pub fn winit_size_to_euclid_size<T>(size: PhysicalSize<T>) -> Size2D<T, DevicePixel> {
    Size2D::new(size.width, size.height)
}

pub fn winit_position_to_euclid_point<T>(position: PhysicalPosition<T>) -> Point2D<T, DevicePixel> {
    Point2D::new(position.x, position.y)
}
