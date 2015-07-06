/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use canvas::webgl_paint_task::WebGLPaintTask;
use canvas_traits::
            {CanvasMsg, CanvasWebGLMsg, CanvasCommonMsg, WebGLError,
             WebGLShaderParameter, WebGLFramebufferBindingRequest};
use dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::
            {self, WebGLContextAttributes, WebGLRenderingContextMethods};
use dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLRenderingContextConstants as constants;

use dom::bindings::global::{GlobalRef, GlobalField};
use dom::bindings::js::{JS, LayoutJS, Root};
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::bindings::conversions::ToJSValConvertible;
use dom::htmlcanvaselement::{HTMLCanvasElement};
use dom::webglbuffer::{WebGLBuffer, WebGLBufferHelpers};
use dom::webglframebuffer::{WebGLFramebuffer, WebGLFramebufferHelpers};
use dom::webglrenderbuffer::{WebGLRenderbuffer, WebGLRenderbufferHelpers};
use dom::webgltexture::{WebGLTexture, WebGLTextureHelpers};
use dom::webglshader::{WebGLShader, WebGLShaderHelpers};
use dom::webglprogram::{WebGLProgram, WebGLProgramHelpers};
use dom::webgluniformlocation::{WebGLUniformLocation, WebGLUniformLocationHelpers};
use euclid::size::Size2D;
use js::jsapi::{JSContext, JSObject, RootedValue};
use js::jsapi::{JS_GetFloat32ArrayData, JS_GetObjectAsArrayBufferView};
use js::jsval::{JSVal, UndefinedValue, NullValue, Int32Value, BooleanValue};
use std::cell::Cell;
use std::mem;
use std::ptr;
use std::slice;
use std::sync::mpsc::{channel, Sender};
use util::str::DOMString;
use offscreen_gl_context::GLContextAttributes;

pub const MAX_UNIFORM_AND_ATTRIBUTE_LEN: usize = 256;

macro_rules! handle_potential_webgl_error {
    ($context:ident, $call:expr, $return_on_error:expr) => {
        match $call {
            Ok(ret) => ret,
            Err(error) => {
                $context.handle_webgl_error(error);
                $return_on_error
            }
        }
    }
}

#[dom_struct]
pub struct WebGLRenderingContext {
    reflector_: Reflector,
    global: GlobalField,
    renderer: Sender<CanvasMsg>,
    canvas: JS<HTMLCanvasElement>,
    last_error: Cell<Option<WebGLError>>,
}

impl WebGLRenderingContext {
    fn new_inherited(global: GlobalRef,
                     canvas: &HTMLCanvasElement,
                     size: Size2D<i32>,
                     attrs: GLContextAttributes)
                     -> Result<WebGLRenderingContext, &'static str> {
        let chan = try!(WebGLPaintTask::start(size, attrs));

        Ok(WebGLRenderingContext {
            reflector_: Reflector::new(),
            global: GlobalField::from_rooted(&global),
            renderer: chan,
            last_error: Cell::new(None),
            canvas: JS::from_ref(canvas),
        })
    }

    pub fn new(global: GlobalRef, canvas: &HTMLCanvasElement, size: Size2D<i32>, attrs: GLContextAttributes)
               -> Option<Root<WebGLRenderingContext>> {
        match WebGLRenderingContext::new_inherited(global, canvas, size, attrs) {
            Ok(ctx) => Some(reflect_dom_object(box ctx, global,
                                               WebGLRenderingContextBinding::Wrap)),
            Err(msg) => {
                error!("Couldn't create WebGLRenderingContext: {}", msg);
                None
            }
        }
    }

    pub fn recreate(&self, size: Size2D<i32>) {
        self.renderer.send(CanvasMsg::Common(CanvasCommonMsg::Recreate(size))).unwrap();
    }
}

impl Drop for WebGLRenderingContext {
    fn drop(&mut self) {
        self.renderer.send(CanvasMsg::Common(CanvasCommonMsg::Close)).unwrap();
    }
}

