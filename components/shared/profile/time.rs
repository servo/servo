/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base::cross_process_instant::CrossProcessInstant;
use ipc_channel::ipc::IpcSender;
use log::warn;
use serde::{Deserialize, Serialize};
use servo_config::opts;
use time_03::Duration;

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct TimerMetadata {
    pub url: String,
    pub iframe: TimerMetadataFrameType,
    pub incremental: TimerMetadataReflowType,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProfilerChan(pub IpcSender<ProfilerMsg>);

impl ProfilerChan {
    pub fn send(&self, msg: ProfilerMsg) {
        if let Err(e) = self.0.send(msg) {
            warn!("Error communicating with the time profiler thread: {}", e);
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ProfilerData {
    NoRecords,
    Record(Vec<Duration>),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ProfilerMsg {
    /// Normal message used for reporting time
    Time(
        (ProfilerCategory, Option<TimerMetadata>),
        (CrossProcessInstant, CrossProcessInstant),
    ),
    /// Message used to get time spend entries for a particular ProfilerBuckets (in nanoseconds)
    Get(
        (ProfilerCategory, Option<TimerMetadata>),
        IpcSender<ProfilerData>,
    ),
    /// Message used to force print the profiling metrics
    Print,

    /// Report a layout query that could not be processed immediately for a particular URL.
    BlockedLayoutQuery(String),

    /// Tells the profiler to shut down.
    Exit(IpcSender<()>),
}

/// Usage sites of variants marked “Rust tracing only” are not visible to rust-analyzer.
#[repr(u32)]
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum ProfilerCategory {
    /// The compositor is rasterising or presenting.
    ///
    /// Not associated with a specific URL.
    Compositing = 0x00,

    /// The script thread is doing layout work.
    LayoutPerform = 0x10,

    /// Events currently only used by Layout 2013.
    LayoutStyleRecalc = 0x11,
    LayoutTextShaping = 0x12,
    LayoutRestyleDamagePropagation = 0x13,
    LayoutGeneratedContent = 0x18,
    LayoutFloatPlacementSpeculation = 0x1a,
    LayoutMain = 0x1b,
    LayoutStoreOverflow = 0x1c,
    LayoutParallelWarmup = 0x1d,
    LayoutDispListBuild = 0x1e,

    ImageSaving = 0x51,
    ScriptAttachLayout = 0x60,
    ScriptConstellationMsg = 0x61,
    ScriptDevtoolsMsg = 0x62,
    ScriptDocumentEvent = 0x63,
    ScriptDomEvent = 0x64,

    /// Rust tracing only: the script thread is executing a script.
    /// This may include time doing layout or parse work initiated by the script.
    ScriptEvaluate = 0x65,

    ScriptEvent = 0x66,
    ScriptFileRead = 0x67,
    ScriptImageCacheMsg = 0x68,
    ScriptInputEvent = 0x69,
    ScriptNetworkEvent = 0x6a,

    /// The script thread is parsing HTML, rather than doing other work like evaluating scripts or doing layout.
    ScriptParseHTML = 0x6b,

    ScriptPlannedNavigation = 0x6c,
    ScriptResize = 0x6d,
    ScriptSetScrollState = 0x6e,
    ScriptSetViewport = 0x6f,
    ScriptTimerEvent = 0x70,
    ScriptStylesheetLoad = 0x71,
    ScriptUpdateReplacedElement = 0x72,
    ScriptWebSocketEvent = 0x73,
    ScriptWorkerEvent = 0x74,
    ScriptServiceWorkerEvent = 0x75,

    /// The script thread is parsing XML, rather than doing other work like evaluating scripts or doing layout.
    ScriptParseXML = 0x76,

    ScriptEnterFullscreen = 0x77,
    ScriptExitFullscreen = 0x78,
    ScriptWorkletEvent = 0x7a,
    ScriptPerformanceEvent = 0x7b,
    ScriptHistoryEvent = 0x7c,
    ScriptPortMessage = 0x7d,
    ScriptWebGPUMsg = 0x7e,

    /// Web performance metrics.
    TimeToFirstPaint = 0x80,
    TimeToFirstContentfulPaint = 0x81,
    TimeToInteractive = 0x82,

    IpcReceiver = 0x83,
    IpcBytesReceiver = 0x84,
}

impl ProfilerCategory {
    pub const fn variant_name(&self) -> &'static str {
        match self {
            ProfilerCategory::Compositing => "Compositing",
            ProfilerCategory::LayoutPerform => "LayoutPerform",
            ProfilerCategory::LayoutStyleRecalc => "LayoutStyleRecalc",
            ProfilerCategory::LayoutTextShaping => "LayoutTextShaping",
            ProfilerCategory::LayoutRestyleDamagePropagation => "LayoutRestyleDamagePropagation",
            ProfilerCategory::LayoutGeneratedContent => "LayoutGeneratedContent",
            ProfilerCategory::LayoutFloatPlacementSpeculation => "LayoutFloatPlacementSpeculation",
            ProfilerCategory::LayoutMain => "LayoutMain",
            ProfilerCategory::LayoutStoreOverflow => "LayoutStoreOverflow",
            ProfilerCategory::LayoutParallelWarmup => "LayoutParallelWarmup",
            ProfilerCategory::LayoutDispListBuild => "LayoutDispListBuild",
            ProfilerCategory::ImageSaving => "ImageSaving",
            ProfilerCategory::ScriptAttachLayout => "ScriptAttachLayout",
            ProfilerCategory::ScriptConstellationMsg => "ScriptConstellationMsg",
            ProfilerCategory::ScriptDevtoolsMsg => "ScriptDevtoolsMsg",
            ProfilerCategory::ScriptDocumentEvent => "ScriptDocumentEvent",
            ProfilerCategory::ScriptDomEvent => "ScriptDomEvent",
            ProfilerCategory::ScriptEvaluate => "ScriptEvaluate",
            ProfilerCategory::ScriptEvent => "ScriptEvent",
            ProfilerCategory::ScriptFileRead => "ScriptFileRead",
            ProfilerCategory::ScriptImageCacheMsg => "ScriptImageCacheMsg",
            ProfilerCategory::ScriptInputEvent => "ScriptInputEvent",
            ProfilerCategory::ScriptNetworkEvent => "ScriptNetworkEvent",
            ProfilerCategory::ScriptParseHTML => "ScriptParseHTML",
            ProfilerCategory::ScriptPlannedNavigation => "ScriptPlannedNavigation",
            ProfilerCategory::ScriptResize => "ScriptResize",
            ProfilerCategory::ScriptSetScrollState => "ScriptSetScrollState",
            ProfilerCategory::ScriptSetViewport => "ScriptSetViewport",
            ProfilerCategory::ScriptTimerEvent => "ScriptTimerEvent",
            ProfilerCategory::ScriptStylesheetLoad => "ScriptStylesheetLoad",
            ProfilerCategory::ScriptUpdateReplacedElement => "ScriptUpdateReplacedElement",
            ProfilerCategory::ScriptWebSocketEvent => "ScriptWebSocketEvent",
            ProfilerCategory::ScriptWorkerEvent => "ScriptWorkerEvent",
            ProfilerCategory::ScriptServiceWorkerEvent => "ScriptServiceWorkerEvent",
            ProfilerCategory::ScriptParseXML => "ScriptParseXML",
            ProfilerCategory::ScriptEnterFullscreen => "ScriptEnterFullscreen",
            ProfilerCategory::ScriptExitFullscreen => "ScriptExitFullscreen",
            ProfilerCategory::ScriptWorkletEvent => "ScriptWorkletEvent",
            ProfilerCategory::ScriptPerformanceEvent => "ScriptPerformanceEvent",
            ProfilerCategory::ScriptHistoryEvent => "ScriptHistoryEvent",
            ProfilerCategory::ScriptPortMessage => "ScriptPortMessage",
            ProfilerCategory::ScriptWebGPUMsg => "ScriptWebGPUMsg",
            ProfilerCategory::TimeToFirstPaint => "TimeToFirstPaint",
            ProfilerCategory::TimeToFirstContentfulPaint => "TimeToFirstContentfulPaint",
            ProfilerCategory::TimeToInteractive => "TimeToInteractive",
            ProfilerCategory::IpcReceiver => "IpcReceiver",
            ProfilerCategory::IpcBytesReceiver => "IpcBytesReceiver",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum TimerMetadataFrameType {
    RootWindow,
    IFrame,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum TimerMetadataReflowType {
    Incremental,
    FirstReflow,
}

#[cfg(feature = "tracing")]
pub type Span = tracing::Span;
#[cfg(not(feature = "tracing"))]
pub type Span = ();

pub fn profile<T, F>(
    category: ProfilerCategory,
    meta: Option<TimerMetadata>,
    profiler_chan: ProfilerChan,
    #[cfg(feature = "tracing")] span: Span,
    #[cfg(not(feature = "tracing"))] _span: Span,
    callback: F,
) -> T
where
    F: FnOnce() -> T,
{
    if opts::get().debug.signpost {
        signpost::start(category as u32, &[0, 0, 0, (category as usize) >> 4]);
    }
    let start_time = CrossProcessInstant::now();
    let val = {
        #[cfg(feature = "tracing")]
        let _enter = span.enter();
        callback()
    };
    let end_time = CrossProcessInstant::now();

    if opts::get().debug.signpost {
        signpost::end(category as u32, &[0, 0, 0, (category as usize) >> 4]);
    }

    send_profile_data(category, meta, &profiler_chan, start_time, end_time);
    val
}

pub fn send_profile_data(
    category: ProfilerCategory,
    meta: Option<TimerMetadata>,
    profiler_chan: &ProfilerChan,
    start_time: CrossProcessInstant,
    end_time: CrossProcessInstant,
) {
    profiler_chan.send(ProfilerMsg::Time((category, meta), (start_time, end_time)));
}
