/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::GPURenderBundleBinding::GPURenderBundleMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::USVString;
use crate::dom::globalscope::GlobalScope;
use dom_struct::dom_struct;
use webgpu::{WebGPU, WebGPUDevice, WebGPURenderBundle};

#[dom_struct]
pub struct GPURenderBundle {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "channels are hard"]
    channel: WebGPU,
    device: WebGPUDevice,
    render_bundle: WebGPURenderBundle,
    label: DomRefCell<Option<USVString>>,
}

impl GPURenderBundle {
    fn new_inherited(
        render_bundle: WebGPURenderBundle,
        device: WebGPUDevice,
        channel: WebGPU,
        label: Option<USVString>,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            render_bundle,
            device,
            channel,
            label: DomRefCell::new(label),
        }
    }

    pub fn new(
        global: &GlobalScope,
        render_bundle: WebGPURenderBundle,
        device: WebGPUDevice,
        channel: WebGPU,
        label: Option<USVString>,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(GPURenderBundle::new_inherited(
                render_bundle,
                device,
                channel,
                label,
            )),
            global,
        )
    }
}

impl GPURenderBundle {
    pub fn id(&self) -> WebGPURenderBundle {
        self.render_bundle
    }
}

impl GPURenderBundleMethods for GPURenderBundle {
    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn GetLabel(&self) -> Option<USVString> {
        self.label.borrow().clone()
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn SetLabel(&self, value: Option<USVString>) {
        *self.label.borrow_mut() = value;
    }
}
