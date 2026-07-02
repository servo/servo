/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::cell::DomRefCell;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};
use webgpu_traits::{WebGPU, WebGPUDevice, WebGPURenderBundle, WebGPURequest};

use crate::dom::bindings::codegen::Bindings::WebGPUBinding::GPURenderBundleMethods;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::USVString;
use crate::dom::globalscope::GlobalScope;

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
pub(crate) struct GPURenderBundle {
    reflector_: Reflector,
    #[no_trace]
    device: WebGPUDevice,
    label: DomRefCell<USVString>,
    droppable: DroppableGPURenderBundle,
}

impl GPURenderBundle {
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
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        render_bundle: WebGPURenderBundle,
        device: WebGPUDevice,
        channel: WebGPU,
        label: USVString,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_cx(
            Box::new(GPURenderBundle::new_inherited(
                render_bundle,
                device,
                channel,
                label,
            )),
            global,
            cx,
        )
    }
}

impl GPURenderBundle {
    pub(crate) fn id(&self) -> WebGPURenderBundle {
        self.droppable.render_bundle
    }
}

impl GPURenderBundleMethods<crate::DomTypeHolder> for GPURenderBundle {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn Label(&self) -> USVString {
        self.label.borrow().clone()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn SetLabel(&self, value: USVString) {
        *self.label.borrow_mut() = value;
    }
}
