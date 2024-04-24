/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use webgpu::{WebGPU, WebGPURequest, WebGPUTextureView};

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::GPUTextureViewMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::USVString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::gputexture::GPUTexture;

#[dom_struct]
pub struct GPUTextureView {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "defined in webgpu"]
    #[no_trace]
    channel: WebGPU,
    label: DomRefCell<USVString>,
    #[no_trace]
    texture_view: WebGPUTextureView,
    texture: Dom<GPUTexture>,
}

impl GPUTextureView {
    fn new_inherited(
        channel: WebGPU,
        texture_view: WebGPUTextureView,
        texture: &GPUTexture,
        label: USVString,
    ) -> GPUTextureView {
        Self {
            reflector_: Reflector::new(),
            channel,
            texture: Dom::from_ref(texture),
            label: DomRefCell::new(label),
            texture_view,
        }
    }

    pub fn new(
        global: &GlobalScope,
        channel: WebGPU,
        texture_view: WebGPUTextureView,
        texture: &GPUTexture,
        label: USVString,
    ) -> DomRoot<GPUTextureView> {
        reflect_dom_object(
            Box::new(GPUTextureView::new_inherited(
                channel,
                texture_view,
                texture,
                label,
            )),
            global,
        )
    }
}

impl GPUTextureView {
    pub fn id(&self) -> WebGPUTextureView {
        self.texture_view
    }
}

impl GPUTextureViewMethods for GPUTextureView {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn Label(&self) -> USVString {
        self.label.borrow().clone()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn SetLabel(&self, value: USVString) {
        *self.label.borrow_mut() = value;
    }
}

impl Drop for GPUTextureView {
    fn drop(&mut self) {
        if let Err(e) = self
            .channel
            .0
            .send((None, WebGPURequest::DropTextureView(self.texture_view.0)))
        {
            warn!(
                "Failed to send DropTextureView ({:?}) ({})",
                self.texture_view.0, e
            );
        }
    }
}
