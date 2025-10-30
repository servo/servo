/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

use euclid::{Scale, Size2D};
use servo::{
    InputEvent, RenderingContext, Servo, ServoBuilder, WebView, WebViewBuilder, WheelDelta,
    WheelEvent, WheelMode, WindowRenderingContext,
};
use tracing::warn;
use url::Url;
use webrender_api::units::{DevicePixel, DevicePoint};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
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
    window: Window,
    servo: Servo,
    rendering_context: Rc<WindowRenderingContext>,
    webviews: RefCell<Vec<WebView>>,
}

impl ::servo::WebViewDelegate for AppState {
    fn notify_new_frame_ready(&self, _: WebView) {
        self.window.request_redraw();
    }

    fn request_open_auxiliary_webview(&self, parent_webview: WebView) -> Option<WebView> {
        let webview = WebViewBuilder::new_auxiliary(&self.servo)
            .hidpi_scale_factor(Scale::new(self.window.scale_factor() as f32))
            .delegate(parent_webview.delegate())
            .build();
        webview.focus_and_raise_to_top(true);

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
                WindowRenderingContext::new(display_handle, window_handle, window.inner_size())
                    .expect("Could not create RenderingContext for window."),
            );

            let _ = rendering_context.make_current();

            let servo = ServoBuilder::new(rendering_context.clone())
                .event_loop_waker(Box::new(waker.clone()))
                .build();
            servo.setup_logging();

            let app_state = Rc::new(AppState {
                window,
                servo,
                rendering_context,
                webviews: Default::default(),
            });

            // Make a new WebView and assign the `AppState` as the delegate.
            let url = Url::parse("https://demo.servo.org/experiments/twgl-tunnel/")
                .expect("Guaranteed by argument");

            let webview = WebViewBuilder::new(&app_state.servo)
                .url(url)
                .hidpi_scale_factor(Scale::new(app_state.window.scale_factor() as f32))
                .delegate(app_state.clone())
                .build();

            webview.focus_and_raise_to_top(true);

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
                        let (delta_x, delta_y, mode) = match delta {
                            MouseScrollDelta::LineDelta(dx, dy) => {
                                ((dx * 76.0) as f64, (dy * 76.0) as f64, WheelMode::DeltaLine)
                            },
                            MouseScrollDelta::PixelDelta(delta) => {
                                (delta.x, delta.y, WheelMode::DeltaPixel)
                            },
                        };

                        webview.notify_input_event(InputEvent::Wheel(WheelEvent::new(
                            WheelDelta {
                                x: delta_x,
                                y: delta_y,
                                z: 0.0,
                                mode,
                            },
                            DevicePoint::default().into(),
                        )));
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
            WindowEvent::Resized(new_size) => {
                if let Self::Running(state) = self {
                    if let Some(webview) = state.webviews.borrow().last() {
                        let mut rect = webview.rect();
                        rect.set_size(winit_size_to_euclid_size(new_size).to_f32());
                        webview.move_resize(rect);
                        webview.resize(new_size);
                    }
                }
            },
            _ => (),
        }
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

pub fn winit_size_to_euclid_size<T>(size: PhysicalSize<T>) -> Size2D<T, DevicePixel> {
    Size2D::new(size.width, size.height)
}
