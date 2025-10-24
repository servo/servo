/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use compositing_traits::rendering_context::{RenderingContext, SoftwareRenderingContext};
use dpi::PhysicalSize;
use embedder_traits::EventLoopWaker;
use servo::{
    EmbedderControl, JSValue, JavaScriptEvaluationError, LoadStatus, Servo, ServoBuilder, WebView,
    WebViewDelegate,
};

pub struct ServoTest {
    pub servo: Rc<Servo>,
    #[allow(dead_code)]
    pub rendering_context: Rc<dyn RenderingContext>,
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
        Self::new_with_builder(|builder| builder)
    }

    pub(crate) fn new_with_builder<F>(customize: F) -> Self
    where
        F: FnOnce(ServoBuilder) -> ServoBuilder,
    {
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
        let builder = ServoBuilder::new(rendering_context.clone())
            .event_loop_waker(Box::new(EventLoopWakerImpl(user_event_triggered)));
        let builder = customize(builder);
        let servo = Rc::new(builder.build());
        Self {
            servo,
            rendering_context,
        }
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
    pub fn spin(&self, callback: impl Fn() -> bool + 'static) {
        while callback() {
            std::thread::sleep(Duration::from_millis(1));
            if !self.servo.spin_event_loop() {
                return;
            }
        }
    }
}

#[derive(Default)]
pub(crate) struct WebViewDelegateImpl {
    pub(crate) url_changed: Cell<bool>,
    pub(crate) cursor_changed: Cell<bool>,
    pub(crate) new_frame_ready: Cell<bool>,
    pub(crate) load_status_changed: Cell<bool>,
    pub(crate) controls_shown: RefCell<Vec<EmbedderControl>>,
    pub(crate) number_of_controls_shown: Cell<usize>,
    pub(crate) number_of_controls_hidden: Cell<usize>,
}

impl WebViewDelegateImpl {
    pub(crate) fn reset(&self) {
        self.url_changed.set(false);
        self.cursor_changed.set(false);
        self.new_frame_ready.set(false);
        self.controls_shown.borrow_mut().clear();
        self.number_of_controls_shown.set(0);
        self.number_of_controls_hidden.set(0);
    }
}

impl WebViewDelegate for WebViewDelegateImpl {
    fn notify_url_changed(&self, _webview: servo::WebView, _url: url::Url) {
        self.url_changed.set(true);
    }

    fn notify_cursor_changed(&self, _webview: WebView, _: servo::Cursor) {
        self.cursor_changed.set(true);
    }

    fn notify_new_frame_ready(&self, webview: WebView) {
        self.new_frame_ready.set(true);
        webview.paint();
    }

    fn notify_load_status_changed(&self, _webview: WebView, status: LoadStatus) {
        if status == LoadStatus::Complete {
            self.load_status_changed.set(true);
        }
    }

    fn show_embedder_control(&self, _: WebView, embedder_control: EmbedderControl) {
        // Even if not used, controls must be stored so that they do not automatically reply
        // when dropped.
        self.controls_shown.borrow_mut().push(embedder_control);

        self.number_of_controls_shown
            .set(self.number_of_controls_shown.get() + 1);
    }

    fn hide_embedder_control(&self, _webview: WebView, _control_id: servo::EmbedderControlId) {
        self.number_of_controls_hidden
            .set(self.number_of_controls_hidden.get() + 1);
    }
}

pub(crate) fn evaluate_javascript(
    servo_test: &ServoTest,
    webview: WebView,
    script: impl ToString,
) -> Result<JSValue, JavaScriptEvaluationError> {
    let load_webview = webview.clone();
    let _ = servo_test.spin(move || load_webview.load_status() != LoadStatus::Complete);

    let saved_result = Rc::new(RefCell::new(None));
    let callback_result = saved_result.clone();
    webview.evaluate_javascript(script, move |result| {
        *callback_result.borrow_mut() = Some(result)
    });

    let spin_result = saved_result.clone();
    let _ = servo_test.spin(move || spin_result.borrow().is_none());

    (*saved_result.borrow())
        .clone()
        .expect("Should have waited until value available")
}
