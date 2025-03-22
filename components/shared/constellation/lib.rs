/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The interface to the `Constellation`, which prevents other crates from depending directly on
//! the `constellation` crate itself. In addition to all messages to the `Constellation`, this
//! crate is responsible for defining types that cross the process boundary from the
//! embedding/rendering layer all the way to script, thus it should have very minimal dependencies
//! on other parts of Servo.

use std::collections::HashMap;
use std::ffi::c_void;
use std::fmt;
use std::time::Duration;

use base::Epoch;
use base::cross_process_instant::CrossProcessInstant;
use base::id::{PipelineId, ScrollTreeNodeId, WebViewId};
use bitflags::bitflags;
use embedder_traits::{Cursor, InputEvent, MediaSessionActionType, Theme, WebDriverCommandMsg};
use euclid::{Scale, Size2D, Vector2D};
use ipc_channel::ipc::IpcSender;
use malloc_size_of::malloc_size_of_is_0;
use malloc_size_of_derive::MallocSizeOf;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use servo_url::ServoUrl;
use strum_macros::IntoStaticStr;
use style_traits::CSSPixel;
use webrender_api::ExternalScrollId;
use webrender_api::units::{DevicePixel, LayoutPixel};

/// Messages to the constellation.
#[derive(IntoStaticStr)]
pub enum ConstellationMsg {
    /// Exit the constellation.
    Exit,
    /// Request that the constellation send the current focused top-level browsing context id,
    /// over a provided channel.
    GetFocusTopLevelBrowsingContext(IpcSender<Option<WebViewId>>),
    /// Query the constellation to see if the current compositor output is stable
    IsReadyToSaveImage(HashMap<PipelineId, Epoch>),
    /// Whether to allow script to navigate.
    AllowNavigationResponse(PipelineId, bool),
    /// Request to load a page.
    LoadUrl(WebViewId, ServoUrl),
    /// Clear the network cache.
    ClearCache,
    /// Request to traverse the joint session history of the provided browsing context.
    TraverseHistory(WebViewId, TraversalDirection),
    /// Inform the constellation of a window being resized.
    WindowSize(WebViewId, WindowSizeData, WindowSizeType),
    /// Inform the constellation of a theme change.
    ThemeChange(Theme),
    /// Requests that the constellation instruct layout to begin a new tick of the animation.
    TickAnimation(PipelineId, AnimationTickType),
    /// Dispatch a webdriver command
    WebDriverCommand(WebDriverCommandMsg),
    /// Reload a top-level browsing context.
    Reload(WebViewId),
    /// A log entry, with the top-level browsing context id and thread name
    LogEntry(Option<WebViewId>, Option<String>, LogEntry),
    /// Create a new top level browsing context.
    NewWebView(ServoUrl, WebViewId),
    /// Close a top level browsing context.
    CloseWebView(WebViewId),
    /// Panic a top level browsing context.
    SendError(Option<WebViewId>, String),
    /// Make a webview focused.
    FocusWebView(WebViewId),
    /// Make none of the webviews focused.
    BlurWebView,
    /// Forward an input event to an appropriate ScriptTask.
    ForwardInputEvent(WebViewId, InputEvent, Option<CompositorHitTestResult>),
    /// Requesting a change to the onscreen cursor.
    SetCursor(WebViewId, Cursor),
    /// Enable the sampling profiler, with a given sampling rate and max total sampling duration.
    ToggleProfiler(Duration, Duration),
    /// Request to exit from fullscreen mode
    ExitFullScreen(WebViewId),
    /// Media session action.
    MediaSessionAction(MediaSessionActionType),
    /// Set whether to use less resources, by stopping animations and running timers at a heavily limited rate.
    SetWebViewThrottled(WebViewId, bool),
    /// The Servo renderer scrolled and is updating the scroll states of the nodes in the
    /// given pipeline via the constellation.
    SetScrollStates(PipelineId, Vec<ScrollState>),
    /// Notify the constellation that a particular paint metric event has happened for the given pipeline.
    PaintMetric(PipelineId, PaintMetricEvent),
}

/// A description of a paint metric that is sent from the Servo renderer to the
/// constellation.
pub enum PaintMetricEvent {
    FirstPaint(CrossProcessInstant, bool /* first_reflow */),
    FirstContentfulPaint(CrossProcessInstant, bool /* first_reflow */),
}

