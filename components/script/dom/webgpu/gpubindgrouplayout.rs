/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;

use dom_struct::dom_struct;
use webgpu::wgc::binding_model::BindGroupLayoutDescriptor;
use webgpu::{WebGPU, WebGPUBindGroupLayout, WebGPURequest};

use crate::conversions::Convert;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::{
    GPUBindGroupLayoutDescriptor, GPUBindGroupLayoutMethods,
};
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::USVString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::webgpu::gpuconvert::convert_bind_group_layout_entry;
use crate::dom::webgpu::gpudevice::GPUDevice;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct GPUBindGroupLayout {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "channels are hard"]
    #[no_trace]
    channel: WebGPU,
    label: DomRefCell<USVString>,
    #[no_trace]
    bind_group_layout: WebGPUBindGroupLayout,
}

impl GPUBindGroupLayout {
    fn new_inherited(
        channel: WebGPU,
        bind_group_layout: WebGPUBindGroupLayout,
        label: USVString,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            channel,
            label: DomRefCell::new(label),
            bind_group_layout,
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        channel: WebGPU,
        bind_group_layout: WebGPUBindGroupLayout,
        label: USVString,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(GPUBindGroupLayout::new_inherited(
                channel,
                bind_group_layout,
                label,
            )),
            global,
            can_gc,
        )
    }
}

impl GPUBindGroupLayout {
    pub(crate) fn id(&self) -> WebGPUBindGroupLayout {
        self.bind_group_layout
    }

    /// <https://gpuweb.github.io/gpuweb/#GPUDevice-createBindGroupLayout>
    pub(crate) fn create(
        device: &GPUDevice,
        descriptor: &GPUBindGroupLayoutDescriptor,
    ) -> Fallible<DomRoot<GPUBindGroupLayout>> {
        let entries = descriptor
            .entries
            .iter()
            .map(|bgle| convert_bind_group_layout_entry(bgle, device))
            .collect::<Fallible<Result<Vec<_>, _>>>()?;

        let desc = match entries {
            Ok(entries) => Some(BindGroupLayoutDescriptor {
                label: (&descriptor.parent).convert(),
                entries: Cow::Owned(entries),
            }),
            Err(error) => {
                device.dispatch_error(error);
                None
            },
        };

        let bind_group_layout_id = device.global().wgpu_id_hub().create_bind_group_layout_id();
        device
            .channel()
            .0
            .send(WebGPURequest::CreateBindGroupLayout {
                device_id: device.id().0,
                bind_group_layout_id,
                descriptor: desc,
            })
            .expect("Failed to create WebGPU BindGroupLayout");

        let bgl = WebGPUBindGroupLayout(bind_group_layout_id);

        Ok(GPUBindGroupLayout::new(
            &device.global(),
            device.channel().clone(),
            bgl,
            descriptor.parent.label.clone(),
            CanGc::note(),
        ))
    }
}

impl Drop for GPUBindGroupLayout {
    fn drop(&mut self) {
        if let Err(e) = self
            .channel
            .0
            .send(WebGPURequest::DropBindGroupLayout(self.bind_group_layout.0))
        {
            warn!(
                "Failed to send WebGPURequest::DropBindGroupLayout({:?}) ({})",
                self.bind_group_layout.0, e
            );
        };
    }
}

impl GPUBindGroupLayoutMethods<crate::DomTypeHolder> for GPUBindGroupLayout {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn Label(&self) -> USVString {
        self.label.borrow().clone()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn SetLabel(&self, value: USVString) {
        *self.label.borrow_mut() = value;
    }
}
