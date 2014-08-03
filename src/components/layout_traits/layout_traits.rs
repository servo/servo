/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![crate_name = "layout_traits"]
#![crate_type = "rlib"]

#![comment = "The Servo Parallel Browser Project"]
#![license = "MPL"]

extern crate gfx;
extern crate script;
extern crate servo_msg = "msg";
extern crate servo_net = "net";
extern crate servo_util = "util";

// This module contains traits in layout used generically
//   in the rest of Servo.
// The traits are here instead of in layout so
//   that these modules won't have to depend on layout.

use gfx::font_cache_task::FontCacheTask;
use gfx::render_task::RenderChan;
use servo_msg::constellation_msg::{ConstellationChan, PipelineId};
use servo_msg::constellation_msg::Failure;
use servo_net::image_cache_task::ImageCacheTask;
use servo_util::opts::Opts;
use servo_util::time::TimeProfilerChan;
use script::layout_interface::{LayoutChan, Msg};
use script::script_task::ScriptChan;
use std::comm::{Sender, Receiver};

// A static method creating a layout task
// Here to remove the compositor -> layout dependency
pub trait LayoutTaskFactory {
    // FIXME: use a proper static method
    fn create(_phantom: Option<&mut Self>,
              id: PipelineId,
              port: Receiver<Msg>,
              chan: LayoutChan,
              constellation_chan: ConstellationChan,
              failure_msg: Failure,
              script_chan: ScriptChan,
              render_chan: RenderChan,
              img_cache_task: ImageCacheTask,
              font_cache_task: FontCacheTask,
              opts: Opts,
              time_profiler_chan: TimeProfilerChan,
              shutdown_chan: Sender<()>);
}
