/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(missing_copy_implementations)]

extern crate gfx;
extern crate script_traits;
extern crate msg;
extern crate net;
extern crate util;

// This module contains traits in layout used generically
//   in the rest of Servo.
// The traits are here instead of in layout so
//   that these modules won't have to depend on layout.

use gfx::font_cache_task::FontCacheTask;
use gfx::paint_task::PaintChan;
use msg::constellation_msg::{ConstellationChan, Failure, PipelineId, PipelineExitType};
use net::image_cache_task::ImageCacheTask;
use net::resource_task::ResourceTask;
use util::time::TimeProfilerChan;
use script_traits::{ScriptControlChan, OpaqueScriptLayoutChannel};
use std::sync::mpsc::{Sender, Receiver};

/// Messages sent to the layout task from the constellation
pub enum LayoutControlMsg {
    ExitNowMsg(PipelineExitType),
}

/// A channel wrapper for constellation messages
pub struct LayoutControlChan(pub Sender<LayoutControlMsg>);

// A static method creating a layout task
// Here to remove the compositor -> layout dependency
pub trait LayoutTaskFactory {
    // FIXME: use a proper static method
    fn create(_phantom: Option<&mut Self>,
              id: PipelineId,
              chan: OpaqueScriptLayoutChannel,
              pipeline_port: Receiver<LayoutControlMsg>,
              constellation_chan: ConstellationChan,
              failure_msg: Failure,
              script_chan: ScriptControlChan,
              paint_chan: PaintChan,
              resource_task: ResourceTask,
              img_cache_task: ImageCacheTask,
              font_cache_task: FontCacheTask,
              time_profiler_chan: TimeProfilerChan,
              shutdown_chan: Sender<()>);
}
