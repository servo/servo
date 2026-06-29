/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};
use servo_canvas_traits::webgl::WebGLVersion;

use super::{WebGLExtension, WebGLExtensionSpec, WebGLExtensions};
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::DomRoot;
use crate::dom::webgl::extensions::oestexturefloat::OESTextureFloat;
use crate::dom::webgl::webglrenderingcontext::WebGLRenderingContext;

#[dom_struct]
pub(crate) struct WEBGLColorBufferFloat {
    reflector_: Reflector,
}

impl WEBGLColorBufferFloat {
    fn new_inherited() -> WEBGLColorBufferFloat {
        Self {
            reflector_: Reflector::new(),
        }
    }
}

impl WebGLExtension for WEBGLColorBufferFloat {
    type Extension = WEBGLColorBufferFloat;
    fn new(cx: &mut JSContext, ctx: &WebGLRenderingContext) -> DomRoot<WEBGLColorBufferFloat> {
        reflect_dom_object_with_cx(Box::new(Self::new_inherited()), &*ctx.global(), cx)
    }

    fn spec() -> WebGLExtensionSpec {
        WebGLExtensionSpec::Specific(WebGLVersion::WebGL1)
    }

    fn is_supported(ext: &WebGLExtensions) -> bool {
        OESTextureFloat::is_supported(ext)
    }

    fn enable(_ext: &WebGLExtensions) {}

    fn name() -> &'static str {
        "WEBGL_color_buffer_float"
    }
}
