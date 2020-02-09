/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::GPUBindGroupBinding::{
    GPUBindGroupBinding, GPUBindGroupMethods,
};
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use dom_struct::dom_struct;
use std::cell::Cell;
use webgpu::WebGPUBindGroup;

#[dom_struct]
pub struct GPUBindGroup {
    reflector_: Reflector,
    label: DomRefCell<Option<DOMString>>,
    bind_group: WebGPUBindGroup,
    valid: Cell<bool>,
}

impl GPUBindGroup {
    fn new_inherited(bind_group: WebGPUBindGroup, valid: bool) -> GPUBindGroup {
        Self {
            reflector_: Reflector::new(),
            label: DomRefCell::new(None),
            bind_group,
            valid: Cell::new(valid),
        }
    }

    pub fn new(
        global: &GlobalScope,
        bind_group: WebGPUBindGroup,
        valid: bool,
    ) -> DomRoot<GPUBindGroup> {
        reflect_dom_object(
            Box::new(GPUBindGroup::new_inherited(bind_group, valid)),
            global,
            GPUBindGroupBinding::Wrap,
        )
    }
}

impl GPUBindGroupMethods for GPUBindGroup {
    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn GetLabel(&self) -> Option<DOMString> {
        self.label.borrow().clone()
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn SetLabel(&self, value: Option<DOMString>) {
        *self.label.borrow_mut() = value;
    }
}
