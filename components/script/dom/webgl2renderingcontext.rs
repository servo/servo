/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// https://www.khronos.org/registry/webgl/specs/latest/2.0/webgl.idl
use canvas_traits::webgl::{WebGLCommand, WebGLError, WebGLMsgSender, WebGLVersion};
use canvas_traits::webgl::{WebGLParameter, WebGLParameterVec, WebGLUniformBlockParameter};
use canvas_traits::webgl::WebGLError::*;
use canvas_traits::webgl::webgl_channel;
use dom::bindings::codegen::Bindings::WebGL2RenderingContextBinding;
use dom::bindings::codegen::Bindings::WebGL2RenderingContextBinding::WebGL2RenderingContextConstants as constants;
use dom::bindings::codegen::Bindings::WebGL2RenderingContextBinding::WebGL2RenderingContextMethods;
use dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLContextAttributes;
use dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLRenderingContextMethods;
use dom::bindings::codegen::UnionTypes::ImageDataOrHTMLImageElementOrHTMLCanvasElementOrHTMLVideoElement;
use dom::bindings::error::Fallible;
use dom::bindings::nonnull::NonNullJSObjectPtr;
use dom::bindings::reflector::{reflect_dom_object, Reflector};
use dom::bindings::root::{Dom, DomRoot, LayoutDom, MutNullableDom};
use dom::bindings::str::DOMString;
use dom::htmlcanvaselement::HTMLCanvasElement;
use dom::htmliframeelement::HTMLIFrameElement;
use dom::webglactiveinfo::WebGLActiveInfo;
use dom::webglbuffer::WebGLBuffer;
use dom::webglframebuffer::WebGLFramebuffer;
use dom::webglprogram::WebGLProgram;
use dom::webglrenderbuffer::WebGLRenderbuffer;
use dom::webglrenderingcontext::{LayoutCanvasWebGLRenderingContextHelpers, WebGLRenderingContext};
use dom::webglshader::WebGLShader;
use dom::webglshaderprecisionformat::WebGLShaderPrecisionFormat;
use dom::webgltexture::WebGLTexture;
use dom::webgluniformlocation::WebGLUniformLocation;
use dom::window::Window;
use dom_struct::dom_struct;
use euclid::Size2D;
use js::conversions::ToJSValConvertible;
use js::jsapi::{JSContext, JSObject};
use js::jsval::{BooleanValue, DoubleValue, Int32Value, JSVal, NullValue, UndefinedValue};
use offscreen_gl_context::GLContextAttributes;
use script_layout_interface::HTMLCanvasDataSource;

#[dom_struct]
pub struct WebGL2RenderingContext {
    reflector_: Reflector,
    base: Dom<WebGLRenderingContext>,
    #[ignore_malloc_size_of = "Channels are hard"]
    webgl_sender: WebGLMsgSender,
    bound_uniform_buffer: MutNullableDom<WebGLBuffer>,
    bound_transform_feedback_buffer: MutNullableDom<WebGLBuffer>,
}

impl WebGL2RenderingContext {
    fn new_inherited(
        window: &Window,
        canvas: &HTMLCanvasElement,
        size: Size2D<i32>,
        attrs: GLContextAttributes
    ) -> Option<WebGL2RenderingContext> {
        let base = WebGLRenderingContext::new(window, canvas, WebGLVersion::WebGL2, size, attrs)?;
        Some(WebGL2RenderingContext {
            reflector_: Reflector::new(),
            base: Dom::from_ref(&*base),
            webgl_sender: base.webgl_sender(),
            bound_uniform_buffer: MutNullableDom::new(None),
            bound_transform_feedback_buffer: MutNullableDom::new(None),
        })
    }

    #[allow(unrooted_must_root)]
    pub fn new(
        window: &Window,
        canvas: &HTMLCanvasElement,
        size: Size2D<i32>,
        attrs: GLContextAttributes
    ) -> Option<DomRoot<WebGL2RenderingContext>> {
        WebGL2RenderingContext::new_inherited(window, canvas, size, attrs).map(|ctx| {
            reflect_dom_object(Box::new(ctx), window, WebGL2RenderingContextBinding::Wrap)
        })
    }
}

impl WebGL2RenderingContext {
    pub fn recreate(&self, size: Size2D<i32>) {
        self.base.recreate(size)
    }

    pub fn base_context(&self) -> DomRoot<WebGLRenderingContext> {
        DomRoot::from_ref(&*self.base)
    }

    #[inline]
    fn send_command(&self, command: WebGLCommand) {
        self.webgl_sender.send(command).unwrap();
    }

    #[inline]
    fn webgl_error(&self, err: WebGLError) {
        self.base.webgl_error(err)
    }
}

