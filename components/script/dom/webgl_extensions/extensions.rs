/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::iter::FromIterator;
use std::ptr::NonNull;

use canvas_traits::webgl::{GlType, TexFormat, WebGLSLVersion, WebGLVersion};
use fnv::{FnvHashMap, FnvHashSet};
use js::jsapi::JSObject;
use malloc_size_of::MallocSizeOf;
use sparkle::gl::{self, GLenum};

use super::wrapper::{TypedWebGLExtensionWrapper, WebGLExtensionWrapper};
use super::{ext, WebGLExtension, WebGLExtensionSpec};
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::ANGLEInstancedArraysBinding::ANGLEInstancedArraysConstants;
use crate::dom::bindings::codegen::Bindings::EXTTextureFilterAnisotropicBinding::EXTTextureFilterAnisotropicConstants;
use crate::dom::bindings::codegen::Bindings::OESStandardDerivativesBinding::OESStandardDerivativesConstants;
use crate::dom::bindings::codegen::Bindings::OESTextureHalfFloatBinding::OESTextureHalfFloatConstants;
use crate::dom::bindings::codegen::Bindings::OESVertexArrayObjectBinding::OESVertexArrayObjectConstants;
use crate::dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLRenderingContextConstants as constants;
use crate::dom::bindings::trace::JSTraceable;
use crate::dom::extcolorbufferhalffloat::EXTColorBufferHalfFloat;
use crate::dom::oestexturefloat::OESTextureFloat;
use crate::dom::oestexturehalffloat::OESTextureHalfFloat;
use crate::dom::webglcolorbufferfloat::WEBGLColorBufferFloat;
use crate::dom::webglrenderingcontext::WebGLRenderingContext;
use crate::dom::webgltexture::TexCompression;

// Data types that are implemented for texImage2D and texSubImage2D in a WebGL 1.0 context
// but must trigger a InvalidValue error until the related WebGL Extensions are enabled.
// Example: https://www.khronos.org/registry/webgl/extensions/OES_texture_float/
const DEFAULT_DISABLED_TEX_TYPES_WEBGL1: [GLenum; 2] = [
    constants::FLOAT,
    OESTextureHalfFloatConstants::HALF_FLOAT_OES,
];

// Data types that are implemented for textures in WebGLRenderingContext
// but not allowed to use with linear filtering until the related WebGL Extensions are enabled.
// Example: https://www.khronos.org/registry/webgl/extensions/OES_texture_float_linear/
const DEFAULT_NOT_FILTERABLE_TEX_TYPES: [GLenum; 2] = [
    constants::FLOAT,
    OESTextureHalfFloatConstants::HALF_FLOAT_OES,
];

// Param names that are implemented for glGetParameter in a WebGL 1.0 context
// but must trigger a InvalidEnum error until the related WebGL Extensions are enabled.
// Example: https://www.khronos.org/registry/webgl/extensions/OES_standard_derivatives/
const DEFAULT_DISABLED_GET_PARAMETER_NAMES_WEBGL1: [GLenum; 3] = [
    EXTTextureFilterAnisotropicConstants::MAX_TEXTURE_MAX_ANISOTROPY_EXT,
    OESStandardDerivativesConstants::FRAGMENT_SHADER_DERIVATIVE_HINT_OES,
    OESVertexArrayObjectConstants::VERTEX_ARRAY_BINDING_OES,
];

// Param names that are implemented for glGetParameter in a WebGL 2.0 context
// but must trigger a InvalidEnum error until the related WebGL Extensions are enabled.
// Example: https://www.khronos.org/registry/webgl/extensions/EXT_texture_filter_anisotropic/
const DEFAULT_DISABLED_GET_PARAMETER_NAMES_WEBGL2: [GLenum; 1] =
    [EXTTextureFilterAnisotropicConstants::MAX_TEXTURE_MAX_ANISOTROPY_EXT];

// Param names that are implemented for glGetTexParameter in a WebGL 1.0 context
// but must trigger a InvalidEnum error until the related WebGL Extensions are enabled.
// Example: https://www.khronos.org/registry/webgl/extensions/OES_standard_derivatives/
const DEFAULT_DISABLED_GET_TEX_PARAMETER_NAMES_WEBGL1: [GLenum; 1] =
    [EXTTextureFilterAnisotropicConstants::TEXTURE_MAX_ANISOTROPY_EXT];

