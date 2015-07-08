/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! This module contains traits in script used generically in the rest of Servo.
//! The traits are here instead of in script so that these modules won't have
//! to depend on script.

#![deny(missing_docs)]

extern crate devtools_traits;
extern crate euclid;
extern crate libc;
extern crate msg;
extern crate net_traits;
extern crate util;
extern crate url;

use devtools_traits::DevtoolsControlChan;
use libc::c_void;
use msg::constellation_msg::{ConstellationChan, PipelineId, Failure, WindowSizeData};
use msg::constellation_msg::{LoadData, SubpageId, Key, KeyState, KeyModifiers};
use msg::constellation_msg::{MozBrowserEvent, PipelineExitType};
use msg::compositor_msg::ScriptListener;
use msg::webdriver_msg::WebDriverScriptCommand;
use net_traits::ResourceTask;
use net_traits::image_cache_task::ImageCacheTask;
use net_traits::storage_task::StorageTask;
use std::any::Any;
use std::sync::mpsc::{Sender, Receiver};
use url::Url;

use euclid::point::Point2D;
use euclid::rect::Rect;

/// The address of a node. Layout sends these back. They must be validated via
/// `from_untrusted_node_address` before they can be used, because we do not trust layout.
#[allow(raw_pointer_derive)]
#[derive(Copy, Clone)]
pub struct UntrustedNodeAddress(pub *const c_void);
unsafe impl Send for UntrustedNodeAddress {}

/// The initial data associated with a newly-created framed pipeline.
pub struct NewLayoutInfo {
    /// Id of the parent of this new pipeline.
    pub containing_pipeline_id: PipelineId,
    /// Id of the newly-created pipeline.
    pub new_pipeline_id: PipelineId,
    /// Id of the new frame associated with this pipeline.
    pub subpage_id: SubpageId,
    /// Channel for communicating with this new pipeline's layout task.
    /// (This is a LayoutChannel.)
    pub layout_chan: Box<Any+Send>,
    /// Network request data which will be initiated by the script task.
    pub load_data: LoadData,
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

/// Encapsulates external communication with the script task.
#[derive(Clone)]
pub struct ScriptControlChan(pub Sender<ConstellationControlMsg>);

/// This trait allows creating a `ScriptTask` without depending on the `script`
/// crate.
pub trait ScriptTaskFactory {
    /// Create a `ScriptTask`.
    fn create<C>(_phantom: Option<&mut Self>,
                 id: PipelineId,
                 parent_info: Option<(PipelineId, SubpageId)>,
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
                 window_size: Option<WindowSizeData>,
                 load_data: LoadData)
                 where C: ScriptListener + Send;
    /// Create a script -> layout channel (`Sender`, `Receiver` pair).
    fn create_layout_channel(_phantom: Option<&mut Self>) -> OpaqueScriptLayoutChannel;
    /// Clone the `Sender` in `pair`.
    fn clone_layout_channel(_phantom: Option<&mut Self>, pair: &OpaqueScriptLayoutChannel)
                            -> Box<Any+Send>;
}
