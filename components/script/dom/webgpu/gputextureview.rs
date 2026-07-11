/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::{JSContext, NoGC};
use js::cell::JSCell;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};
use webgpu_traits::{WebGPU, WebGPURequest, WebGPUTextureView};

use crate::dom::bindings::codegen::Bindings::WebGPUBinding::GPUTextureViewMethods;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::USVString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::webgpu::gputexture::GPUTexture;

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
pub(crate) struct GPUTextureView {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "JSCell is hard to measure"]
    label: JSCell<USVString>,
    texture: Dom<GPUTexture>,
    droppable: DroppableGPUTextureView,
}

impl GPUTextureView {
    fn new_inherited(
        channel: WebGPU,
        texture_view: WebGPUTextureView,
        texture: &GPUTexture,
        label: USVString,
    ) -> GPUTextureView {
        Self {
            reflector_: Reflector::new(),
            texture: Dom::from_ref(texture),
            label: JSCell::new(label),
            droppable: DroppableGPUTextureView {
                channel,
                texture_view,
            },
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        channel: WebGPU,
        texture_view: WebGPUTextureView,
        texture: &GPUTexture,
        label: USVString,
    ) -> DomRoot<GPUTextureView> {
        reflect_dom_object_with_cx(
            Box::new(GPUTextureView::new_inherited(
                channel,
                texture_view,
                texture,
                label,
            )),
            global,
            cx,
        )
    }
}

impl GPUTextureView {
    pub(crate) fn id(&self) -> WebGPUTextureView {
        self.droppable.texture_view
    }
}

impl GPUTextureViewMethods<crate::DomTypeHolder> for GPUTextureView {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn Label(&self, no_gc: &NoGC) -> USVString {
        self.label.borrow(no_gc).clone()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn SetLabel(&self, no_gc_mut: &mut NoGC, value: USVString) {
        *self.label.borrow_mut(no_gc_mut) = value;
    }
}
