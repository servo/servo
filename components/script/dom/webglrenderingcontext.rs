/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use canvas::webgl_paint_task::WebGLPaintTask;
use canvas::canvas_msg::{CanvasMsg, CanvasWebGLMsg, CanvasCommonMsg};
use dom::bindings::codegen::Bindings::WebGLRenderingContextBinding;
use dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::{ WebGLRenderingContextMethods, WebGLRenderingContextConstants};
use dom::bindings::global::{GlobalRef, GlobalField};
use dom::bindings::js::{JS, JSRef, LayoutJS, Temporary};
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::htmlcanvaselement::{HTMLCanvasElement};
use dom::webglbuffer::{WebGLBuffer, WebGLBufferHelpers};
use dom::webglshader::{WebGLShader, WebGLShaderHelpers};
use dom::webglprogram::{WebGLProgram, WebGLProgramHelpers};
use dom::webgluniformlocation::{WebGLUniformLocation, WebGLUniformLocationHelpers};
use geom::size::Size2D;
use js::jsapi::{JSContext, JSObject};
use js::jsfriendapi::bindgen::{JS_GetFloat32ArrayData, JS_GetObjectAsArrayBufferView};
use js::jsval::{JSVal, NullValue, Int32Value};
use std::ptr;
use std::sync::mpsc::{channel, Sender};
use util::str::DOMString;

#[dom_struct]
pub struct WebGLRenderingContext {
    reflector_: Reflector,
    global: GlobalField,
    renderer: Sender<CanvasMsg>,
    canvas: JS<HTMLCanvasElement>,
}

impl WebGLRenderingContext {
    fn new_inherited(global: GlobalRef, canvas: JSRef<HTMLCanvasElement>, size: Size2D<i32>)
                     -> Result<WebGLRenderingContext, &'static str> {
        let chan = try!(WebGLPaintTask::start(size));

