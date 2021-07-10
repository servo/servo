/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::GPUSwapChainBinding::GPUSwapChainMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::USVString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::gpucanvascontext::GPUCanvasContext;
use crate::dom::gputexture::GPUTexture;
use dom_struct::dom_struct;
use webgpu::{WebGPU, WebGPURequest, WebGPUTexture};

#[dom_struct]
pub struct GPUSwapChain {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "channels are hard"]
    channel: WebGPU,
    label: DomRefCell<Option<USVString>>,
    context: Dom<GPUCanvasContext>,
    texture: Dom<GPUTexture>,
}

impl GPUSwapChain {
    fn new_inherited(
        channel: WebGPU,
        context: &GPUCanvasContext,
        texture: &GPUTexture,
        label: Option<USVString>,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            channel,
            context: Dom::from_ref(context),
            texture: Dom::from_ref(texture),
            label: DomRefCell::new(label),
        }
    }

    pub fn new(
        global: &GlobalScope,
        channel: WebGPU,
        context: &GPUCanvasContext,
        texture: &GPUTexture,
        label: Option<USVString>,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(GPUSwapChain::new_inherited(
                channel, context, texture, label,
            )),
            global,
        )
    }
}

impl GPUSwapChain {
    pub fn destroy(&self, external_id: u64, image_key: webrender_api::ImageKey) {
        if let Err(e) = self.channel.0.send((
            None,
            WebGPURequest::DestroySwapChain {
                external_id,
                image_key,
            },
        )) {
            warn!(
                "Failed to send DestroySwapChain-ImageKey({:?}) ({})",
                image_key, e
            );
        }
    }

    pub fn texture_id(&self) -> WebGPUTexture {
        self.texture.id()
    }
}

impl GPUSwapChainMethods for GPUSwapChain {
    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn GetLabel(&self) -> Option<USVString> {
        self.label.borrow().clone()
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn SetLabel(&self, value: Option<USVString>) {
        *self.label.borrow_mut() = value;
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuswapchain-getcurrenttexture
    fn GetCurrentTexture(&self) -> DomRoot<GPUTexture> {
        self.context.mark_as_dirty();
        DomRoot::from_ref(&*self.texture)
    }
}