impl fmt::Debug for ConstellationMsg {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let variant_string: &'static str = self.into();
        write!(formatter, "ConstellationMsg::{variant_string}")
    }
}

/// A log entry reported to the constellation
/// We don't report all log entries, just serious ones.
/// We need a separate type for this because `LogLevel` isn't serializable.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum LogEntry {
    /// Panic, with a reason and backtrace
    Panic(String, String),
    /// Error, with a reason
    Error(String),
    /// warning, with a reason
    Warn(String),
}

/// Data about the window size.
#[derive(Clone, Copy, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub struct WindowSizeData {
    /// The size of the initial layout viewport, before parsing an
    /// <http://www.w3.org/TR/css-device-adapt/#initial-viewport>
    pub initial_viewport: Size2D<f32, CSSPixel>,

    /// The resolution of the window in dppx, not including any "pinch zoom" factor.
    pub device_pixel_ratio: Scale<f32, CSSPixel, DevicePixel>,
}

/// The type of window size change.
#[derive(Clone, Copy, Debug, Deserialize, Eq, MallocSizeOf, PartialEq, Serialize)]
pub enum WindowSizeType {
    /// Initial load.
    Initial,
    /// Window resize.
    Resize,
}

bitflags! {
    #[derive(Debug, Default, Deserialize, Serialize)]
    /// Specifies if rAF should be triggered and/or CSS Animations and Transitions.
    pub struct AnimationTickType: u8 {
        /// Trigger a call to requestAnimationFrame.
        const REQUEST_ANIMATION_FRAME = 0b001;
        /// Trigger restyles for CSS Animations and Transitions.
        const CSS_ANIMATIONS_AND_TRANSITIONS = 0b010;
    }
}

/// The result of a hit test in the compositor.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CompositorHitTestResult {
    /// The pipeline id of the resulting item.
    pub pipeline_id: PipelineId,

    /// The hit test point in the item's viewport.
    pub point_in_viewport: euclid::default::Point2D<f32>,

    /// The hit test point relative to the item itself.
    pub point_relative_to_item: euclid::default::Point2D<f32>,

    /// The node address of the hit test result.
    pub node: UntrustedNodeAddress,

    /// The cursor that should be used when hovering the item hit by the hit test.
    pub cursor: Option<Cursor>,

    /// The scroll tree node associated with this hit test item.
    pub scroll_tree_node: ScrollTreeNodeId,
}

/// The scroll state of a stacking context.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct ScrollState {
    /// The ID of the scroll root.
    pub scroll_id: ExternalScrollId,
    /// The scrolling offset of this stacking context.
    pub scroll_offset: Vector2D<f32, LayoutPixel>,
}

/// The address of a node. Layout sends these back. They must be validated via
/// `from_untrusted_node_address` before they can be used, because we do not trust layout.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UntrustedNodeAddress(pub *const c_void);

malloc_size_of_is_0!(UntrustedNodeAddress);

#[allow(unsafe_code)]
unsafe impl Send for UntrustedNodeAddress {}

impl From<style_traits::dom::OpaqueNode> for UntrustedNodeAddress {
    fn from(o: style_traits::dom::OpaqueNode) -> Self {
        UntrustedNodeAddress(o.0 as *const c_void)
    }
}

impl Serialize for UntrustedNodeAddress {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        (self.0 as usize).serialize(s)
    }
}

impl<'de> Deserialize<'de> for UntrustedNodeAddress {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<UntrustedNodeAddress, D::Error> {
        let value: usize = Deserialize::deserialize(d)?;
        Ok(UntrustedNodeAddress::from_id(value))
    }
}

impl UntrustedNodeAddress {
    /// Creates an `UntrustedNodeAddress` from the given pointer address value.
    #[inline]
    pub fn from_id(id: usize) -> UntrustedNodeAddress {
        UntrustedNodeAddress(id as *const c_void)
    }
}

/// The direction of a history traversal
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum TraversalDirection {
    /// Travel forward the given number of documents.
    Forward(usize),
    /// Travel backward the given number of documents.
    Back(usize),
}
