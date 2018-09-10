/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use canvas_traits::webgl::WebGLVersion;
use dom::bindings::codegen::Bindings::WEBGLColorBufferFloatBinding;
use dom::bindings::reflector::{DomObject, Reflector, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::webgl_extensions::ext::oestexturefloat::OESTextureFloat;
use dom::webglrenderingcontext::WebGLRenderingContext;
use dom_struct::dom_struct;
use super::{WebGLExtension, WebGLExtensions, WebGLExtensionSpec};

#[dom_struct]
pub struct WEBGLColorBufferFloat {
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
    fn new(ctx: &WebGLRenderingContext) -> DomRoot<WEBGLColorBufferFloat> {
        reflect_dom_object(Box::new(WEBGLColorBufferFloat::new_inherited()),
                           &*ctx.global(),
                           WEBGLColorBufferFloatBinding::Wrap)
    }

    fn spec() -> WebGLExtensionSpec {
        WebGLExtensionSpec::Specific(WebGLVersion::WebGL1)
    }

    fn is_supported(ext: &WebGLExtensions) -> bool {
        OESTextureFloat::is_supported(ext)
    }

    fn enable(_ext: &WebGLExtensions) {
    }

    fn name() -> &'static str {
        "WEBGL_color_buffer_float"
    }
}
