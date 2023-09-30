/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use webgpu::WebGPUBindGroupLayout;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::GPUBindGroupLayoutMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::USVString;
use crate::dom::globalscope::GlobalScope;

#[dom_struct]
pub struct GPUBindGroupLayout {
    reflector_: Reflector,
    label: DomRefCell<USVString>,
    #[no_trace]
    bind_group_layout: WebGPUBindGroupLayout,
}

impl GPUBindGroupLayout {
    fn new_inherited(bind_group_layout: WebGPUBindGroupLayout, label: USVString) -> Self {
        Self {
            reflector_: Reflector::new(),
            label: DomRefCell::new(label),
            bind_group_layout,
        }
    }

    pub fn new(
        global: &GlobalScope,
        bind_group_layout: WebGPUBindGroupLayout,
        label: USVString,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(GPUBindGroupLayout::new_inherited(bind_group_layout, label)),
            global,
        )
    }
}

impl GPUBindGroupLayout {
    pub fn id(&self) -> WebGPUBindGroupLayout {
        self.bind_group_layout
    }
}

impl GPUBindGroupLayoutMethods for GPUBindGroupLayout {
    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn Label(&self) -> USVString {
        self.label.borrow().clone()
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn SetLabel(&self, value: USVString) {
        *self.label.borrow_mut() = value;
    }
}
