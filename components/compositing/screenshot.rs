/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::rc::Rc;

use base::Epoch;
use base::id::{PipelineId, WebViewId};
use embedder_traits::ScreenshotCaptureError;
use euclid::{Point2D, Size2D};
use image::RgbaImage;
use log::error;
use rustc_hash::FxHashMap;
use webrender_api::units::{DeviceIntRect, DeviceRect};

use crate::paint::RepaintReason;
use crate::painter::Painter;

pub(crate) struct ScreenshotRequest {
    webview_id: WebViewId,
    rect: Option<DeviceRect>,
    callback: Box<dyn FnOnce(Result<RgbaImage, ScreenshotCaptureError>) + 'static>,
    phase: ScreenshotRequestPhase,
}

/// Screenshots requests happen in three phases:
#[derive(PartialEq)]
pub(crate) enum ScreenshotRequestPhase {
    /// A request is sent to the Constellation, asking each Pipeline in a WebView,
    /// to report the display list epoch to render for the screenshot. Each Pipeline
    /// will wait to send an epoch that happens after the Pipeline is ready in a
    /// variety of ways:
    ///
    ///  - The `load` event has fired.
    ///  - All render blocking elements are no longer blocking the rendering.
    ///  - All images are loaded and displayed.
    ///  - All web fonts are loaded.
    ///  - The `reftest-wait` and `test-wait` classes have been removed from the root element.
    ///  - The rendering is up-to-date
    ///
    /// When all Pipelines have reported this epoch to the Constellation it sends a
    /// ScreenshotReadinessResponse back to the renderer.
    ConstellationRequest,
    /// The renderer has received the ScreenshotReadinessReponse from the Constellation
    /// and is now waiting for all display lists to be received from the Pipelines and
    /// sent to WebRender.
    WaitingOnPipelineDisplayLists(Rc<FxHashMap<PipelineId, Epoch>>),
    /// Once the renderer has received all of the Pipeline display lists necessary to take
    /// the screenshot and uploaded them to WebRender, it waits for an appropriate frame to
    /// be ready. Currently this just waits for the [`FrameDelayer`] to stop delaying frames
    /// and for there to be no pending WebRender frames (ones sent to WebRender that are not
    /// ready yet). Once that happens, and a potential extra repaint is triggered, the renderer
    /// will take the screenshot and fufill the request.
    WaitingOnFrame,
}

#[derive(Default)]
pub(crate) struct ScreenshotTaker {
    /// A vector of pending screenshots to be taken. These will be resolved once the
    /// pages have finished loading all content and the rendering reflects the finished
    /// state. See [`ScreenshotRequestPhase`] for more information.
    requests: RefCell<Vec<ScreenshotRequest>>,
}

impl ScreenshotTaker {
    pub(crate) fn request_screenshot(
        &self,
        webview_id: WebViewId,
        rect: Option<DeviceRect>,
        callback: Box<dyn FnOnce(Result<RgbaImage, ScreenshotCaptureError>) + 'static>,
    ) {
        self.requests.borrow_mut().push(ScreenshotRequest {
            webview_id,
            rect,
            callback,
            phase: ScreenshotRequestPhase::ConstellationRequest,
        });
    }

    pub(crate) fn handle_screenshot_readiness_reply(
        &self,
        webview_id: WebViewId,
        expected_epochs: FxHashMap<PipelineId, Epoch>,
        renderer: &Painter,
    ) {
        let expected_epochs = Rc::new(expected_epochs);

        for screenshot_request in self.requests.borrow_mut().iter_mut() {
            if screenshot_request.webview_id != webview_id ||
                screenshot_request.phase != ScreenshotRequestPhase::ConstellationRequest
            {
                continue;
            }
            screenshot_request.phase =
                ScreenshotRequestPhase::WaitingOnPipelineDisplayLists(expected_epochs.clone());
        }

        // Maybe when the message is received, the renderer already has the all of the necessary
        // display lists from the Pipelines. In that case, the renderer should move immediately
        // to the next phase of the screenshot state machine.
        self.prepare_screenshot_requests_for_render(renderer);
    }

    pub(crate) fn prepare_screenshot_requests_for_render(&self, renderer: &Painter) {
        let mut any_became_ready = false;

        for screenshot_request in self.requests.borrow_mut().iter_mut() {
            let ScreenshotRequestPhase::WaitingOnPipelineDisplayLists(pipelines) =
                &screenshot_request.phase
            else {
                continue;
            };

            let Some(webview) = renderer.webview_renderer(screenshot_request.webview_id) else {
                continue;
            };

            if pipelines.iter().all(|(pipeline_id, expected_epoch)| {
                webview
                    .pipelines
                    .get(pipeline_id)
                    .and_then(|pipeline| pipeline.display_list_epoch)
                    .is_some_and(|epoch| epoch >= *expected_epoch)
            }) {
                screenshot_request.phase = ScreenshotRequestPhase::WaitingOnFrame;
                any_became_ready = true;
            }
        }

        // If there are now screenshots waiting on a frame, and there are no pending frames,
        // immediately trigger a repaint so that screenshots can be taken when the repaint
        // is done.
        if any_became_ready {
            self.maybe_trigger_paint_for_screenshot(renderer);
        }
    }

    pub(crate) fn maybe_trigger_paint_for_screenshot(&self, renderer: &Painter) {
        if renderer.has_pending_frames() {
            return;
        }

        if self.requests.borrow().iter().any(|screenshot_request| {
            matches!(
                screenshot_request.phase,
                ScreenshotRequestPhase::WaitingOnFrame
            )
        }) {
            renderer.set_needs_repaint(RepaintReason::ReadyForScreenshot);
        }
    }

    pub(crate) fn maybe_take_screenshots(&self, renderer: &Painter) {
        if renderer.has_pending_frames() {
            return;
        }

        let mut requests = self.requests.borrow_mut();
        if requests.is_empty() {
            return;
        }

        let ready_screenshot_requests = requests.extract_if(.., |screenshot_request| {
            !matches!(
                screenshot_request.phase,
                ScreenshotRequestPhase::WaitingOnFrame
            )
        });

        for screenshot_request in ready_screenshot_requests {
            let callback = screenshot_request.callback;
            let Some(webview_renderer) = renderer.webview_renderer(screenshot_request.webview_id)
            else {
                callback(Err(ScreenshotCaptureError::WebViewDoesNotExist));
                continue;
            };

            let viewport_rect = webview_renderer.rect.to_i32();
            let viewport_size = viewport_rect.size();
            let rect = screenshot_request.rect.map_or(viewport_rect, |rect| {
                // We need to convert to the bottom-left origin coordinate
                // system used by OpenGL
                // If dpi > 1, y can be computed to be -1 due to rounding issue, resulting in panic.
                // https://github.com/servo/servo/issues/39306#issuecomment-3342204869
                let x = rect.min.x as i32;
                let y =
                    0.max((viewport_size.height as f32 - rect.min.y - rect.size().height) as i32);
                let w = rect.size().width as i32;
                let h = rect.size().height as i32;

                DeviceIntRect::from_origin_and_size(Point2D::new(x, y), Size2D::new(w, h))
            });
            if let Err(error) = renderer.rendering_context.make_current() {
                error!("Failed to make the rendering context current: {error:?}");
            }
            let result = renderer
                .rendering_context
                .read_to_image(rect)
                .ok_or(ScreenshotCaptureError::CouldNotReadImage);
            callback(result);
        }
    }
}
