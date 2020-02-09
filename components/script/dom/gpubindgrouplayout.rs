/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::GPUBindGroupLayoutBinding::{
    self, GPUBindGroupLayoutBindings, GPUBindGroupLayoutMethods,
};
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use dom_struct::dom_struct;
use std::cell::Cell;
use webgpu::{WebGPU, WebGPUBindGroupLayout};

#[dom_struct]
pub struct GPUBindGroupLayout {
    reflector_: Reflector,
    label: DomRefCell<Option<DOMString>>,
    bind_group_layout: WebGPUBindGroupLayout,
    #[ignore_malloc_size_of = "defined in webgpu"]
    bindings: Vec<GPUBindGroupLayoutBindings>,
    #[ignore_malloc_size_of = "defined in webgpu"]
    channel: WebGPU,
    valid: Cell<bool>,
}

impl GPUBindGroupLayout {
    fn new_inherited(
        channel: WebGPU,
        bind_group_layout: WebGPUBindGroupLayout,
        bindings: Vec<GPUBindGroupLayoutBindings>,
        valid: bool,
    ) -> GPUBindGroupLayout {
        Self {
            reflector_: Reflector::new(),
            channel,
            label: DomRefCell::new(None),
            bind_group_layout,
            bindings,
            valid: Cell::new(valid),
        }
    }

    pub fn new(
        global: &GlobalScope,
        channel: WebGPU,
        bind_group_layout: WebGPUBindGroupLayout,
        bindings: Vec<GPUBindGroupLayoutBindings>,
        valid: bool,
    ) -> DomRoot<GPUBindGroupLayout> {
        reflect_dom_object(
            Box::new(GPUBindGroupLayout::new_inherited(
                channel,
                bind_group_layout,
                bindings,
                valid,
            )),
            global,
            GPUBindGroupLayoutBinding::Wrap,
        )
    }
}

impl GPUBindGroupLayout {
    pub fn is_valid(&self) -> bool {
        self.valid.get()
    }

    pub fn id(&self) -> WebGPUBindGroupLayout {
        self.bind_group_layout
    }

    pub fn bindings(&self) -> &[GPUBindGroupLayoutBindings] {
        &self.bindings
    }
}

impl GPUBindGroupLayoutMethods for GPUBindGroupLayout {
    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn GetLabel(&self) -> Option<DOMString> {
        self.label.borrow().clone()
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn SetLabel(&self, value: Option<DOMString>) {
        *self.label.borrow_mut() = value;
    }
}
