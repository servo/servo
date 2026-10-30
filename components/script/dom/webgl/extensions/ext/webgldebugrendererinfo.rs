/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::reflector::{Reflector, reflect_dom_object};

use super::{WebGLExtension, WebGLExtensionSpec, WebGLExtensions};
use crate::dom::bindings::codegen::Bindings::WEBGLDebugRendererInfoBinding::WEBGLDebugRendererInfoConstants;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::DomRoot;
use crate::dom::webgl::webglrenderingcontext::WebGLRenderingContext;

#[dom_struct]
pub(crate) struct WEBGLDebugRendererInfo {
    reflector_: Reflector,
}

impl WEBGLDebugRendererInfo {
    fn new_inherited() -> WEBGLDebugRendererInfo {
        Self {
            reflector_: Reflector::new(),
        }
    }
}

impl WebGLExtension for WEBGLDebugRendererInfo {
    type Extension = WEBGLDebugRendererInfo;
    fn new(cx: &mut JSContext, ctx: &WebGLRenderingContext) -> DomRoot<WEBGLDebugRendererInfo> {
        reflect_dom_object(cx, Box::new(Self::new_inherited()), &*ctx.global())
    }

    fn spec() -> WebGLExtensionSpec {
        WebGLExtensionSpec::All
    }

    fn is_supported(_ext: &WebGLExtensions) -> bool {
        true
    }

    fn enable(ext: &WebGLExtensions) {
        ext.enable_get_parameter_name(WEBGLDebugRendererInfoConstants::UNMASKED_VENDOR_WEBGL);
        ext.enable_get_parameter_name(WEBGLDebugRendererInfoConstants::UNMASKED_RENDERER_WEBGL);
    }

    fn name() -> &'static str {
        "WEBGL_debug_renderer_info"
    }
}
