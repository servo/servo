/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use canvas_traits::webgl::WebGLVersion;
use dom::bindings::cell::DomRefCell;
use dom::bindings::codegen::Bindings::ANGLEInstancedArraysBinding::ANGLEInstancedArraysConstants;
use dom::bindings::codegen::Bindings::EXTTextureFilterAnisotropicBinding::EXTTextureFilterAnisotropicConstants;
use dom::bindings::codegen::Bindings::OESStandardDerivativesBinding::OESStandardDerivativesConstants;
use dom::bindings::codegen::Bindings::OESTextureHalfFloatBinding::OESTextureHalfFloatConstants;
use dom::bindings::codegen::Bindings::OESVertexArrayObjectBinding::OESVertexArrayObjectConstants;
use dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLRenderingContextConstants as constants;
use dom::bindings::trace::JSTraceable;
use dom::webglrenderingcontext::WebGLRenderingContext;
use fnv::{FnvHashMap, FnvHashSet};
use gleam::gl::GLenum;
use js::jsapi::JSObject;
use malloc_size_of::MallocSizeOf;
use std::collections::HashMap;
use std::iter::FromIterator;
use std::ptr::NonNull;
use super::{ext, WebGLExtension, WebGLExtensionSpec};
use super::wrapper::{WebGLExtensionWrapper, TypedWebGLExtensionWrapper};
use typeholder::TypeHolderTrait;

// Data types that are implemented for texImage2D and texSubImage2D in a WebGL 1.0 context
// but must trigger a InvalidValue error until the related WebGL Extensions are enabled.
// Example: https://www.khronos.org/registry/webgl/extensions/OES_texture_float/
const DEFAULT_DISABLED_TEX_TYPES_WEBGL1: [GLenum; 2] = [
    constants::FLOAT, OESTextureHalfFloatConstants::HALF_FLOAT_OES
];

// Data types that are implemented for textures in WebGLRenderingContext
// but not allowed to use with linear filtering until the related WebGL Extensions are enabled.
// Example: https://www.khronos.org/registry/webgl/extensions/OES_texture_float_linear/
const DEFAULT_NOT_FILTERABLE_TEX_TYPES: [GLenum; 2] = [
    constants::FLOAT, OESTextureHalfFloatConstants::HALF_FLOAT_OES
];

// Param names that are implemented for glGetParameter in a WebGL 1.0 context
// but must trigger a InvalidEnum error until the related WebGL Extensions are enabled.
// Example: https://www.khronos.org/registry/webgl/extensions/OES_standard_derivatives/
const DEFAULT_DISABLED_GET_PARAMETER_NAMES_WEBGL1: [GLenum; 3] = [
    EXTTextureFilterAnisotropicConstants::MAX_TEXTURE_MAX_ANISOTROPY_EXT,
    OESStandardDerivativesConstants::FRAGMENT_SHADER_DERIVATIVE_HINT_OES,
    OESVertexArrayObjectConstants::VERTEX_ARRAY_BINDING_OES,
];

// Param names that are implemented for glGetTexParameter in a WebGL 1.0 context
// but must trigger a InvalidEnum error until the related WebGL Extensions are enabled.
// Example: https://www.khronos.org/registry/webgl/extensions/OES_standard_derivatives/
const DEFAULT_DISABLED_GET_TEX_PARAMETER_NAMES_WEBGL1: [GLenum; 1] = [
    EXTTextureFilterAnisotropicConstants::TEXTURE_MAX_ANISOTROPY_EXT,
];

// Param names that are implemented for glGetVertexAttrib in a WebGL 1.0 context
// but must trigger a InvalidEnum error until the related WebGL Extensions are enabled.
// Example: https://www.khronos.org/registry/webgl/extensions/ANGLE_instanced_arrays/
const DEFAULT_DISABLED_GET_VERTEX_ATTRIB_NAMES_WEBGL1: [GLenum; 1] = [
    ANGLEInstancedArraysConstants::VERTEX_ATTRIB_ARRAY_DIVISOR_ANGLE,
];

