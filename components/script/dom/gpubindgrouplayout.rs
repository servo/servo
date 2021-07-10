/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::GPUBindGroupLayoutBinding::GPUBindGroupLayoutMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::USVString;
use crate::dom::globalscope::GlobalScope;
use dom_struct::dom_struct;
use webgpu::WebGPUBindGroupLayout;

#[dom_struct]
pub struct GPUBindGroupLayout {
    reflector_: Reflector,
    label: DomRefCell<Option<USVString>>,
    bind_group_layout: WebGPUBindGroupLayout,
}

impl GPUBindGroupLayout {
    fn new_inherited(bind_group_layout: WebGPUBindGroupLayout, label: Option<USVString>) -> Self {
        Self {
            reflector_: Reflector::new(),
            label: DomRefCell::new(label),
            bind_group_layout,
        }
    }

    pub fn new(
        global: &GlobalScope,
        bind_group_layout: WebGPUBindGroupLayout,
        label: Option<USVString>,
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
    fn GetLabel(&self) -> Option<USVString> {
        self.label.borrow().clone()
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn SetLabel(&self, value: Option<USVString>) {
        *self.label.borrow_mut() = value;
    }
}
