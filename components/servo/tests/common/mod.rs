/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use anyhow::Error;
use compositing_traits::rendering_context::{RenderingContext, SoftwareRenderingContext};
use crossbeam_channel::{Receiver, Sender, unbounded};
use dpi::PhysicalSize;
use embedder_traits::EventLoopWaker;
use parking_lot::Mutex;
use servo::{Servo, ServoBuilder};

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
    fn new() -> Self {
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

    /// Run a Servo test. All tests are run in a `ServoTestThread` and serially. Currently
    /// Servo does not support launching concurrent instances, in order to ensure
    /// isolation and allow for more than a single test per instance.
    pub fn run(
        test_function: impl FnOnce(&ServoTest) -> Result<(), anyhow::Error> + Send + Sync + 'static,
    ) {
        static SERVO_TEST_THREAD: Mutex<OnceLock<ServoTestThread>> = Mutex::new(OnceLock::new());
        let test_thread = SERVO_TEST_THREAD.lock();
        test_thread
            .get_or_init(ServoTestThread::new)
            .run_test(Box::new(test_function));
    }
}

type TestFunction =
    Box<dyn FnOnce(&ServoTest) -> Result<(), anyhow::Error> + Send + Sync + 'static>;

struct ServoTestThread {
    test_function_sender: Sender<TestFunction>,
    result_receiver: Receiver<Result<(), Error>>,
}

impl ServoTestThread {
    fn new() -> Self {
        let (result_sender, result_receiver) = unbounded();
        let (test_function_sender, test_function_receiver) = unbounded();

        // Defined here rather than at the end of this method in order to take advantage
        // of Rust type inference.
        let thread = Self {
            test_function_sender,
            result_receiver,
        };

        let _ = std::thread::spawn(move || {
            let servo_test = ServoTest::new();
            while let Ok(incoming_test_function) = test_function_receiver.recv() {
                let _ = result_sender.send(incoming_test_function(&servo_test));
            }
        });

        thread
    }

    fn run_test(&self, test_function: TestFunction) {
        let _ = self.test_function_sender.send(Box::new(test_function));
        let result = self
            .result_receiver
            .recv()
            .expect("Servo test thread should always return a result.");
        if let Err(result) = result {
            unreachable!("{result}");
        }
    }
}
