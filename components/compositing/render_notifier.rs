/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use compositing_traits::{CompositorMsg, CompositorProxy};
use webrender_api::{DocumentId, FramePublishId, FrameReadyParams};

#[derive(Clone)]
pub(crate) struct RenderNotifier {
    compositor_proxy: CompositorProxy,
}

impl RenderNotifier {
    pub(crate) fn new(compositor_proxy: CompositorProxy) -> RenderNotifier {
        RenderNotifier { compositor_proxy }
    }
}

impl webrender_api::RenderNotifier for RenderNotifier {
    fn clone(&self) -> Box<dyn webrender_api::RenderNotifier> {
        Box::new(RenderNotifier::new(self.compositor_proxy.clone()))
    }

    fn wake_up(&self, _composite_needed: bool) {}

    fn new_frame_ready(
        &self,
        document_id: DocumentId,
        _: FramePublishId,
        frame_ready_params: &FrameReadyParams,
    ) {
        self.compositor_proxy
            .send(CompositorMsg::NewWebRenderFrameReady(
                document_id,
                frame_ready_params.render,
            ));
    }
}
