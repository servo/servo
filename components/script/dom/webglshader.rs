/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::webgl_extensions::ext::extfragdepth::EXTFragDepth;
use crate::dom::webgl_extensions::ext::extshadertexturelod::EXTShaderTextureLod;
use crate::dom::webgl_extensions::ext::oesstandardderivatives::OESStandardDerivatives;
use crate::dom::webgl_extensions::WebGLExtensions;
use crate::dom::webglobject::WebGLObject;
use crate::dom::webglrenderingcontext::{Operation, WebGLRenderingContext};
use canvas_traits::webgl::{webgl_channel, GlType, WebGLVersion};
use canvas_traits::webgl::{GLLimits, WebGLCommand, WebGLError};
use canvas_traits::webgl::{WebGLResult, WebGLSLVersion, WebGLShaderId};
use dom_struct::dom_struct;
use mozangle::shaders::{ffi, BuiltInResources, Output, ShaderValidator};
use std::cell::Cell;
use std::os::raw::c_int;
use std::sync::Once;

#[derive(Clone, Copy, Debug, JSTraceable, MallocSizeOf, PartialEq)]
pub enum ShaderCompilationStatus {
    NotCompiled,
    Succeeded,
    Failed,
}

#[dom_struct]
pub struct WebGLShader {
    webgl_object: WebGLObject,
    id: WebGLShaderId,
    gl_type: u32,
    source: DomRefCell<DOMString>,
    info_log: DomRefCell<DOMString>,
    marked_for_deletion: Cell<bool>,
    attached_counter: Cell<u32>,
    compilation_status: Cell<ShaderCompilationStatus>,
}

static GLSLANG_INITIALIZATION: Once = Once::new();

impl WebGLShader {
    fn new_inherited(context: &WebGLRenderingContext, id: WebGLShaderId, shader_type: u32) -> Self {
        GLSLANG_INITIALIZATION.call_once(|| ::mozangle::shaders::initialize().unwrap());
        Self {
            webgl_object: WebGLObject::new_inherited(context),
            id: id,
            gl_type: shader_type,
            source: Default::default(),
            info_log: Default::default(),
            marked_for_deletion: Cell::new(false),
            attached_counter: Cell::new(0),
            compilation_status: Cell::new(ShaderCompilationStatus::NotCompiled),
        }
    }

    pub fn maybe_new(context: &WebGLRenderingContext, shader_type: u32) -> Option<DomRoot<Self>> {
        let (sender, receiver) = webgl_channel().unwrap();
        context.send_command(WebGLCommand::CreateShader(shader_type, sender));
        receiver
            .recv()
            .unwrap()
            .map(|id| WebGLShader::new(context, id, shader_type))
    }

    pub fn new(
        context: &WebGLRenderingContext,
        id: WebGLShaderId,
        shader_type: u32,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(WebGLShader::new_inherited(context, id, shader_type)),
            &*context.global(),
        )
    }
}

