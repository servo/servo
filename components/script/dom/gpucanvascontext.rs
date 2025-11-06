/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use webrender_api::ImageKey;

use crate::dom::bindings::codegen::Bindings::GPUCanvasContextBinding::GPUCanvasContextMethods;
use crate::dom::bindings::codegen::UnionTypes::HTMLCanvasElementOrOffscreenCanvas as RootedHTMLCanvasElementOrOffscreenCanvas;
use crate::dom::bindings::reflector::Reflector;
use crate::dom::bindings::root::LayoutDom;

#[dom_struct]
pub(crate) struct GPUCanvasContext {
    reflector_: Reflector,
}

impl GPUCanvasContext {
    #[expect(dead_code)]
    fn new_inherited() -> Self {
        unimplemented!()
    }
}

impl GPUCanvasContextMethods<crate::DomTypeHolder> for GPUCanvasContext {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpucanvascontext-canvas>
    fn Canvas(&self) -> RootedHTMLCanvasElementOrOffscreenCanvas {
        unimplemented!()
    }
}
