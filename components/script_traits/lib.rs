/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(int_uint)]

#![deny(unused_imports)]
#![deny(unused_variables)]
#![allow(missing_copy_implementations)]

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
use servo_msg::constellation_msg::PipelineExitType;
use servo_msg::compositor_msg::ScriptListener;
use servo_net::image_cache_task::ImageCacheTask;
use servo_net::resource_task::ResourceTask;
use servo_net::storage_task::StorageTask;
use servo_util::smallvec::SmallVec1;
use std::any::Any;
use std::sync::mpsc::{Sender, Receiver};

use geom::point::Point2D;
use geom::rect::Rect;

/// The address of a node. Layout sends these back. They must be validated via
/// `from_untrusted_node_address` before they can be used, because we do not trust layout.
#[allow(raw_pointer_deriving)]
#[derive(Copy, Clone)]
pub struct UntrustedNodeAddress(pub *const c_void);
unsafe impl Send for UntrustedNodeAddress {}

pub struct NewLayoutInfo {
    pub old_pipeline_id: PipelineId,
    pub new_pipeline_id: PipelineId,
    pub subpage_id: SubpageId,
    pub layout_chan: Box<Any+Send>, // opaque reference to a LayoutChannel
}

/// Messages sent from the constellation to the script task
pub enum ConstellationControlMsg {
    /// Loads a new URL on the specified pipeline.
    Load(PipelineId, LoadData),
    /// Gives a channel and ID to a layout task, as well as the ID of that layout's parent
    AttachLayout(NewLayoutInfo),
    /// Window resized.  Sends a DOM event eventually, but first we combine events.
    Resize(PipelineId, WindowSizeData),
    /// Notifies script that window has been resized but to not take immediate action.
    ResizeInactive(PipelineId, WindowSizeData),
    /// Notifies the script that a pipeline should be closed.
    ExitPipeline(PipelineId, PipelineExitType),
    /// Sends a DOM event.
    SendEvent(PipelineId, CompositorEvent),
    /// Notifies script that reflow is finished.
    ReflowComplete(PipelineId, uint),
    /// Notifies script of the viewport.
    Viewport(PipelineId, Rect<f32>),
    /// Requests that the script task immediately send the constellation the title of a pipeline.
    GetTitle(PipelineId),
}

unsafe impl Send for ConstellationControlMsg {
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
#[derive(Clone)]
pub struct ScriptControlChan(pub Sender<ConstellationControlMsg>);

pub trait ScriptTaskFactory {
    fn create<C>(_phantom: Option<&mut Self>,
                 id: PipelineId,
                 compositor: C,
                 layout_chan: &OpaqueScriptLayoutChannel,
                 control_chan: ScriptControlChan,
                 control_port: Receiver<ConstellationControlMsg>,
                 constellation_msg: ConstellationChan,
                 failure_msg: Failure,
                 resource_task: ResourceTask,
                 storage_task: StorageTask,
                 image_cache_task: ImageCacheTask,
                 devtools_chan: Option<DevtoolsControlChan>,
                 window_size: WindowSizeData)
                 where C: ScriptListener + Send;
    fn create_layout_channel(_phantom: Option<&mut Self>) -> OpaqueScriptLayoutChannel;
    fn clone_layout_channel(_phantom: Option<&mut Self>, pair: &OpaqueScriptLayoutChannel)
                            -> Box<Any+Send>;
}
