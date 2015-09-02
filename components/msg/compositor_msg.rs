/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use azure::azure_hl::Color;
use constellation_msg::{Key, KeyState, KeyModifiers};
use euclid::Matrix4;
use euclid::{Size2D, Point2D, Rect};
use ipc_channel::ipc::IpcSender;
use layers::layers::{BufferRequest, LayerBufferSet};
use layers::platform::surface::NativeDisplay;
use std::fmt;
use std::fmt::{Formatter, Debug};

use constellation_msg::PipelineId;

/// A newtype struct for denoting the age of messages; prevents race conditions.
#[derive(PartialEq, Eq, Debug, Copy, Clone, PartialOrd, Ord, Deserialize, Serialize)]
pub struct Epoch(pub u32);

impl Epoch {
    pub fn next(&mut self) {
        let Epoch(ref mut u) = *self;
        *u += 1;
    }
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub struct FrameTreeId(pub u32);

impl FrameTreeId {
    pub fn next(&mut self) {
        let FrameTreeId(ref mut u) = *self;
        *u += 1;
    }
}

#[derive(Clone, PartialEq, Eq, Copy, Hash, Deserialize, Serialize)]
pub struct LayerId(pub usize, pub u32);

impl Debug for LayerId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let LayerId(a, b) = *self;
        write!(f, "Layer({}, {})", a, b)
    }
}

impl LayerId {
    /// FIXME(#2011, pcwalton): This is unfortunate. Maybe remove this in the future.
    pub fn null() -> LayerId {
        LayerId(0, 0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LayerKind {
    NoTransform,
    HasTransform,
}

/// The scrolling policy of a layer.
#[derive(Clone, PartialEq, Eq, Copy, Deserialize, Serialize, Debug)]
pub enum ScrollPolicy {
    /// These layers scroll when the parent receives a scrolling message.
    Scrollable,
    /// These layers do not scroll when the parent receives a scrolling message.
    FixedPosition,
}

/// All layer-specific information that the painting task sends to the compositor other than the
/// buffer contents of the layer itself.
#[derive(Copy, Clone)]
pub struct LayerProperties {
    /// An opaque ID. This is usually the address of the flow and index of the box within it.
    pub id: LayerId,
    /// The id of the parent layer.
    pub parent_id: Option<LayerId>,
    /// The position and size of the layer in pixels.
    pub rect: Rect<f32>,
    /// The background color of the layer.
    pub background_color: Color,
    /// The scrolling policy of this layer.
    pub scroll_policy: ScrollPolicy,
    /// The transform for this layer
    pub transform: Matrix4,
    /// The perspective transform for this layer
    pub perspective: Matrix4,
    /// Whether this layer establishes a new 3d rendering context.
    pub establishes_3d_context: bool,
    /// Whether this layer scrolls its overflow area.
    pub scrolls_overflow_area: bool,
}

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
    fn notify_paint_task_exiting(&mut self, pipeline_id: PipelineId);
}

#[derive(Deserialize, Serialize)]
pub enum ScriptToCompositorMsg {
    ScrollFragmentPoint(PipelineId, LayerId, Point2D<f32>, bool),
    SetTitle(PipelineId, Option<String>),
    SendKeyEvent(Key, KeyState, KeyModifiers),
    GetClientWindow(IpcSender<(Size2D<u32>, Point2D<i32>)>),
    MoveTo(Point2D<i32>),
    ResizeTo(Size2D<u32>),
    Exit,
}
