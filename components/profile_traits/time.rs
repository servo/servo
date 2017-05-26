/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate time as std_time;

use energy::read_energy_uj;
use ipc_channel::ipc::IpcSender;
use self::std_time::precise_time_ns;
use servo_config::opts;
use signpost;

#[derive(PartialEq, Clone, PartialOrd, Eq, Ord, Debug, Deserialize, Serialize)]
pub struct TimerMetadata {
    pub url:         String,
    pub iframe:      TimerMetadataFrameType,
    pub incremental: TimerMetadataReflowType,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct ProfilerChan(pub IpcSender<ProfilerMsg>);

impl ProfilerChan {
    pub fn send(&self, msg: ProfilerMsg) {
        if let Err(e) = self.0.send(msg) {
            warn!("Error communicating with the time profiler thread: {}", e);
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub enum ProfilerMsg {
    /// Normal message used for reporting time
    Time((ProfilerCategory, Option<TimerMetadata>), (u64, u64), (u64, u64)),
    /// Message used to force print the profiling metrics
    Print,
    /// Tells the profiler to shut down.
    Exit(IpcSender<()>),
}

#[repr(u32)]
#[derive(PartialEq, Clone, Copy, PartialOrd, Eq, Ord, Deserialize, Serialize, Debug, Hash)]
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
    TimeToFirstPaint = 0x80,
    TimeToFirstContentfulPaint = 0x81,
    ApplicationHeartbeat = 0x90,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Deserialize, Serialize)]
pub enum TimerMetadataFrameType {
    RootWindow,
    IFrame,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Deserialize, Serialize)]
pub enum TimerMetadataReflowType {
    Incremental,
    FirstReflow,
}

pub fn profile<T, F>(category: ProfilerCategory,
                     meta: Option<TimerMetadata>,
                     profiler_chan: ProfilerChan,
                     callback: F)
                  -> T
    where F: FnOnce() -> T
{
    if opts::get().signpost {
        signpost::start(category as u32, &[0, 0, 0, (category as usize) >> 4]);
    }
    let start_energy = read_energy_uj();
    let start_time = precise_time_ns();

    let val = callback();

    let end_time = precise_time_ns();
    let end_energy = read_energy_uj();
    if opts::get().signpost {
        signpost::end(category as u32, &[0, 0, 0, (category as usize) >> 4]);
    }

    send_profile_data(category,
                      meta,
                      &profiler_chan,
                      start_time,
                      end_time,
                      start_energy,
                      end_energy);
    val
}

pub fn send_profile_data(category: ProfilerCategory,
                         meta: Option<TimerMetadata>,
                         profiler_chan: &ProfilerChan,
                         start_time: u64,
                         end_time: u64,
                         start_energy: u64,
                         end_energy: u64) {
    profiler_chan.send(ProfilerMsg::Time((category, meta),
                                         (start_time, end_time),
                                         (start_energy, end_energy)));
}
