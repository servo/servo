/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::rpc::LayoutRPC;
use crate::{OpaqueStyleAndLayoutData, PendingImage, TrustedNodeAddress};
use app_units::Au;
use crossbeam_channel::{Receiver, Sender};
use euclid::default::{Point2D, Rect};
use gfx_traits::Epoch;
use ipc_channel::ipc::{IpcReceiver, IpcSender};
use metrics::PaintTimeMetrics;
use msg::constellation_msg::{BackgroundHangMonitorRegister, PipelineId};
use net_traits::image_cache::ImageCache;
use profile_traits::mem::ReportsChan;
use script_traits::Painter;
use script_traits::{ConstellationControlMsg, LayoutControlMsg, LayoutMsg as ConstellationMsg};
use script_traits::{ScrollState, UntrustedNodeAddress, WindowSizeData};
use servo_arc::Arc as ServoArc;
use servo_atoms::Atom;
use servo_url::ServoUrl;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use style::context::QuirksMode;
use style::dom::OpaqueNode;
use style::properties::PropertyId;
use style::selector_parser::PseudoElement;
use style::stylesheets::Stylesheet;

/// Asynchronous messages that script can send to layout.
pub enum Msg {
    /// Adds the given stylesheet to the document. The second stylesheet is the
    /// insertion point (if it exists, the sheet needs to be inserted before
    /// it).
    AddStylesheet(ServoArc<Stylesheet>, Option<ServoArc<Stylesheet>>),

    /// Removes a stylesheet from the document.
    RemoveStylesheet(ServoArc<Stylesheet>),

    /// Change the quirks mode.
    SetQuirksMode(QuirksMode),

    /// Requests a reflow.
    Reflow(ScriptReflow),

    /// Get an RPC interface.
    GetRPC(Sender<Box<dyn LayoutRPC + Send>>),

    /// Requests that the layout thread render the next frame of all animations.
    TickAnimations,

    /// Updates layout's timer for animation testing from script.
    ///
    /// The inner field is the number of *milliseconds* to advance, and the bool
    /// field is whether animations should be force-ticked.
    AdvanceClockMs(i32, bool),

    /// Destroys layout data associated with a DOM node.
    ///
    /// TODO(pcwalton): Maybe think about batching to avoid message traffic.
    ReapStyleAndLayoutData(OpaqueStyleAndLayoutData),

    /// Requests that the layout thread measure its memory usage. The resulting reports are sent back
    /// via the supplied channel.
    CollectReports(ReportsChan),

    /// Requests that the layout thread enter a quiescent state in which no more messages are
    /// accepted except `ExitMsg`. A response message will be sent on the supplied channel when
    /// this happens.
    PrepareToExit(Sender<()>),

    /// Requests that the layout thread immediately shut down. There must be no more nodes left after
    /// this, or layout will crash.
    ExitNow,

    /// Get the last epoch counter for this layout thread.
    GetCurrentEpoch(IpcSender<Epoch>),

    /// Asks the layout thread whether any Web fonts have yet to load (if true, loads are pending;
    /// false otherwise).
    GetWebFontLoadState(IpcSender<bool>),

    /// Creates a new layout thread.
    ///
    /// This basically exists to keep the script-layout dependency one-way.
    CreateLayoutThread(LayoutThreadInit),

    /// Set the final Url.
    SetFinalUrl(ServoUrl),

    /// Tells layout about the new scrolling offsets of each scrollable stacking context.
    SetScrollStates(Vec<ScrollState>),

    /// Tells layout about a single new scrolling offset from the script. The rest will
    /// remain untouched and layout won't forward this back to script.
    UpdateScrollStateFromScript(ScrollState),

    /// Tells layout that script has added some paint worklet modules.
    RegisterPaint(Atom, Vec<Atom>, Box<dyn Painter>),

    /// Send to layout the precise time when the navigation started.
    SetNavigationStart(u64),

    /// Request the current number of animations that are running.
    GetRunningAnimations(IpcSender<usize>),
}

#[derive(Debug, PartialEq)]
pub enum NodesFromPointQueryType {
    All,
    Topmost,
}

#[derive(Debug, PartialEq)]
pub enum QueryMsg {
    ContentBoxQuery(OpaqueNode),
    ContentBoxesQuery(OpaqueNode),
    NodeGeometryQuery(OpaqueNode),
    NodeScrollGeometryQuery(OpaqueNode),
    OffsetParentQuery(OpaqueNode),
    TextIndexQuery(OpaqueNode, Point2D<f32>),
    NodesFromPointQuery(Point2D<f32>, NodesFromPointQueryType),