/// WebGL features that are enabled/disabled by WebGL Extensions.
#[derive(JSTraceable, MallocSizeOf)]
struct WebGLExtensionFeatures {
    gl_extensions: FnvHashSet<String>,
    disabled_tex_types: FnvHashSet<GLenum>,
    not_filterable_tex_types: FnvHashSet<GLenum>,
    effective_tex_internal_formats: FnvHashMap<TexFormatType, u32>,
    /// WebGL Hint() targets enabled by extensions.
    hint_targets: FnvHashSet<GLenum>,
    /// WebGL GetParameter() names enabled by extensions.
    disabled_get_parameter_names: FnvHashSet<GLenum>,
    /// WebGL GetTexParameter() names enabled by extensions.
    disabled_get_tex_parameter_names: FnvHashSet<GLenum>,
    /// WebGL GetAttribVertex() names enabled by extensions.
    disabled_get_vertex_attrib_names: FnvHashSet<GLenum>,
    /// WebGL OES_element_index_uint extension.
    element_index_uint_enabled: bool,
    /// WebGL EXT_blend_minmax extension.
    blend_minmax_enabled: bool,
}

impl WebGLExtensionFeatures {
    fn new(webgl_version: WebGLVersion) -> Self {
        let (
            disabled_tex_types,
            disabled_get_parameter_names,
            disabled_get_tex_parameter_names,
            disabled_get_vertex_attrib_names,
            element_index_uint_enabled,
            blend_minmax_enabled,
        ) = match webgl_version {
            WebGLVersion::WebGL1 => {
                (
                    DEFAULT_DISABLED_TEX_TYPES_WEBGL1.iter().cloned().collect(),
                    DEFAULT_DISABLED_GET_PARAMETER_NAMES_WEBGL1.iter().cloned().collect(),
                    DEFAULT_DISABLED_GET_TEX_PARAMETER_NAMES_WEBGL1.iter().cloned().collect(),
                    DEFAULT_DISABLED_GET_VERTEX_ATTRIB_NAMES_WEBGL1.iter().cloned().collect(),
                    false,
                    false,
                )
            },
            WebGLVersion::WebGL2 => {
                (
                    Default::default(),
                    Default::default(),
                    Default::default(),
                    Default::default(),
                    true,
                    true,
                )
            }
        };
        Self {
            gl_extensions: Default::default(),
            disabled_tex_types,
            not_filterable_tex_types: DEFAULT_NOT_FILTERABLE_TEX_TYPES.iter().cloned().collect(),
            effective_tex_internal_formats: Default::default(),
            hint_targets: Default::default(),
            disabled_get_parameter_names,
            disabled_get_tex_parameter_names,
            disabled_get_vertex_attrib_names,
            element_index_uint_enabled,
            blend_minmax_enabled,
        }
    }
}

/// Handles the list of implemented, supported and enabled WebGL extensions.
#[must_root]
#[derive(JSTraceable, MallocSizeOf)]
pub struct WebGLExtensions<TH: TypeHolderTrait> {
    extensions: DomRefCell<HashMap<String, Box<WebGLExtensionWrapper<TH>>>>,
    features: DomRefCell<WebGLExtensionFeatures>,
    webgl_version: WebGLVersion,
}

impl<TH: TypeHolderTrait> WebGLExtensions<TH> {
    pub fn new(webgl_version: WebGLVersion) -> WebGLExtensions<TH> {
        Self {
            extensions: DomRefCell::new(HashMap::new()),
            features: DomRefCell::new(WebGLExtensionFeatures::new(webgl_version)),
            webgl_version,
        }
    }

    pub fn init_once<F>(&self, cb: F) where F: FnOnce() -> String {
        if self.extensions.borrow().len() == 0 {
            let gl_str = cb();
            self.features.borrow_mut().gl_extensions = FnvHashSet::from_iter(gl_str.split(&[',', ' '][..])
                                                                                   .map(|s| s.into()));
            self.register_all_extensions();
        }
    }

