/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Tests for how the refresh driver paces the animations of a WebView.
mod common;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};

use dpi::PhysicalSize;
use embedder_traits::RefreshDriver;
use euclid::default::Size2D as UntypedSize2D;
use image::RgbaImage;
use servo::{InputEvent, JSValue, MouseMoveEvent, RenderingContext, WebView, WebViewBuilder};
use surfman::{Error, Surface, SurfaceTexture};
use url::Url;
use webrender_api::units::{DeviceIntRect, DevicePoint};

use crate::common::{ServoTest, WebViewDelegateImpl, evaluate_javascript};

/// A [`RefreshDriver`] whose frames only start when the test starts them.
#[derive(Default)]
struct ManualRefreshDriver {
    start_frame_callbacks: RefCell<Vec<Box<dyn Fn() + Send + 'static>>>,
}

impl ManualRefreshDriver {
    fn waiting_for_frame(&self) -> bool {
        !self.start_frame_callbacks.borrow().is_empty()
    }

    fn start_frame(&self) {
        for callback in self.start_frame_callbacks.take() {
            callback();
        }
    }
}

impl RefreshDriver for ManualRefreshDriver {
    fn observe_next_frame(&self, start_frame_callback: Box<dyn Fn() + Send + 'static>) {
        self.start_frame_callbacks
            .borrow_mut()
            .push(start_frame_callback);
    }
}

/// A [`RenderingContext`] whose frames start with a [`ManualRefreshDriver`]. Every other method
/// forwards to the inner context.
struct ManuallyRefreshedRenderingContext(Rc<dyn RenderingContext>, Rc<ManualRefreshDriver>);

impl RenderingContext for ManuallyRefreshedRenderingContext {
    fn prepare_for_rendering(&self) {
        self.0.prepare_for_rendering();
    }

    fn read_to_image(&self, source_rectangle: DeviceIntRect) -> Option<RgbaImage> {
        self.0.read_to_image(source_rectangle)
    }

    fn size(&self) -> PhysicalSize<u32> {
        self.0.size()
    }

    fn resize(&self, size: PhysicalSize<u32>) {
        self.0.resize(size);
    }

    fn present(&self) {
        self.0.present();
    }

    fn make_current(&self) -> Result<(), Error> {
        self.0.make_current()
    }

    fn gleam_gl_api(&self) -> Rc<dyn gleam::gl::Gl> {
        self.0.gleam_gl_api()
    }

    fn glow_gl_api(&self) -> Arc<glow::Context> {
        self.0.glow_gl_api()
    }

    fn create_texture(
        &self,
        surface: Surface,
    ) -> Option<(SurfaceTexture, u32, UntypedSize2D<i32>)> {
        self.0.create_texture(surface)
    }

    fn destroy_texture(&self, surface_texture: SurfaceTexture) -> Option<Surface> {
        self.0.destroy_texture(surface_texture)
    }

    fn connection(&self) -> Option<surfman::Connection> {
        self.0.connection()
    }

    fn refresh_driver(&self) -> Option<Rc<dyn RefreshDriver>> {
        Some(self.1.clone())
    }
}

/// A [`WebView`] whose page counts the animation frames it runs in `frames_run`. Each frame
/// changes the page, as script runs animation frames that change nothing on a timer of its own.
struct AnimatingWebView<'a> {
    servo_test: &'a ServoTest,
    webview: WebView,
}

impl<'a> AnimatingWebView<'a> {
    /// Loads the page in a [`WebView`] rendering to `rendering_context` and waits for it to
    /// animate.
    fn new(
        servo_test: &'a ServoTest,
        rendering_context: Rc<dyn RenderingContext>,
        delegate: Rc<WebViewDelegateImpl>,
    ) -> Self {
        let webview = WebViewBuilder::new(servo_test.servo(), rendering_context)
            .delegate(delegate)
            .url(
                Url::parse(
                    "data:text/html,<body><script>window.frames_run = 0; \
                     (function frame() { \
                       document.body.textContent = ++frames_run; requestAnimationFrame(frame); \
                     })();</script>",
                )
                .unwrap(),
            )
            .build();
        servo_test.spin(|| !webview.animating());
        Self {
            servo_test,
            webview,
        }
    }

