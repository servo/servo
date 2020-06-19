/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::GPUSamplerBinding::GPUSamplerMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use dom_struct::dom_struct;
use std::cell::Cell;
use webgpu::{WebGPUDevice, WebGPUSampler};

#[dom_struct]
pub struct GPUSampler {
    reflector_: Reflector,
    label: DomRefCell<Option<DOMString>>,
    device: WebGPUDevice,
    compare_enable: bool,
    sampler: WebGPUSampler,
    valid: Cell<bool>,
}

impl GPUSampler {
    fn new_inherited(
        device: WebGPUDevice,
        compare_enable: bool,
        sampler: WebGPUSampler,
        valid: bool,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            label: DomRefCell::new(None),
            valid: Cell::new(valid),
            device,
            sampler,
            compare_enable,
        }
    }

    pub fn new(
        global: &GlobalScope,
        device: WebGPUDevice,
        compare_enable: bool,
        sampler: WebGPUSampler,
        valid: bool,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(GPUSampler::new_inherited(
                device,
                compare_enable,
                sampler,
                valid,
            )),
            global,
        )
    }
}

impl GPUSampler {
    pub fn id(&self) -> WebGPUSampler {
        self.sampler
    }

    pub fn is_valid(&self) -> bool {
        self.valid.get()
    }
}

impl GPUSamplerMethods for GPUSampler {
    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn GetLabel(&self) -> Option<DOMString> {
        self.label.borrow().clone()
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn SetLabel(&self, value: Option<DOMString>) {
        *self.label.borrow_mut() = value;
    }
}
