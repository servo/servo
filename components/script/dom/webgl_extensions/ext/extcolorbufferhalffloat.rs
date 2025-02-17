/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use canvas_traits::webgl::WebGLVersion;
use dom_struct::dom_struct;

use super::{WebGLExtension, WebGLExtensionSpec, WebGLExtensions};
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::webgl_extensions::ext::oestexturehalffloat::OESTextureHalfFloat;
use crate::dom::webglrenderingcontext::WebGLRenderingContext;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct EXTColorBufferHalfFloat {
    reflector_: Reflector,
}

impl EXTColorBufferHalfFloat {
    fn new_inherited() -> EXTColorBufferHalfFloat {
        Self {
            reflector_: Reflector::new(),
        }
    }
}

impl WebGLExtension for EXTColorBufferHalfFloat {
    type Extension = EXTColorBufferHalfFloat;
    fn new(ctx: &WebGLRenderingContext, can_gc: CanGc) -> DomRoot<EXTColorBufferHalfFloat> {
        reflect_dom_object(
            Box::new(EXTColorBufferHalfFloat::new_inherited()),
            &*ctx.global(),
            can_gc,
        )
    }

    fn spec() -> WebGLExtensionSpec {
        WebGLExtensionSpec::Specific(WebGLVersion::WebGL1)
    }

    fn is_supported(ext: &WebGLExtensions) -> bool {
        OESTextureHalfFloat::is_supported(ext)
    }

    fn enable(_ext: &WebGLExtensions) {}

    fn name() -> &'static str {
        "EXT_color_buffer_half_float"
    }
}
