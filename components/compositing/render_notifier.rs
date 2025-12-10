/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base::id::PainterId;
use compositing_traits::{PaintMessage, PaintProxy};
use webrender_api::{DocumentId, FramePublishId, FrameReadyParams};

#[derive(Clone)]
pub(crate) struct RenderNotifier {
    painter_id: PainterId,
    paint_proxy: PaintProxy,
}

impl RenderNotifier {
    pub(crate) fn new(painter_id: PainterId, paint_proxy: PaintProxy) -> RenderNotifier {
        RenderNotifier {
            painter_id,
            paint_proxy,
        }
    }
}

impl webrender_api::RenderNotifier for RenderNotifier {
    fn clone(&self) -> Box<dyn webrender_api::RenderNotifier> {
        Box::new(RenderNotifier::new(
            self.painter_id,
            self.paint_proxy.clone(),
        ))
    }

    fn wake_up(&self, _composite_needed: bool) {}

    fn new_frame_ready(
        &self,
        document_id: DocumentId,
        _: FramePublishId,
        frame_ready_params: &FrameReadyParams,
    ) {
        self.paint_proxy.send(PaintMessage::NewWebRenderFrameReady(
            self.painter_id,
            document_id,
            frame_ready_params.render,
        ));
    }
}