// Based on https://searchfox.org/mozilla-central/rev/efdf9bb55789ea782ae3a431bda6be74a87b041e/gfx/angle/checkout/src/compiler/translator/ShaderLang.cpp#173
fn default_validator() -> BuiltInResources {
    BuiltInResources {
        // Constants.
        MaxVertexAttribs: 8,
        MaxVertexUniformVectors: 128,
        MaxVaryingVectors: 8,
        MaxVertexTextureImageUnits: 0,
        MaxCombinedTextureImageUnits: 8,
        MaxTextureImageUnits: 8,
        MaxFragmentUniformVectors: 16,
        MaxDrawBuffers: 1,

        // Extensions.
        OES_standard_derivatives: 0,
        OES_EGL_image_external: 0,
        OES_EGL_image_external_essl3: 0,
        NV_EGL_stream_consumer_external: 0,
        ARB_texture_rectangle: 0,
        EXT_blend_func_extended: 0,
        EXT_draw_buffers: 0,
        EXT_frag_depth: 0,
        EXT_shader_texture_lod: 0,
        WEBGL_debug_shader_precision: 0,
        EXT_shader_framebuffer_fetch: 0,
        NV_shader_framebuffer_fetch: 0,
        NV_draw_buffers: 0,
        ARM_shader_framebuffer_fetch: 0,
        //OVR_multiview: 0,
        OVR_multiview2: 0,
        EXT_YUV_target: 0,
        EXT_geometry_shader: 0,
        OES_texture_storage_multisample_2d_array: 0,
        //OES_texture_3d: 0,
        ANGLE_texture_multisample: 0,
        ANGLE_multi_draw: 0,

        // Disable highp precision in fragment shader by default.
        FragmentPrecisionHigh: 0,

        // GLSL ES 3.0 constants.
        MaxVertexOutputVectors: 16,
        MaxFragmentInputVectors: 15,
        MinProgramTexelOffset: -8,
        MaxProgramTexelOffset: 7,

        // Extension constants.
        MaxDualSourceDrawBuffers: 0,
        MaxViewsOVR: 4,

        // Disable name hashing by default.
        HashFunction: None,
        ArrayIndexClampingStrategy:
            ffi::ShArrayIndexClampingStrategy::SH_CLAMP_WITH_CLAMP_INTRINSIC,

        MaxExpressionComplexity: 256,
        MaxCallStackDepth: 256,
        MaxFunctionParameters: 1024,

        // ES 3.1 Revision 4, 7.2 Built-in Constants

        // ES 3.1, Revision 4, 8.13 Texture minification
        // "The value of MIN_PROGRAM_TEXTURE_GATHER_OFFSET must be less than or equal to the value of
        // MIN_PROGRAM_TEXEL_OFFSET. The value of MAX_PROGRAM_TEXTURE_GATHER_OFFSET must be greater than
        // or equal to the value of MAX_PROGRAM_TEXEL_OFFSET"
        MinProgramTextureGatherOffset: -8,
        MaxProgramTextureGatherOffset: 7,

        MaxImageUnits: 4,
        MaxVertexImageUniforms: 0,
        MaxFragmentImageUniforms: 0,
        MaxComputeImageUniforms: 0,
        MaxCombinedImageUniforms: 0,

        MaxUniformLocations: 1024,

        MaxCombinedShaderOutputResources: 4,

        MaxComputeWorkGroupCount: [65535, 65535, 65535],
        MaxComputeWorkGroupSize: [128, 128, 64],
        MaxComputeUniformComponents: 512,
        MaxComputeTextureImageUnits: 16,

        MaxComputeAtomicCounters: 8,
        MaxComputeAtomicCounterBuffers: 1,

        MaxVertexAtomicCounters: 0,
        MaxFragmentAtomicCounters: 0,
        MaxCombinedAtomicCounters: 8,
        MaxAtomicCounterBindings: 1,

        MaxVertexAtomicCounterBuffers: 0,
        MaxFragmentAtomicCounterBuffers: 0,
        MaxCombinedAtomicCounterBuffers: 1,
        MaxAtomicCounterBufferSize: 32,

        MaxUniformBufferBindings: 32,
        MaxShaderStorageBufferBindings: 4,
        MaxPointSize: 0.0,

        MaxGeometryUniformComponents: 1024,
        MaxGeometryUniformBlocks: 12,
        MaxGeometryInputComponents: 64,
        MaxGeometryOutputComponents: 64,
        MaxGeometryOutputVertices: 256,
        MaxGeometryTotalOutputComponents: 1024,
        MaxGeometryTextureImageUnits: 16,
        MaxGeometryAtomicCounterBuffers: 0,
        MaxGeometryAtomicCounters: 0,
        MaxGeometryShaderStorageBlocks: 0,
        MaxGeometryShaderInvocations: 32,
        MaxGeometryImageUniforms: 0,
    }
}

impl WebGLShader {
    pub fn id(&self) -> WebGLShaderId {
        self.id
    }

    pub fn gl_type(&self) -> u32 {
        self.gl_type
    }

