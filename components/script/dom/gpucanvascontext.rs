/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use script_layout_interface::HTMLCanvasDataSource;

use crate::dom::bindings::codegen::Bindings::GPUCanvasContextBinding::GPUCanvasContextMethods;
use crate::dom::bindings::codegen::UnionTypes;
use crate::dom::bindings::reflector::Reflector;
use crate::dom::bindings::root::LayoutDom;
use crate::dom::htmlcanvaselement::LayoutCanvasRenderingContextHelpers;

#[dom_struct]
pub(crate) struct GPUCanvasContext {
    reflector_: Reflector,
}

impl GPUCanvasContext {
    #[allow(dead_code)]
    fn new_inherited() -> Self {
        unimplemented!()
    }
}

impl GPUCanvasContextMethods<crate::DomTypeHolder> for GPUCanvasContext {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpucanvascontext-canvas>
    fn Canvas(&self) -> UnionTypes::HTMLCanvasElementOrOffscreenCanvas {
        unimplemented!()
    }
}

impl LayoutCanvasRenderingContextHelpers for LayoutDom<'_, GPUCanvasContext> {
    fn canvas_data_source(self) -> HTMLCanvasDataSource {
        unimplemented!()
    }
}
