/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use core::iter::FromIterator;
use core::nonzero::NonZero;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::OESTextureHalfFloatBinding::OESTextureHalfFloatConstants;
use dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLRenderingContextConstants as constants;
use dom::bindings::js::Root;
use dom::bindings::trace::JSTraceable;
use dom::webglrenderingcontext::WebGLRenderingContext;
use gleam::gl::GLenum;
use heapsize::HeapSizeOf;
use js::jsapi::{JSContext, JSObject};
use js::jsval::JSVal;
use ref_filter_map::ref_filter_map;
use std::cell::Ref;
use std::collections::{HashMap, HashSet};
use super::{ext, WebGLExtension};
use super::wrapper::{WebGLExtensionWrapper, TypedWebGLExtensionWrapper};
use webrender_api::WebGLError;

// Data types that are implemented for texImage2D and texSubImage2D in WebGLRenderingContext
// but must trigger a InvalidValue error until the related WebGL Extensions are enabled.
// Example: https://www.khronos.org/registry/webgl/extensions/OES_texture_float/
const DEFAULT_DISABLED_TEX_TYPES: [GLenum; 2] = [
    constants::FLOAT, OESTextureHalfFloatConstants::HALF_FLOAT_OES
];

// Data types that are implemented for textures in WebGLRenderingContext
// but not allowed to use with linear filtering until the related WebGL Extensions are enabled.
// Example: https://www.khronos.org/registry/webgl/extensions/OES_texture_float_linear/
const DEFAULT_NOT_FILTERABLE_TEX_TYPES: [GLenum; 2] = [
    constants::FLOAT, OESTextureHalfFloatConstants::HALF_FLOAT_OES
];

/// WebGL features that are enabled/disabled by WebGL Extensions.
#[derive(JSTraceable, HeapSizeOf)]
struct WebGLExtensionFeatures {
    gl_extensions: HashSet<String>,
    disabled_tex_types: HashSet<GLenum>,
    not_filterable_tex_types: HashSet<GLenum>,
    effective_tex_internal_formats: HashMap<TexFormatType, u32>,
    query_parameter_handlers: HashMap<GLenum, WebGLQueryParameterHandler>
}

impl Default for WebGLExtensionFeatures {
    fn default() -> WebGLExtensionFeatures {
        WebGLExtensionFeatures {
            gl_extensions: HashSet::new(),
            disabled_tex_types: DEFAULT_DISABLED_TEX_TYPES.iter().cloned().collect(),
            not_filterable_tex_types: DEFAULT_NOT_FILTERABLE_TEX_TYPES.iter().cloned().collect(),
            effective_tex_internal_formats: HashMap::new(),
            query_parameter_handlers: HashMap::new()
        }
    }
}

/// Handles the list of implemented, supported and enabled WebGL extensions.
#[must_root]
#[derive(JSTraceable, HeapSizeOf)]
pub struct WebGLExtensions {
    extensions: DOMRefCell<HashMap<String, Box<WebGLExtensionWrapper>>>,
    features: DOMRefCell<WebGLExtensionFeatures>,
}

impl WebGLExtensions {
    pub fn new() -> WebGLExtensions {
        Self {
            extensions: DOMRefCell::new(HashMap::new()),
            features: DOMRefCell::new(Default::default())
        }
    }

    pub fn init_once<F>(&self, cb: F) where F: FnOnce() -> String {
        if self.extensions.borrow().len() == 0 {
            let gl_str = cb();
            self.features.borrow_mut().gl_extensions = HashSet::from_iter(gl_str.split(&[',', ' '][..])
                                                                                .map(|s| s.into()));
            self.register_all_extensions();
        }
    }

