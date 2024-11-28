/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// Prevent console window from appearing on Windows
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use servo::servo::EventLoopProxyMessage;
use servo::{Result, Servo};
use winit::application::ApplicationHandler;
use winit::event_loop::{self, DeviceEvents, EventLoop, EventLoopProxy};

struct App {
    servo: Option<Servo>,
    proxy: EventLoopProxy<EventLoopProxyMessage>,
}

impl ApplicationHandler<EventLoopProxyMessage> for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.servo = Some(Servo::new(event_loop, self.proxy.clone()));
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        if let Some(v) = self.servo.as_mut() {
            v.handle_window_event(event_loop, window_id, event);
        }
    }

    fn user_event(
        &mut self,
        event_loop: &event_loop::ActiveEventLoop,
        event: EventLoopProxyMessage,
    ) {
        if let Some(v) = self.servo.as_mut() {
            match event {
                EventLoopProxyMessage::Wake => {
                    v.request_redraw(event_loop);
                },
            }
        }
    }
}

fn main() -> Result<()> {
    let event_loop = EventLoop::<EventLoopProxyMessage>::with_user_event().build()?;
    event_loop.listen_device_events(DeviceEvents::Never);
    let proxy = event_loop.create_proxy();
    let mut app = App { servo: None, proxy };
    event_loop.run_app(&mut app)?;

    Ok(())
}