    /// glCompileShader
    pub fn compile(
        &self,
        api_type: GlType,
        webgl_version: WebGLVersion,
        glsl_version: WebGLSLVersion,
        limits: &GLLimits,
        ext: &WebGLExtensions,
    ) -> WebGLResult<()> {
        if self.marked_for_deletion.get() && !self.is_attached() {
            return Err(WebGLError::InvalidValue);
        }
        if self.compilation_status.get() != ShaderCompilationStatus::NotCompiled {
            debug!("Compiling already compiled shader {}", self.id);
        }

        let source = self.source.borrow();

        let mut params = BuiltInResources {
            MaxVertexAttribs: limits.max_vertex_attribs as c_int,
            MaxVertexUniformVectors: limits.max_vertex_uniform_vectors as c_int,
            MaxVertexTextureImageUnits: limits.max_vertex_texture_image_units as c_int,
            MaxCombinedTextureImageUnits: limits.max_combined_texture_image_units as c_int,
            MaxTextureImageUnits: limits.max_texture_image_units as c_int,
            MaxFragmentUniformVectors: limits.max_fragment_uniform_vectors as c_int,

            MaxVertexOutputVectors: limits.max_vertex_output_vectors as c_int,
            MaxFragmentInputVectors: limits.max_fragment_input_vectors as c_int,
            MaxVaryingVectors: limits.max_varying_vectors as c_int,

            OES_standard_derivatives: ext.is_enabled::<OESStandardDerivatives>() as c_int,
            EXT_shader_texture_lod: ext.is_enabled::<EXTShaderTextureLod>() as c_int,
            EXT_frag_depth: ext.is_enabled::<EXTFragDepth>() as c_int,

            FragmentPrecisionHigh: 1,
            ..default_validator()
        };

        if webgl_version == WebGLVersion::WebGL2 {
            params.MinProgramTexelOffset = limits.min_program_texel_offset as c_int;
            params.MaxProgramTexelOffset = limits.max_program_texel_offset as c_int;
            params.MaxDrawBuffers = limits.max_draw_buffers as c_int;
        }

        let validator = match webgl_version {
            WebGLVersion::WebGL1 => {
                let output_format = if api_type == GlType::Gles {
                    Output::Essl
                } else {
                    Output::Glsl
                };
                ShaderValidator::for_webgl(self.gl_type, output_format, &params).unwrap()
            },
            WebGLVersion::WebGL2 => {
                let output_format = if api_type == GlType::Gles {
                    Output::Essl
                } else {
                    match (glsl_version.major, glsl_version.minor) {
                        (1, 30) => Output::Glsl130,
                        (1, 40) => Output::Glsl140,
                        (1, 50) => Output::Glsl150Core,
                        (3, 30) => Output::Glsl330Core,
                        (4, 0) => Output::Glsl400Core,
                        (4, 10) => Output::Glsl410Core,
                        (4, 20) => Output::Glsl420Core,
                        (4, 30) => Output::Glsl430Core,
                        (4, 40) => Output::Glsl440Core,
                        (4, _) => Output::Glsl450Core,
                        _ => Output::Glsl140,
                    }
                };
                ShaderValidator::for_webgl2(self.gl_type, output_format, &params).unwrap()
            },
        };

        // Replicating
        // https://searchfox.org/mozilla-central/rev/c621276fbdd9591f52009042d959b9e19b66d49f/dom/canvas/WebGLShaderValidator.cpp#32
        let options = mozangle::shaders::ffi::SH_VARIABLES |
            mozangle::shaders::ffi::SH_ENFORCE_PACKING_RESTRICTIONS |
            mozangle::shaders::ffi::SH_OBJECT_CODE |
            mozangle::shaders::ffi::SH_INIT_GL_POSITION |
            mozangle::shaders::ffi::SH_INITIALIZE_UNINITIALIZED_LOCALS |
            mozangle::shaders::ffi::SH_INIT_OUTPUT_VARIABLES |
            mozangle::shaders::ffi::SH_LIMIT_EXPRESSION_COMPLEXITY |
            mozangle::shaders::ffi::SH_LIMIT_CALL_STACK_DEPTH |
            if cfg!(target_os = "macos") {
                // Work around https://bugs.webkit.org/show_bug.cgi?id=124684,
                // https://chromium.googlesource.com/angle/angle/+/5e70cf9d0b1bb
                mozangle::shaders::ffi::SH_UNFOLD_SHORT_CIRCUIT |
                // Work around that Mac drivers handle struct scopes incorrectly.
                mozangle::shaders::ffi::SH_REGENERATE_STRUCT_NAMES |
                // Work around that Intel drivers on Mac OSX handle for-loop incorrectly.
                mozangle::shaders::ffi::SH_ADD_AND_TRUE_TO_LOOP_CONDITION
            } else {
                // We want to do this everywhere, but to do this on Mac, we need
                // to do it only on Mac OSX > 10.6 as this causes the shader
                // compiler in 10.6 to crash
                mozangle::shaders::ffi::SH_CLAMP_INDIRECT_ARRAY_BOUNDS
            };

        // Replicating
        // https://github.com/servo/mozangle/blob/706a9baaf8026c1a3cb6c67ba63aa5f4734264d0/src/shaders/mod.rs#L226
        let options = options |
            mozangle::shaders::ffi::SH_VALIDATE |
            mozangle::shaders::ffi::SH_OBJECT_CODE |
            mozangle::shaders::ffi::SH_VARIABLES | // For uniform_name_map()
            mozangle::shaders::ffi::SH_EMULATE_ABS_INT_FUNCTION | // To workaround drivers
            mozangle::shaders::ffi::SH_EMULATE_ISNAN_FLOAT_FUNCTION | // To workaround drivers
            mozangle::shaders::ffi::SH_EMULATE_ATAN2_FLOAT_FUNCTION | // To workaround drivers
            mozangle::shaders::ffi::SH_CLAMP_INDIRECT_ARRAY_BOUNDS |
            mozangle::shaders::ffi::SH_INIT_GL_POSITION |
            mozangle::shaders::ffi::SH_ENFORCE_PACKING_RESTRICTIONS |
            mozangle::shaders::ffi::SH_LIMIT_EXPRESSION_COMPLEXITY |
            mozangle::shaders::ffi::SH_LIMIT_CALL_STACK_DEPTH;

        match validator.compile(&[&source], options) {
            Ok(()) => {
                let translated_source = validator.object_code();
                debug!("Shader translated: {}", translated_source);
                // NOTE: At this point we should be pretty sure that the compilation in the paint thread
                // will succeed.
                // It could be interesting to retrieve the info log from the paint thread though
                self.upcast::<WebGLObject>()
                    .context()
                    .send_command(WebGLCommand::CompileShader(self.id, translated_source));
                self.compilation_status
                    .set(ShaderCompilationStatus::Succeeded);
            },
            Err(error) => {
                self.compilation_status.set(ShaderCompilationStatus::Failed);
                debug!("Shader {} compilation failed: {}", self.id, error);
            },
        }

        *self.info_log.borrow_mut() = validator.info_log().into();

        Ok(())
    }

