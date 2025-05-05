/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use anyhow::Error;
use compositing_traits::rendering_context::{RenderingContext, SoftwareRenderingContext};
use dpi::PhysicalSize;
use embedder_traits::EventLoopWaker;
use servo::{Servo, ServoBuilder};

macro_rules! run_api_tests {
    ($($test_function:ident), +) => {
        let mut failed = false;

        // Be sure that `servo_test` is dropped before exiting early.
        {
            let servo_test = ServoTest::new();
            $(
                common::run_test($test_function, stringify!($test_function), &servo_test, &mut failed);
            )+
        }

        if failed {
            std::process::exit(1);
        }
    }
}

pub(crate) use run_api_tests;

pub(crate) fn run_test(
    test_function: fn(&ServoTest) -> Result<(), Error>,
    test_name: &str,
    servo_test: &ServoTest,
    failed: &mut bool,
) {
    match test_function(servo_test) {
        Ok(_) => println!("    ✅ {test_name}"),
        Err(error) => {
            *failed = true;
            println!("    ❌ {test_name}");
            println!("{}", format!("\n{error:?}").replace("\n", "\n        "));
        },
    }
}

pub struct ServoTest {
    servo: Servo,
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

impl ServoTest {
    pub(crate) fn new() -> Self {
        let rendering_context = Rc::new(
            SoftwareRenderingContext::new(PhysicalSize {
                width: 500,
                height: 500,
            })
            .expect("Could not create SoftwareRenderingContext"),
        );
        assert!(rendering_context.make_current().is_ok());

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
        let servo = ServoBuilder::new(rendering_context.clone())
            .event_loop_waker(Box::new(EventLoopWakerImpl(user_event_triggered)))
            .build();
        Self { servo }
    }

    pub fn servo(&self) -> &Servo {
        &self.servo
    }

    /// Spin the Servo event loop until one of:
    ///  - The given callback returns `Ok(false)`.
    ///  - The given callback returns an `Error`, in which case the `Error` will be returned.
    ///  - Servo has indicated that shut down is complete and we cannot spin the event loop
    ///    any longer.
    // The dead code exception here is because not all test suites that use `common` also
    // use `spin()`.
    #[allow(dead_code)]
    pub fn spin(&self, callback: impl Fn() -> Result<bool, Error> + 'static) -> Result<(), Error> {
        let mut keep_going = true;
        while keep_going {
            std::thread::sleep(Duration::from_millis(1));
            if !self.servo.spin_event_loop() {
                return Ok(());
            }
            let result = callback();
            match result {
                Ok(result) => keep_going = result,
                Err(error) => return Err(error),
            }
        }

        Ok(())
    }
}
