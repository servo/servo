/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use compositor_task::{Msg, Exit, ChangeReadyState, SetIds};
use compositor_task::{GetGraphicsMetadata, CreateOrUpdateRootLayer, CreateOrUpdateDescendantLayer};
use compositor_task::{SetLayerClipRect, Paint, ScrollFragmentPoint, LoadComplete};
use compositor_task::{ShutdownComplete, ChangeRenderState, ReRenderMsgDiscarded};

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
    pub port: Receiver<Msg>,
}

impl NullCompositor {
    fn new(port: Receiver<Msg>) -> NullCompositor {
        NullCompositor {
            port: port,
        }
    }

    pub fn create(port: Receiver<Msg>,
                  constellation_chan: ConstellationChan,
                  time_profiler_chan: TimeProfilerChan,
                  memory_profiler_chan: MemoryProfilerChan) {
        let compositor = NullCompositor::new(port);

        // Tell the constellation about the initial fake size.
        {
            let ConstellationChan(ref chan) = constellation_chan;
            chan.send(ResizedWindowMsg(WindowSizeData {
                initial_viewport: TypedSize2D(640_f32, 480_f32),
                visible_viewport: TypedSize2D(640_f32, 480_f32),
                device_pixel_ratio: ScaleFactor(1.0),
            }));
        }
        compositor.handle_message(constellation_chan);

        // Drain compositor port, sometimes messages contain channels that are blocking
        // another task from finishing (i.e. SetIds)
        loop {
            match compositor.port.try_recv() {
                Err(_) => break,
                Ok(_) => {},
            }
        }

        time_profiler_chan.send(time::ExitMsg);
        memory_profiler_chan.send(memory::ExitMsg);
    }

    fn handle_message(&self, constellation_chan: ConstellationChan) {
        loop {
            match self.port.recv() {
                Exit(chan) => {
                    debug!("shutting down the constellation");
                    let ConstellationChan(ref con_chan) = constellation_chan;
                    con_chan.send(ExitMsg);
                    chan.send(());
                }

                ShutdownComplete => {
                    debug!("constellation completed shutdown");
                    break
                }

                GetGraphicsMetadata(chan) => {
                    chan.send(None);
                }

                SetIds(_, response_chan, _) => {
                    response_chan.send(());
                }

                // Explicitly list ignored messages so that when we add a new one,
                // we'll notice and think about whether it needs a response, like
                // SetIds.

                CreateOrUpdateRootLayer(..) |
                CreateOrUpdateDescendantLayer(..) |
                SetLayerClipRect(..) | Paint(..) |
                ChangeReadyState(..) | ChangeRenderState(..) | ScrollFragmentPoint(..) |
                LoadComplete(..) | ReRenderMsgDiscarded(..) => ()
            }
        }
    }
}