    pub fn register<T:'static + WebGLExtension + JSTraceable + HeapSizeOf>(&self) {
        let name = T::name().to_uppercase();
        self.extensions.borrow_mut().insert(name, box TypedWebGLExtensionWrapper::<T>::new());
    }

    pub fn get_suported_extensions(&self) -> Vec<&'static str> {
        self.extensions.borrow().iter()
                                .filter(|ref v| v.1.is_supported(&self))
                                .map(|ref v| v.1.name())
                                .collect()
    }

    pub fn get_or_init_extension(&self, name: &str, ctx: &WebGLRenderingContext) -> Option<NonZero<*mut JSObject>> {
        let name = name.to_uppercase();
        self.extensions.borrow().get(&name).and_then(|extension| {
            if extension.is_supported(self) {
                Some(extension.instance_or_init(ctx, self))
            } else {
                None
            }
        })
    }

    pub fn get_dom_object<T>(&self) -> Option<Root<T::Extension>>
           where T: 'static + WebGLExtension + JSTraceable + HeapSizeOf {
        let name = T::name().to_uppercase();
        self.extensions.borrow().get(&name).and_then(|extension| {
            extension.as_any().downcast_ref::<TypedWebGLExtensionWrapper<T>>().and_then(|extension| {
                extension.dom_object()
            })
        })
    }

    pub fn supports_gl_extension(&self, name: &str) -> bool {
        self.features.borrow().gl_extensions.contains(name)
    }

    pub fn supports_any_gl_extension(&self, names: &[&str]) -> bool {
        let features = self.features.borrow();
        names.iter().any(|name| features.gl_extensions.contains(*name))
    }

    pub fn enable_tex_type(&self, data_type: GLenum) {
        self.features.borrow_mut().disabled_tex_types.remove(&data_type);
    }

    pub fn is_tex_type_enabled(&self, data_type: GLenum) -> bool {
        self.features.borrow().disabled_tex_types.get(&data_type).is_none()
    }

    pub fn add_effective_tex_internal_format(&self,
                                             source_internal_format: u32,
                                             source_data_type: u32,
                                             effective_internal_format: u32)
    {
        let format = TexFormatType(source_internal_format, source_data_type);
        self.features.borrow_mut().effective_tex_internal_formats.insert(format,
                                                                         effective_internal_format);

    }

    pub fn get_effective_tex_internal_format(&self,
                                             source_internal_format: u32,
                                             source_data_type: u32) -> u32 {
        let format = TexFormatType(source_internal_format, source_data_type);
        *(self.features.borrow().effective_tex_internal_formats.get(&format)
                                                               .unwrap_or(&source_internal_format))
    }

    pub fn enable_filterable_tex_type(&self, text_data_type: GLenum) {
        self.features.borrow_mut().not_filterable_tex_types.remove(&text_data_type);
    }

    pub fn is_filterable(&self, text_data_type: u32) -> bool {
        self.features.borrow().not_filterable_tex_types.get(&text_data_type).is_none()
    }

    pub fn add_query_parameter_handler(&self, name: GLenum, f: Box<WebGLQueryParameterFunc>) {
        let handler = WebGLQueryParameterHandler {
            func: f
        };
        self.features.borrow_mut().query_parameter_handlers.insert(name, handler);
    }

    pub fn get_query_parameter_handler(&self, name: GLenum) -> Option<Ref<Box<WebGLQueryParameterFunc>>> {
        ref_filter_map(self.features.borrow(), |features| {
            features.query_parameter_handlers.get(&name).map(|item| &item.func)
        })
    }

    fn register_all_extensions(&self) {
        self.register::<ext::oestexturefloat::OESTextureFloat>();
        self.register::<ext::oestexturefloatlinear::OESTextureFloatLinear>();
        self.register::<ext::oestexturehalffloat::OESTextureHalfFloat>();
        self.register::<ext::oestexturehalffloatlinear::OESTextureHalfFloatLinear>();
        self.register::<ext::oesvertexarrayobject::OESVertexArrayObject>();
    }
}

// Helper structs
#[derive(JSTraceable, HeapSizeOf, PartialEq, Eq, Hash)]
struct TexFormatType(u32, u32);

type WebGLQueryParameterFunc = Fn(*mut JSContext, &WebGLRenderingContext)
                               -> Result<JSVal, WebGLError>;

#[derive(HeapSizeOf)]
struct WebGLQueryParameterHandler {
    #[ignore_heap_size_of = "Closures are hard"]
    func: Box<WebGLQueryParameterFunc>
}

unsafe_no_jsmanaged_fields!(WebGLQueryParameterHandler);
