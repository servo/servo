/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The task that handles all painting.

use azure::azure_hl::Color;
use layers::layers::BufferRequest;
use msg::compositor_msg::{Epoch, FrameTreeId, LayerId, LayerKind, ScrollPolicy};
use msg::constellation_msg::PipelineExitType;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender};

/// Information about a hardware graphics layer that layout sends to the painting task.
#[derive(Clone, Deserialize, Serialize)]
pub struct PaintLayer {
    /// A per-pipeline ID describing this layer that should be stable across reflows.
    pub id: LayerId,
    /// The color of the background in this layer. Used for unpainted content.
    pub background_color: Color,
    /// The scrolling policy of this layer.
    pub scroll_policy: ScrollPolicy,
}

impl PaintLayer {
    /// Creates a new `PaintLayer`.
    pub fn new(id: LayerId, background_color: Color, scroll_policy: ScrollPolicy) -> PaintLayer {
        PaintLayer {
            id: id,
            background_color: background_color,
            scroll_policy: scroll_policy,
        }
    }
}

pub struct PaintRequest {
    pub buffer_requests: Vec<BufferRequest>,
    pub scale: f32,
    pub layer_id: LayerId,
    pub epoch: Epoch,
    pub layer_kind: LayerKind,
}

pub enum CompositorPaintMsg {
    Paint(Vec<PaintRequest>, FrameTreeId),
    PaintPermissionGranted,
    PaintPermissionRevoked,
    Exit(Option<Sender<()>>, PipelineExitType),
}

#[derive(Clone)]
pub struct CompositorPaintChan(Sender<CompositorPaintMsg>);

impl CompositorPaintChan {
    pub fn new() -> (Receiver<CompositorPaintMsg>, CompositorPaintChan) {
        let (chan, port) = channel();
        (port, CompositorPaintChan(chan))
    }

    pub fn send(&self, msg: CompositorPaintMsg) {
        assert!(self.send_opt(msg).is_ok(), "PaintChan.send: paint port closed")
    }

    pub fn send_opt(&self, msg: CompositorPaintMsg) -> Result<(), CompositorPaintMsg> {
        let &CompositorPaintChan(ref chan) = self;
        chan.send(msg).map_err(|e| e.0)
    }
}
