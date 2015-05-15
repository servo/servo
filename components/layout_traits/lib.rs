/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate geom;
extern crate gfx;
extern crate script_traits;
extern crate msg;
extern crate profile_traits;
extern crate net_traits;
extern crate url;
extern crate util;

// This module contains traits in layout used generically
//   in the rest of Servo.
// The traits are here instead of in layout so
//   that these modules won't have to depend on layout.

use geom::rect::Rect;
use gfx::font_cache_task::FontCacheTask;
use gfx::paint_task::PaintChan;
use msg::compositor_msg::{Epoch, LayerId};
use msg::constellation_msg::{ConstellationChan, Failure, PipelineId, PipelineExitType};
use profile_traits::mem;
use profile_traits::time;
use net_traits::image_cache_task::ImageCacheTask;
use script_traits::{ScriptControlChan, OpaqueScriptLayoutChannel};
use std::sync::mpsc::{Sender, Receiver};
use util::geometry::Au;
use url::Url;

/// Messages sent to the layout task from the constellation and/or compositor.
pub enum LayoutControlMsg {
    ExitNow(PipelineExitType),
    GetCurrentEpoch(Sender<Epoch>),
    TickAnimations,
    SetVisibleRects(Vec<(LayerId, Rect<Au>)>),
}

/// A channel wrapper for constellation messages
#[derive(Clone)]
pub struct LayoutControlChan(pub Sender<LayoutControlMsg>);

// A static method creating a layout task
// Here to remove the compositor -> layout dependency
pub trait LayoutTaskFactory {
    // FIXME: use a proper static method
    fn create(_phantom: Option<&mut Self>,
              id: PipelineId,
              url: Url,
              is_iframe: bool,
              chan: OpaqueScriptLayoutChannel,
              pipeline_port: Receiver<LayoutControlMsg>,
              constellation_chan: ConstellationChan,
              failure_msg: Failure,
              script_chan: ScriptControlChan,
              paint_chan: PaintChan,
              image_cache_task: ImageCacheTask,
              font_cache_task: FontCacheTask,
              time_profiler_chan: time::ProfilerChan,
              mem_profiler_chan: mem::ProfilerChan,
              shutdown_chan: Sender<()>);
}
