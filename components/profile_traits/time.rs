/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate time as std_time;
extern crate url;

use self::std_time::precise_time_ns;
use self::url::Url;
use std::sync::mpsc::Sender;

#[derive(PartialEq, Clone, PartialOrd, Eq, Ord)]
pub struct TimerMetadata {
    pub url:         String,
    pub iframe:      bool,
    pub incremental: bool,
}

#[derive(Clone)]
pub struct ProfilerChan(pub Sender<ProfilerMsg>);

impl ProfilerChan {
    pub fn send(&self, msg: ProfilerMsg) {
        let ProfilerChan(ref c) = *self;
        c.send(msg).unwrap();
    }
}

#[derive(Clone)]
pub enum ProfilerMsg {
    /// Normal message used for reporting time
    Time((ProfilerCategory, Option<TimerMetadata>), f64),
    /// Message used to force print the profiling metrics
    Print,
    /// Tells the profiler to shut down.
    Exit,
}

#[repr(u32)]
#[derive(PartialEq, Clone, PartialOrd, Eq, Ord)]
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
}

#[derive(Eq, PartialEq)]
pub enum TimerMetadataFrameType {
    RootWindow,
    IFrame,
}

#[derive(Eq, PartialEq)]
pub enum TimerMetadataReflowType {
    Incremental,
    FirstReflow,
}

pub type ProfilerMetadata<'a> =
    Option<(&'a Url, TimerMetadataFrameType, TimerMetadataReflowType)>;

pub fn profile<T, F>(category: ProfilerCategory,
                     meta: ProfilerMetadata,
                     profiler_chan: ProfilerChan,
                     callback: F)
                  -> T
    where F: FnOnce() -> T
{
    let start_time = precise_time_ns();
    let val = callback();
    let end_time = precise_time_ns();
    let ms = (end_time - start_time) as f64 / 1000000f64;
    let meta = meta.map(|(url, iframe, reflow_type)|
        TimerMetadata {
            url: url.serialize(),
            iframe: iframe == TimerMetadataFrameType::IFrame,
            incremental: reflow_type == TimerMetadataReflowType::Incremental,
        });
    profiler_chan.send(ProfilerMsg::Time((category, meta), ms));
    return val;
}
