/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! This module contains traits in script used generically in the rest of Servo.
//! The traits are here instead of in script so that these modules won't have
//! to depend on script.

#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]
#![deny(missing_docs)]

extern crate devtools_traits;
extern crate euclid;
extern crate ipc_channel;
extern crate libc;
extern crate msg;
extern crate net_traits;
extern crate profile_traits;
extern crate serde;
extern crate util;
extern crate url;

use devtools_traits::ScriptToDevtoolsControlMsg;
use ipc_channel::ipc::{IpcReceiver, IpcSender};
use libc::c_void;
use msg::compositor_msg::{Epoch, LayerId, ScriptToCompositorMsg};
use msg::constellation_msg::{ConstellationChan, PipelineId, Failure, WindowSizeData};
use msg::constellation_msg::{LoadData, SubpageId, Key, KeyState, KeyModifiers};
use msg::constellation_msg::{MozBrowserEvent, PipelineExitType};
use msg::webdriver_msg::WebDriverScriptCommand;
use net_traits::ResourceTask;
use net_traits::image_cache_task::ImageCacheTask;
use net_traits::storage_task::StorageTask;
use profile_traits::mem;
use std::any::Any;
use std::sync::mpsc::{Receiver, Sender};
use url::Url;
use util::geometry::Au;

use euclid::point::Point2D;
use euclid::rect::Rect;

/// The address of a node. Layout sends these back. They must be validated via
/// `from_untrusted_node_address` before they can be used, because we do not trust layout.
#[allow(raw_pointer_derive)]
#[derive(Copy, Clone, Debug)]
pub struct UntrustedNodeAddress(pub *const c_void);
unsafe impl Send for UntrustedNodeAddress {}

/// Messages sent to the layout task from the constellation and/or compositor.
#[derive(Deserialize, Serialize)]
pub enum LayoutControlMsg {
    /// Requests that this layout task exit.
    ExitNow(PipelineExitType),
    /// Requests the current epoch (layout counter) from this layout.
    GetCurrentEpoch(IpcSender<Epoch>),
    /// Asks layout to run another step in its animation.
    TickAnimations,
    /// Informs layout as to which regions of the page are visible.
    SetVisibleRects(Vec<(LayerId, Rect<Au>)>),
}

/// The initial data associated with a newly-created framed pipeline.
pub struct NewLayoutInfo {
    /// Id of the parent of this new pipeline.
    pub containing_pipeline_id: PipelineId,
    /// Id of the newly-created pipeline.
    pub new_pipeline_id: PipelineId,
    /// Id of the new frame associated with this pipeline.
    pub subpage_id: SubpageId,
    /// Network request data which will be initiated by the script task.
    pub load_data: LoadData,
    /// The paint channel, cast to `Box<Any>`.
    ///
    /// TODO(pcwalton): When we convert this to use IPC, this will need to become an
    /// `IpcAnySender`.
    pub paint_chan: Box<Any + Send>,
    /// Information on what to do on task failure.
    pub failure: Failure,
    /// A port on which layout can receive messages from the pipeline.
    pub pipeline_port: IpcReceiver<LayoutControlMsg>,
    /// A shutdown channel so that layout can notify others when it's done.
    pub layout_shutdown_chan: Sender<()>,
}

/// `StylesheetLoadResponder` is used to notify a responder that a style sheet
/// has loaded.
pub trait StylesheetLoadResponder {
    /// Respond to a loaded style sheet.
    fn respond(self: Box<Self>);
}

/// Used to determine if a script has any pending asynchronous activity.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ScriptState {
    /// The document has been loaded.
    DocumentLoaded,
    /// The document is still loading.
    DocumentLoading,
}

/// Messages sent from the constellation to the script task
pub enum ConstellationControlMsg {
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
    ReflowComplete(PipelineId, u32),
    /// Notifies script of the viewport.
    Viewport(PipelineId, Rect<f32>),
    /// Requests that the script task immediately send the constellation the title of a pipeline.
    GetTitle(PipelineId),
    /// Notifies script task to suspend all its timers
    Freeze(PipelineId),
    /// Notifies script task to resume all its timers
    Thaw(PipelineId),
    /// Notifies script task that a url should be loaded in this iframe.
    Navigate(PipelineId, SubpageId, LoadData),
    /// Requests the script task forward a mozbrowser event to an iframe it owns
    MozBrowserEvent(PipelineId, SubpageId, MozBrowserEvent),
    /// Updates the current subpage id of a given iframe
    UpdateSubpageId(PipelineId, SubpageId, SubpageId),
    /// Set an iframe to be focused. Used when an element in an iframe gains focus.
    FocusIFrame(PipelineId, SubpageId),
    /// Passes a webdriver command to the script task for execution
    WebDriverScriptCommand(PipelineId, WebDriverScriptCommand),
    /// Notifies script task that all animations are done
    TickAllAnimations(PipelineId),
    /// Notifies script that a stylesheet has finished loading.
    StylesheetLoadComplete(PipelineId, Url, Box<StylesheetLoadResponder+Send>),
    /// Get the current state of the script task for a given pipeline.
    GetCurrentState(Sender<ScriptState>, PipelineId),
}

/// The mouse button involved in the event.
#[derive(Clone, Debug)]
pub enum MouseButton {
    /// The left mouse button.
    Left,
    /// The middle mouse button.
    Middle,
    /// The right mouse button.
    Right,
}

/// Events from the compositor that the script task needs to know about
pub enum CompositorEvent {
    /// The window was resized.
    ResizeEvent(WindowSizeData),
    /// A point was clicked.
    ClickEvent(MouseButton, Point2D<f32>),
    /// A mouse button was pressed on a point.
    MouseDownEvent(MouseButton, Point2D<f32>),
    /// A mouse button was released on a point.
    MouseUpEvent(MouseButton, Point2D<f32>),
    /// The mouse was moved over a point.
    MouseMoveEvent(Point2D<f32>),
    /// A key was pressed.
    KeyEvent(Key, KeyState, KeyModifiers),
}

/// An opaque wrapper around script<->layout channels to avoid leaking message types into
/// crates that don't need to know about them.
pub struct OpaqueScriptLayoutChannel(pub (Box<Any+Send>, Box<Any+Send>));

/// This trait allows creating a `ScriptTask` without depending on the `script`
/// crate.
pub trait ScriptTaskFactory {
    /// Create a `ScriptTask`.
    fn create(_phantom: Option<&mut Self>,
              id: PipelineId,
              parent_info: Option<(PipelineId, SubpageId)>,
              compositor: IpcSender<ScriptToCompositorMsg>,
              layout_chan: &OpaqueScriptLayoutChannel,
              control_chan: Sender<ConstellationControlMsg>,
              control_port: Receiver<ConstellationControlMsg>,
              constellation_msg: ConstellationChan,
              failure_msg: Failure,
              resource_task: ResourceTask,
              storage_task: StorageTask,
              image_cache_task: ImageCacheTask,
              mem_profiler_chan: mem::ProfilerChan,
              devtools_chan: Option<IpcSender<ScriptToDevtoolsControlMsg>>,
              window_size: Option<WindowSizeData>,
              load_data: LoadData);
    /// Create a script -> layout channel (`Sender`, `Receiver` pair).
    fn create_layout_channel(_phantom: Option<&mut Self>) -> OpaqueScriptLayoutChannel;
    /// Clone the `Sender` in `pair`.
    fn clone_layout_channel(_phantom: Option<&mut Self>, pair: &OpaqueScriptLayoutChannel)
                            -> Box<Any+Send>;
}
