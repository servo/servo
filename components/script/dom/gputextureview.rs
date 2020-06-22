/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::GPUTextureViewBinding::GPUTextureViewMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::gputexture::GPUTexture;
use dom_struct::dom_struct;
use std::cell::Cell;
use webgpu::WebGPUTextureView;

#[dom_struct]
pub struct GPUTextureView {
    reflector_: Reflector,
    label: DomRefCell<Option<DOMString>>,
    texture_view: WebGPUTextureView,
    texture: Dom<GPUTexture>,
    valid: Cell<bool>,
}

impl GPUTextureView {
    fn new_inherited(
        texture_view: WebGPUTextureView,
        texture: &GPUTexture,
        valid: bool,
    ) -> GPUTextureView {
        Self {
            reflector_: Reflector::new(),
            texture: Dom::from_ref(texture),
            label: DomRefCell::new(None),
            texture_view,
            valid: Cell::new(valid),
        }
    }

    pub fn new(
        global: &GlobalScope,
        texture_view: WebGPUTextureView,
        texture: &GPUTexture,
        valid: bool,
    ) -> DomRoot<GPUTextureView> {
        reflect_dom_object(
            Box::new(GPUTextureView::new_inherited(texture_view, texture, valid)),
            global,
        )
    }
}

impl GPUTextureView {
    pub fn id(&self) -> WebGPUTextureView {
        self.texture_view
    }

    pub fn is_valid(&self) -> bool {
        self.valid.get()
    }
}

impl GPUTextureViewMethods for GPUTextureView {
    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn GetLabel(&self) -> Option<DOMString> {
        self.label.borrow().clone()
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn SetLabel(&self, value: Option<DOMString>) {
        *self.label.borrow_mut() = value;
    }
}