impl<'a> WebGLRenderingContextMethods for &'a WebGLRenderingContext {
    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.1
    fn Canvas(self) -> Root<HTMLCanvasElement> {
        self.canvas.root()
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.1
    fn DrawingBufferWidth(self) -> i32 {
        let (sender, receiver) = channel();
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::DrawingBufferWidth(sender))).unwrap();
        receiver.recv().unwrap()
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.1
    fn DrawingBufferHeight(self) -> i32 {
        let (sender, receiver) = channel();
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::DrawingBufferHeight(sender))).unwrap();
        receiver.recv().unwrap()
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn GetParameter(self, cx: *mut JSContext, parameter: u32) -> JSVal {
        // TODO(ecoal95): Implement the missing parameters from the spec
        let mut rval = RootedValue::new(cx, UndefinedValue());
        match parameter {
            constants::VERSION =>
                "WebGL 1.0".to_jsval(cx, rval.handle_mut()),
            constants::RENDERER |
            constants::VENDOR =>
                "Mozilla/Servo".to_jsval(cx, rval.handle_mut()),
            _ => rval.ptr = NullValue(),
        }
        rval.ptr
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn GetError(self) -> u32 {
        let error_code = if let Some(error) = self.last_error.get() {
            match error {
                WebGLError::InvalidEnum => constants::INVALID_ENUM,
                WebGLError::InvalidValue => constants::INVALID_VALUE,
                WebGLError::InvalidOperation => constants::INVALID_OPERATION,
                WebGLError::OutOfMemory => constants::OUT_OF_MEMORY,
                WebGLError::ContextLost => constants::CONTEXT_LOST_WEBGL,
            }
        } else {
            constants::NO_ERROR
        };
        self.last_error.set(None);
        error_code
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.2
    fn GetContextAttributes(self) -> Option<WebGLContextAttributes> {
        let (sender, receiver) = channel();

        // If the send does not succeed, assume context lost
        if let Err(_) = self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::GetContextAttributes(sender))) {
            return None;
        }
        let attrs = receiver.recv().unwrap();

        Some(WebGLContextAttributes {
            alpha: attrs.alpha,
            antialias: attrs.antialias,
            depth: attrs.depth,
            failIfMajorPerformanceCaveat: false,
            preferLowPowerToHighPerformance: false,
            premultipliedAlpha: attrs.premultiplied_alpha,
            preserveDrawingBuffer: attrs.preserve_drawing_buffer,
            stencil: attrs.stencil
        })
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.14
    fn GetExtension(self, _cx: *mut JSContext, _name: DOMString) -> *mut JSObject {
        // TODO(ecoal95) we actually do not support extensions.
        // `getSupportedExtensions` cannot be implemented as of right now (see #544)
        0 as *mut JSObject
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn ActiveTexture(self, texture: u32) {
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::ActiveTexture(texture))).unwrap();
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn BlendColor(self, r: f32, g: f32, b: f32, a: f32) {
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::BlendColor(r, g, b, a))).unwrap();
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn BlendEquation(self, mode: u32) {
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::BlendEquation(mode))).unwrap();
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn BlendEquationSeparate(self, mode_rgb: u32, mode_alpha: u32) {
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::BlendEquationSeparate(mode_rgb, mode_alpha))).unwrap();
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn BlendFunc(self, src_factor: u32, dest_factor: u32) {
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::BlendFunc(src_factor, dest_factor))).unwrap();
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn BlendFuncSeparate(self, src_rgb: u32, dest_rgb: u32, src_alpha: u32, dest_alpha: u32) {
        self.renderer.send(
            CanvasMsg::WebGL(CanvasWebGLMsg::BlendFuncSeparate(src_rgb, dest_rgb, src_alpha, dest_alpha))).unwrap();
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn AttachShader(self, program: Option<&WebGLProgram>, shader: Option<&WebGLShader>) {
        if let Some(program) = program {
            if let Some(shader) = shader {
                handle_potential_webgl_error!(self, program.attach_shader(shader), ());
            }
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.5
    fn BindBuffer(self, target: u32, buffer: Option<&WebGLBuffer>) {
        if let Some(buffer) = buffer {
            buffer.bind(target)
        } else {
            // Unbind the current buffer
            self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::BindBuffer(target, 0))).unwrap()
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.6
    fn BindFramebuffer(self, target: u32, framebuffer: Option<&WebGLFramebuffer>) {
        if let Some(framebuffer) = framebuffer {
            framebuffer.bind(target)
        } else {
            // Bind the default framebuffer
            let cmd = CanvasWebGLMsg::BindFramebuffer(target, WebGLFramebufferBindingRequest::Default);
            self.renderer.send(CanvasMsg::WebGL(cmd)).unwrap();
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.7
    fn BindRenderbuffer(self, target: u32, renderbuffer: Option<&WebGLRenderbuffer>) {
        if let Some(renderbuffer) = renderbuffer {
            renderbuffer.bind(target)
        } else {
            // Unbind the currently bound renderbuffer
            self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::BindRenderbuffer(target, 0))).unwrap()
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn BindTexture(self, target: u32, texture: Option<&WebGLTexture>) {
        if let Some(texture) = texture {
            texture.bind(target)
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.5
    #[allow(unsafe_code)]
    fn BufferData(self, _cx: *mut JSContext, target: u32, data: Option<*mut JSObject>, usage: u32) {
        let data = match data {
            Some(data) => data,
            None => return,
        };
        let data_vec = unsafe {
            let mut length = 0;
            let mut ptr = ptr::null_mut();
            let buffer_data = JS_GetObjectAsArrayBufferView(data, &mut length, &mut ptr);
            if buffer_data.is_null() {
                panic!("Argument data to WebGLRenderingContext.bufferdata is not a Float32Array")
            }
            let data_f32 = JS_GetFloat32ArrayData(buffer_data, ptr::null());
            let data_vec_length = length / mem::size_of::<f32>() as u32;
            slice::from_raw_parts(data_f32, data_vec_length as usize).to_vec()
        };
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::BufferData(target, data_vec, usage))).unwrap()
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.11
    fn Clear(self, mask: u32) {
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::Clear(mask))).unwrap()
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn ClearColor(self, red: f32, green: f32, blue: f32, alpha: f32) {
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::ClearColor(red, green, blue, alpha))).unwrap()
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn CompileShader(self, shader: Option<&WebGLShader>) {
        if let Some(shader) = shader {
            shader.compile()
        }
    }

    // TODO(ecoal95): Probably in the future we should keep track of the
    // generated objects, either here or in the webgl task
    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.5
    fn CreateBuffer(self) -> Option<Root<WebGLBuffer>> {
        WebGLBuffer::maybe_new(self.global.root().r(), self.renderer.clone())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.6
    fn CreateFramebuffer(self) -> Option<Root<WebGLFramebuffer>> {
        WebGLFramebuffer::maybe_new(self.global.root().r(), self.renderer.clone())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.7
    fn CreateRenderbuffer(self) -> Option<Root<WebGLRenderbuffer>> {
        WebGLRenderbuffer::maybe_new(self.global.root().r(), self.renderer.clone())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn CreateTexture(self) -> Option<Root<WebGLTexture>> {
        WebGLTexture::maybe_new(self.global.root().r(), self.renderer.clone())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn CreateProgram(self) -> Option<Root<WebGLProgram>> {
        WebGLProgram::maybe_new(self.global.root().r(), self.renderer.clone())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    // TODO(ecoal95): Check if constants are cross-platform or if we must make a translation
    // between WebGL constants and native ones.
    fn CreateShader(self, shader_type: u32) -> Option<Root<WebGLShader>> {
        WebGLShader::maybe_new(self.global.root().r(), self.renderer.clone(), shader_type)
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.5
    fn DeleteBuffer(self, buffer: Option<&WebGLBuffer>) {
        if let Some(buffer) = buffer {
            buffer.delete()
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.6
    fn DeleteFramebuffer(self, framebuffer: Option<&WebGLFramebuffer>) {
        if let Some(framebuffer) = framebuffer {
            framebuffer.delete()
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.7
    fn DeleteRenderbuffer(self, renderbuffer: Option<&WebGLRenderbuffer>) {
        if let Some(renderbuffer) = renderbuffer {
            renderbuffer.delete()
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn DeleteTexture(self, texture: Option<&WebGLTexture>) {
        if let Some(texture) = texture {
            texture.delete()
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn DeleteProgram(self, program: Option<&WebGLProgram>) {
        if let Some(program) = program {
            program.delete()
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn DeleteShader(self, shader: Option<&WebGLShader>) {
        if let Some(shader) = shader {
            shader.delete()
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.11
    fn DrawArrays(self, mode: u32, first: i32, count: i32) {
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::DrawArrays(mode, first, count))).unwrap()
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn EnableVertexAttribArray(self, attrib_id: u32) {
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::EnableVertexAttribArray(attrib_id))).unwrap()
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn GetAttribLocation(self, program: Option<&WebGLProgram>, name: DOMString) -> i32 {
        if let Some(program) = program {
            handle_potential_webgl_error!(self, program.get_attrib_location(name), None).unwrap_or(-1)
        } else {
            -1
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn GetShaderInfoLog(self, shader: Option<&WebGLShader>) -> Option<DOMString> {
        if let Some(shader) = shader {
            shader.info_log()
        } else {
            None
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn GetShaderParameter(self, _: *mut JSContext, shader: Option<&WebGLShader>, param_id: u32) -> JSVal {
        if let Some(shader) = shader {
            match handle_potential_webgl_error!(self, shader.parameter(param_id), WebGLShaderParameter::Invalid) {
                WebGLShaderParameter::Int(val) => Int32Value(val),
                WebGLShaderParameter::Bool(val) => BooleanValue(val),
                WebGLShaderParameter::Invalid => NullValue(),
            }
        } else {
            NullValue()
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn GetUniformLocation(self,
                          program: Option<&WebGLProgram>,
                          name: DOMString) -> Option<Root<WebGLUniformLocation>> {
        if let Some(program) = program {
            handle_potential_webgl_error!(self, program.get_uniform_location(name), None)
                .map(|location| WebGLUniformLocation::new(self.global.root().r(), location))
        } else {
            None
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn LinkProgram(self, program: Option<&WebGLProgram>) {
        if let Some(program) = program {
            program.link()
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn ShaderSource(self, shader: Option<&WebGLShader>, source: DOMString) {
        if let Some(shader) = shader {
            shader.set_source(source)
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn GetShaderSource(self, shader: Option<&WebGLShader>) -> Option<DOMString> {
        if let Some(shader) = shader {
            shader.source()
        } else {
            None
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    #[allow(unsafe_code)]
    fn Uniform4fv(self,
                  _cx: *mut JSContext,
                  uniform: Option<&WebGLUniformLocation>,
                  data: Option<*mut JSObject>) {
        let uniform_id = match uniform {
            Some(uniform) => uniform.id(),
            None => return,
        };

        let data = match data {
            Some(data) => data,
            None => return,
        };

        let data_vec = unsafe {
            let data_f32 = JS_GetFloat32ArrayData(data, ptr::null());
            slice::from_raw_parts(data_f32, 4).to_vec()
        };
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::Uniform4fv(uniform_id, data_vec))).unwrap()
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn UseProgram(self, program: Option<&WebGLProgram>) {
        if let Some(program) = program {
            program.use_program()
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn VertexAttribPointer(self, attrib_id: u32, size: i32, data_type: u32,
                           normalized: bool, stride: i32, offset: i64) {
        match data_type {
            constants::FLOAT => {
               let msg = CanvasMsg::WebGL(
                   CanvasWebGLMsg::VertexAttribPointer2f(attrib_id, size, normalized, stride, offset));
                self.renderer.send(msg).unwrap()
            }
            _ => panic!("VertexAttribPointer: Data Type not supported")
        }

    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.4
    fn Viewport(self, x: i32, y: i32, width: i32, height: i32) {
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::Viewport(x, y, width, height))).unwrap()
    }
}

pub trait WebGLRenderingContextHelpers {
    fn handle_webgl_error(&self, err: WebGLError);
}

impl<'a> WebGLRenderingContextHelpers for &'a WebGLRenderingContext {
    fn handle_webgl_error(&self, err: WebGLError) {
        // If an error has been detected no further errors must be
        // recorded until `getError` has been called
        if self.last_error.get().is_none() {
            self.last_error.set(Some(err));
        }
    }
}

pub trait LayoutCanvasWebGLRenderingContextHelpers {
    #[allow(unsafe_code)]
    unsafe fn get_renderer(&self) -> Sender<CanvasMsg>;
}

impl LayoutCanvasWebGLRenderingContextHelpers for LayoutJS<WebGLRenderingContext> {
    #[allow(unsafe_code)]
    unsafe fn get_renderer(&self) -> Sender<CanvasMsg> {
        (*self.unsafe_get()).renderer.clone()
    }
}
