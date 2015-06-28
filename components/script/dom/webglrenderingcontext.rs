/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use canvas::webgl_paint_task::WebGLPaintTask;
use canvas_traits::{CanvasMsg, CanvasWebGLMsg, CanvasCommonMsg};
use dom::bindings::codegen::Bindings::WebGLRenderingContextBinding;
use dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::{
    WebGLContextAttributes, WebGLRenderingContextMethods, WebGLRenderingContextConstants};
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
use js::jsval::{JSVal, UndefinedValue, NullValue, Int32Value};
use std::mem;
use std::ptr;
use std::slice;
use std::sync::mpsc::{channel, Sender};
use util::str::DOMString;
use offscreen_gl_context::GLContextAttributes;

#[dom_struct]
pub struct WebGLRenderingContext {
    reflector_: Reflector,
    global: GlobalField,
    renderer: Sender<CanvasMsg>,
    canvas: JS<HTMLCanvasElement>,
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
            WebGLRenderingContextConstants::VERSION =>
                "WebGL 1.0".to_jsval(cx, rval.handle_mut()),
            WebGLRenderingContextConstants::RENDERER |
            WebGLRenderingContextConstants::VENDOR =>
                "Mozilla/Servo".to_jsval(cx, rval.handle_mut()),
            _ => rval.ptr = NullValue(),
        }
        rval.ptr
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
        let program_id = match program {
            Some(program) => program.get_id(),
            None => return,
        };
        let shader_id = match shader {
            Some(shader) => shader.get_id(),
            None => return,
        };
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::AttachShader(program_id, shader_id))).unwrap()
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.5
    fn BindBuffer(self, target: u32, buffer: Option<&WebGLBuffer>) {
        let id = buffer.map(|buf| buf.get_id()).unwrap_or(0);
        self.renderer.send(
            CanvasMsg::WebGL(CanvasWebGLMsg::BindBuffer(target, id))).unwrap()
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.6
    fn BindFramebuffer(self, target: u32, framebuffer: Option<&WebGLFramebuffer>) {
        let id = framebuffer.map(|fb| fb.get_id()).unwrap_or(0);
        self.renderer.send(
            CanvasMsg::WebGL(CanvasWebGLMsg::BindFramebuffer(target, id))).unwrap()
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.7
    fn BindRenderbuffer(self, target: u32, renderbuffer: Option<&WebGLRenderbuffer>) {
        let id = renderbuffer.map(|rb| rb.get_id()).unwrap_or(0);
        self.renderer.send(
            CanvasMsg::WebGL(CanvasWebGLMsg::BindRenderbuffer(target, id))).unwrap()
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn BindTexture(self, target: u32, texture: Option<&WebGLTexture>) {
        let id = texture.map(|tex| tex.get_id()).unwrap_or(0);
        self.renderer.send(
            CanvasMsg::WebGL(CanvasWebGLMsg::BindTexture(target, id))).unwrap()
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
        let shader_id = match shader {
            Some(shader) => shader.get_id(),
            None => return,
        };
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::CompileShader(shader_id))).unwrap()
    }

    // TODO(ecoal95): Probably in the future we should keep track of the
    // generated objects, either here or in the webgl task
    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.5
    fn CreateBuffer(self) -> Option<Root<WebGLBuffer>> {
        let (sender, receiver) = channel();
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::CreateBuffer(sender))).unwrap();
        receiver.recv().unwrap()
            .map(|buffer_id| WebGLBuffer::new(self.global.root().r(), *buffer_id))
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.6
    fn CreateFramebuffer(self) -> Option<Root<WebGLFramebuffer>> {
        let (sender, receiver) = channel();
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::CreateFramebuffer(sender))).unwrap();
        receiver.recv().unwrap()
            .map(|fb_id| WebGLFramebuffer::new(self.global.root().r(), *fb_id))
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.7
    fn CreateRenderbuffer(self) -> Option<Root<WebGLRenderbuffer>> {
        let (sender, receiver) = channel();
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::CreateRenderbuffer(sender))).unwrap();
        receiver.recv().unwrap()
            .map(|program_id| WebGLRenderbuffer::new(self.global.root().r(), *program_id))
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn CreateTexture(self) -> Option<Root<WebGLTexture>> {
        let (sender, receiver) = channel();
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::CreateTexture(sender))).unwrap();
        receiver.recv().unwrap()
            .map(|texture_id| WebGLTexture::new(self.global.root().r(), *texture_id))
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn CreateProgram(self) -> Option<Root<WebGLProgram>> {
        let (sender, receiver) = channel();
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::CreateProgram(sender))).unwrap();
        receiver.recv().unwrap()
            .map(|program_id| WebGLProgram::new(self.global.root().r(), *program_id))
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    // TODO(ecoal95): Check if constants are cross-platform or if we must make a translation
    // between WebGL constants and native ones.
    fn CreateShader(self, shader_type: u32) -> Option<Root<WebGLShader>> {
        let (sender, receiver) = channel();
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::CreateShader(shader_type, sender))).unwrap();
        receiver.recv().unwrap()
            .map(|shader_id| WebGLShader::new(self.global.root().r(), *shader_id))
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.5
    fn DeleteBuffer(self, buffer: Option<&WebGLBuffer>) {
        if let Some(buffer) = buffer {
            self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::DeleteBuffer(buffer.get_id()))).unwrap();
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.6
    fn DeleteFramebuffer(self, framebuffer: Option<&WebGLFramebuffer>) {
        if let Some(framebuffer) = framebuffer {
            self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::DeleteFramebuffer(framebuffer.get_id()))).unwrap();
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.7
    fn DeleteRenderbuffer(self, renderbuffer: Option<&WebGLRenderbuffer>) {
        if let Some(renderbuffer) = renderbuffer {
            self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::DeleteRenderbuffer(renderbuffer.get_id()))).unwrap();
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn DeleteTexture(self, texture: Option<&WebGLTexture>) {
        if let Some(texture) = texture {
            self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::DeleteTexture(texture.get_id()))).unwrap();
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn DeleteProgram(self, program: Option<&WebGLProgram>) {
        if let Some(program) = program {
            self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::DeleteProgram(program.get_id()))).unwrap();
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn DeleteShader(self, shader: Option<&WebGLShader>) {
        if let Some(shader) = shader {
            self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::DeleteShader(shader.get_id()))).unwrap();
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
        let program_id = match program {
            Some(program) => program.get_id(),
            None => return -1,
        };
        let (sender, receiver) = channel();
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::GetAttribLocation(program_id, name, sender))).unwrap();
        receiver.recv().unwrap()
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn GetShaderInfoLog(self, shader: Option<&WebGLShader>) -> Option<DOMString> {
        if let Some(shader) = shader {
            let (sender, receiver) = channel();
            self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::GetShaderInfoLog(shader.get_id(), sender))).unwrap();
            Some(receiver.recv().unwrap())
        } else {
            None
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn GetShaderParameter(self, _: *mut JSContext, shader: Option<&WebGLShader>, param_id: u32) -> JSVal {
        if let Some(shader) = shader {
            let (sender, receiver) = channel();
            self.renderer.send(
                CanvasMsg::WebGL(CanvasWebGLMsg::GetShaderParameter(shader.get_id(), param_id, sender))).unwrap();
            Int32Value(receiver.recv().unwrap())
        } else {
            NullValue()
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn GetUniformLocation(self,
                          program: Option<&WebGLProgram>,
                          name: DOMString) -> Option<Root<WebGLUniformLocation>> {
        if let Some(program) = program {
            let (sender, receiver) = channel();
            self.renderer.send(
                CanvasMsg::WebGL(CanvasWebGLMsg::GetUniformLocation(program.get_id(), name, sender))).unwrap();
            receiver.recv().unwrap()
                .map(|location| WebGLUniformLocation::new(self.global.root().r(), location))
        } else {
            None
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn LinkProgram(self, program: Option<&WebGLProgram>) {
        if let Some(program) = program {
            self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::LinkProgram(program.get_id()))).unwrap()
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn ShaderSource(self, shader: Option<&WebGLShader>, source: DOMString) {
        if let Some(shader) = shader {
            self.renderer.send(
                CanvasMsg::WebGL(CanvasWebGLMsg::ShaderSource(shader.get_id(), source))).unwrap();
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    #[allow(unsafe_code)]
    fn Uniform4fv(self,
                  _cx: *mut JSContext,
                  uniform: Option<&WebGLUniformLocation>,
                  data: Option<*mut JSObject>) {
        let uniform_id = match uniform {
            Some(uniform) => uniform.get_id(),
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
        let program_id = match program {
            Some(program) => program.get_id(),
            None => return,
        };
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::UseProgram(program_id as u32))).unwrap()
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn VertexAttribPointer(self, attrib_id: u32, size: i32, data_type: u32,
                           normalized: bool, stride: i32, offset: i64) {
        match data_type {
            WebGLRenderingContextConstants::FLOAT => {
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
