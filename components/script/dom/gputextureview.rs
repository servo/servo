/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::GPUTextureViewBinding::GPUTextureViewMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::USVString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::gputexture::GPUTexture;
use dom_struct::dom_struct;
use webgpu::WebGPUTextureView;

#[dom_struct]
pub struct GPUTextureView {
    reflector_: Reflector,
    label: DomRefCell<Option<USVString>>,
    texture_view: WebGPUTextureView,
    texture: Dom<GPUTexture>,
}

impl GPUTextureView {
    fn new_inherited(
        texture_view: WebGPUTextureView,
        texture: &GPUTexture,
        label: Option<USVString>,
    ) -> GPUTextureView {
        Self {
            reflector_: Reflector::new(),
            texture: Dom::from_ref(texture),
            label: DomRefCell::new(label),
            texture_view,
        }
    }

    pub fn new(
        global: &GlobalScope,
        texture_view: WebGPUTextureView,
        texture: &GPUTexture,
        label: Option<USVString>,
    ) -> DomRoot<GPUTextureView> {
        reflect_dom_object(
            Box::new(GPUTextureView::new_inherited(texture_view, texture, label)),
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
    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn GetLabel(&self) -> Option<USVString> {
        self.label.borrow().clone()
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn SetLabel(&self, value: Option<USVString>) {
        *self.label.borrow_mut() = value;
    }
}
