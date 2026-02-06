/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use dpi::PhysicalSize;
use embedder_traits::EventLoopWaker;
use paint_api::rendering_context::{RenderingContext, SoftwareRenderingContext};
use servo::{
    EmbedderControl, JSValue, JavaScriptEvaluationError, LoadStatus, Preferences, Servo,
    ServoBuilder, SimpleDialog, WebView, WebViewDelegate,
};

pub struct ServoTest {
    pub servo: Servo,
    pub rendering_context: Rc<dyn RenderingContext>,
}

impl ServoTest {
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
        // Set the proxy to null as the tests will all be on localhost, hence, proxy might interfere.
        let mut preferences = Preferences::default();
        preferences.network_http_proxy_uri = String::new();
        preferences.network_https_proxy_uri = String::new();

        let builder = ServoBuilder::default()
            .preferences(preferences)
            .event_loop_waker(Box::new(EventLoopWakerImpl(user_event_triggered)));
        let builder = customize(builder);
        Self {
            servo: builder.build(),
            rendering_context,
        }
    }

    pub fn servo(&self) -> &Servo {
        &self.servo
    }

    /// Spin the Servo event loop until one of:
    ///  - The given callback returns `Ok(false)`.
    ///  - The given callback returns an `Error`, in which case the `Error` will be returned.
    pub fn spin(&self, callback: impl Fn() -> bool + 'static) {
        while callback() {
            self.servo.spin_event_loop();
            std::thread::sleep(Duration::from_millis(1));
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
    pub(crate) active_dialog: RefCell<Option<SimpleDialog>>,
    pub(crate) number_of_controls_shown: Cell<usize>,
    pub(crate) number_of_controls_hidden: Cell<usize>,
}

#[allow(dead_code)] // Used by some tests and not others
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
        if let EmbedderControl::SimpleDialog(simple_dialog) = embedder_control {
            let previous_dialog = self.active_dialog.borrow_mut().replace(simple_dialog);
            assert!(previous_dialog.is_none());
            return;
        }
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

// Used by some unit tests only. Since they compile into different binaries,
// it will be flagged as unused for certain unit tests.
#[allow(dead_code)]
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

// Used by some unit tests only. Since they compile into different binaries,
// it will be flagged as unused for certain unit tests.
#[allow(dead_code)]
pub(crate) fn show_webview_and_wait_for_rendering_to_be_ready(
    servo_test: &ServoTest,
    webview: &WebView,
    delegate: &Rc<WebViewDelegateImpl>,
) {
    let load_webview = webview.clone();
    servo_test.spin(move || load_webview.load_status() != LoadStatus::Complete);

    delegate.reset();

    // Trigger a change to the display of the document, so that we get at least one
    // new frame after load is complete.
    let _ = evaluate_javascript(
        &servo_test,
        webview.clone(),
        "requestAnimationFrame(() => { \
           document.body.style.background = 'red'; \
           document.body.style.background = 'green'; \
        });",
    );

    // Wait for at least one frame after the load completes.
    let captured_delegate = delegate.clone();
    servo_test.spin(move || !captured_delegate.new_frame_ready.get());
}
