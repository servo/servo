/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::time::{SystemTime, UNIX_EPOCH};

use ipc_channel::ipc::IpcSender;
use log::warn;
use serde::{Deserialize, Serialize};
use servo_config::opts;

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
    Record(Vec<f64>),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ProfilerMsg {
    /// Normal message used for reporting time
    Time((ProfilerCategory, Option<TimerMetadata>), (u64, u64)),
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

#[repr(u32)]
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum ProfilerCategory {
    Compositing = 0x00,
    LayoutPerform = 0x10,
    LayoutStyleRecalc = 0x11,
    LayoutTextShaping = 0x12,
    LayoutRestyleDamagePropagation = 0x13,
    LayoutNonIncrementalReset = 0x14,
    LayoutSelectorMatch = 0x15,
    LayoutTreeBuilder = 0x16,
    LayoutDamagePropagate = 0x17,
    LayoutGeneratedContent = 0x18,
    LayoutDisplayListSorting = 0x19,
    LayoutFloatPlacementSpeculation = 0x1a,
    LayoutMain = 0x1b,
    LayoutStoreOverflow = 0x1c,
    LayoutParallelWarmup = 0x1d,
    LayoutDispListBuild = 0x1e,
    NetHTTPRequestResponse = 0x30,
    PaintingPerTile = 0x41,
    PaintingPrepBuff = 0x42,
    Painting = 0x43,
    ImageDecoding = 0x50,
    ImageSaving = 0x51,
    ScriptAttachLayout = 0x60,
    ScriptConstellationMsg = 0x61,
    ScriptDevtoolsMsg = 0x62,
    ScriptDocumentEvent = 0x63,
    ScriptDomEvent = 0x64,
    ScriptEvaluate = 0x65,
    ScriptEvent = 0x66,
    ScriptFileRead = 0x67,
    ScriptImageCacheMsg = 0x68,
    ScriptInputEvent = 0x69,
    ScriptNetworkEvent = 0x6a,
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
    ScriptParseXML = 0x76,
    ScriptEnterFullscreen = 0x77,
    ScriptExitFullscreen = 0x78,
    ScriptWebVREvent = 0x79,
    ScriptWorkletEvent = 0x7a,
    ScriptPerformanceEvent = 0x7b,
    ScriptHistoryEvent = 0x7c,
    ScriptPortMessage = 0x7d,
    ScriptWebGPUMsg = 0x7e,
    TimeToFirstPaint = 0x80,
    TimeToFirstContentfulPaint = 0x81,
    TimeToInteractive = 0x82,
    IpcReceiver = 0x83,
    IpcBytesReceiver = 0x84,
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

pub fn profile<T, F>(
    category: ProfilerCategory,
    meta: Option<TimerMetadata>,
    profiler_chan: ProfilerChan,
    callback: F,
) -> T
where
    F: FnOnce() -> T,
{
    if opts::get().debug.signpost {
        signpost::start(category as u32, &[0, 0, 0, (category as usize) >> 4]);
    }
    let start_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    let val = callback();

    let end_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    if opts::get().debug.signpost {
        signpost::end(category as u32, &[0, 0, 0, (category as usize) >> 4]);
    }

    send_profile_data(
        category,
        meta,
        &profiler_chan,
        start_time as u64,
        end_time as u64,
    );
    val
}

pub fn send_profile_data(
    category: ProfilerCategory,
    meta: Option<TimerMetadata>,
    profiler_chan: &ProfilerChan,
    start_time: u64,
    end_time: u64,
) {
    profiler_chan.send(ProfilerMsg::Time((category, meta), (start_time, end_time)));
}
