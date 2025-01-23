/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::error::Error;
use std::mem::replace;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use base::id::WebViewId;
use compositing::windowing::{AnimationState, EmbedderEvent, EmbedderMethods, WindowMethods};
use embedder_traits::EmbedderMsg;
use euclid::{Point2D, Scale, Size2D};
use servo::Servo;
use servo_geometry::DeviceIndependentPixel;
use servo_url::ServoUrl;
use surfman::{Connection, SurfaceType};
use tracing::warn;
use webrender_api::units::{DeviceIntPoint, DeviceIntRect, DevicePixel};
use webrender_traits::RenderingContext;
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::WindowEvent;
use winit::event_loop::EventLoop;
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::window::Window;

fn main() -> Result<(), Box<dyn Error>> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install crypto provider");

    let event_loop = EventLoop::with_user_event()
        .build()
        .expect("Failed to create EventLoop");
    let mut app = App::new(&event_loop);
    event_loop.run_app(&mut app)?;

    Ok(())
}

enum App {
    Initial(Waker),
    Running {
        window_delegate: Rc<WindowDelegate>,
        servo: Servo<WindowDelegate>,
    },
    Exiting,
}

impl App {
    fn new(event_loop: &EventLoop<WakerEvent>) -> Self {
        Self::Initial(Waker::new(event_loop))
    }
}

impl ApplicationHandler<WakerEvent> for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Self::Initial(waker) = self {
            let window = event_loop
                .create_window(Window::default_attributes())
                .expect("Failed to create winit Window");
            let display_handle = event_loop
                .display_handle()
                .expect("Failed to get display handle");
            let connection = Connection::from_display_handle(display_handle)
                .expect("Failed to create connection");
            let adapter = connection
                .create_adapter()
                .expect("Failed to create adapter");
            let rendering_context = RenderingContext::create(&connection, &adapter, None)
                .expect("Failed to create rendering context");
            let native_widget = rendering_context
                .connection()
                .create_native_widget_from_window_handle(
                    window.window_handle().expect("Failed to get window handle"),
                    winit_size_to_euclid_size(window.inner_size())
                        .to_i32()
                        .to_untyped(),
                )
                .expect("Failed to create native widget");
            let surface = rendering_context
                .create_surface(SurfaceType::Widget { native_widget })
                .expect("Failed to create surface");
            rendering_context
                .bind_surface(surface)
                .expect("Failed to bind surface");
            rendering_context
                .make_gl_context_current()
                .expect("Failed to make context current");
            let window_delegate = Rc::new(WindowDelegate::new(window));
            let mut servo = Servo::new(
                Default::default(),
                Default::default(),
                rendering_context,
                Box::new(EmbedderDelegate {
                    waker: waker.clone(),
                }),
                window_delegate.clone(),
                Default::default(),
                compositing::CompositeTarget::Window,
            );
            servo.setup_logging();
            servo.handle_events([EmbedderEvent::NewWebView(
                ServoUrl::parse("https://demo.servo.org/experiments/twgl-tunnel/")
                    .expect("Guaranteed by argument"),
                WebViewId::new(),
            )]);
            *self = Self::Running {
                window_delegate,
                servo,
            };
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if let Self::Running {
            window_delegate,
            servo,
        } = self
        {
            let mut events_for_servo = vec![];
            for (_webview_id, message) in servo.get_events() {
                match message {
                    // FIXME: rust-analyzer autocompletes this as top_level_browsing_context_id
                    EmbedderMsg::WebViewOpened(webview_id) => {
                        let rect = window_delegate.get_coordinates().get_viewport().to_f32();
                        events_for_servo.extend([
                            EmbedderEvent::FocusWebView(webview_id),
                            EmbedderEvent::MoveResizeWebView(webview_id, rect),
                            EmbedderEvent::RaiseWebViewToTop(webview_id, true),
                        ]);
                    },
                    _ => {},
                }
            }
            servo.handle_events(events_for_servo);
        }
        match event {
            WindowEvent::CloseRequested => {
                if matches!(self, Self::Running { .. }) {
                    let Self::Running { servo, .. } = replace(self, Self::Exiting) else {
                        unreachable!()
                    };
                    // TODO: ask Servo to shut down and wait for EmbedderMsg::Shutdown?
                    servo.deinit();
                }
                event_loop.exit();
            },
            WindowEvent::RedrawRequested => {
                if let Self::Running {
                    window_delegate,
                    servo,
                } = self
                {
                    servo.present();
                    window_delegate.window.request_redraw();
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
struct Waker(Arc<Mutex<winit::event_loop::EventLoopProxy<WakerEvent>>>);
#[derive(Debug)]
struct WakerEvent;

impl Waker {
    fn new(event_loop: &EventLoop<WakerEvent>) -> Self {
        Self(Arc::new(Mutex::new(event_loop.create_proxy())))
    }
}

impl embedder_traits::EventLoopWaker for Waker {
    fn clone_box(&self) -> Box<dyn embedder_traits::EventLoopWaker> {
        Box::new(Self(self.0.clone()))
    }

    fn wake(&self) {
        if let Err(error) = self
            .0
            .lock()
            .expect("Failed to lock EventLoopProxy")
            .send_event(WakerEvent)
        {
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