// Param names that are implemented for glGetTexParameter in a WebGL 2.0 context
// but must trigger a InvalidEnum error until the related WebGL Extensions are enabled.
// Example: https://www.khronos.org/registry/webgl/extensions/EXT_texture_filter_anisotropic/
const DEFAULT_DISABLED_GET_TEX_PARAMETER_NAMES_WEBGL2: [GLenum; 1] =
    [EXTTextureFilterAnisotropicConstants::TEXTURE_MAX_ANISOTROPY_EXT];

// Param names that are implemented for glGetVertexAttrib in a WebGL 1.0 context
// but must trigger a InvalidEnum error until the related WebGL Extensions are enabled.
// Example: https://www.khronos.org/registry/webgl/extensions/ANGLE_instanced_arrays/
const DEFAULT_DISABLED_GET_VERTEX_ATTRIB_NAMES_WEBGL1: [GLenum; 1] =
    [ANGLEInstancedArraysConstants::VERTEX_ATTRIB_ARRAY_DIVISOR_ANGLE];

/// WebGL features that are enabled/disabled by WebGL Extensions.
#[derive(JSTraceable, MallocSizeOf)]
struct WebGLExtensionFeatures {
    gl_extensions: FnvHashSet<String>,
    disabled_tex_types: FnvHashSet<GLenum>,
    not_filterable_tex_types: FnvHashSet<GLenum>,
    #[no_trace]
    effective_tex_internal_formats: FnvHashMap<TexFormatType, TexFormat>,
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
    /// WebGL supported texture compression formats enabled by extensions.
    tex_compression_formats: FnvHashMap<GLenum, TexCompression>,
}

impl WebGLExtensionFeatures {
    fn new(webgl_version: WebGLVersion) -> Self {
        let (
            disabled_tex_types,
            disabled_get_parameter_names,
            disabled_get_tex_parameter_names,
            disabled_get_vertex_attrib_names,
            not_filterable_tex_types,
            element_index_uint_enabled,
            blend_minmax_enabled,
        ) = match webgl_version {
            WebGLVersion::WebGL1 => (
                DEFAULT_DISABLED_TEX_TYPES_WEBGL1.iter().cloned().collect(),
                DEFAULT_DISABLED_GET_PARAMETER_NAMES_WEBGL1
                    .iter()
                    .cloned()
                    .collect(),
                DEFAULT_DISABLED_GET_TEX_PARAMETER_NAMES_WEBGL1
                    .iter()
                    .cloned()
                    .collect(),
                DEFAULT_DISABLED_GET_VERTEX_ATTRIB_NAMES_WEBGL1
                    .iter()
                    .cloned()
                    .collect(),
                DEFAULT_NOT_FILTERABLE_TEX_TYPES.iter().cloned().collect(),
                false,
                false,
            ),
            WebGLVersion::WebGL2 => (
                Default::default(),
                DEFAULT_DISABLED_GET_PARAMETER_NAMES_WEBGL2
                    .iter()
                    .cloned()
                    .collect(),
                DEFAULT_DISABLED_GET_TEX_PARAMETER_NAMES_WEBGL2
                    .iter()
                    .cloned()
                    .collect(),
                Default::default(),
                Default::default(),
                true,
                true,
            ),
        };
        Self {
            gl_extensions: Default::default(),
            disabled_tex_types,
            not_filterable_tex_types,
            effective_tex_internal_formats: Default::default(),
            hint_targets: Default::default(),
            disabled_get_parameter_names,
            disabled_get_tex_parameter_names,
            disabled_get_vertex_attrib_names,
            element_index_uint_enabled,
            blend_minmax_enabled,
            tex_compression_formats: Default::default(),
        }
    }
}

/// Handles the list of implemented, supported and enabled WebGL extensions.
#[crown::unrooted_must_root_lint::must_root]
#[derive(JSTraceable, MallocSizeOf)]
pub struct WebGLExtensions {
    extensions: DomRefCell<HashMap<String, Box<dyn WebGLExtensionWrapper>>>,
    features: DomRefCell<WebGLExtensionFeatures>,
    #[no_trace]
    webgl_version: WebGLVersion,
    #[no_trace]
    api_type: GlType,
    #[no_trace]
    glsl_version: WebGLSLVersion,
}

