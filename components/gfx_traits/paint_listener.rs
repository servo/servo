/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use Epoch;
use FrameTreeId;
use LayerId;
use LayerProperties;
use layers::layers::{BufferRequest, LayerBufferSet};
use layers::platform::surface::NativeDisplay;
use msg::constellation_msg::PipelineId;

/// The interface used by the painter to acquire draw targets for each paint frame and
/// submit them to be drawn to the display.
pub trait PaintListener {
    fn native_display(&mut self) -> Option<NativeDisplay>;

    /// Informs the compositor of the layers for the given pipeline. The compositor responds by
    /// creating and/or destroying paint layers as necessary.
    fn initialize_layers_for_pipeline(&mut self,
                                      pipeline_id: PipelineId,
                                      properties: Vec<LayerProperties>,
                                      epoch: Epoch);

    /// Sends new buffers for the given layers to the compositor.
    fn assign_painted_buffers(&mut self,
                              pipeline_id: PipelineId,
                              epoch: Epoch,
                              replies: Vec<(LayerId, Box<LayerBufferSet>)>,
                              frame_tree_id: FrameTreeId);

    /// Inform the compositor that these buffer requests will be ignored.
    fn ignore_buffer_requests(&mut self, buffer_requests: Vec<BufferRequest>);

    // Notification that the paint task wants to exit.
    fn notify_paint_thread_exiting(&mut self, pipeline_id: PipelineId);
}
