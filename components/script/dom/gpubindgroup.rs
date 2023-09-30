/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use webgpu::{WebGPUBindGroup, WebGPUDevice};

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::GPUBindGroupMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::USVString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::gpubindgrouplayout::GPUBindGroupLayout;

#[dom_struct]
pub struct GPUBindGroup {
    reflector_: Reflector,
    label: DomRefCell<USVString>,
    #[no_trace]
    bind_group: WebGPUBindGroup,
    #[no_trace]
    device: WebGPUDevice,
    layout: Dom<GPUBindGroupLayout>,
}

impl GPUBindGroup {
    fn new_inherited(
        bind_group: WebGPUBindGroup,
        device: WebGPUDevice,
        layout: &GPUBindGroupLayout,
        label: USVString,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            label: DomRefCell::new(label),
            bind_group,
            device,
            layout: Dom::from_ref(layout),
        }
    }

    pub fn new(
        global: &GlobalScope,
        bind_group: WebGPUBindGroup,
        device: WebGPUDevice,
        layout: &GPUBindGroupLayout,
        label: USVString,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(GPUBindGroup::new_inherited(
                bind_group, device, layout, label,
            )),
            global,
        )
    }
}

impl GPUBindGroup {
    pub fn id(&self) -> &WebGPUBindGroup {
        &self.bind_group
    }
}

impl GPUBindGroupMethods for GPUBindGroup {
    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn Label(&self) -> USVString {
        self.label.borrow().clone()
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn SetLabel(&self, value: USVString) {
        *self.label.borrow_mut() = value;
    }
}