    // FIXME(nox): The following queries use the TrustedNodeAddress to
    // access actual DOM nodes, but those values can be constructed from
    // garbage values such as `0xdeadbeef as *const _`, this is unsound.
    NodeScrollIdQuery(TrustedNodeAddress),
    ResolvedStyleQuery(TrustedNodeAddress, Option<PseudoElement>, PropertyId),
    StyleQuery(TrustedNodeAddress),
    ElementInnerTextQuery(TrustedNodeAddress),
}

/// Any query to perform with this reflow.
#[derive(Debug, PartialEq)]
pub enum ReflowGoal {
    Full,
    TickAnimations,
    LayoutQuery(QueryMsg, u64),
}

impl ReflowGoal {
    /// Returns true if the given ReflowQuery needs a full, up-to-date display list to
    /// be present or false if it only needs stacking-relative positions.
    pub fn needs_display_list(&self) -> bool {
        match *self {
            ReflowGoal::Full | ReflowGoal::TickAnimations => true,
            ReflowGoal::LayoutQuery(ref querymsg, _) => match *querymsg {
                QueryMsg::NodesFromPointQuery(..) |
                QueryMsg::TextIndexQuery(..) |
                QueryMsg::ElementInnerTextQuery(_) => true,
                QueryMsg::ContentBoxQuery(_) |
                QueryMsg::ContentBoxesQuery(_) |
                QueryMsg::NodeGeometryQuery(_) |
                QueryMsg::NodeScrollGeometryQuery(_) |
                QueryMsg::NodeScrollIdQuery(_) |
                QueryMsg::ResolvedStyleQuery(..) |
                QueryMsg::OffsetParentQuery(_) |
                QueryMsg::StyleQuery(_) => false,
            },
        }
    }

    /// Returns true if the given ReflowQuery needs its display list send to WebRender or
    /// false if a layout_thread display list is sufficient.
    pub fn needs_display(&self) -> bool {
        match *self {
            ReflowGoal::Full | ReflowGoal::TickAnimations => true,
            ReflowGoal::LayoutQuery(ref querymsg, _) => match *querymsg {
                QueryMsg::NodesFromPointQuery(..) |
                QueryMsg::TextIndexQuery(..) |
                QueryMsg::ElementInnerTextQuery(_) => true,
                QueryMsg::ContentBoxQuery(_) |
                QueryMsg::ContentBoxesQuery(_) |
                QueryMsg::NodeGeometryQuery(_) |
                QueryMsg::NodeScrollGeometryQuery(_) |
                QueryMsg::NodeScrollIdQuery(_) |
                QueryMsg::ResolvedStyleQuery(..) |
                QueryMsg::OffsetParentQuery(_) |
                QueryMsg::StyleQuery(_) => false,
            },
        }
    }
}

/// Information needed for a reflow.
pub struct Reflow {
    ///  A clipping rectangle for the page, an enlarged rectangle containing the viewport.
    pub page_clip_rect: Rect<Au>,
}

/// Information derived from a layout pass that needs to be returned to the script thread.
#[derive(Default)]
pub struct ReflowComplete {
    /// The list of images that were encountered that are in progress.
    pub pending_images: Vec<PendingImage>,
    /// The list of nodes that initiated a CSS transition.
    pub newly_transitioning_nodes: Vec<UntrustedNodeAddress>,
}

/// Information needed for a script-initiated reflow.
pub struct ScriptReflow {
    /// General reflow data.
    pub reflow_info: Reflow,
    /// The document node.
    pub document: TrustedNodeAddress,
    /// Whether the document's stylesheets have changed since the last script reflow.
    pub stylesheets_changed: bool,
    /// The current window size.
    pub window_size: WindowSizeData,
    /// The channel that we send a notification to.
    pub script_join_chan: Sender<ReflowComplete>,
    /// The goal of this reflow.
    pub reflow_goal: ReflowGoal,
    /// The number of objects in the dom #10110
    pub dom_count: u32,
}

pub struct LayoutThreadInit {
    pub id: PipelineId,
    pub url: ServoUrl,
    pub is_parent: bool,
    pub layout_pair: (Sender<Msg>, Receiver<Msg>),
    pub pipeline_port: IpcReceiver<LayoutControlMsg>,
    pub background_hang_monitor_register: Box<dyn BackgroundHangMonitorRegister>,
    pub constellation_chan: IpcSender<ConstellationMsg>,
    pub script_chan: IpcSender<ConstellationControlMsg>,
    pub image_cache: Arc<dyn ImageCache>,
    pub paint_time_metrics: PaintTimeMetrics,
    pub layout_is_busy: Arc<AtomicBool>,
}
