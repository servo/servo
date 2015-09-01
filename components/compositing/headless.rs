/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use compositor_task::{CompositorEventListener, CompositorReceiver, Msg};
use windowing::WindowEvent;

use euclid::scale_factor::ScaleFactor;
use euclid::{Size2D, Point2D};
use msg::constellation_msg::AnimationState;
use msg::constellation_msg::Msg as ConstellationMsg;
use msg::constellation_msg::{ConstellationChan, WindowSizeData};
use profile_traits::mem;
use profile_traits::time;

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
    time_profiler_chan: time::ProfilerChan,
    /// A channel to the memory profiler.
    mem_profiler_chan: mem::ProfilerChan,
}

impl NullCompositor {
    fn new(port: Box<CompositorReceiver>,
           constellation_chan: ConstellationChan,
           time_profiler_chan: time::ProfilerChan,
           mem_profiler_chan: mem::ProfilerChan)
           -> NullCompositor {
        NullCompositor {
            port: port,
            constellation_chan: constellation_chan,
            time_profiler_chan: time_profiler_chan,
            mem_profiler_chan: mem_profiler_chan,
        }
    }

    pub fn create(port: Box<CompositorReceiver>,
                  constellation_chan: ConstellationChan,
                  time_profiler_chan: time::ProfilerChan,
                  mem_profiler_chan: mem::ProfilerChan)
                  -> NullCompositor {
        let compositor = NullCompositor::new(port,
                                             constellation_chan,
                                             time_profiler_chan,
                                             mem_profiler_chan);

        // Tell the constellation about the initial fake size.
        {
            let ConstellationChan(ref chan) = compositor.constellation_chan;
            chan.send(ConstellationMsg::ResizedWindow(WindowSizeData {
                initial_viewport: Size2D::typed(640_f32, 480_f32),
                visible_viewport: Size2D::typed(640_f32, 480_f32),
                device_pixel_ratio: ScaleFactor::new(1.0),
            })).unwrap();
        }

        compositor
    }
}

impl CompositorEventListener for NullCompositor {
    fn handle_events(&mut self, _: Vec<WindowEvent>) -> bool {
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

            Msg::GetNativeDisplay(chan) => {
                chan.send(None).unwrap();
            }

            Msg::SetFrameTree(_, response_chan, _) => {
                response_chan.send(()).unwrap();
            }

            Msg::GetClientWindow(send) => {
               let rect = (Size2D::zero(), Point2D::zero());
                send.send(rect).unwrap();
            }

            Msg::ChangeRunningAnimationsState(pipeline_id, animation_state) => {
                match animation_state {
                    AnimationState::AnimationsPresent |
                    AnimationState::NoAnimationsPresent |
                    AnimationState::NoAnimationCallbacksPresent => {}
                    AnimationState::AnimationCallbacksPresent => {
                        let msg = ConstellationMsg::TickAnimation(pipeline_id);
                        self.constellation_chan.0.send(msg).unwrap()
                    }
                }
            }

            // Explicitly list ignored messages so that when we add a new one,
            // we'll notice and think about whether it needs a response, like
            // SetFrameTree.

            Msg::InitializeLayersForPipeline(..) |
            Msg::SetLayerRect(..) |
            Msg::AssignPaintedBuffers(..) |
            Msg::ScrollFragmentPoint(..) |
            Msg::Status(..) |
            Msg::LoadStart(..) |
            Msg::LoadComplete(..) |
            Msg::ScrollTimeout(..) |
            Msg::RecompositeAfterScroll |
            Msg::ChangePageTitle(..) |
            Msg::ChangePageUrl(..) |
            Msg::KeyEvent(..) |
            Msg::SetCursor(..) |
            Msg::ViewportConstrained(..) => {}
            Msg::CreatePng(..) |
            Msg::PaintTaskExited(..) |
            Msg::MoveTo(..) |
            Msg::ResizeTo(..) |
            Msg::IsReadyToSaveImageReply(..) => {}
            Msg::NewFavicon(..) => {}
            Msg::HeadParsed => {}
            Msg::ReturnUnusedNativeSurfaces(..) => {}
            Msg::CollectMemoryReports(..) => {}
        }
        true
    }

    fn repaint_synchronously(&mut self) {}

    fn shutdown(&mut self) {
        // Drain compositor port, sometimes messages contain channels that are blocking
        // another task from finishing (i.e. SetIds)
        while self.port.try_recv_compositor_msg().is_some() {}

        self.time_profiler_chan.send(time::ProfilerMsg::Exit);
        self.mem_profiler_chan.send(mem::ProfilerMsg::Exit);
    }

    fn pinch_zoom_level(&self) -> f32 {
        1.0
    }

    fn get_title_for_main_frame(&self) {}
}
