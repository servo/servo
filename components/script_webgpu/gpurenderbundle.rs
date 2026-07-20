/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::marker::PhantomData;

use dom_struct::dom_struct;
use js::context::{JSContext, NoGC};
use jstraceable_derive::JSTraceable;
use log::warn;
use malloc_size_of_derive::MallocSizeOf;
use script_bindings::DomTypes;
use script_bindings::cell::DomRefCell;
use script_bindings::codegen::GenericBindings::WebGPUBinding::{
    GPURenderBundleMethods, GPURenderBundleWrap,
};
use script_bindings::reflector::{Reflector, reflect_dom_object_with_wrap};
use webgpu_traits::{WebGPU, WebGPUDevice, WebGPURenderBundle, WebGPURequest};

use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::USVString;

#[derive(JSTraceable, MallocSizeOf)]
struct DroppableGPURenderBundle {
    #[no_trace]
    channel: WebGPU,
    #[no_trace]
    render_bundle: WebGPURenderBundle,
}

impl Drop for DroppableGPURenderBundle {
    fn drop(&mut self) {
        if let Err(e) = self
            .channel
            .0
            .send(WebGPURequest::DropRenderBundle(self.render_bundle.0))
        {
            warn!(
                "Failed to send DropRenderBundle ({:?}) ({})",
                self.render_bundle.0, e
            );
        }
    }
}

#[dom_struct]
pub struct GPURenderBundle<D: DomTypes> {
    reflector_: Reflector,
    #[no_trace]
    device: WebGPUDevice,
    label: DomRefCell<USVString>,
    droppable: DroppableGPURenderBundle,
    #[no_trace = "PhantomData does not exist"]
    phantom: PhantomData<D>,
}

impl<D> GPURenderBundle<D>
where
    D: DomTypes<GPURenderBundle = GPURenderBundle<D>>,
{
    fn new_inherited(
        render_bundle: WebGPURenderBundle,
        device: WebGPUDevice,
        channel: WebGPU,
        label: USVString,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            device,
            label: DomRefCell::new(label),
            droppable: DroppableGPURenderBundle {
                channel,
                render_bundle,
            },
            phantom: PhantomData,
        }
    }

    pub fn new(
        cx: &mut JSContext,
        global: &D::GlobalScope,
        render_bundle: WebGPURenderBundle,
        device: WebGPUDevice,
        channel: WebGPU,
        label: USVString,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_wrap::<D, _, _>(
            Box::new(GPURenderBundle::new_inherited(
                render_bundle,
                device,
                channel,
                label,
            )),
            global,
            cx,
            GPURenderBundleWrap::<D>,
        )
    }
}

impl<D: DomTypes> GPURenderBundle<D> {
    pub fn id(&self) -> WebGPURenderBundle {
        self.droppable.render_bundle
    }
}

impl<D: DomTypes> GPURenderBundleMethods<D> for GPURenderBundle<D> {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn Label(&self) -> USVString {
        self.label.borrow().clone()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn SetLabel(&self, no_gc: &NoGC, value: USVString) {
        *self.label.safe_borrow_mut(no_gc) = value;
    }
}
