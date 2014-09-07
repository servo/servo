/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![comment = "The Servo Parallel Browser Project"]
#![license = "MPL"]

extern crate geom;
extern crate servo_msg = "msg";
extern crate servo_net = "net";
extern crate url;
extern crate std;
extern crate serialize;

// This module contains traits in script used generically
//   in the rest of Servo.
// The traits are here instead of in layout so
//   that these modules won't have to depend on script.

use servo_msg::constellation_msg::{ConstellationChan, PipelineId, Failure, WindowSizeData};
use servo_msg::constellation_msg::SubpageId;
use servo_msg::compositor_msg::ScriptListener;
use servo_net::image_cache_task::ImageCacheTask;
use servo_net::resource_task::ResourceTask;
use std::any::Any;
use url::Url;

use geom::point::Point2D;

use serialize::{Encodable, Encoder};

pub struct NewLayoutInfo {
    pub old_pipeline_id: PipelineId,
    pub new_pipeline_id: PipelineId,
    pub subpage_id: SubpageId,
    pub layout_chan: Box<Any+Send>, // opaque reference to a LayoutChannel
}

/// Messages sent from the constellation to the script task
pub enum ConstellationControlMsg {
    /// Loads a new URL on the specified pipeline.
    LoadMsg(PipelineId, Url),
    /// Gives a channel and ID to a layout task, as well as the ID of that layout's parent
    AttachLayoutMsg(NewLayoutInfo),
    /// Window resized.  Sends a DOM event eventually, but first we combine events.
    ResizeMsg(PipelineId, WindowSizeData),
    /// Notifies script that window has been resized but to not take immediate action.
    ResizeInactiveMsg(PipelineId, WindowSizeData),
    /// Notifies the script that a pipeline should be closed.
    ExitPipelineMsg(PipelineId),
    /// Sends a DOM event.
    SendEventMsg(PipelineId, CompositorEvent),
    /// Notifies script that reflow is finished.
    ReflowCompleteMsg(PipelineId, uint),
}

/// Events from the compositor that the script task needs to know about
pub enum CompositorEvent {
    ResizeEvent(WindowSizeData),
    ReflowEvent,
    ClickEvent(uint, Point2D<f32>),
    MouseDownEvent(uint, Point2D<f32>),
    MouseUpEvent(uint, Point2D<f32>),
    MouseMoveEvent(Point2D<f32>)
}

/// An opaque wrapper around script<->layout channels to avoid leaking message types into
/// crates that don't need to know about them.
pub struct OpaqueScriptLayoutChannel(pub (Box<Any+Send>, Box<Any+Send>));

/// Encapsulates external communication with the script task.
#[deriving(Clone)]
pub struct ScriptControlChan(pub Sender<ConstellationControlMsg>);

impl<S: Encoder<E>, E> Encodable<S, E> for ScriptControlChan {
    fn encode(&self, _s: &mut S) -> Result<(), E> {
        Ok(())
    }
}

pub trait ScriptTaskFactory {
    fn create<C: ScriptListener + Send>(_phantom: Option<&mut Self>,
                                        id: PipelineId,
                                        compositor: Box<C>,
                                        layout_chan: &OpaqueScriptLayoutChannel,
                                        control_chan: ScriptControlChan,
                                        control_port: Receiver<ConstellationControlMsg>,
                                        constellation_msg: ConstellationChan,
                                        failure_msg: Failure,
                                        resource_task: ResourceTask,
                                        image_cache_task: ImageCacheTask,
                                        window_size: WindowSizeData);
    fn create_layout_channel(_phantom: Option<&mut Self>) -> OpaqueScriptLayoutChannel;
    fn clone_layout_channel(_phantom: Option<&mut Self>, pair: &OpaqueScriptLayoutChannel) -> Box<Any+Send>;
}
