/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use AnimationTickType;
use CompositorMsg as ConstellationMsg;
use compositor_thread::{CompositorEventListener, CompositorReceiver};
use compositor_thread::{InitialCompositorState, Msg};
use euclid::scale_factor::ScaleFactor;
use euclid::{Point2D, Size2D};
use msg::constellation_msg::WindowSizeData;
use profile_traits::mem;
use profile_traits::time;
use script_traits::AnimationState;
use std::sync::mpsc::Sender;
use util::opts;
use windowing::WindowEvent;

/// Starts the compositor, which listens for messages on the specified port.
///
/// This is the null compositor which doesn't draw anything to the screen.
/// It's intended for headless testing.
pub struct NullCompositor {
    /// The port on which we receive messages.
    pub port: Box<CompositorReceiver>,
    /// A channel to the constellation.
    constellation_chan: Sender<ConstellationMsg>,
    /// A channel to the time profiler.
    time_profiler_chan: time::ProfilerChan,
    /// A channel to the memory profiler.
    mem_profiler_chan: mem::ProfilerChan,
}

impl NullCompositor {
    fn new(state: InitialCompositorState) -> NullCompositor {
        NullCompositor {
            port: state.receiver,
            constellation_chan: state.constellation_chan,
            time_profiler_chan: state.time_profiler_chan,
            mem_profiler_chan: state.mem_profiler_chan,
        }
    }

    pub fn create(state: InitialCompositorState) -> NullCompositor {
        let compositor = NullCompositor::new(state);

        // Tell the constellation about the initial fake size.
        {
            compositor.constellation_chan.send(ConstellationMsg::ResizedWindow(WindowSizeData {
                initial_viewport: Size2D::typed(800_f32, 600_f32),
                visible_viewport: Size2D::typed(800_f32, 600_f32),
                device_pixel_ratio:
                    ScaleFactor::new(opts::get().device_pixels_per_px.unwrap_or(1.0)),
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
                self.constellation_chan.send(ConstellationMsg::Exit).unwrap();
                chan.send(()).unwrap();
            }

            Msg::ShutdownComplete => {
                debug!("constellation completed shutdown");

                // Drain compositor port, sometimes messages contain channels that are blocking
                // another thread from finishing (i.e. SetIds)
                while self.port.try_recv_compositor_msg().is_some() {}

                self.time_profiler_chan.send(time::ProfilerMsg::Exit);
                self.mem_profiler_chan.send(mem::ProfilerMsg::Exit);

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
                        let msg = ConstellationMsg::TickAnimation(pipeline_id, AnimationTickType::Script);
                        self.constellation_chan.send(msg).unwrap()
                    }
                }
            }

            // Explicitly list ignored messages so that when we add a new one,
            // we'll notice and think about whether it needs a response, like
            // SetFrameTree.

            Msg::InitializeLayersForPipeline(..) |
            Msg::AssignPaintedBuffers(..) |
            Msg::ScrollFragmentPoint(..) |
            Msg::Status(..) |
            Msg::LoadStart(..) |
            Msg::LoadComplete(..) |
            Msg::DelayedCompositionTimeout(..) |
            Msg::Recomposite(..) |
            Msg::ChangePageTitle(..) |
            Msg::ChangePageUrl(..) |
            Msg::KeyEvent(..) |
            Msg::TouchEventProcessed(..) |
            Msg::SetCursor(..) |
            Msg::ViewportConstrained(..) => {}
            Msg::CreatePng(..) |
            Msg::PaintThreadExited(..) |
            Msg::MoveTo(..) |
            Msg::ResizeTo(..) |
            Msg::IsReadyToSaveImageReply(..) => {}
            Msg::NewFavicon(..) => {}
            Msg::HeadParsed => {}
            Msg::ReturnUnusedNativeSurfaces(..) => {}
            Msg::CollectMemoryReports(..) => {}
            Msg::PipelineExited(..) => {}
        }
        true
    }

    fn repaint_synchronously(&mut self) {}

    fn pinch_zoom_level(&self) -> f32 {
        1.0
    }

    fn title_for_main_frame(&self) {}
}
