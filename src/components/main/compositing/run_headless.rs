/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use compositing::*;

use std::unstable::intrinsics;

/// Starts the compositor, which listens for messages on the specified port.
///
/// This is the null compositor which doesn't draw anything to the screen.
/// It's intended for headless testing.
pub fn run_compositor(compositor: &CompositorTask) {
    loop {
        match compositor.port.recv() {
            Exit => break,

             GetGraphicsMetadata(chan) => {
                unsafe {
                    chan.send(intrinsics::uninit());
                }
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
