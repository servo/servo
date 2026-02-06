use std::ffi::{CString, CStr};
use std::ptr;

#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[allow(non_upper_case_globals)]
mod bindings;

pub enum ShaderType {
    Vertex,
    Fragment,
}

pub enum Target {
    OpenGl,
    OpenGles20,
    OpenGles30,
    Metal,
}

pub struct Context {
    ctx: *mut bindings::glslopt_ctx,
}

impl Context {
    pub fn new(target: Target) -> Self {
        let target = match target {
            Target::OpenGl => bindings::glslopt_target_kGlslTargetOpenGL,
            Target::OpenGles20 => bindings::glslopt_target_kGlslTargetOpenGLES20,
            Target::OpenGles30 => bindings::glslopt_target_kGlslTargetOpenGLES30,
            Target::Metal => bindings::glslopt_target_kGlslTargetMetal,
        };

        let ctx = unsafe { bindings::glslopt_initialize(target) };

        Self {
            ctx,
        }
    }

    pub fn optimize(&self, shader_type: ShaderType, source: String) -> Shader {
        let shader_type = match shader_type {
            ShaderType::Vertex => bindings::glslopt_shader_type_kGlslOptShaderVertex,
            ShaderType::Fragment => bindings::glslopt_shader_type_kGlslOptShaderFragment,
        };
        let source = CString::new(source).unwrap();

        let shader = unsafe { bindings::glslopt_optimize(self.ctx, shader_type, source.as_ptr(), 0) };
        assert_ne!(shader, ptr::null_mut());
        Shader {
            shader,
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            bindings::glslopt_cleanup(self.ctx);
        }
    }
}

pub struct Shader {
    shader: *mut bindings::glslopt_shader,
}

impl Shader {
    pub fn get_status(&self) -> bool {
        unsafe { bindings::glslopt_get_status(self.shader) }
    }

    pub fn get_output(&self) -> Result<&str, ()> {
        unsafe {
            let cstr = bindings::glslopt_get_output(self.shader);
            if cstr == ptr::null() {
                Err(())
            } else {
                Ok(CStr::from_ptr(cstr).to_str().unwrap())
            }
        }
    }

    pub fn get_log(&self) -> &str {
        unsafe {
            let cstr = bindings::glslopt_get_log(self.shader);
            if cstr == ptr::null() {
                ""
            } else {
                CStr::from_ptr(cstr).to_str().unwrap()
            }
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            bindings::glslopt_shader_delete(self.shader);
        }
    }
}
