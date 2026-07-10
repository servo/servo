/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::{JSContext, NoGC};
use script_bindings::cell::DomRefCell;
use script_bindings::codegen::GenericBindings::WebGPUBinding::{GPUDeviceMethods, GPUQueryType};
use script_bindings::error::{Error, Fallible};
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};
use script_bindings::root::DomRoot;
use webgpu_traits::{WebGPU, WebGPUQuerySet, WebGPURequest};

use crate::conversions::Convert;
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::{
    GPUQuerySetDescriptor, GPUQuerySetMethods,
};
use crate::dom::bindings::reflector::DomGlobal as _;
use crate::dom::bindings::str::USVString;
use crate::dom::types::{GPUDevice, GlobalScope};

#[derive(JSTraceable, MallocSizeOf)]
struct DroppableGPUQuerySet {
    #[no_trace]
    channel: WebGPU,
    #[no_trace]
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
pub(crate) struct GPUQuerySet {
    reflector_: Reflector,
    droppable: DroppableGPUQuerySet,
    label: DomRefCell<USVString>,
    r#type: GPUQueryType,
    count: u32,
}

impl GPUQuerySet {
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
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        label: USVString,
        channel: WebGPU,
        query_set: WebGPUQuerySet,
        r#type: GPUQueryType,
        count: u32,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_cx(
            Box::new(GPUQuerySet::new_inherited(
                label, channel, query_set, r#type, count,
            )),
            global,
            cx,
        )
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createqueryset>
    pub(crate) fn create(
        cx: &mut JSContext,
        device: &GPUDevice,
        descriptor: &GPUQuerySetDescriptor,
    ) -> Fallible<DomRoot<Self>> {
        // 1. If descriptor.type is "timestamp", but "timestamp-query" is not enabled for this:
        if descriptor.type_ == GPUQueryType::Timestamp
            && !device
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
        let query_set_id = device.global().wgpu_id_hub().create_query_set_id();
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
            &device.global(),
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

impl GPUQuerySetMethods<crate::DomTypeHolder> for GPUQuerySet {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpuqueryset-destroy>
    fn Destroy(&self) {
        // TODO: wgpu does not implement proper destroy for query sets.
        // Waiting for https://github.com/gfx-rs/wgpu/pull/9671 to be released.
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
