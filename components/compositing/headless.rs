/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use compositor_task::{GetGraphicsMetadata, CreateOrUpdateRootLayer, CreateOrUpdateDescendantLayer};
use compositor_task::{Exit, ChangeReadyState, LoadComplete, Paint, ScrollFragmentPoint, SetIds};
use compositor_task::{SetLayerOrigin, ShutdownComplete, ChangePaintState, PaintMsgDiscarded};
use compositor_task::{CompositorEventListener, CompositorReceiver, ScrollTimeout, ChangePageTitle};
use compositor_task::{ChangePageLoadData, FrameTreeUpdateMsg, KeyEvent};
use windowing::WindowEvent;

use geom::scale_factor::ScaleFactor;
use geom::size::TypedSize2D;
use servo_msg::constellation_msg::{ConstellationChan, ExitMsg, ResizedWindowMsg, WindowSizeData};
use servo_util::memory::MemoryProfilerChan;
use servo_util::memory;
use servo_util::time::TimeProfilerChan;
use servo_util::time;

/// Starts the compositor, which listens for messages on the specified port.
///
/// This is the null compositor which doesn't draw anything to the screen.
/// It's intended for headless testing.
pub struct NullCompositor {
    /// The port on which we receive messages.
    pub port: Box<CompositorReceiver>,
    /// A channel to the constellation.
    constellation_chan: ConstellationChan,
    /// A channel to the time profiler.
    time_profiler_chan: TimeProfilerChan,
    /// A channel to the memory profiler.
    memory_profiler_chan: MemoryProfilerChan,
}

impl NullCompositor {
    fn new(port: Box<CompositorReceiver>,
           constellation_chan: ConstellationChan,
           time_profiler_chan: TimeProfilerChan,
           memory_profiler_chan: MemoryProfilerChan)
           -> NullCompositor {
        NullCompositor {
            port: port,
            constellation_chan: constellation_chan,
            time_profiler_chan: time_profiler_chan,
            memory_profiler_chan: memory_profiler_chan,
        }
    }

    pub fn create(port: Box<CompositorReceiver>,
                  constellation_chan: ConstellationChan,
                  time_profiler_chan: TimeProfilerChan,
                  memory_profiler_chan: MemoryProfilerChan)
                  -> NullCompositor {
        let compositor = NullCompositor::new(port,
                                             constellation_chan,
                                             time_profiler_chan,
                                             memory_profiler_chan);

        // Tell the constellation about the initial fake size.
        {
            let ConstellationChan(ref chan) = compositor.constellation_chan;
            chan.send(ResizedWindowMsg(WindowSizeData {
                initial_viewport: TypedSize2D(640_f32, 480_f32),
                visible_viewport: TypedSize2D(640_f32, 480_f32),
                device_pixel_ratio: ScaleFactor(1.0),
            }));
        }

        compositor
    }
}

impl CompositorEventListener for NullCompositor {
    fn handle_event(&mut self, _: WindowEvent) -> bool {
        match self.port.recv_compositor_msg() {
            Exit(chan) => {
                debug!("shutting down the constellation");
                let ConstellationChan(ref con_chan) = self.constellation_chan;
                con_chan.send(ExitMsg);
                chan.send(());
            }

            ShutdownComplete => {
                debug!("constellation completed shutdown");
                return false
            }

            GetGraphicsMetadata(chan) => {
                chan.send(None);
            }

            SetIds(_, response_chan, _) => {
                response_chan.send(());
            }

            FrameTreeUpdateMsg(_, response_channel) => {
                response_channel.send(());
            }

            // Explicitly list ignored messages so that when we add a new one,
            // we'll notice and think about whether it needs a response, like
            // SetIds.

            CreateOrUpdateRootLayer(..) |
            CreateOrUpdateDescendantLayer(..) |
            SetLayerOrigin(..) | Paint(..) |
            ChangeReadyState(..) | ChangePaintState(..) | ScrollFragmentPoint(..) |
            LoadComplete | PaintMsgDiscarded(..) | ScrollTimeout(..) | ChangePageTitle(..) |
            ChangePageLoadData(..) | KeyEvent(..) => ()
        }
        true
    }

    fn repaint_synchronously(&mut self) {}

    fn shutdown(&mut self) {
        // Drain compositor port, sometimes messages contain channels that are blocking
        // another task from finishing (i.e. SetIds)
        while self.port.try_recv_compositor_msg().is_some() {}

        self.time_profiler_chan.send(time::ExitMsg);
        self.memory_profiler_chan.send(memory::ExitMsg);
    }

    fn pinch_zoom_level(&self) -> f32 {
        1.0
    }

    fn get_title_for_main_frame(&self) {}
}
