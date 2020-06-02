/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::GPUBindGroupBinding::{
    GPUBindGroupEntry, GPUBindGroupMethods,
};
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::gpubindgrouplayout::GPUBindGroupLayout;
use crate::dom::gpubuffer::GPUBuffer;
use crate::dom::gputextureview::TextureSubresource;
use dom_struct::dom_struct;
use std::cell::Cell;
use std::collections::HashMap;
use webgpu::{WebGPUBindGroup, WebGPUDevice};

#[dom_struct]
pub struct GPUBindGroup {
    reflector_: Reflector,
    label: DomRefCell<Option<DOMString>>,
    bind_group: WebGPUBindGroup,
    device: WebGPUDevice,
    layout: Dom<GPUBindGroupLayout>,
    #[ignore_malloc_size_of = "defined in webgpu"]
    entries: Vec<GPUBindGroupEntry>,
    used_buffers: HashMap<Dom<GPUBuffer>, u32>,
    used_textures: HashMap<TextureSubresource, u32>,
    valid: Cell<bool>,
}

impl GPUBindGroup {
    fn new_inherited(
        bind_group: WebGPUBindGroup,
        device: WebGPUDevice,
        valid: bool,
        entries: Vec<GPUBindGroupEntry>,
        layout: &GPUBindGroupLayout,
        used_buffers: HashMap<DomRoot<GPUBuffer>, u32>,
        used_textures: HashMap<TextureSubresource, u32>,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            label: DomRefCell::new(None),
            bind_group,
            device,
            valid: Cell::new(valid),
            layout: Dom::from_ref(layout),
            entries,
            used_buffers: used_buffers
                .into_iter()
                .map(|(key, value)| (Dom::from_ref(&*key), value))
                .collect(),
            used_textures,
        }
    }

    pub fn new(
        global: &GlobalScope,
        bind_group: WebGPUBindGroup,
        device: WebGPUDevice,
        valid: bool,
        entries: Vec<GPUBindGroupEntry>,
        layout: &GPUBindGroupLayout,
        used_buffers: HashMap<DomRoot<GPUBuffer>, u32>,
        used_textures: HashMap<TextureSubresource, u32>,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(GPUBindGroup::new_inherited(
                bind_group,
                device,
                valid,
                entries,
                layout,
                used_buffers,
                used_textures,
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
    fn GetLabel(&self) -> Option<DOMString> {
        self.label.borrow().clone()
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn SetLabel(&self, value: Option<DOMString>) {
        *self.label.borrow_mut() = value;
    }
}
