/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![comment = "The Servo Parallel Browser Project"]
#![license = "MPL"]

#![deny(unused_imports)]
#![deny(unused_variables)]

extern crate devtools_traits;
extern crate geom;
extern crate libc;
extern crate "msg" as servo_msg;
extern crate "net" as servo_net;
extern crate "util" as servo_util;
extern crate url;
extern crate serialize;

// This module contains traits in script used generically
//   in the rest of Servo.
// The traits are here instead of in script so
//   that these modules won't have to depend on script.

use devtools_traits::DevtoolsControlChan;
use libc::c_void;
use servo_msg::constellation_msg::{ConstellationChan, PipelineId, Failure, WindowSizeData};
use servo_msg::constellation_msg::{LoadData, SubpageId, Key, KeyState, KeyModifiers};
use servo_msg::compositor_msg::{ScriptToCompositorThreadProxy, ScriptToMainThreadProxy};
use servo_net::image_cache_task::ImageCacheTask;
use servo_net::resource_task::ResourceTask;
use servo_net::storage_task::StorageTask;
use servo_util::smallvec::SmallVec1;
use std::any::Any;

use geom::point::Point2D;
use geom::rect::Rect;

use serialize::{Encodable, Encoder};

/// The address of a node. Layout sends these back. They must be validated via
/// `from_untrusted_node_address` before they can be used, because we do not trust layout.
pub type UntrustedNodeAddress = *const c_void;

pub struct NewLayoutInfo {
    pub old_pipeline_id: PipelineId,
    pub new_pipeline_id: PipelineId,
    pub subpage_id: SubpageId,
    pub layout_chan: Box<Any+Send>, // opaque reference to a LayoutChannel
}

/// Messages sent from the constellation to the script task
pub enum ConstellationControlMsg {
    /// Loads a new URL on the specified pipeline.
    LoadMsg(PipelineId, LoadData),
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
    ViewportMsg(PipelineId, Rect<f32>),
}

/// Events from the compositor that the script task needs to know about
pub enum CompositorEvent {
    ResizeEvent(WindowSizeData),
    ReflowEvent(SmallVec1<UntrustedNodeAddress>),
    ClickEvent(uint, Point2D<f32>),
    MouseDownEvent(uint, Point2D<f32>),
    MouseUpEvent(uint, Point2D<f32>),
    MouseMoveEvent(Point2D<f32>),
    KeyEvent(Key, KeyState, KeyModifiers),
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

/// Data needed to construct a script thread.
pub struct InitialScriptState<C> {
    /// The ID of the pipeline with which this script thread is associated.
    pub id: PipelineId,
    /// A proxy to the main thread.
    pub main_thread_proxy: Box<ScriptToMainThreadProxy + Send>,
    /// The compositor. If `None`, this is a headless script thread.
    pub compositor: Option<C>,
    /// A channel with which messages can be sent to us (the script task).
    pub control_chan: ScriptControlChan,
    /// A port on which messages sent by the constellation to script can be received.
    pub control_port: Receiver<ConstellationControlMsg>,
    /// A channel on which messages can be sent to the constellation from script.
    pub constellation_proxy: ConstellationChan,
    /// Information that script sends out when it panics.
    pub failure_info: Failure,
    /// A channel to the resource manager task.
    pub resource_task: ResourceTask,
    /// A channel to the storage task.
    pub storage_task: StorageTask,
    /// A channel to the image cache task.
    pub image_cache_task: ImageCacheTask,
    /// A channel to the developer tools, if applicable.
    pub devtools_chan: Option<DevtoolsControlChan>,
    /// Information about the initial window size.
    pub window_size: WindowSizeData,
}

pub trait ScriptTaskFactory {
    fn create<C>(_phantom: Option<&mut Self>,
                 state: InitialScriptState<C>,
                 layout_chan: &OpaqueScriptLayoutChannel)
                 where C: ScriptToCompositorThreadProxy + Send;
    fn create_layout_channel(_phantom: Option<&mut Self>) -> OpaqueScriptLayoutChannel;
    fn clone_layout_channel(_phantom: Option<&mut Self>, pair: &OpaqueScriptLayoutChannel)
                            -> Box<Any+Send>;
}
