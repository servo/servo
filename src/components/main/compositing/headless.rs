/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use compositing::*;

use geom::size::Size2D;
use servo_msg::constellation_msg::{ConstellationChan, ExitMsg, ResizedWindowMsg};
use std::comm::{Empty, Disconnected, Data, Receiver};
use servo_util::time::ProfilerChan;
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
                  profiler_chan: ProfilerChan) {
        let compositor = NullCompositor::new(port);

        // Tell the constellation about the initial fake size.
        {
            let ConstellationChan(ref chan) = constellation_chan;
            chan.send(ResizedWindowMsg(Size2D(640u, 480u)));
        }
        compositor.handle_message(constellation_chan);

        // Drain compositor port, sometimes messages contain channels that are blocking
        // another task from finishing (i.e. SetIds)
        loop {
            match compositor.port.try_recv() {
                Empty | Disconnected => break,
                Data(_) => {},
            }
        }

        profiler_chan.send(time::ExitMsg);
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

                CreateRootCompositorLayerIfNecessary(..) |
                CreateDescendantCompositorLayerIfNecessary(..) | SetLayerPageSize(..) |
                SetLayerClipRect(..) | DeleteLayerGroup(..) | Paint(..) |
                ChangeReadyState(..) | ChangeRenderState(..) | ScrollFragmentPoint(..) |
                SetUnRenderedColor(..) | LoadComplete(..) => ()
            }
        }
    }
}
