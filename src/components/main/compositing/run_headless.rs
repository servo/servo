/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use compositing::*;

use geom::size::Size2D;
use servo_msg::constellation_msg::ResizedWindowMsg;

/// Starts the compositor, which listens for messages on the specified port.
///
/// This is the null compositor which doesn't draw anything to the screen.
/// It's intended for headless testing.
pub fn run_compositor(compositor: &CompositorTask) {
    // Tell the constellation about the initial fake size.
    compositor.constellation_chan.send(ResizedWindowMsg(Size2D(640u, 480u)));

    loop {
        match compositor.port.recv() {
            Exit => break,

            GetGraphicsMetadata(chan) => {
                chan.send(None);
            }

            SetIds(_, response_chan, _) => {
                response_chan.send(());
            }

            // Explicitly list ignored messages so that when we add a new one,
            // we'll notice and think about whether it needs a response, like
            // SetIds.

            NewLayer(*) | SetLayerPageSize(*) | SetLayerClipRect(*) | DeleteLayer(*) |
            Paint(*) | InvalidateRect(*) | ChangeReadyState(*) | ChangeRenderState(*)|
            ScrollFragmentPoint(*) | SetUnRenderedColor(*)
                => ()
        }
    }
    compositor.shutdown_chan.send(())
}
