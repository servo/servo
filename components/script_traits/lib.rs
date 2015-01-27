/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

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
use libc::{c_int, c_void};
use servo_msg::compositor_msg::ScriptToCompositorMsg;
use servo_msg::constellation_msg::{ConstellationChan, PipelineId, Failure, WindowSizeData};
use servo_msg::constellation_msg::{LoadData, SubpageId, Key, KeyState, KeyModifiers};
use servo_msg::constellation_msg::PipelineExitType;
use servo_net::image_cache_task::ImageCacheTask;
use servo_net::resource_task::ResourceTask;
use servo_net::server::SharedServerProxy;
use servo_net::storage_task::StorageTask;
use servo_util::ipc::{IpcReceiver, IpcSender};
use std::any::Any;

use geom::point::Point2D;
use geom::rect::Rect;

use serialize::{Decodable, Decoder, Encodable, Encoder};

/// The address of a node. Layout sends these back. They must be validated via
/// `from_untrusted_node_address` before they can be used, because we do not trust layout.
#[allow(raw_pointer_deriving)]
#[deriving(Copy, Clone, Show)]
pub struct UntrustedNodeAddress(pub *const c_void);

impl UntrustedNodeAddress {
    #[inline]
    pub fn to_uint(&self) -> uint {
        let UntrustedNodeAddress(ptr) = *self;
        ptr as uint
    }
}

impl<E,S:Encoder<E>> Encodable<S,E> for UntrustedNodeAddress {
    fn encode(&self, encoder: &mut S) -> Result<(),E> {
        let UntrustedNodeAddress(ptr) = *self;
        (ptr as uint).encode(encoder)
    }
}

impl<E,D:Decoder<E>> Decodable<D,E> for UntrustedNodeAddress {
    fn decode(decoder: &mut D) -> Result<UntrustedNodeAddress,E> {
        let ptr: uint = try!(Decodable::decode(decoder));
        Ok(UntrustedNodeAddress(ptr as *const c_void))
    }
}


#[deriving(Encodable, Decodable)]
pub struct NewLayoutInfo {
    pub old_pipeline_id: PipelineId,
    pub new_pipeline_id: PipelineId,
    pub subpage_id: SubpageId,
    // FIXME(pcwalton): Terrible, pass an FD instead.
    pub layout_chan: (uint, uint),
}

/// Messages sent from the constellation to the script task
#[deriving(Encodable, Decodable)]
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

/// Events from the compositor that the script task needs to know about
#[deriving(Encodable, Decodable)]
pub enum CompositorEvent {
    ResizeEvent(WindowSizeData),
    ReflowEvent(Vec<UntrustedNodeAddress>),
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
pub struct ScriptControlChan(pub IpcSender<ConstellationControlMsg>);

impl ScriptControlChan {
    #[inline]
    pub fn fd(&self) -> c_int {
        let ScriptControlChan(ref ipc_channel) = *self;
        ipc_channel.fd()
    }
}

pub trait ScriptTaskFactory {
    fn create(_phantom: Option<&mut Self>,
              id: PipelineId,
              compositor: SharedServerProxy<ScriptToCompositorMsg,()>,
              layout_chan: &OpaqueScriptLayoutChannel,
              contellation_to_script_receiver: IpcReceiver<ConstellationControlMsg>,
              constellation_msg: ConstellationChan,
              failure_msg: Failure,
              layout_to_script_receiver: Receiver<ConstellationControlMsg>,
              resource_task: ResourceTask,
              storage_task: StorageTask,
              image_cache_task: ImageCacheTask,
              devtools_chan: Option<DevtoolsControlChan>,
              window_size: WindowSizeData);
    fn create_layout_channel(_phantom: Option<&mut Self>) -> OpaqueScriptLayoutChannel;
    fn clone_layout_channel(_phantom: Option<&mut Self>, pair: &OpaqueScriptLayoutChannel)
                            -> Box<Any+Send>;
}