impl WebGL2RenderingContextMethods for WebGL2RenderingContext {
    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.1
    fn Canvas(&self) -> DomRoot<HTMLCanvasElement> {
        self.base.Canvas()
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.11
    fn Flush(&self) {
        self.base.Flush()
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.11
    fn Finish(&self) {
        self.base.Finish()
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.1
    fn DrawingBufferWidth(&self) -> i32 {
        self.base.DrawingBufferWidth()
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.1
    fn DrawingBufferHeight(&self) -> i32 {
        self.base.DrawingBufferHeight()
    }

    #[allow(unsafe_code)]
    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.5
    unsafe fn GetBufferParameter(&self, _cx: *mut JSContext, target: u32, parameter: u32) -> JSVal {
        self.base.GetBufferParameter(_cx, target, parameter)
    }

    #[allow(unsafe_code)]
    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    unsafe fn GetParameter(&self, cx: *mut JSContext, parameter: u32) -> JSVal {
        match parameter {
            constants::UNIFORM_BUFFER_BINDING =>
                return object_binding_to_js_or_null!(cx, &self.bound_uniform_buffer),
            constants::TRANSFORM_FEEDBACK_BUFFER_BINDING =>
                return object_binding_to_js_or_null!(cx, &self.bound_transform_feedback_buffer),
            _ => return self.base.GetParameter(cx, parameter),
        }
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn GetError(&self) -> u32 {
        self.base.GetError()
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.2
    fn GetContextAttributes(&self) -> Option<WebGLContextAttributes> {
        self.base.GetContextAttributes()
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.14
    fn GetSupportedExtensions(&self) -> Option<Vec<DOMString>> {
        self.base.GetSupportedExtensions()
    }

    #[allow(unsafe_code)]
    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.14
    unsafe fn GetExtension(&self, cx: *mut JSContext, name: DOMString) -> Option<NonNullJSObjectPtr> {
        self.base.GetExtension(cx, name)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn ActiveTexture(&self, texture: u32) {
        self.base.ActiveTexture(texture)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn BlendColor(&self, r: f32, g: f32, b: f32, a: f32) {
        self.base.BlendColor(r, g, b, a)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn BlendEquation(&self, mode: u32) {
        self.base.BlendEquation(mode)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn BlendEquationSeparate(&self, mode_rgb: u32, mode_alpha: u32) {
        self.base.BlendEquationSeparate(mode_rgb, mode_alpha)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn BlendFunc(&self, src_factor: u32, dest_factor: u32) {
        self.base.BlendFunc(src_factor, dest_factor)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn BlendFuncSeparate(&self, src_rgb: u32, dest_rgb: u32, src_alpha: u32, dest_alpha: u32) {
        self.base.BlendFuncSeparate(src_rgb, dest_rgb, src_alpha, dest_alpha)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn AttachShader(&self, program: Option<&WebGLProgram>, shader: Option<&WebGLShader>) {
        self.base.AttachShader(program, shader)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn DetachShader(&self, program: Option<&WebGLProgram>, shader: Option<&WebGLShader>) {
        self.base.DetachShader(program, shader)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn BindAttribLocation(&self, program: Option<&WebGLProgram>,
                          index: u32, name: DOMString) {
        self.base.BindAttribLocation(program, index, name)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.5
    fn BindBuffer(&self, target: u32, buffer: Option<&WebGLBuffer>) {
        let slot = match target {
            constants::UNIFORM_BUFFER => &self.bound_uniform_buffer,
            constants::TRANSFORM_FEEDBACK_BUFFER => &self.bound_transform_feedback_buffer,
            _ => return self.base.BindBuffer(target, buffer),
        };

        if let Some(buffer) = buffer {
            match buffer.bind(target) {
                Ok(_) => WebGLBuffer::update_slot(slot, Some(buffer)),
                Err(e) => return self.webgl_error(e),
            }
        } else {
            WebGLBuffer::update_slot(slot, None);
            // Unbind the current buffer
            self.send_command(WebGLCommand::BindBuffer(target, None));
        }
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.6
    fn BindFramebuffer(&self, target: u32, framebuffer: Option<&WebGLFramebuffer>) {
        self.base.BindFramebuffer(target, framebuffer)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.7
    fn BindRenderbuffer(&self, target: u32, renderbuffer: Option<&WebGLRenderbuffer>) {
        self.base.BindRenderbuffer(target, renderbuffer)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn BindTexture(&self, target: u32, texture: Option<&WebGLTexture>) {
        self.base.BindTexture(target, texture)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn GenerateMipmap(&self, target: u32) {
        self.base.GenerateMipmap(target)
    }

    #[allow(unsafe_code)]
    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.5
    unsafe fn BufferData(&self, cx: *mut JSContext, target: u32, data: *mut JSObject, usage: u32) -> Fallible<()> {
        let bound_buffer = match target {
            constants::ARRAY_BUFFER => self.base.bound_buffer_array(),
            constants::ELEMENT_ARRAY_BUFFER => self.base.bound_buffer_element_array(),
            constants::UNIFORM_BUFFER => self.bound_uniform_buffer.get(),
            constants::TRANSFORM_FEEDBACK_BUFFER => self.bound_transform_feedback_buffer.get(),
            _ => return Ok(self.webgl_error(InvalidEnum)),
        };

        self.base.buffer_data(cx, target, bound_buffer, data, usage)
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.5
    fn BufferData_(&self, target: u32, size: i64, usage: u32) -> Fallible<()> {
        let bound_buffer = match target {
            constants::ARRAY_BUFFER => self.base.bound_buffer_array(),
            constants::ELEMENT_ARRAY_BUFFER => self.base.bound_buffer_element_array(),
            constants::UNIFORM_BUFFER => self.bound_uniform_buffer.get(),
            constants::TRANSFORM_FEEDBACK_BUFFER => self.bound_transform_feedback_buffer.get(),
            _ => return Ok(self.webgl_error(InvalidEnum)),
        };

        self.base.buffer_data_(target, bound_buffer, size, usage)
    }

    #[allow(unsafe_code)]
    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.5
    unsafe fn BufferSubData(&self, cx: *mut JSContext, target: u32, offset: i64, data: *mut JSObject) -> Fallible<()> {
        let bound_buffer = match target {
            constants::ARRAY_BUFFER => self.base.bound_buffer_array(),
            constants::ELEMENT_ARRAY_BUFFER => self.base.bound_buffer_element_array(),
            constants::UNIFORM_BUFFER => self.bound_uniform_buffer.get(),
            constants::TRANSFORM_FEEDBACK_BUFFER => self.bound_transform_feedback_buffer.get(),
            _ => return Ok(self.webgl_error(InvalidEnum)),
        };

        self.base.buffer_sub_data(cx, target, bound_buffer, offset, data)
    }

    #[allow(unsafe_code)]
    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    unsafe fn CompressedTexImage2D(&self, cx: *mut JSContext, target: u32, level: i32, internal_format: u32,
                                   width: i32, height: i32, border: i32, pixels: *mut JSObject) -> Fallible<()> {
        self.base.CompressedTexImage2D(cx, target, level, internal_format, width, height, border, pixels)
    }

    #[allow(unsafe_code)]
    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    unsafe fn CompressedTexSubImage2D(&self, cx: *mut JSContext, target: u32, level: i32,
                                      xoffset: i32, yoffset: i32, width: i32, height: i32,
                                      format: u32, pixels: *mut JSObject) -> Fallible<()> {
        self.base.CompressedTexSubImage2D(cx, target, level, xoffset, yoffset, width, height, format, pixels)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn CopyTexImage2D(&self, target: u32, level: i32, internal_format: u32,
                      x: i32, y: i32, width: i32, height: i32, border: i32) {
        self.base.CopyTexImage2D(target, level, internal_format, x, y, width, height, border)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn CopyTexSubImage2D(&self, target: u32, level: i32, xoffset: i32, yoffset: i32,
                         x: i32, y: i32, width: i32, height: i32) {
        self.base.CopyTexSubImage2D(target, level, xoffset, yoffset, x, y, width, height)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.11
    fn Clear(&self, mask: u32) {
        self.base.Clear(mask)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn ClearColor(&self, red: f32, green: f32, blue: f32, alpha: f32) {
        self.base.ClearColor(red, green, blue, alpha)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn ClearDepth(&self, depth: f32) {
        self.base.ClearDepth(depth)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn ClearStencil(&self, stencil: i32) {
        self.base.ClearStencil(stencil)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn ColorMask(&self, r: bool, g: bool, b: bool, a: bool) {
        self.base.ColorMask(r, g, b, a)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn CullFace(&self, mode: u32) {
        self.base.CullFace(mode)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn FrontFace(&self, mode: u32) {
        self.base.FrontFace(mode)
    }
    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn DepthFunc(&self, func: u32) {
        self.base.DepthFunc(func)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn DepthMask(&self, flag: bool) {
        self.base.DepthMask(flag)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn DepthRange(&self, near: f32, far: f32) {
        self.base.DepthRange(near, far)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn Enable(&self, cap: u32) {
        self.base.Enable(cap)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn Disable(&self, cap: u32) {
        self.base.Disable(cap)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn CompileShader(&self, shader: Option<&WebGLShader>) {
        self.base.CompileShader(shader)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.5
    fn CreateBuffer(&self) -> Option<DomRoot<WebGLBuffer>> {
        self.base.CreateBuffer()
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.6
    fn CreateFramebuffer(&self) -> Option<DomRoot<WebGLFramebuffer>> {
        self.base.CreateFramebuffer()
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.7
    fn CreateRenderbuffer(&self) -> Option<DomRoot<WebGLRenderbuffer>> {
        self.base.CreateRenderbuffer()
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn CreateTexture(&self) -> Option<DomRoot<WebGLTexture>> {
        self.base.CreateTexture()
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn CreateProgram(&self) -> Option<DomRoot<WebGLProgram>> {
        self.base.CreateProgram()
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn CreateShader(&self, shader_type: u32) -> Option<DomRoot<WebGLShader>> {
        self.base.CreateShader(shader_type)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.5
    fn DeleteBuffer(&self, buffer: Option<&WebGLBuffer>) {
        self.base.DeleteBuffer(buffer);
        if let Some(buffer) = buffer {
            handle_object_deletion!(self, self.bound_uniform_buffer, buffer,
                                    Some(WebGLCommand::BindBuffer(constants::UNIFORM_BUFFER, None)));
        }
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.6
    fn DeleteFramebuffer(&self, framebuffer: Option<&WebGLFramebuffer>) {
        self.base.DeleteFramebuffer(framebuffer)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.7
    fn DeleteRenderbuffer(&self, renderbuffer: Option<&WebGLRenderbuffer>) {
        self.base.DeleteRenderbuffer(renderbuffer)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn DeleteTexture(&self, texture: Option<&WebGLTexture>) {
        self.base.DeleteTexture(texture)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn DeleteProgram(&self, program: Option<&WebGLProgram>) {
        self.base.DeleteProgram(program)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn DeleteShader(&self, shader: Option<&WebGLShader>) {
        self.base.DeleteShader(shader)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.11
    fn DrawArrays(&self, mode: u32, first: i32, count: i32) {
        self.base.DrawArrays(mode, first, count)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.11
    fn DrawElements(&self, mode: u32, count: i32, type_: u32, offset: i64) {
        self.base.DrawElements(mode, count, type_, offset)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn EnableVertexAttribArray(&self, attrib_id: u32) {
        self.base.EnableVertexAttribArray(attrib_id)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn DisableVertexAttribArray(&self, attrib_id: u32) {
        self.base.DisableVertexAttribArray(attrib_id)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn GetActiveUniform(&self, program: Option<&WebGLProgram>, index: u32) -> Option<DomRoot<WebGLActiveInfo>> {
        self.base.GetActiveUniform(program, index)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn GetActiveAttrib(&self, program: Option<&WebGLProgram>, index: u32) -> Option<DomRoot<WebGLActiveInfo>> {
        self.base.GetActiveAttrib(program, index)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn GetAttribLocation(&self, program: Option<&WebGLProgram>, name: DOMString) -> i32 {
        self.base.GetAttribLocation(program, name)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn GetProgramInfoLog(&self, program: Option<&WebGLProgram>) -> Option<DOMString> {
        self.base.GetProgramInfoLog(program)
    }

    #[allow(unsafe_code)]
    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    unsafe fn GetProgramParameter(&self, cx: *mut JSContext, program: Option<&WebGLProgram>, param_id: u32) -> JSVal {
        self.base.GetProgramParameter(cx, program, param_id)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn GetShaderInfoLog(&self, shader: Option<&WebGLShader>) -> Option<DOMString> {
        self.base.GetShaderInfoLog(shader)
    }

    #[allow(unsafe_code)]
    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    unsafe fn GetShaderParameter(&self, cx: *mut JSContext, shader: Option<&WebGLShader>, param_id: u32) -> JSVal {
        self.base.GetShaderParameter(cx, shader, param_id)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn GetShaderPrecisionFormat(&self,
                                shader_type: u32,
                                precision_type: u32)
                                -> Option<DomRoot<WebGLShaderPrecisionFormat>> {
        self.base.GetShaderPrecisionFormat(shader_type, precision_type)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn GetUniformLocation(&self,
                          program: Option<&WebGLProgram>,
                          name: DOMString) -> Option<DomRoot<WebGLUniformLocation>> {
        self.base.GetUniformLocation(program, name)
    }

    #[allow(unsafe_code)]
    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    unsafe fn GetVertexAttrib(&self, cx: *mut JSContext, index: u32, pname: u32) -> JSVal {
        self.base.GetVertexAttrib(cx, index, pname)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn GetVertexAttribOffset(&self, index: u32, pname: u32) -> i64 {
        self.base.GetVertexAttribOffset(index, pname)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn Hint(&self, target: u32, mode: u32) {
        self.base.Hint(target, mode)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.5
    fn IsBuffer(&self, buffer: Option<&WebGLBuffer>) -> bool {
        self.base.IsBuffer(buffer)
    }

    // TODO: We could write this without IPC, recording the calls to `enable` and `disable`.
    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn IsEnabled(&self, cap: u32) -> bool {
        self.base.IsEnabled(cap)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.6
    fn IsFramebuffer(&self, frame_buffer: Option<&WebGLFramebuffer>) -> bool {
        self.base.IsFramebuffer(frame_buffer)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn IsProgram(&self, program: Option<&WebGLProgram>) -> bool {
        self.base.IsProgram(program)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.7
    fn IsRenderbuffer(&self, render_buffer: Option<&WebGLRenderbuffer>) -> bool {
        self.base.IsRenderbuffer(render_buffer)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn IsShader(&self, shader: Option<&WebGLShader>) -> bool {
        self.base.IsShader(shader)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn IsTexture(&self, texture: Option<&WebGLTexture>) -> bool {
        self.base.IsTexture(texture)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn LineWidth(&self, width: f32) {
        self.base.LineWidth(width)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn PixelStorei(&self, param_name: u32, param_value: i32) {
        self.base.PixelStorei(param_name, param_value)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn PolygonOffset(&self, factor: f32, units: f32) {
        self.base.PolygonOffset(factor, units)
    }

    #[allow(unsafe_code)]
    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.12
    unsafe fn ReadPixels(&self, cx: *mut JSContext, x: i32, y: i32, width: i32, height: i32,
                  format: u32, pixel_type: u32, pixels: *mut JSObject) -> Fallible<()> {
        self.base.ReadPixels(cx, x, y, width, height, format, pixel_type, pixels)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn SampleCoverage(&self, value: f32, invert: bool) {
        self.base.SampleCoverage(value, invert)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.4
    fn Scissor(&self, x: i32, y: i32, width: i32, height: i32) {
        self.base.Scissor(x, y, width, height)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn StencilFunc(&self, func: u32, ref_: i32, mask: u32) {
        self.base.StencilFunc(func, ref_, mask)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn StencilFuncSeparate(&self, face: u32, func: u32, ref_: i32, mask: u32) {
        self.base.StencilFuncSeparate(face, func, ref_, mask)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn StencilMask(&self, mask: u32) {
        self.base.StencilMask(mask)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn StencilMaskSeparate(&self, face: u32, mask: u32) {
        self.base.StencilMaskSeparate(face, mask)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn StencilOp(&self, fail: u32, zfail: u32, zpass: u32) {
        self.base.StencilOp(fail, zfail, zpass)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn StencilOpSeparate(&self, face: u32, fail: u32, zfail: u32, zpass: u32) {
        self.base.StencilOpSeparate(face, fail, zfail, zpass)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn LinkProgram(&self, program: Option<&WebGLProgram>) {
        self.base.LinkProgram(program)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn ShaderSource(&self, shader: Option<&WebGLShader>, source: DOMString) {
        self.base.ShaderSource(shader, source)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn GetShaderSource(&self, shader: Option<&WebGLShader>) -> Option<DOMString> {
        self.base.GetShaderSource(shader)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform1f(&self,
                  uniform: Option<&WebGLUniformLocation>,
                  val: f32) {
        self.base.Uniform1f(uniform, val)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform1i(&self,
                  uniform: Option<&WebGLUniformLocation>,
                  val: i32) {
        self.base.Uniform1i(uniform, val)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    #[allow(unsafe_code)]
    unsafe fn Uniform1iv(&self,
                  cx: *mut JSContext,
                  uniform: Option<&WebGLUniformLocation>,
                  data: *mut JSObject) -> Fallible<()> {
        self.base.Uniform1iv(cx, uniform, data)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    #[allow(unsafe_code)]
    unsafe fn Uniform1fv(&self,
                  cx: *mut JSContext,
                  uniform: Option<&WebGLUniformLocation>,
                  data: *mut JSObject) -> Fallible<()> {
        self.base.Uniform1fv(cx, uniform, data)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform2f(&self,
                  uniform: Option<&WebGLUniformLocation>,
                  x: f32, y: f32) {
        self.base.Uniform2f(uniform, x, y)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    #[allow(unsafe_code)]
    unsafe fn Uniform2fv(&self,
                  cx: *mut JSContext,
                  uniform: Option<&WebGLUniformLocation>,
                  data: *mut JSObject) -> Fallible<()> {
        self.base.Uniform2fv(cx, uniform, data)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform2i(&self,
                  uniform: Option<&WebGLUniformLocation>,
                  x: i32, y: i32) {
        self.base.Uniform2i(uniform, x, y)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    #[allow(unsafe_code)]
    unsafe fn Uniform2iv(&self,
                  cx: *mut JSContext,
                  uniform: Option<&WebGLUniformLocation>,
                  data: *mut JSObject) -> Fallible<()> {
        self.base.Uniform2iv(cx, uniform, data)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform3f(&self,
                  uniform: Option<&WebGLUniformLocation>,
                  x: f32, y: f32, z: f32) {
        self.base.Uniform3f(uniform, x, y, z)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    #[allow(unsafe_code)]
    unsafe fn Uniform3fv(&self,
                  cx: *mut JSContext,
                  uniform: Option<&WebGLUniformLocation>,
                  data: *mut JSObject) -> Fallible<()> {
        self.base.Uniform3fv(cx, uniform, data)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform3i(&self,
                  uniform: Option<&WebGLUniformLocation>,
                  x: i32, y: i32, z: i32) {
        self.base.Uniform3i(uniform, x, y, z)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    #[allow(unsafe_code)]
    unsafe fn Uniform3iv(&self,
                  cx: *mut JSContext,
                  uniform: Option<&WebGLUniformLocation>,
                  data: *mut JSObject) -> Fallible<()> {
        self.base.Uniform3iv(cx, uniform, data)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform4i(&self,
                  uniform: Option<&WebGLUniformLocation>,
                  x: i32, y: i32, z: i32, w: i32) {
        self.base.Uniform4i(uniform, x, y, z, w)
    }


    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    #[allow(unsafe_code)]
    unsafe fn Uniform4iv(&self,
                  cx: *mut JSContext,
                  uniform: Option<&WebGLUniformLocation>,
                  data: *mut JSObject) -> Fallible<()> {
        self.base.Uniform4iv(cx, uniform, data)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform4f(&self,
                  uniform: Option<&WebGLUniformLocation>,
                  x: f32, y: f32, z: f32, w: f32) {
        self.base.Uniform4f(uniform, x, y, z, w)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    #[allow(unsafe_code)]
    unsafe fn Uniform4fv(&self,
                  cx: *mut JSContext,
                  uniform: Option<&WebGLUniformLocation>,
                  data: *mut JSObject) -> Fallible<()> {
        self.base.Uniform4fv(cx, uniform, data)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    #[allow(unsafe_code)]
    unsafe fn UniformMatrix2fv(&self,
                        cx: *mut JSContext,
                        uniform: Option<&WebGLUniformLocation>,
                        transpose: bool,
                        data: *mut JSObject) -> Fallible<()> {
        self.base.UniformMatrix2fv(cx, uniform, transpose, data)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    #[allow(unsafe_code)]
    unsafe fn UniformMatrix3fv(&self,
                        cx: *mut JSContext,
                        uniform: Option<&WebGLUniformLocation>,
                        transpose: bool,
                        data: *mut JSObject) -> Fallible<()> {
        self.base.UniformMatrix3fv(cx, uniform, transpose, data)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    #[allow(unsafe_code)]
    unsafe fn UniformMatrix4fv(&self,
                        cx: *mut JSContext,
                        uniform: Option<&WebGLUniformLocation>,
                        transpose: bool,
                        data: *mut JSObject) -> Fallible<()> {
        self.base.UniformMatrix4fv(cx, uniform, transpose, data)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn UseProgram(&self, program: Option<&WebGLProgram>) {
        self.base.UseProgram(program)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn ValidateProgram(&self, program: Option<&WebGLProgram>) {
        self.base.ValidateProgram(program)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn VertexAttrib1f(&self, indx: u32, x: f32) {
        self.base.VertexAttrib1f(indx, x)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    #[allow(unsafe_code)]
    unsafe fn VertexAttrib1fv(&self, cx: *mut JSContext, indx: u32, data: *mut JSObject) -> Fallible<()> {
        self.base.VertexAttrib1fv(cx, indx, data)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn VertexAttrib2f(&self, indx: u32, x: f32, y: f32) {
        self.base.VertexAttrib2f(indx, x, y)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    #[allow(unsafe_code)]
    unsafe fn VertexAttrib2fv(&self, cx: *mut JSContext, indx: u32, data: *mut JSObject) -> Fallible<()> {
        self.base.VertexAttrib2fv(cx, indx, data)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn VertexAttrib3f(&self, indx: u32, x: f32, y: f32, z: f32) {
        self.base.VertexAttrib3f(indx, x, y, z)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    #[allow(unsafe_code)]
    unsafe fn VertexAttrib3fv(&self, cx: *mut JSContext, indx: u32, data: *mut JSObject) -> Fallible<()> {
        self.base.VertexAttrib3fv(cx, indx, data)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn VertexAttrib4f(&self, indx: u32, x: f32, y: f32, z: f32, w: f32) {
        self.base.VertexAttrib4f(indx, x, y, z, w)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    #[allow(unsafe_code)]
    unsafe fn VertexAttrib4fv(&self, cx: *mut JSContext, indx: u32, data: *mut JSObject) -> Fallible<()> {
        self.base.VertexAttrib4fv(cx, indx, data)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn VertexAttribPointer(&self, attrib_id: u32, size: i32, data_type: u32,
                           normalized: bool, stride: i32, offset: i64) {
        self.base.VertexAttribPointer(attrib_id, size, data_type, normalized, stride, offset)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.4
    fn Viewport(&self, x: i32, y: i32, width: i32, height: i32) {
        self.base.Viewport(x, y, width, height)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    #[allow(unsafe_code)]
    unsafe fn TexImage2D(&self,
                         cx: *mut JSContext,
                         target: u32,
                         level: i32,
                         internal_format: u32,
                         width: i32,
                         height: i32,
                         border: i32,
                         format: u32,
                         data_type: u32,
                         data_ptr: *mut JSObject) -> Fallible<()> {
        self.base.TexImage2D(cx, target, level, internal_format, width, height, border, format, data_type, data_ptr)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn TexImage2D_(&self,
                   target: u32,
                   level: i32,
                   internal_format: u32,
                   format: u32,
                   data_type: u32,
                   source: Option<ImageDataOrHTMLImageElementOrHTMLCanvasElementOrHTMLVideoElement>) -> Fallible<()> {
        self.base.TexImage2D_(target, level, internal_format, format, data_type, source)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn TexImageDOM(&self,
                   target: u32,
                   level: i32,
                   internal_format: u32,
                   width: i32,
                   height: i32,
                   format: u32,
                   data_type: u32,
                   source: &HTMLIFrameElement) -> Fallible<()> {
        self.base.TexImageDOM(target, level, internal_format, width, height, format, data_type, source)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    #[allow(unsafe_code)]
    unsafe fn TexSubImage2D(&self,
                            cx: *mut JSContext,
                            target: u32,
                            level: i32,
                            xoffset: i32,
                            yoffset: i32,
                            width: i32,
                            height: i32,
                            format: u32,
                            data_type: u32,
                            data_ptr: *mut JSObject) -> Fallible<()> {
        self.base.TexSubImage2D(cx, target, level, xoffset, yoffset, width, height, format, data_type, data_ptr)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn TexSubImage2D_(&self,
                      target: u32,
                      level: i32,
                      xoffset: i32,
                      yoffset: i32,
                      format: u32,
                      data_type: u32,
                      source: Option<ImageDataOrHTMLImageElementOrHTMLCanvasElementOrHTMLVideoElement>)
                      -> Fallible<()> {
        self.base.TexSubImage2D_(target, level, xoffset, yoffset, format, data_type, source)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn TexParameterf(&self, target: u32, name: u32, value: f32) {
        self.base.TexParameterf(target, name, value)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn TexParameteri(&self, target: u32, name: u32, value: i32) {
        self.base.TexParameteri(target, name, value)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.6
    fn CheckFramebufferStatus(&self, target: u32) -> u32 {
        self.base.CheckFramebufferStatus(target)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.7
    fn RenderbufferStorage(&self, target: u32, internal_format: u32,
                           width: i32, height: i32) {
        self.base.RenderbufferStorage(target, internal_format, width, height)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.6
    fn FramebufferRenderbuffer(&self, target: u32, attachment: u32,
                               renderbuffertarget: u32,
                               rb: Option<&WebGLRenderbuffer>) {
        self.base.FramebufferRenderbuffer(target, attachment, renderbuffertarget, rb)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.6
    fn FramebufferTexture2D(&self, target: u32, attachment: u32,
                            textarget: u32, texture: Option<&WebGLTexture>,
                            level: i32) {
        self.base.FramebufferTexture2D(target, attachment, textarget, texture, level)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.16
    fn BindBufferBase(&self, target: u32, index: u32, buffer: Option<&WebGLBuffer>) {
        let slot = match target {
            constants::UNIFORM_BUFFER => &self.bound_uniform_buffer,
            constants::TRANSFORM_FEEDBACK_BUFFER => &self.bound_transform_feedback_buffer,
            // Invalid parameters
            _ => return self.webgl_error(InvalidEnum)
        };

        if let Some(error) = buffer.and_then(|buffer| buffer.update_target(target).err()) {
            return self.webgl_error(error);
        }
        WebGLBuffer::update_slot(slot, buffer);

        self.send_command(WebGLCommand::BindBufferBase(target, index, buffer.map(|b| b.id())));
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.16
    fn BindBufferRange(&self, target: u32, index: u32, buffer: Option<&WebGLBuffer>, offset: i64, size: i64) {
        let slot = match target {
            constants::UNIFORM_BUFFER => &self.bound_uniform_buffer,
            constants::TRANSFORM_FEEDBACK_BUFFER => &self.bound_transform_feedback_buffer,
            // Invalid parameters
            _ => return self.webgl_error(InvalidEnum)
        };

        if let Some(error) = buffer.and_then(|buffer| buffer.update_target(target).err()) {
            return self.webgl_error(error);
        }
        WebGLBuffer::update_slot(slot, buffer);

        self.send_command(WebGLCommand::BindBufferRange(target, index, buffer.map(|b| b.id()), offset, size));
    }

    #[allow(unsafe_code)]
    /// https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.16
    unsafe fn GetIndexedParameter(&self, cx: *mut JSContext, target: u32, index: u32) -> JSVal {
        let value = match target {
            constants::TRANSFORM_FEEDBACK_BUFFER_BINDING =>
                return object_binding_to_js_or_null!(cx, &self.bound_transform_feedback_buffer),
            constants::UNIFORM_BUFFER_BINDING =>
                return object_binding_to_js_or_null!(cx, &self.bound_uniform_buffer),
            _ => {
                let (sender, receiver) = webgl_channel().unwrap();
                self.send_command(WebGLCommand::GetIndexedParameter(target, index, sender));
                receiver.recv().unwrap()
            }
        };


        match handle_potential_webgl_error!(self, value, WebGLParameter::Invalid) {
            WebGLParameter::Int(val) => Int32Value(val),
            WebGLParameter::Int64(val) => DoubleValue(val as f64),
            WebGLParameter::Bool(val) => BooleanValue(val),
            WebGLParameter::Float(val) => DoubleValue(val as f64),
            WebGLParameter::FloatArray(_) => panic!("Parameter should not be float array"),
            WebGLParameter::String(val) => {
                rooted!(in(cx) let mut rval = UndefinedValue());
                val.to_jsval(cx, rval.handle_mut());
                rval.get()
            }
            WebGLParameter::Invalid => NullValue(),
        }
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.16
    fn GetUniformIndices(&self, program: &WebGLProgram, mut uniform_names: Vec<DOMString>) -> Option<Vec<u32>> {
        if !program.is_linked() {
            self.webgl_error(InvalidOperation);
            return None;
        }

        let (sender, receiver) = webgl_channel().unwrap();
        let uniform_names = uniform_names.drain(..).map(|name| String::from(name)).collect();
        self.send_command(WebGLCommand::GetUniformIndices(program.id(), uniform_names, sender));
        Some(receiver.recv().unwrap())
    }

    #[allow(unsafe_code)]
    /// https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.16
    unsafe fn GetActiveUniforms(
        &self,
        cx: *mut JSContext,
        program: &WebGLProgram,
        uniform_indices: Vec<u32>,
        pname: u32
    ) -> JSVal {
        let (sender, receiver) = webgl_channel().unwrap();
        self.send_command(WebGLCommand::GetActiveUniforms(program.id(), uniform_indices, pname, sender));

        let value = match receiver.recv().unwrap() {
            Ok(value) => value,
            Err(error) => {
                self.webgl_error(error);
                return NullValue();
            }
        };

        match value {
            WebGLParameterVec::Int(val) => {
                rooted!(in(cx) let mut result = UndefinedValue());
                val.to_jsval(cx, result.handle_mut());
                result.get()
            },
            WebGLParameterVec::Uint(val) => {
                rooted!(in(cx) let mut result = UndefinedValue());
                val.to_jsval(cx, result.handle_mut());
                result.get()
            },
            WebGLParameterVec::Bool(val) => {
                rooted!(in(cx) let mut result = UndefinedValue());
                val.to_jsval(cx, result.handle_mut());
                result.get()
            },
            WebGLParameterVec::Invalid => NullValue()
        }
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.16
    fn GetUniformBlockIndex(&self, program: &WebGLProgram, block_name: DOMString) -> u32 {
        let (sender, receiver) = webgl_channel().unwrap();
        self.send_command(WebGLCommand::GetUniformBlockIndex(program.id(), String::from(block_name), sender));
        receiver.recv().unwrap()
    }

    #[allow(unsafe_code)]
    /// https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.16
    unsafe fn GetActiveUniformBlockParameter(
        &self,
        cx: *mut JSContext,
        program: &WebGLProgram,
        block_index: u32,
        pname: u32
    ) -> JSVal {
        let (sender, receiver) = webgl_channel().unwrap();
        self.send_command(WebGLCommand::GetActiveUniformBlockParameter(program.id(), block_index, pname, sender));
        let value = receiver.recv().unwrap();

        match handle_potential_webgl_error!(self, value, WebGLUniformBlockParameter::Invalid) {
            WebGLUniformBlockParameter::Uint(val) => Int32Value(val as i32),
            WebGLUniformBlockParameter::Bool(val) => BooleanValue(val),
            WebGLUniformBlockParameter::IntArray(val) => {
                rooted!(in(cx) let mut result = UndefinedValue());
                val.to_jsval(cx, result.handle_mut());
                result.get()
            },
            WebGLUniformBlockParameter::Invalid => NullValue(),
        }
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.16
    fn GetActiveUniformBlockName(&self, program: &WebGLProgram, block_index: u32) -> Option<DOMString> {
        let (sender, receiver) = webgl_channel().unwrap();
        self.send_command(WebGLCommand::GetActiveUniformBlockName(program.id(), block_index, sender));
        Some(DOMString::from(receiver.recv().unwrap()))
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.16
    fn UniformBlockBinding(&self, program: &WebGLProgram, block_index: u32, block_binding: u32) {
        self.send_command(WebGLCommand::UniformBlockBinding(program.id(), block_index, block_binding));
    }
}


impl LayoutCanvasWebGLRenderingContextHelpers for LayoutDom<WebGL2RenderingContext> {
    #[allow(unsafe_code)]
    unsafe fn canvas_data_source(&self) -> HTMLCanvasDataSource {
        let this = &*self.unsafe_get();
        HTMLCanvasDataSource::WebGL((*this.base.to_layout().unsafe_get()).layout_handle())
    }
}
