/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use std::cell::Cell;
use std::os::raw::c_int;
use std::sync::Once;

use canvas_traits::webgl::{
    webgl_channel, GLLimits, GlType, WebGLCommand, WebGLError, WebGLResult, WebGLSLVersion,
    WebGLShaderId, WebGLVersion,
};
use dom_struct::dom_struct;
use mozangle::shaders::{BuiltInResources, CompileOptions, Output, ShaderValidator};

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::webgl_extensions::ext::extfragdepth::EXTFragDepth;
use crate::dom::webgl_extensions::ext::extshadertexturelod::EXTShaderTextureLod;
use crate::dom::webgl_extensions::ext::oesstandardderivatives::OESStandardDerivatives;
use crate::dom::webgl_extensions::WebGLExtensions;
use crate::dom::webglobject::WebGLObject;
use crate::dom::webglrenderingcontext::{Operation, WebGLRenderingContext};
use crate::script_runtime::CanGc;

#[derive(Clone, Copy, Debug, JSTraceable, MallocSizeOf, PartialEq)]
pub(crate) enum ShaderCompilationStatus {
    NotCompiled,
    Succeeded,
    Failed,
}

#[dom_struct]
pub(crate) struct WebGLShader {
    webgl_object: WebGLObject,
    #[no_trace]
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
            id,
            gl_type: shader_type,
            source: Default::default(),
            info_log: Default::default(),
            marked_for_deletion: Cell::new(false),
            attached_counter: Cell::new(0),
            compilation_status: Cell::new(ShaderCompilationStatus::NotCompiled),
        }
    }

    pub(crate) fn maybe_new(
        context: &WebGLRenderingContext,
        shader_type: u32,
    ) -> Option<DomRoot<Self>> {
        let (sender, receiver) = webgl_channel().unwrap();
        context.send_command(WebGLCommand::CreateShader(shader_type, sender));
        receiver
            .recv()
            .unwrap()
            .map(|id| WebGLShader::new(context, id, shader_type))
    }

    pub(crate) fn new(
        context: &WebGLRenderingContext,
        id: WebGLShaderId,
        shader_type: u32,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(WebGLShader::new_inherited(context, id, shader_type)),
            &*context.global(),
            CanGc::note(),
        )
    }
}

impl WebGLShader {
    pub(crate) fn id(&self) -> WebGLShaderId {
        self.id
    }

    pub(crate) fn gl_type(&self) -> u32 {
        self.gl_type
    }

    /// glCompileShader
    pub(crate) fn compile(
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
            ..Default::default()
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
        // https://searchfox.org/mozilla-esr115/rev/f1fb0868dc63b89ccf9eea157960d1ec27fb55a2/dom/canvas/WebGLShaderValidator.cpp#29
        let mut options = CompileOptions::mozangle();
        options.set_variables(1);
        options.set_enforcePackingRestrictions(1);
        options.set_objectCode(1);
        options.set_initGLPosition(1);
        options.set_initializeUninitializedLocals(1);
        options.set_initOutputVariables(1);

        options.set_limitExpressionComplexity(1);
        options.set_limitCallStackDepth(1);

        if cfg!(target_os = "macos") {
            options.set_removeInvariantAndCentroidForESSL3(1);

            // Work around https://bugs.webkit.org/show_bug.cgi?id=124684,
            // https://chromium.googlesource.com/angle/angle/+/5e70cf9d0b1bb
            options.set_unfoldShortCircuit(1);
            // Work around that Mac drivers handle struct scopes incorrectly.
            options.set_regenerateStructNames(1);
            // TODO: Only apply this workaround to Intel hardware
            // Work around that Intel drivers on Mac OSX handle for-loop incorrectly.
            options.set_addAndTrueToLoopCondition(1);
            options.set_rewriteTexelFetchOffsetToTexelFetch(1);
        } else {
            // We want to do this everywhere, but to do this on Mac, we need
            // to do it only on Mac OSX > 10.6 as this causes the shader
            // compiler in 10.6 to crash
            options.set_clampIndirectArrayBounds(1);
        }

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
    pub(crate) fn mark_for_deletion(&self, operation_fallibility: Operation) {
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

    pub(crate) fn is_marked_for_deletion(&self) -> bool {
        self.marked_for_deletion.get()
    }

    pub(crate) fn is_deleted(&self) -> bool {
        self.marked_for_deletion.get() && !self.is_attached()
    }

    pub(crate) fn is_attached(&self) -> bool {
        self.attached_counter.get() > 0
    }

    pub(crate) fn increment_attached_counter(&self) {
        self.attached_counter.set(self.attached_counter.get() + 1);
    }

    pub(crate) fn decrement_attached_counter(&self) {
        assert!(self.attached_counter.get() > 0);
        self.attached_counter.set(self.attached_counter.get() - 1);
    }

    /// glGetShaderInfoLog
    pub(crate) fn info_log(&self) -> DOMString {
        self.info_log.borrow().clone()
    }

    /// Get the shader source
    pub(crate) fn source(&self) -> DOMString {
        self.source.borrow().clone()
    }

    /// glShaderSource
    pub(crate) fn set_source(&self, source: DOMString) {
        *self.source.borrow_mut() = source;
    }

    pub(crate) fn successfully_compiled(&self) -> bool {
        self.compilation_status.get() == ShaderCompilationStatus::Succeeded
    }
}

impl Drop for WebGLShader {
    fn drop(&mut self) {
        self.mark_for_deletion(Operation::Fallible);
    }
}
