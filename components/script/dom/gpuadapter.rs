/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::GPUAdapterBinding::{self, GPUAdapterMethods};
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::JSContext as SafeJSContext;
use dom_struct::dom_struct;
use js::jsapi::{Heap, JSObject};
use std::ptr::NonNull;
use webgpu::WebGPUAdapter;

#[dom_struct]
pub struct GPUAdapter {
    reflector_: Reflector,
    name: DOMString,
    #[ignore_malloc_size_of = "mozjs"]
    extensions: Heap<*mut JSObject>,
    adapter: WebGPUAdapter,
}

impl GPUAdapter {
    pub fn new_inherited(
        name: DOMString,
        extensions: Heap<*mut JSObject>,
        adapter: WebGPUAdapter,
    ) -> GPUAdapter {
        GPUAdapter {
            reflector_: Reflector::new(),
            name,
            extensions,
            adapter,
        }
    }

    pub fn new(
        global: &GlobalScope,
        name: DOMString,
        extensions: Heap<*mut JSObject>,
        adapter: WebGPUAdapter,
    ) -> DomRoot<GPUAdapter> {
        reflect_dom_object(
            Box::new(GPUAdapter::new_inherited(name, extensions, adapter)),
            global,
            GPUAdapterBinding::Wrap,
        )
    }
}

impl GPUAdapterMethods for GPUAdapter {
    // https://gpuweb.github.io/gpuweb/#dom-gpuadapter-name
    fn Name(&self) -> DOMString {
        self.name.clone()
    }

    // https://gpuweb.github.io/gpuweb/#dom-gpuadapter-extensions
    fn Extensions(&self, _cx: SafeJSContext) -> NonNull<JSObject> {
        NonNull::new(self.extensions.get()).unwrap()
    }
}
