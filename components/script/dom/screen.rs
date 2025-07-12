/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use embedder_traits::{EmbedderMsg, ScreenMetrics};
use ipc_channel::ipc;

use crate::dom::bindings::codegen::Bindings::ScreenBinding::ScreenMethods;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct Screen {
    reflector_: Reflector,
    window: Dom<Window>,
}

impl Screen {
    fn new_inherited(window: &Window) -> Screen {
        Screen {
            reflector_: Reflector::new(),
            window: Dom::from_ref(window),
        }
    }

    pub(crate) fn new(window: &Window, can_gc: CanGc) -> DomRoot<Screen> {
        reflect_dom_object(Box::new(Screen::new_inherited(window)), window, can_gc)
    }

    /// Retrives [`ScreenMetrics`] from the embedder.
    fn screen_metrics(&self) -> ScreenMetrics {
        let (sender, receiver) = ipc::channel().expect("Failed to create IPC channel!");

        self.window.send_to_embedder(EmbedderMsg::GetScreenMetrics(
            self.window.webview_id(),
            sender,
        ));

        receiver.recv().unwrap_or_default()
    }
}

impl ScreenMethods<crate::DomTypeHolder> for Screen {
    // https://drafts.csswg.org/cssom-view/#dom-screen-availwidth
    fn AvailWidth(&self) -> Finite<f64> {
        Finite::wrap(self.screen_metrics().available_size.width as f64)
    }

    // https://drafts.csswg.org/cssom-view/#dom-screen-availheight
    fn AvailHeight(&self) -> Finite<f64> {
        Finite::wrap(self.screen_metrics().available_size.height as f64)
    }

    // https://drafts.csswg.org/cssom-view/#dom-screen-width
    fn Width(&self) -> Finite<f64> {
        Finite::wrap(self.screen_metrics().screen_size.width as f64)
    }

    // https://drafts.csswg.org/cssom-view/#dom-screen-height
    fn Height(&self) -> Finite<f64> {
        Finite::wrap(self.screen_metrics().screen_size.height as f64)
    }

    // https://drafts.csswg.org/cssom-view/#dom-screen-colordepth
    fn ColorDepth(&self) -> u32 {
        24
    }

    // https://drafts.csswg.org/cssom-view/#dom-screen-pixeldepth
    fn PixelDepth(&self) -> u32 {
        24
    }
}
