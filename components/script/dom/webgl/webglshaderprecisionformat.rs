/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::reflector::{Reflector, reflect_dom_object, reflect_dom_object_with_cx};

use crate::dom::bindings::codegen::Bindings::WebGLShaderPrecisionFormatBinding::WebGLShaderPrecisionFormatMethods;
use crate::dom::bindings::root::DomRoot;
use crate::dom::window::Window;

#[dom_struct]
pub(crate) struct WebGLShaderPrecisionFormat {
    reflector_: Reflector,
    range_min: i32,
    range_max: i32,
    precision: i32,
}

impl WebGLShaderPrecisionFormat {
    fn new_inherited(range_min: i32, range_max: i32, precision: i32) -> WebGLShaderPrecisionFormat {
        WebGLShaderPrecisionFormat {
            reflector_: Reflector::new(),
            range_min,
            range_max,
            precision,
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        window: &Window,
        range_min: i32,
        range_max: i32,
        precision: i32,
    ) -> DomRoot<WebGLShaderPrecisionFormat> {
        reflect_dom_object_with_cx(
            Box::new(WebGLShaderPrecisionFormat::new_inherited(
                range_min, range_max, precision,
            )),
            window,
            cx,
        )
    }
}

impl WebGLShaderPrecisionFormatMethods<crate::DomTypeHolder> for WebGLShaderPrecisionFormat {
    /// <https://www.khronos.org/registry/webgl/specs/1.0/#5.12.1>
    fn RangeMin(&self) -> i32 {
        self.range_min
    }

    /// <https://www.khronos.org/registry/webgl/specs/1.0/#5.12.1>
    fn RangeMax(&self) -> i32 {
        self.range_max
    }

    /// <https://www.khronos.org/registry/webgl/specs/1.0/#5.12.1>
    fn Precision(&self) -> i32 {
        self.precision
    }
}
