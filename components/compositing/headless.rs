/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use compositor_task::{CompositorEventListener, CompositorReceiver, Msg};
use windowing::WindowEvent;

use geom::scale_factor::ScaleFactor;
use geom::size::TypedSize2D;
use msg::constellation_msg::Msg as ConstellationMsg;
use msg::constellation_msg::{ConstellationChan, WindowSizeData};
use util::memory::MemoryProfilerChan;
use util::memory;
use util::time::TimeProfilerChan;
use util::time;

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
            chan.send(ConstellationMsg::ResizedWindow(WindowSizeData {
                initial_viewport: TypedSize2D(640_f32, 480_f32),
                visible_viewport: TypedSize2D(640_f32, 480_f32),
                device_pixel_ratio: ScaleFactor(1.0),
            })).unwrap();
        }

        compositor
    }
}

impl CompositorEventListener for NullCompositor {
    fn handle_event(&mut self, _: WindowEvent) -> bool {
        match self.port.recv_compositor_msg() {
            Msg::Exit(chan) => {
                debug!("shutting down the constellation");
                let ConstellationChan(ref con_chan) = self.constellation_chan;
                con_chan.send(ConstellationMsg::Exit).unwrap();
                chan.send(()).unwrap();
            }

            Msg::ShutdownComplete => {
                debug!("constellation completed shutdown");
                return false
            }

            Msg::GetGraphicsMetadata(chan) => {
                chan.send(None).unwrap();
            }

            Msg::SetFrameTree(_, response_chan, _) => {
                response_chan.send(()).unwrap();
            }

            Msg::ChangeLayerPipelineAndRemoveChildren(_, _, response_channel) => {
                response_channel.send(()).unwrap();
            }

            Msg::CreateRootLayerForPipeline(_, _, _, response_channel) => {
                response_channel.send(()).unwrap();
            }

            // Explicitly list ignored messages so that when we add a new one,
            // we'll notice and think about whether it needs a response, like
            // SetFrameTree.

            Msg::CreateOrUpdateBaseLayer(..) |
            Msg::CreateOrUpdateDescendantLayer(..) |
            Msg::SetLayerOrigin(..) |
            Msg::AssignPaintedBuffers(..) |
            Msg::ChangeReadyState(..) |
            Msg::ChangePaintState(..) |
            Msg::ScrollFragmentPoint(..) |
            Msg::LoadComplete |
            Msg::PaintMsgDiscarded(..) |
            Msg::ScrollTimeout(..) |
            Msg::ChangePageTitle(..) |
            Msg::ChangePageLoadData(..) |
            Msg::KeyEvent(..) |
            Msg::SetCursor(..) => {}
            Msg::PaintTaskExited(..) => {}
        }
        true
    }

    fn repaint_synchronously(&mut self) {}

    fn shutdown(&mut self) {
        // Drain compositor port, sometimes messages contain channels that are blocking
        // another task from finishing (i.e. SetIds)
        while self.port.try_recv_compositor_msg().is_some() {}

        self.time_profiler_chan.send(time::TimeProfilerMsg::Exit);
        self.memory_profiler_chan.send(memory::MemoryProfilerMsg::Exit);
    }

    fn pinch_zoom_level(&self) -> f32 {
        1.0
    }

    fn get_title_for_main_frame(&self) {}
}
