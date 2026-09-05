/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::marker::PhantomData;

use dom_struct::dom_struct;
use js::context::{JSContext, NoGC};
use log::warn;
use malloc_size_of_derive::MallocSizeOf;
use script_bindings::DomTypes;
use script_bindings::cell::DomRefCell;
use script_bindings::codegen::GenericBindings::WebGPUBinding::{
    GPUDeviceMethods, GPUQuerySetDescriptor, GPUQuerySetMethods, GPUQuerySetWrap, GPUQueryType,
};
use script_bindings::error::{Error, Fallible};
use script_bindings::reflector::{DomGlobalGeneric, Reflector, reflect_dom_object_with_wrap};
use script_bindings::root::DomRoot;
use webgpu_traits::{WebGPU, WebGPUQuerySet, WebGPURequest};

use crate::JSTraceable;
use crate::dom::bindings::str::USVString;
use crate::gpuconvert::WebGPUConvert;
use crate::traits::{Equivalence, GPUDeviceTrait, WebGPUGlobalTrait};

#[derive(MallocSizeOf)]
struct DroppableGPUQuerySet {
    channel: WebGPU,
    query_set: WebGPUQuerySet,
}

impl Drop for DroppableGPUQuerySet {
    fn drop(&mut self) {
        if let Err(error) = self
            .channel
            .0
            .send(WebGPURequest::DropQuerySet(self.query_set.0))
        {
            warn!(
                "Failed to send WebGPURequest::DropQuerySet({:?}) ({error})",
                self.query_set.0
            );
        }
    }
}

#[dom_struct]
pub struct GPUQuerySet<D: DomTypes> {
    reflector_: Reflector,
    #[no_trace]
    droppable: DroppableGPUQuerySet,
    label: DomRefCell<USVString>,
    r#type: GPUQueryType,
    count: u32,
    #[no_trace = "PhantomData does not exist"]
    phantom: PhantomData<D>,
}

impl<D> GPUQuerySet<D>
where
    D: Equivalence,
    D::GPUDevice: GPUDeviceTrait<D>,
    D::GlobalScope: WebGPUGlobalTrait,
{
    pub(crate) fn new_inherited(
        label: USVString,
        channel: WebGPU,
        query_set: WebGPUQuerySet,
        r#type: GPUQueryType,
        count: u32,
    ) -> Self {
        GPUQuerySet {
            reflector_: Reflector::new(),
            label: DomRefCell::new(label),
            droppable: DroppableGPUQuerySet { channel, query_set },
            r#type,
            count,
            phantom: PhantomData,
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        global: &D::GlobalScope,
        label: USVString,
        channel: WebGPU,
        query_set: WebGPUQuerySet,
        r#type: GPUQueryType,
        count: u32,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_wrap::<D, _, _>(
            Box::new(GPUQuerySet::new_inherited(
                label, channel, query_set, r#type, count,
            )),
            global,
            cx,
            GPUQuerySetWrap::<D>,
        )
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createqueryset>
    pub fn create(
        cx: &mut JSContext,
        device: &D::GPUDevice,
        descriptor: &GPUQuerySetDescriptor,
    ) -> Fallible<DomRoot<Self>> {
        // 1. If descriptor.type is "timestamp", but "timestamp-query" is not enabled for this:
        if descriptor.type_ == GPUQueryType::Timestamp &&
            !device
                .Features()
                .wgpu_features()
                .contains(wgpu_types::Features::TIMESTAMP_QUERY)
        {
            // Throw a TypeError.
            return Err(Error::Type(
                c"The device does not support timestamp queries".to_owned(),
            ));
        }
        // 2. Let q be ! create a new WebGPU object(this, GPUQuerySet, descriptor).
        let query_set_id = device
            .global_from_reflector()
            .global_wgpu_id_hub()
            .create_query_set_id();
        // 5. Issue the initialization steps on the Device timeline of this.
        let channel = device.channel();
        if let Err(error) = channel.0.send(WebGPURequest::CreateQuerySet {
            device_id: device.id().0,
            query_set_id,
            descriptor: descriptor.convert(),
        }) {
            warn!("Failed to send WebGPURequest::CreateQuerySet: {error}");
        }
        // 6. Return q
        Ok(Self::new(
            cx,
            &device.global_from_reflector(),
            descriptor.parent.label.clone(),
            channel,
            WebGPUQuerySet(query_set_id),
            // 3. Set q.type to descriptor.type.
            descriptor.type_,
            // 4. Set q.count to descriptor.count.
            descriptor.count,
        ))
    }

    pub(crate) fn id(&self) -> WebGPUQuerySet {
        self.droppable.query_set
    }
}

impl<D> GPUQuerySetMethods<D> for GPUQuerySet<D>
where
    D: Equivalence,
    D::GPUDevice: GPUDeviceTrait<D>,
    D::GlobalScope: WebGPUGlobalTrait,
{
    /// <https://gpuweb.github.io/gpuweb/#dom-gpuqueryset-destroy>
    fn Destroy(&self) {
        // 1. Issue the subsequent steps on the device timeline.
        if let Err(error) = self
            .droppable
            .channel
            .0
            .send(WebGPURequest::DestroyQuerySet(self.id().0))
        {
            warn!(
                "Failed to send WebGPURequest::DestroyQuerySet({:?}) ({error})",
                self.id().0
            );
        }
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn Label(&self) -> USVString {
        self.label.borrow().clone()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn SetLabel(&self, no_gc: &NoGC, value: USVString) {
        *self.label.safe_borrow_mut(no_gc) = value;
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuqueryset-type>
    fn Type(&self) -> GPUQueryType {
        self.r#type
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuqueryset-count>
    fn Count(&self) -> u32 {
        self.count
    }
}