    /// Like [`AnimatingWebView::new`], with frames that start with `refresh_driver`, once it
    /// waits for a frame to start.
    fn new_with_manual_refresh_driver(
        servo_test: &'a ServoTest,
        refresh_driver: Rc<ManualRefreshDriver>,
        delegate: Rc<WebViewDelegateImpl>,
    ) -> Self {
        let rendering_context = Rc::new(ManuallyRefreshedRenderingContext(
            servo_test.rendering_context.clone(),
            refresh_driver.clone(),
        ));
        let animating_webview = Self::new(servo_test, rendering_context, delegate);
        servo_test.spin(|| !refresh_driver.waiting_for_frame());
        animating_webview
    }

    fn frames_run(&self) -> f64 {
        match evaluate_javascript(self.servo_test, self.webview.clone(), "frames_run") {
            Ok(JSValue::Number(frames_run)) => frames_run,
            result => panic!("Could not read the number of frames run: {result:?}"),
        }
    }

    /// The number of animation frames the page runs over [`PAINTS`] paints that an embedder
    /// does for reasons of its own, without starting a frame.
    fn frames_run_over_paints(&self) -> f64 {
        let before = self.frames_run();
        for _ in 0..PAINTS {
            self.webview.paint();
            self.servo_test.servo().spin_event_loop();
        }
        self.frames_run() - before
    }
}

/// How many paints [`AnimatingWebView::frames_run_over_paints`] does in a row.
const PAINTS: u32 = 100;

#[test]
fn test_paints_between_frames_do_not_advance_animations() {
    let servo_test = ServoTest::new();
    let refresh_driver = Rc::new(ManualRefreshDriver::default());
    let delegate = Rc::new(WebViewDelegateImpl::default());
    let animating_webview =
        AnimatingWebView::new_with_manual_refresh_driver(&servo_test, refresh_driver, delegate);

    // Only script's timer of its own can still run a few, while no frame starts.
    assert!(animating_webview.frames_run_over_paints() < f64::from(PAINTS / 10));
}

#[test]
fn test_frame_start_after_paints_asks_for_a_paint() {
    let servo_test = ServoTest::new();
    let refresh_driver = Rc::new(ManualRefreshDriver::default());
    let delegate = Rc::new(WebViewDelegateImpl::default());
    let animating_webview = AnimatingWebView::new_with_manual_refresh_driver(
        &servo_test,
        refresh_driver.clone(),
        delegate.clone(),
    );

    // A paint while no frame has started uses up what servo would have asked for a paint for.
    animating_webview.webview.paint();
    servo_test.servo().spin_event_loop();
    delegate.reset();
    servo_test.servo().spin_event_loop();
    assert!(!delegate.new_frame_ready.get());

    // Servo still asks for the paint that advances the animations once the frame starts.
    refresh_driver.start_frame();
    servo_test.servo().spin_event_loop();
    assert!(delegate.new_frame_ready.get());
}

#[test]
fn test_input_does_not_speed_up_animations() {
    let servo_test = ServoTest::new();
    let delegate = Rc::new(WebViewDelegateImpl::default());
    let animating_webview =
        AnimatingWebView::new(&servo_test, servo_test.rendering_context.clone(), delegate);

    // Moving the mouse over the page, with a paint for each move, for a quarter of a second.
    let before = animating_webview.frames_run();
    let start = Instant::now();
    let mut x = 0.;
    while start.elapsed() < Duration::from_millis(250) {
        x = (x + 1.) % 100.;
        animating_webview
            .webview
            .notify_input_event(InputEvent::MouseMove(MouseMoveEvent::new(
                DevicePoint::new(x, 10.).into(),
            )));
        animating_webview.webview.paint();
        servo_test.servo().spin_event_loop();
    }
    let frames = animating_webview.frames_run() - before;

    // The page animates no faster than the 120 frames per second of the default refresh driver,
    // whose `FRAME_DURATION` is in `components/paint/refresh_driver.rs`.
    assert!(
        frames <= 40.,
        "{frames} animation frames in a quarter of a second"
    );
}