    /// Mark this shader as deleted (if it wasn't previously)
    /// and delete it as if calling glDeleteShader.
    /// Currently does not check if shader is attached
    pub fn mark_for_deletion(&self, operation_fallibility: Operation) {
        if !self.marked_for_deletion.get() {
            self.marked_for_deletion.set(true);
            let context = self.upcast::<WebGLObject>().context();
            let cmd = WebGLCommand::DeleteShader(self.id);
            match operation_fallibility {
                Operation::Fallible => context.send_command_ignored(cmd),
                Operation::Infallible => context.send_command(cmd),
            }
        }
    }

    pub fn is_marked_for_deletion(&self) -> bool {
        self.marked_for_deletion.get()
    }

    pub fn is_deleted(&self) -> bool {
        self.marked_for_deletion.get() && !self.is_attached()
    }

    pub fn is_attached(&self) -> bool {
        self.attached_counter.get() > 0
    }

    pub fn increment_attached_counter(&self) {
        self.attached_counter.set(self.attached_counter.get() + 1);
    }

    pub fn decrement_attached_counter(&self) {
        assert!(self.attached_counter.get() > 0);
        self.attached_counter.set(self.attached_counter.get() - 1);
    }

    /// glGetShaderInfoLog
    pub fn info_log(&self) -> DOMString {
        self.info_log.borrow().clone()
    }

    /// Get the shader source
    pub fn source(&self) -> DOMString {
        self.source.borrow().clone()
    }

    /// glShaderSource
    pub fn set_source(&self, source: DOMString) {
        *self.source.borrow_mut() = source;
    }

    pub fn successfully_compiled(&self) -> bool {
        self.compilation_status.get() == ShaderCompilationStatus::Succeeded
    }
}

impl Drop for WebGLShader {
    fn drop(&mut self) {
        self.mark_for_deletion(Operation::Fallible);
    }
}
