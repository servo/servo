/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;

use servo::protocol_handler::ProtocolRegistry;
use servo::{EventLoopWaker, Opts, Preferences, ServoBuilder};
use url::Url;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::WindowId;

mod window;

pub const VERSION: &str = concat!("Ringtail ", env!("CARGO_PKG_VERSION"), "-", env!("GIT_SHA"));

#[cfg(not(any(target_os = "android", target_env = "ohos")))]
pub fn main() {
    crate::init_crypto();

    let event_loop = winit::event_loop::EventLoop::with_user_event().build().unwrap();
    let proxy = event_loop.create_proxy();
    
    let mut app = App::new(&event_loop, proxy);
    event_loop.run_app(&mut app).unwrap();
}

pub fn init_crypto() {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Error initializing crypto provider");
}

struct App {
    waker: Box<dyn EventLoopWaker>,
    proxy: winit::event_loop::EventLoopProxy<AppEvent>,
    state: AppState,
    t_start: Instant,
    t: Instant,
}

enum AppState {
    Initializing,
    Running(Rc<RunningAppState>),
    ShuttingDown,
}

struct RunningAppState {
    servo: servo::Servo,
    window: Rc<window::RingtailWindow>,
}

impl App {
    fn new(event_loop: &winit::event_loop::EventLoop<AppEvent>, proxy: winit::event_loop::EventLoopProxy<AppEvent>) -> Self {
        let t = Instant::now();
        let waker = Box::new(HeadedEventLoopWaker::new(event_loop));
        App {
            waker,
            proxy,
            state: AppState::Initializing,
            t_start: t,
            t,
        }
    }

    fn init(&mut self, active_event_loop: &ActiveEventLoop) {
        let mut protocol_registry = ProtocolRegistry::default();
        let _ = protocol_registry.register(
            "resource",
            window::resource_protocol::ResourceProtocolHandler::default(),
        );

        let url = Url::parse("resource:///resource_protocol/newtab.html").unwrap();

        let servo_builder = ServoBuilder::default()
            .opts(Opts::default())
            .preferences(Preferences::default())
            .protocol_registry(protocol_registry)
            .event_loop_waker(self.waker.clone());

        let mut window = window::RingtailWindow::new(active_event_loop, url.clone());
        
        let servo = servo_builder.build();
        servo.setup_logging();

        window.load_url(url, &servo);

        let running_state = Rc::new(RunningAppState {
            servo,
            window: Rc::new(window),
        });

        self.state = AppState::Running(running_state);
    }
}

impl ApplicationHandler<AppEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.init(event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        window_event: WindowEvent,
    ) {
        // 1. Temporarily extract the state to run the event handler, then drop it.
        if let AppState::Running(state) = &self.state {
            state.window.handle_winit_event(&window_event);
        }

        // 2. Handle specific structural events like Redraw or Close
        match window_event {
            WindowEvent::RedrawRequested => {
                if let AppState::Running(state) = &self.state {
                    state.window.paint();
                }
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
                return;
            }
            _ => {
                // 3. Now self can be safely borrowed mutably here 
                // because no outer 'state' binding exists anymore!
                if !self.pump_servo_event_loop() {
                    event_loop.exit();
                }
                
                // 4. Temporarily borrow again to request the redraw safely
                if let AppState::Running(state) = &self.state {
                    state.window.request_redraw();
                }
            }
        }

        event_loop.set_control_flow(ControlFlow::Wait);
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, _app_event: AppEvent) {
        if !self.pump_servo_event_loop() {
            event_loop.exit();
        }

        // When a background channel wakes up the event loop, ask Winit to refresh the window
        if let AppState::Running(state) = &self.state {
            state.window.request_redraw();
        }

        event_loop.set_control_flow(ControlFlow::Wait);
    }
}

impl App {
    fn pump_servo_event_loop(&mut self) -> bool {
        let AppState::Running(state) = &self.state else {
            return false;
        };

        state.servo.spin_event_loop();
        true
    }
}

#[derive(Clone, Debug)]
enum AppEvent {
    Wakeup,
}

#[derive(Clone)]
struct HeadedEventLoopWaker {
    proxy: Arc<Mutex<winit::event_loop::EventLoopProxy<AppEvent>>>,
}

impl HeadedEventLoopWaker {
    fn new(event_loop: &winit::event_loop::EventLoop<AppEvent>) -> Self {
        let proxy = Arc::new(Mutex::new(event_loop.create_proxy()));
        Self { proxy }
    }
}

impl EventLoopWaker for HeadedEventLoopWaker {
    fn wake(&self) {
        let _ = self.proxy.lock().unwrap().send_event(AppEvent::Wakeup);
    }

    fn clone_box(&self) -> Box<dyn EventLoopWaker> {
        Box::new(self.clone())
    }
}
