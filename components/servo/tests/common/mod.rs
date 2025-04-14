/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use compositing::windowing::EmbedderMethods;
use compositing_traits::rendering_context::{RenderingContext, SoftwareRenderingContext};
use dpi::PhysicalSize;
use embedder_traits::EventLoopWaker;
use euclid::Scale;
use servo::Servo;

pub struct ServoTest {
    servo: Servo,
}

impl ServoTest {
    pub fn new() -> Self {
        let rendering_context = Rc::new(
            SoftwareRenderingContext::new(PhysicalSize {
                width: 500,
                height: 500,
            })
            .expect("Could not create SoftwareRenderingContext"),
        );
        assert!(rendering_context.make_current().is_ok());

        #[derive(Clone)]
        struct EmbedderMethodsImpl(Arc<AtomicBool>);
        impl EmbedderMethods for EmbedderMethodsImpl {
            fn create_event_loop_waker(&mut self) -> Box<dyn embedder_traits::EventLoopWaker> {
                Box::new(EventLoopWakerImpl(self.0.clone()))
            }
        }

        #[derive(Clone)]
        struct EventLoopWakerImpl(Arc<AtomicBool>);
        impl EventLoopWaker for EventLoopWakerImpl {
            fn clone_box(&self) -> Box<dyn EventLoopWaker> {
                Box::new(self.clone())
            }

            fn wake(&self) {
                self.0.store(true, Ordering::Relaxed);
            }
        }

        let user_event_triggered = Arc::new(AtomicBool::new(false));
        let servo = Servo::new(
            Default::default(),
            Default::default(),
            rendering_context.clone(),
            Box::new(EmbedderMethodsImpl(user_event_triggered)),
            Default::default(),
        );
        Self { servo }
    }

    pub fn servo(&self) -> &Servo {
        &self.servo
    }
}

impl Drop for ServoTest {
    fn drop(&mut self) {
        self.servo.start_shutting_down();
        while self.servo.spin_event_loop() {
            std::thread::sleep(Duration::from_millis(1));
        }
        self.servo.deinit();
    }
}