        Ok(WebGLRenderingContext {
            reflector_: Reflector::new(),
            global: GlobalField::from_rooted(&global),
            renderer: chan,
            canvas: JS::from_rooted(canvas),
        })
    }

    pub fn new(global: GlobalRef, canvas: JSRef<HTMLCanvasElement>, size: Size2D<i32>)
               -> Option<Temporary<WebGLRenderingContext>> {
        match WebGLRenderingContext::new_inherited(global, canvas, size) {
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

#[unsafe_destructor]
impl Drop for WebGLRenderingContext {
    fn drop(&mut self) {
        self.renderer.send(CanvasMsg::Common(CanvasCommonMsg::Close)).unwrap();
    }
}

impl<'a> WebGLRenderingContextMethods for JSRef<'a, WebGLRenderingContext> {
    fn AttachShader(self, program: Option<JSRef<WebGLProgram>>, shader: Option<JSRef<WebGLShader>>) {
        let program_id = program.unwrap().get_id();
        let shader_id = shader.unwrap().get_id();
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::AttachShader(program_id, shader_id))).unwrap()
    }

    fn BindBuffer(self, buffer_type: u32, buffer: Option<JSRef<WebGLBuffer>>) -> () {
        let buffer_id = buffer.unwrap().get_id();
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::BindBuffer(buffer_type, buffer_id))).unwrap()
    }

    #[allow(unsafe_code)]
    fn BufferData(self, cx: *mut JSContext, target: u32, data: Option<*mut JSObject>, usage: u32) -> () {
        let data = match data {
            Some(data) => data,
            None => return,
        };
        let data_vec;
        unsafe {
            let mut length = 0;
            let mut ptr = ptr::null_mut();
            let buffer_data = JS_GetObjectAsArrayBufferView(cx, data, &mut length, &mut ptr);
            if buffer_data.is_null() {
                panic!("Argument data to WebGLRenderingContext.bufferdata is not an Float32Array")
            }
            let data_f32 = JS_GetFloat32ArrayData(buffer_data, cx);
            let data_vec_length = length / 4;
            data_vec = Vec::from_raw_buf(data_f32, data_vec_length as usize);
        }
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::BufferData(target, data_vec, usage))).unwrap()
    }

    fn Clear(self, mask: u32) -> () {
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::Clear(mask))).unwrap()
    }

    fn ClearColor(self, red: f32, green: f32, blue: f32, alpha: f32) -> () {
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::ClearColor(red, green, blue, alpha))).unwrap()
    }

    fn CompileShader(self, shader: Option<JSRef<WebGLShader>>) -> () {
        if shader.is_none() {
            return;
        }
        let shader_id = shader.unwrap().get_id();
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::CompileShader(shader_id))).unwrap()
    }

    fn CreateBuffer(self) -> Option<Temporary<WebGLBuffer>> {
        let (sender, receiver) = channel::<u32>();
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::CreateBuffer(sender))).unwrap();
        Some(WebGLBuffer::new(self.global.root().r(), receiver.recv().unwrap()))
    }

    fn CreateProgram(self) -> Option<Temporary<WebGLProgram>> {
        let (sender, receiver) = channel::<u32>();
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::CreateProgram(sender))).unwrap();
        Some(WebGLProgram::new(self.global.root().r(), receiver.recv().unwrap()))
    }

    fn CreateShader(self, shader_type: u32) -> Option<Temporary<WebGLShader>> {
        let (sender, receiver) = channel::<u32>();
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::CreateShader(shader_type, sender))).unwrap();
        Some(WebGLShader::new(self.global.root().r(), receiver.recv().unwrap()))
    }

    fn DrawArrays(self, mode: u32, first: i32, count: i32) {
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::DrawArrays(mode, first, count))).unwrap()
    }

    fn EnableVertexAttribArray(self, attrib_id: u32) -> () {
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::EnableVertexAttribArray(attrib_id))).unwrap()
    }

    fn GetAttribLocation(self, program: Option<JSRef<WebGLProgram>>, name: DOMString ) -> i32 {
        let (sender, receiver) = channel::<i32>();
        let program_id = program.unwrap().get_id();
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::GetAttribLocation(program_id, name, sender))).unwrap();
        receiver.recv().unwrap()
    }

    fn GetShaderInfoLog(self, shader: Option<JSRef<WebGLShader>>) -> Option<DOMString> {
        if shader.is_none() {
            return None;
        }
        let (sender, receiver) = channel::<String>();
        let shader_id = shader.unwrap().get_id();
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::GetShaderInfoLog(shader_id, sender))).unwrap();
        let info = receiver.recv().unwrap();
        Some(info)
    }

    fn GetShaderParameter(self, _: *mut JSContext, shader: Option<JSRef<WebGLShader>>, param_id: u32) -> JSVal {
        if shader.is_none() {
            return NullValue();
        }
        let shader_id = shader.unwrap().get_id();
        let (sender, receiver) = channel::<i32>();
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::GetShaderParameter(shader_id, param_id, sender))).unwrap();
        Int32Value(receiver.recv().unwrap())
    }

    fn GetUniformLocation(self, program: Option<JSRef<WebGLProgram>>, name: DOMString) -> Option<Temporary<WebGLUniformLocation>> {
        let program_id = program.unwrap().get_id();
        let (sender, receiver) = channel::<u32>();
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::GetUniformLocation(program_id, name, sender))).unwrap();
        Some(WebGLUniformLocation::new(self.global.root().r(), receiver.recv().unwrap()))
    }

    fn LinkProgram(self, program: Option<JSRef<WebGLProgram>>) {
        let program_id = program.unwrap().get_id();
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::LinkProgram(program_id))).unwrap()
    }

    fn ShaderSource(self, shader: Option<JSRef<WebGLShader>>, source: DOMString) -> () {
        if shader.is_none() {
            return;
        }
        let shader_id = shader.unwrap().get_id();
        let source_lines: Vec<String> = source.trim().split(|c: char| c == '\n').map(|line: &str| String::from_str(line) + "\n").collect();
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::ShaderSource(shader_id, source_lines))).unwrap()
    }

    #[allow(unsafe_code)]
    fn Uniform4fv(self, cx: *mut JSContext, uniform: Option<JSRef<WebGLUniformLocation>>, data: Option<*mut JSObject>) -> () {
        if uniform.is_none() || data.is_none() {
            return;
        }
        let data_vec: Vec<f32>;
        unsafe {
            let data_f32 = JS_GetFloat32ArrayData(data.unwrap(), cx);
            data_vec = Vec::from_raw_buf(data_f32, 4);
        }
        let uniform_id = uniform.unwrap().get_id();
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::Uniform4fv(uniform_id, data_vec))).unwrap()
    }

    fn UseProgram(self, program: Option<JSRef<WebGLProgram>>) -> () {
        let program_id = program.unwrap().get_id();
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::UseProgram(program_id as u32))).unwrap()
    }

    fn VertexAttribPointer(self, attrib_id: u32, size: i32, data_type: u32,
                           normalized: bool, stride: i32, offset: i64) -> (){
        match data_type {
            WebGLRenderingContextConstants::FLOAT => {
                self.renderer.send(
                    CanvasMsg::WebGL(CanvasWebGLMsg::VertexAttribPointer2f(attrib_id, size, normalized, stride, offset))).unwrap()
            }
            _ => panic!("VertexAttribPointer: Data Type not supported")
        }

    }

    fn Viewport(self, x: i32, y: i32, width: i32, height: i32) -> () {
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
