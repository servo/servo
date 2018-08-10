/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use canvas_traits::webgl::{WebGLCommand, WebGLError};
use canvas_traits::webgl::{WebGLResult, WebGLSLVersion, WebGLShaderId};
use canvas_traits::webgl::{WebGLVersion, webgl_channel};
use dom::bindings::cell::DomRefCell;
use dom::bindings::codegen::Bindings::WebGLShaderBinding;
use dom::bindings::inheritance::Castable;
use dom::bindings::reflector::{DomObject, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::bindings::str::DOMString;
use dom::webgl_extensions::WebGLExtensions;
use dom::webgl_extensions::ext::extshadertexturelod::EXTShaderTextureLod;
use dom::webgl_extensions::ext::oesstandardderivatives::OESStandardDerivatives;
use dom::webglobject::WebGLObject;
use dom::webglrenderingcontext::WebGLRenderingContext;
use dom_struct::dom_struct;
use mozangle::shaders::{BuiltInResources, Output, ShaderValidator};
use offscreen_gl_context::GLLimits;
use std::cell::Cell;
use std::os::raw::c_int;
use std::sync::{ONCE_INIT, Once};
use typeholder::TypeHolderTrait;
use std::marker::PhantomData;

#[derive(Clone, Copy, Debug, JSTraceable, MallocSizeOf, PartialEq)]
pub enum ShaderCompilationStatus {
    NotCompiled,
    Succeeded,
    Failed,
}

#[dom_struct]
pub struct WebGLShader<TH: TypeHolderTrait> {
    webgl_object: WebGLObject<TH>,
    id: WebGLShaderId,
    gl_type: u32,
    source: DomRefCell<DOMString>,
    info_log: DomRefCell<DOMString>,
    marked_for_deletion: Cell<bool>,
    attached_counter: Cell<u32>,
    compilation_status: Cell<ShaderCompilationStatus>,
}

static GLSLANG_INITIALIZATION: Once = ONCE_INIT;

impl<TH: TypeHolderTrait> WebGLShader<TH> {
    fn new_inherited(
        context: &WebGLRenderingContext<TH>,
        id: WebGLShaderId,
        shader_type: u32,
    ) -> Self {
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

    pub fn maybe_new(context: &WebGLRenderingContext<TH>, shader_type: u32) -> Option<DomRoot<Self>> {
        let (sender, receiver) = webgl_channel().unwrap();
        context.send_command(WebGLCommand::CreateShader(shader_type, sender));
        receiver.recv().unwrap().map(|id| WebGLShader::new(context, id, shader_type))
    }

    pub fn new(
        context: &WebGLRenderingContext<TH>,
        id: WebGLShaderId,
        shader_type: u32,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(WebGLShader::new_inherited(context, id, shader_type)),
            &*context.global(),
            WebGLShaderBinding::Wrap,
        )
    }
}


impl<TH: TypeHolderTrait> WebGLShader<TH> {
    pub fn id(&self) -> WebGLShaderId {
        self.id
    }

    pub fn gl_type(&self) -> u32 {
        self.gl_type
    }

    /// glCompileShader
    pub fn compile(
        &self,
        webgl_version: WebGLVersion,
        glsl_version: WebGLSLVersion,
        limits: &GLLimits,
        ext: &WebGLExtensions<TH>,
    ) -> WebGLResult<()> {
        if self.marked_for_deletion.get() && !self.is_attached() {
            return Err(WebGLError::InvalidValue);
        }
        if self.compilation_status.get() != ShaderCompilationStatus::NotCompiled {
            debug!("Compiling already compiled shader {}", self.id);
        }

        let source = self.source.borrow();

        let params = BuiltInResources {
            MaxVertexAttribs: limits.max_vertex_attribs as c_int,
            MaxVertexUniformVectors: limits.max_vertex_uniform_vectors as c_int,
            MaxVaryingVectors: limits.max_varying_vectors as c_int,
            MaxVertexTextureImageUnits: limits.max_vertex_texture_image_units as c_int,
            MaxCombinedTextureImageUnits: limits.max_combined_texture_image_units as c_int,
            MaxTextureImageUnits: limits.max_texture_image_units as c_int,
            MaxFragmentUniformVectors: limits.max_fragment_uniform_vectors as c_int,
            OES_standard_derivatives: ext.is_enabled::<OESStandardDerivatives<TH>>() as c_int,
            EXT_shader_texture_lod: ext.is_enabled::<EXTShaderTextureLod<TH>>() as c_int,
            FragmentPrecisionHigh: 1,
            ..BuiltInResources::default()
        };
        let validator = match webgl_version {
            WebGLVersion::WebGL1 => {
                let output_format = if cfg!(any(target_os = "android", target_os = "ios")) {
                    Output::Essl
                } else {
                    Output::Glsl
                };
                ShaderValidator::for_webgl(self.gl_type,
                                            output_format,
                                            &params).unwrap()
            },
            WebGLVersion::WebGL2 => {
                let output_format = if cfg!(any(target_os = "android", target_os = "ios")) {
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
                        _ => Output::Glsl140
                    }
                };
                ShaderValidator::for_webgl2(self.gl_type,
                                            output_format,
                                            &params).unwrap()
            },
        };

        match validator.compile_and_translate(&[&source]) {
            Ok(translated_source) => {
                debug!("Shader translated: {}", translated_source);
                // NOTE: At this point we should be pretty sure that the compilation in the paint thread
                // will succeed.
                // It could be interesting to retrieve the info log from the paint thread though
                self.upcast::<WebGLObject<TH>>()
                    .context()
                    .send_command(WebGLCommand::CompileShader(self.id, translated_source));
                self.compilation_status.set(ShaderCompilationStatus::Succeeded);
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
    pub fn mark_for_deletion(&self) {
        if !self.marked_for_deletion.get() {
            self.marked_for_deletion.set(true);
            self.upcast::<WebGLObject<TH>>()
                .context()
                .send_command(WebGLCommand::DeleteShader(self.id));
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

impl<TH: TypeHolderTrait> Drop for WebGLShader<TH> {
    fn drop(&mut self) {
        self.mark_for_deletion();
    }
}
