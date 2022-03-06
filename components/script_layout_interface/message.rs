/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::rpc::LayoutRPC;
use crossbeam_channel::{Receiver, Sender};
use gfx_traits::Epoch;
use ipc_channel::ipc::{IpcReceiver, IpcSender};
use metrics::PaintTimeMetrics;
use msg::constellation_msg::{BackgroundHangMonitorRegister, PipelineId};
use net_traits::image_cache::ImageCache;
use profile_traits::mem::ReportsChan;
use script_traits::Painter;
use script_traits::{
    ConstellationControlMsg, LayoutControlMsg, LayoutMsg as ConstellationMsg, ScrollState,
    WindowSizeData,
};
use servo_arc::Arc as ServoArc;
use servo_atoms::Atom;
use servo_url::ServoUrl;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use style::context::QuirksMode;
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

    /*/// Requests a reflow.
    Reflow(ScriptReflow),*/

    /// Get an RPC interface.
    GetRPC(Sender<Box<dyn LayoutRPC + Send>>),

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
    pub window_size: WindowSizeData,
}
