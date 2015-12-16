/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate time as std_time;

use energy::read_energy_uj;
use ipc_channel::ipc::IpcSender;
use self::std_time::precise_time_ns;

#[derive(PartialEq, Clone, PartialOrd, Eq, Ord, Deserialize, Serialize)]
pub struct TimerMetadata {
    pub url:         String,
    pub iframe:      TimerMetadataFrameType,
    pub incremental: TimerMetadataReflowType,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct ProfilerChan(pub IpcSender<ProfilerMsg>);

impl ProfilerChan {
    pub fn send(&self, msg: ProfilerMsg) {
        let ProfilerChan(ref c) = *self;
        c.send(msg).unwrap();
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub enum ProfilerMsg {
    /// Normal message used for reporting time
    Time((ProfilerCategory, Option<TimerMetadata>), (u64, u64), (u64, u64)),
    /// Message used to force print the profiling metrics
    Print,
    /// Tells the profiler to shut down.
    Exit,
}

#[repr(u32)]
#[derive(PartialEq, Clone, PartialOrd, Eq, Ord, Deserialize, Serialize, Debug, Hash)]
pub enum ProfilerCategory {
    Compositing,
    LayoutPerform,
    LayoutStyleRecalc,
    LayoutRestyleDamagePropagation,
    LayoutNonIncrementalReset,
    LayoutSelectorMatch,
    LayoutTreeBuilder,
    LayoutDamagePropagate,
    LayoutGeneratedContent,
    LayoutMain,
    LayoutParallelWarmup,
    LayoutShaping,
    LayoutDispListBuild,
    PaintingPerTile,
    PaintingPrepBuff,
    Painting,
    ImageDecoding,
    ScriptAttachLayout,
    ScriptConstellationMsg,
    ScriptDevtoolsMsg,
    ScriptDocumentEvent,
    ScriptDomEvent,
    ScriptEvent,
    ScriptFileRead,
    ScriptImageCacheMsg,
    ScriptInputEvent,
    ScriptNetworkEvent,
    ScriptResize,
    ScriptSetViewport,
    ScriptTimerEvent,
    ScriptStylesheetLoad,
    ScriptUpdateReplacedElement,
    ScriptWebSocketEvent,
    ScriptWorkerEvent,
    ApplicationHeartbeat,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub enum TimerMetadataFrameType {
    RootWindow,
    IFrame,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
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
    profiler_chan.send(ProfilerMsg::Time((category, meta),
                                         (start_time, end_time),
                                         (start_energy, end_energy)));
    val
}
