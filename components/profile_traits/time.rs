/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate time as std_time;

use energy::read_energy_uj;
use ipc_channel::ipc::IpcSender;
use self::std_time::precise_time_ns;

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
        self.0.send(msg).unwrap();
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
    Compositing,
    LayoutPerform,
    LayoutStyleRecalc,
    LayoutTextShaping,
    LayoutRestyleDamagePropagation,
    LayoutNonIncrementalReset,
    LayoutSelectorMatch,
    LayoutTreeBuilder,
    LayoutDamagePropagate,
    LayoutGeneratedContent,
    LayoutDisplayListSorting,
    LayoutFloatPlacementSpeculation,
    LayoutMain,
    LayoutStoreOverflow,
    LayoutParallelWarmup,
    LayoutDispListBuild,
    NetHTTPRequestResponse,
    PaintingPerTile,
    PaintingPrepBuff,
    Painting,
    ImageDecoding,
    ImageSaving,
    ScriptAttachLayout,
    ScriptConstellationMsg,
    ScriptDevtoolsMsg,
    ScriptDocumentEvent,
    ScriptDomEvent,
    ScriptEvaluate,
    ScriptEvent,
    ScriptFileRead,
    ScriptImageCacheMsg,
    ScriptInputEvent,
    ScriptNetworkEvent,
    ScriptParseHTML,
    ScriptPlannedNavigation,
    ScriptResize,
    ScriptSetScrollState,
    ScriptSetViewport,
    ScriptTimerEvent,
    ScriptStylesheetLoad,
    ScriptUpdateReplacedElement,
    ScriptWebSocketEvent,
    ScriptWorkerEvent,
    ScriptServiceWorkerEvent,
    ApplicationHeartbeat,
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
    let start_energy = read_energy_uj();
    let start_time = precise_time_ns();
    let val = callback();
    let end_time = precise_time_ns();
    let end_energy = read_energy_uj();
    send_profile_data(category,
                      meta,
                      profiler_chan,
                      start_time,
                      end_time,
                      start_energy,
                      end_energy);
    val
}

pub fn send_profile_data(category: ProfilerCategory,
                         meta: Option<TimerMetadata>,
                         profiler_chan: ProfilerChan,
                         start_time: u64,
                         end_time: u64,
                         start_energy: u64,
                         end_energy: u64) {
    profiler_chan.send(ProfilerMsg::Time((category, meta),
                                         (start_time, end_time),
                                         (start_energy, end_energy)));
}
