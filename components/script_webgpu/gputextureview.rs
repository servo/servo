/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::{JSContext, NoGC};
use log::warn;
use malloc_size_of_derive::MallocSizeOf;
use script_bindings::DomTypes;
use script_bindings::cell::DomRefCell;
use script_bindings::codegen::GenericBindings::WebGPUBinding::{
    GPUTextureViewMethods, GPUTextureViewWrap,
};
use script_bindings::reflector::{Reflector, reflect_dom_object_with_wrap};
use webgpu_traits::{WebGPU, WebGPURequest, WebGPUTextureView};

use crate::JSTraceable;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::USVString;
use crate::gputexture::GPUTexture;
use crate::traits::Equivalence;

#[derive(JSTraceable, MallocSizeOf)]
struct DroppableGPUTextureView {
    #[ignore_malloc_size_of = "defined in webgpu"]
    #[no_trace]
    channel: WebGPU,
    #[no_trace]
    texture_view: WebGPUTextureView,
}

impl Drop for DroppableGPUTextureView {
    fn drop(&mut self) {
        if let Err(e) = self
            .channel
            .0
            .send(WebGPURequest::DropTextureView(self.texture_view.0))
        {
            warn!(
                "Failed to send DropTextureView ({:?}) ({})",
                self.texture_view.0, e
            );
        }
    }
}

#[dom_struct]
pub struct GPUTextureView<D: DomTypes> {
    reflector_: Reflector,
    label: DomRefCell<USVString>,
    texture: Dom<GPUTexture<D>>,
    droppable: DroppableGPUTextureView,
}

impl<D: Equivalence> GPUTextureView<D> {
    fn new_inherited(
        channel: WebGPU,
        texture_view: WebGPUTextureView,
        texture: &GPUTexture<D>,
        label: USVString,
    ) -> GPUTextureView<D> {
        Self {
            reflector_: Reflector::new(),
            texture: Dom::from_ref(texture),
            label: DomRefCell::new(label),
            droppable: DroppableGPUTextureView {
                channel,
                texture_view,
            },
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        global: &D::GlobalScope,
        channel: WebGPU,
        texture_view: WebGPUTextureView,
        texture: &GPUTexture<D>,
        label: USVString,
    ) -> DomRoot<GPUTextureView<D>> {
        reflect_dom_object_with_wrap::<D, _, _>(
            Box::new(GPUTextureView::new_inherited(
                channel,
                texture_view,
                texture,
                label,
            )),
            global,
            cx,
            GPUTextureViewWrap::<D>,
        )
    }
}

impl<D: DomTypes> GPUTextureView<D> {
    pub(crate) fn id(&self) -> WebGPUTextureView {
        self.droppable.texture_view
    }
}

impl<D: DomTypes> GPUTextureViewMethods<D> for GPUTextureView<D> {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn Label(&self) -> USVString {
        self.label.borrow().clone()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn SetLabel(&self, no_gc: &NoGC, value: USVString) {
        *self.label.safe_borrow_mut(no_gc) = value;
    }
}