impl WebGLExtensions {
    pub fn new(
        webgl_version: WebGLVersion,
        api_type: GlType,
        glsl_version: WebGLSLVersion,
    ) -> WebGLExtensions {
        Self {
            extensions: DomRefCell::new(HashMap::new()),
            features: DomRefCell::new(WebGLExtensionFeatures::new(webgl_version)),
            webgl_version,
            api_type,
            glsl_version,
        }
    }

    pub fn init_once<F>(&self, cb: F)
    where
        F: FnOnce() -> String,
    {
        if self.extensions.borrow().len() == 0 {
            let gl_str = cb();
            self.features.borrow_mut().gl_extensions =
                FnvHashSet::from_iter(gl_str.split(&[',', ' '][..]).map(|s| s.into()));
            self.register_all_extensions();
        }
    }

    pub fn register<T: 'static + WebGLExtension + JSTraceable + MallocSizeOf>(&self) {
        let name = T::name().to_uppercase();
        self.extensions
            .borrow_mut()
            .insert(name, Box::new(TypedWebGLExtensionWrapper::<T>::new()));
    }

    pub fn get_supported_extensions(&self) -> Vec<&'static str> {
        self.extensions
            .borrow()
            .iter()
            .filter(|v| {
                if let WebGLExtensionSpec::Specific(version) = v.1.spec() {
                    if self.webgl_version != version {
                        return false;
                    }
                }
                v.1.is_supported(self)
            })
            .map(|ref v| v.1.name())
            .collect()
    }

    pub fn get_or_init_extension(
        &self,
        name: &str,
        ctx: &WebGLRenderingContext,
    ) -> Option<NonNull<JSObject>> {
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
        T: 'static + WebGLExtension + JSTraceable + MallocSizeOf,
    {
        let name = T::name().to_uppercase();
        self.extensions
            .borrow()
            .get(&name)
            .map_or(false, |ext| ext.is_enabled())
    }

    pub fn supports_gl_extension(&self, name: &str) -> bool {
        self.features.borrow().gl_extensions.contains(name)
    }

    pub fn supports_any_gl_extension(&self, names: &[&str]) -> bool {
        let features = self.features.borrow();
        names
            .iter()
            .any(|name| features.gl_extensions.contains(*name))
    }

    pub fn supports_all_gl_extension(&self, names: &[&str]) -> bool {
        let features = self.features.borrow();
        names
            .iter()
            .all(|name| features.gl_extensions.contains(*name))
    }

    pub fn enable_tex_type(&self, data_type: GLenum) {
        self.features
            .borrow_mut()
            .disabled_tex_types
            .remove(&data_type);
    }

    pub fn is_tex_type_enabled(&self, data_type: GLenum) -> bool {
        !self
            .features
            .borrow()
            .disabled_tex_types
            .contains(&data_type)
    }

    pub fn add_effective_tex_internal_format(
        &self,
        source_internal_format: TexFormat,
        source_data_type: u32,
        effective_internal_format: TexFormat,
    ) {
        let format = TexFormatType(source_internal_format, source_data_type);
        self.features
            .borrow_mut()
            .effective_tex_internal_formats
            .insert(format, effective_internal_format);
    }

    pub fn get_effective_tex_internal_format(
        &self,
        source_internal_format: TexFormat,
        source_data_type: u32,
    ) -> TexFormat {
        let format = TexFormatType(source_internal_format, source_data_type);
        *(self
            .features
            .borrow()
            .effective_tex_internal_formats
            .get(&format)
            .unwrap_or(&source_internal_format))
    }

    pub fn enable_filterable_tex_type(&self, text_data_type: GLenum) {
        self.features
            .borrow_mut()
            .not_filterable_tex_types
            .remove(&text_data_type);
    }

    pub fn is_filterable(&self, text_data_type: u32) -> bool {
        !self
            .features
            .borrow()
            .not_filterable_tex_types
            .contains(&text_data_type)
    }

    pub fn enable_hint_target(&self, name: GLenum) {
        self.features.borrow_mut().hint_targets.insert(name);
    }

    pub fn is_hint_target_enabled(&self, name: GLenum) -> bool {
        self.features.borrow().hint_targets.contains(&name)
    }

    pub fn enable_get_parameter_name(&self, name: GLenum) {
        self.features
            .borrow_mut()
            .disabled_get_parameter_names
            .remove(&name);
    }

    pub fn is_get_parameter_name_enabled(&self, name: GLenum) -> bool {
        !self
            .features
            .borrow()
            .disabled_get_parameter_names
            .contains(&name)
    }

    pub fn enable_get_tex_parameter_name(&self, name: GLenum) {
        self.features
            .borrow_mut()
            .disabled_get_tex_parameter_names
            .remove(&name);
    }

    pub fn is_get_tex_parameter_name_enabled(&self, name: GLenum) -> bool {
        !self
            .features
            .borrow()
            .disabled_get_tex_parameter_names
            .contains(&name)
    }

    pub fn enable_get_vertex_attrib_name(&self, name: GLenum) {
        self.features
            .borrow_mut()
            .disabled_get_vertex_attrib_names
            .remove(&name);
    }

    pub fn is_get_vertex_attrib_name_enabled(&self, name: GLenum) -> bool {
        !self
            .features
            .borrow()
            .disabled_get_vertex_attrib_names
            .contains(&name)
    }

    pub fn add_tex_compression_formats(&self, formats: &[TexCompression]) {
        let formats: FnvHashMap<GLenum, TexCompression> = formats
            .iter()
            .map(|&compression| (compression.format.as_gl_constant(), compression))
            .collect();

        self.features
            .borrow_mut()
            .tex_compression_formats
            .extend(formats.iter());
    }

    pub fn get_tex_compression_format(&self, format_id: GLenum) -> Option<TexCompression> {
        self.features
            .borrow()
            .tex_compression_formats
            .get(&format_id)
            .cloned()
    }

    pub fn get_tex_compression_ids(&self) -> Vec<GLenum> {
        self.features
            .borrow()
            .tex_compression_formats
            .keys()
            .copied()
            .collect()
    }

    fn register_all_extensions(&self) {
        self.register::<ext::angleinstancedarrays::ANGLEInstancedArrays>();
        self.register::<ext::extblendminmax::EXTBlendMinmax>();
        self.register::<ext::extcolorbufferhalffloat::EXTColorBufferHalfFloat>();
        self.register::<ext::extfragdepth::EXTFragDepth>();
        self.register::<ext::extshadertexturelod::EXTShaderTextureLod>();
        self.register::<ext::exttexturefilteranisotropic::EXTTextureFilterAnisotropic>();
        self.register::<ext::oeselementindexuint::OESElementIndexUint>();
        self.register::<ext::oesstandardderivatives::OESStandardDerivatives>();
        self.register::<ext::oestexturefloat::OESTextureFloat>();
        self.register::<ext::oestexturefloatlinear::OESTextureFloatLinear>();
        self.register::<ext::oestexturehalffloat::OESTextureHalfFloat>();
        self.register::<ext::oestexturehalffloatlinear::OESTextureHalfFloatLinear>();
        self.register::<ext::oesvertexarrayobject::OESVertexArrayObject>();
        self.register::<ext::webglcolorbufferfloat::WEBGLColorBufferFloat>();
        self.register::<ext::webglcompressedtextureetc1::WEBGLCompressedTextureETC1>();
        self.register::<ext::webglcompressedtextures3tc::WEBGLCompressedTextureS3TC>();
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

    pub fn is_float_buffer_renderable(&self) -> bool {
        self.is_enabled::<WEBGLColorBufferFloat>() || self.is_enabled::<OESTextureFloat>()
    }

    pub fn is_min_glsl_version_satisfied(&self, min_glsl_version: WebGLSLVersion) -> bool {
        self.glsl_version >= min_glsl_version
    }

    pub fn is_half_float_buffer_renderable(&self) -> bool {
        self.is_enabled::<EXTColorBufferHalfFloat>() || self.is_enabled::<OESTextureHalfFloat>()
    }

    pub fn effective_type(&self, type_: u32) -> u32 {
        if type_ == OESTextureHalfFloatConstants::HALF_FLOAT_OES &&
            !self.supports_gl_extension("GL_OES_texture_half_float")
        {
            return gl::HALF_FLOAT;
        }
        type_
    }

    pub fn is_gles(&self) -> bool {
        self.api_type == GlType::Gles
    }
}

// Helper structs
#[derive(Eq, Hash, MallocSizeOf, PartialEq)]
struct TexFormatType(TexFormat, u32);
