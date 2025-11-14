/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use canvas_traits::webgl::{WebGLContextId, WebGLMsg, WebGLThreads, webgl_channel};
use compositing_traits::{
    CrossProcessCompositorApi, PainterSurfmanDetailsMap, WebRenderExternalImageIdManager,
};
use log::debug;
use surfman::Device;
use surfman::chains::SwapChains;
#[cfg(feature = "webxr")]
use webxr::SurfmanGL as WebXRSurfman;
#[cfg(feature = "webxr")]
use webxr_api::LayerGrandManager as WebXRLayerGrandManager;

use crate::webgl_thread::{WebGLThread, WebGLThreadInit};

pub struct WebGLComm {
    pub webgl_threads: WebGLThreads,
    pub swap_chains: SwapChains<WebGLContextId, Device>,
    #[cfg(feature = "webxr")]
    pub webxr_layer_grand_manager: WebXRLayerGrandManager<WebXRSurfman>,
}

impl WebGLComm {
    /// Creates a new `WebGLComm` object.
    pub fn new(
        compositor_api: CrossProcessCompositorApi,
        external_image_id_manager: WebRenderExternalImageIdManager,
        painter_surfman_details_map: PainterSurfmanDetailsMap,
    ) -> WebGLComm {
        debug!("WebGLThreads::new()");
        let (sender, receiver) = webgl_channel::<WebGLMsg>().unwrap();
        let swap_chains = SwapChains::new();

        #[cfg(feature = "webxr")]
        let webxr_init = crate::webxr::WebXRBridgeInit::new(sender.clone());
        #[cfg(feature = "webxr")]
        let webxr_layer_grand_manager = webxr_init.layer_grand_manager();

        // This implementation creates a single `WebGLThread` for all the pipelines.
        let init = WebGLThreadInit {
            compositor_api,
            external_image_id_manager,
            sender: sender.clone(),
            receiver,
            webrender_swap_chains: swap_chains.clone(),
            #[cfg(feature = "webxr")]
            webxr_init,
            painter_surfman_details_map,
        };

        WebGLThread::run_on_own_thread(init);

        WebGLComm {
            webgl_threads: WebGLThreads(sender),
            swap_chains,
            #[cfg(feature = "webxr")]
            webxr_layer_grand_manager,
        }
    }
}