    pub fn register<T:'static + WebGLExtension<TH> + JSTraceable + MallocSizeOf>(&self) {
        let name = T::name().to_uppercase();
        self.extensions.borrow_mut().insert(name, Box::new(TypedWebGLExtensionWrapper::<T, TH>::new()));
    }

    pub fn get_suported_extensions(&self) -> Vec<&'static str> {
        self.extensions.borrow().iter()
                                .filter(|ref v| {
                                    if let WebGLExtensionSpec::Specific(version) = v.1.spec() {
                                        if self.webgl_version != version {
                                            return false;
                                        }
                                    }
                                    v.1.is_supported(&self)
                                })
                                .map(|ref v| v.1.name())
                                .collect()
    }

    pub fn get_or_init_extension(&self, name: &str, ctx: &WebGLRenderingContext<TH>) -> Option<NonNull<JSObject>> {
        let name = name.to_uppercase();
        self.extensions.borrow().get(&name).and_then(|extension| {
            if extension.is_supported(self) {
                Some(extension.instance_or_init(ctx, self))
            } else {
                None
            }
        })
    }

    pub fn is_enabled<T>(&self) -> bool
    where
        T: 'static + WebGLExtension<TH> + JSTraceable + MallocSizeOf
    {
        let name = T::name().to_uppercase();
        self.extensions.borrow().get(&name).map_or(false, |ext| { ext.is_enabled() })
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

    pub fn enable_hint_target(&self, name: GLenum) {
        self.features.borrow_mut().hint_targets.insert(name);
    }

    pub fn is_hint_target_enabled(&self, name: GLenum) -> bool {
        self.features.borrow().hint_targets.contains(&name)
    }

    pub fn enable_get_parameter_name(&self, name: GLenum) {
        self.features.borrow_mut().disabled_get_parameter_names.remove(&name);
    }

    pub fn is_get_parameter_name_enabled(&self, name: GLenum) -> bool {
        !self.features.borrow().disabled_get_parameter_names.contains(&name)
    }

    pub fn enable_get_tex_parameter_name(&self, name: GLenum) {
        self.features.borrow_mut().disabled_get_tex_parameter_names.remove(&name);
    }

    pub fn is_get_tex_parameter_name_enabled(&self, name: GLenum) -> bool {
        !self.features.borrow().disabled_get_tex_parameter_names.contains(&name)
    }

    pub fn enable_get_vertex_attrib_name(&self, name: GLenum) {
        self.features.borrow_mut().disabled_get_vertex_attrib_names.remove(&name);
    }

    pub fn is_get_vertex_attrib_name_enabled(&self, name: GLenum) -> bool {
        !self.features.borrow().disabled_get_vertex_attrib_names.contains(&name)
    }

    fn register_all_extensions(&self) {
        self.register::<ext::angleinstancedarrays::ANGLEInstancedArrays::<TH>>();
        self.register::<ext::extblendminmax::EXTBlendMinmax::<TH>>();
        self.register::<ext::extshadertexturelod::EXTShaderTextureLod::<TH>>();
        self.register::<ext::exttexturefilteranisotropic::EXTTextureFilterAnisotropic::<TH>>();
        self.register::<ext::oeselementindexuint::OESElementIndexUint::<TH>>();
        self.register::<ext::oesstandardderivatives::OESStandardDerivatives::<TH>>();
        self.register::<ext::oestexturefloat::OESTextureFloat::<TH>>();
        self.register::<ext::oestexturefloatlinear::OESTextureFloatLinear::<TH>>();
        self.register::<ext::oestexturehalffloat::OESTextureHalfFloat::<TH>>();
        self.register::<ext::oestexturehalffloatlinear::OESTextureHalfFloatLinear::<TH>>();
        self.register::<ext::oesvertexarrayobject::OESVertexArrayObject::<TH>>();
    }

    pub fn enable_element_index_uint(&self) {
        self.features.borrow_mut().element_index_uint_enabled = true;
    }

    pub fn is_element_index_uint_enabled(&self) -> bool {
        self.features.borrow().element_index_uint_enabled
    }

    pub fn enable_blend_minmax(&self) {
        self.features.borrow_mut().blend_minmax_enabled = true;
    }

    pub fn is_blend_minmax_enabled(&self) -> bool {
        self.features.borrow().blend_minmax_enabled
    }
}

// Helper structs
#[derive(Eq, Hash, JSTraceable, MallocSizeOf, PartialEq)]
struct TexFormatType(u32, u32);
