/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::WebGL2RenderingContextBinding;
use crate::dom::bindings::codegen::Bindings::WebGL2RenderingContextBinding::WebGL2RenderingContextConstants as constants;
use crate::dom::bindings::codegen::Bindings::WebGL2RenderingContextBinding::WebGL2RenderingContextMethods;
use crate::dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLContextAttributes;
use crate::dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLRenderingContextMethods;
use crate::dom::bindings::codegen::UnionTypes::ArrayBufferViewOrArrayBuffer;
use crate::dom::bindings::codegen::UnionTypes::Float32ArrayOrUnrestrictedFloatSequence;
use crate::dom::bindings::codegen::UnionTypes::ImageDataOrHTMLImageElementOrHTMLCanvasElementOrHTMLVideoElement;
use crate::dom::bindings::codegen::UnionTypes::Int32ArrayOrLongSequence;
use crate::dom::bindings::conversions::ToJSValConvertible;
use crate::dom::bindings::error::{ErrorResult, Fallible};
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot, LayoutDom, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::htmlcanvaselement::HTMLCanvasElement;
use crate::dom::htmliframeelement::HTMLIFrameElement;
use crate::dom::webglactiveinfo::WebGLActiveInfo;
use crate::dom::webglbuffer::WebGLBuffer;
use crate::dom::webglframebuffer::WebGLFramebuffer;
use crate::dom::webglprogram::WebGLProgram;
use crate::dom::webglquery::WebGLQuery;
use crate::dom::webglrenderbuffer::WebGLRenderbuffer;
use crate::dom::webglrenderingcontext::{
    LayoutCanvasWebGLRenderingContextHelpers, WebGLRenderingContext,
};
use crate::dom::webglsampler::{WebGLSampler, WebGLSamplerValue};
use crate::dom::webglshader::WebGLShader;
use crate::dom::webglshaderprecisionformat::WebGLShaderPrecisionFormat;
use crate::dom::webglsync::WebGLSync;
use crate::dom::webgltexture::WebGLTexture;
use crate::dom::webgluniformlocation::WebGLUniformLocation;
use crate::dom::window::Window;
use crate::script_runtime::JSContext;
use canvas_traits::webgl::WebGLError::*;
/// https://www.khronos.org/registry/webgl/specs/latest/2.0/webgl.idl
use canvas_traits::webgl::{webgl_channel, GLContextAttributes, WebGLCommand, WebGLVersion};
use dom_struct::dom_struct;
use euclid::default::Size2D;
use js::jsapi::JSObject;
use js::jsval::{BooleanValue, DoubleValue, Int32Value, JSVal, NullValue, UInt32Value};
use js::rust::CustomAutoRooterGuard;
use js::typedarray::ArrayBufferView;
use script_layout_interface::HTMLCanvasDataSource;
use std::ptr::NonNull;

#[dom_struct]
pub struct WebGL2RenderingContext {
    reflector_: Reflector,
    base: Dom<WebGLRenderingContext>,
    occlusion_query: MutNullableDom<WebGLQuery>,
    primitives_query: MutNullableDom<WebGLQuery>,
    samplers: Box<[MutNullableDom<WebGLSampler>]>,
}

impl WebGL2RenderingContext {
    fn new_inherited(
        window: &Window,
        canvas: &HTMLCanvasElement,
        size: Size2D<u32>,
        attrs: GLContextAttributes,
    ) -> Option<WebGL2RenderingContext> {
        let base = WebGLRenderingContext::new(window, canvas, WebGLVersion::WebGL2, size, attrs)?;

        let samplers = (0..base.limits().max_combined_texture_image_units)
            .map(|_| Default::default())
            .collect::<Vec<_>>()
            .into();

        Some(WebGL2RenderingContext {
            reflector_: Reflector::new(),
            base: Dom::from_ref(&*base),
            occlusion_query: MutNullableDom::new(None),
            primitives_query: MutNullableDom::new(None),
            samplers: samplers,
        })
    }

    #[allow(unrooted_must_root)]
    pub fn new(
        window: &Window,
        canvas: &HTMLCanvasElement,
        size: Size2D<u32>,
        attrs: GLContextAttributes,
    ) -> Option<DomRoot<WebGL2RenderingContext>> {
        WebGL2RenderingContext::new_inherited(window, canvas, size, attrs).map(|ctx| {
            reflect_dom_object(Box::new(ctx), window, WebGL2RenderingContextBinding::Wrap)
        })
    }
}

impl WebGL2RenderingContext {
    pub fn recreate(&self, size: Size2D<u32>) {
        self.base.recreate(size)
    }

    pub fn base_context(&self) -> DomRoot<WebGLRenderingContext> {
        DomRoot::from_ref(&*self.base)
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

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.5
    fn GetBufferParameter(&self, cx: JSContext, target: u32, parameter: u32) -> JSVal {
        self.base.GetBufferParameter(cx, target, parameter)
    }

    #[allow(unsafe_code)]
    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn GetParameter(&self, cx: JSContext, parameter: u32) -> JSVal {
        match parameter {
            constants::MAX_CLIENT_WAIT_TIMEOUT_WEBGL => {
                Int32Value(self.base.limits().max_client_wait_timeout_webgl.as_nanos() as i32)
            },
            constants::SAMPLER_BINDING => unsafe {
                let idx = (self.base.textures().active_unit_enum() - constants::TEXTURE0) as usize;
                assert!(idx < self.samplers.len());
                let sampler = self.samplers[idx].get();
                optional_root_object_to_js_or_null!(*cx, sampler)
            },
            _ => self.base.GetParameter(cx, parameter),
        }
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn GetTexParameter(&self, cx: JSContext, target: u32, pname: u32) -> JSVal {
        self.base.GetTexParameter(cx, target, pname)
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

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.14
    fn GetExtension(&self, cx: JSContext, name: DOMString) -> Option<NonNull<JSObject>> {
        self.base.GetExtension(cx, name)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.4
    fn GetFramebufferAttachmentParameter(
        &self,
        cx: JSContext,
        target: u32,
        attachment: u32,
        pname: u32,
    ) -> JSVal {
        self.base
            .GetFramebufferAttachmentParameter(cx, target, attachment, pname)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.7
    fn GetRenderbufferParameter(&self, cx: JSContext, target: u32, pname: u32) -> JSVal {
        self.base.GetRenderbufferParameter(cx, target, pname)
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
        self.base
            .BlendFuncSeparate(src_rgb, dest_rgb, src_alpha, dest_alpha)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn AttachShader(&self, program: &WebGLProgram, shader: &WebGLShader) {
        self.base.AttachShader(program, shader)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn DetachShader(&self, program: &WebGLProgram, shader: &WebGLShader) {
        self.base.DetachShader(program, shader)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn BindAttribLocation(&self, program: &WebGLProgram, index: u32, name: DOMString) {
        self.base.BindAttribLocation(program, index, name)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.5
    fn BindBuffer(&self, target: u32, buffer: Option<&WebGLBuffer>) {
        self.base.BindBuffer(target, buffer)
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

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.5
    fn BufferData(&self, target: u32, data: Option<ArrayBufferViewOrArrayBuffer>, usage: u32) {
        self.base.BufferData(target, data, usage)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.5
    fn BufferData_(&self, target: u32, size: i64, usage: u32) {
        self.base.BufferData_(target, size, usage)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.5
    fn BufferSubData(&self, target: u32, offset: i64, data: ArrayBufferViewOrArrayBuffer) {
        self.base.BufferSubData(target, offset, data)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn CompressedTexImage2D(
        &self,
        target: u32,
        level: i32,
        internal_format: u32,
        width: i32,
        height: i32,
        border: i32,
        pixels: CustomAutoRooterGuard<ArrayBufferView>,
    ) {
        self.base.CompressedTexImage2D(
            target,
            level,
            internal_format,
            width,
            height,
            border,
            pixels,
        )
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn CompressedTexSubImage2D(
        &self,
        target: u32,
        level: i32,
        xoffset: i32,
        yoffset: i32,
        width: i32,
        height: i32,
        format: u32,
        pixels: CustomAutoRooterGuard<ArrayBufferView>,
    ) {
        self.base.CompressedTexSubImage2D(
            target, level, xoffset, yoffset, width, height, format, pixels,
        )
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn CopyTexImage2D(
        &self,
        target: u32,
        level: i32,
        internal_format: u32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        border: i32,
    ) {
        self.base
            .CopyTexImage2D(target, level, internal_format, x, y, width, height, border)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn CopyTexSubImage2D(
        &self,
        target: u32,
        level: i32,
        xoffset: i32,
        yoffset: i32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) {
        self.base
            .CopyTexSubImage2D(target, level, xoffset, yoffset, x, y, width, height)
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
    fn CompileShader(&self, shader: &WebGLShader) {
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
        self.base.DeleteBuffer(buffer)
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
    fn GetActiveUniform(
        &self,
        program: &WebGLProgram,
        index: u32,
    ) -> Option<DomRoot<WebGLActiveInfo>> {
        self.base.GetActiveUniform(program, index)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn GetActiveAttrib(
        &self,
        program: &WebGLProgram,
        index: u32,
    ) -> Option<DomRoot<WebGLActiveInfo>> {
        self.base.GetActiveAttrib(program, index)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn GetAttribLocation(&self, program: &WebGLProgram, name: DOMString) -> i32 {
        self.base.GetAttribLocation(program, name)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn GetProgramInfoLog(&self, program: &WebGLProgram) -> Option<DOMString> {
        self.base.GetProgramInfoLog(program)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn GetProgramParameter(&self, cx: JSContext, program: &WebGLProgram, param_id: u32) -> JSVal {
        self.base.GetProgramParameter(cx, program, param_id)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn GetShaderInfoLog(&self, shader: &WebGLShader) -> Option<DOMString> {
        self.base.GetShaderInfoLog(shader)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn GetShaderParameter(&self, cx: JSContext, shader: &WebGLShader, param_id: u32) -> JSVal {
        self.base.GetShaderParameter(cx, shader, param_id)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn GetShaderPrecisionFormat(
        &self,
        shader_type: u32,
        precision_type: u32,
    ) -> Option<DomRoot<WebGLShaderPrecisionFormat>> {
        self.base
            .GetShaderPrecisionFormat(shader_type, precision_type)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn GetUniformLocation(
        &self,
        program: &WebGLProgram,
        name: DOMString,
    ) -> Option<DomRoot<WebGLUniformLocation>> {
        self.base.GetUniformLocation(program, name)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn GetVertexAttrib(&self, cx: JSContext, index: u32, pname: u32) -> JSVal {
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

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.12
    fn ReadPixels(
        &self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        format: u32,
        pixel_type: u32,
        pixels: CustomAutoRooterGuard<Option<ArrayBufferView>>,
    ) {
        self.base
            .ReadPixels(x, y, width, height, format, pixel_type, pixels)
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
    fn LinkProgram(&self, program: &WebGLProgram) {
        self.base.LinkProgram(program)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn ShaderSource(&self, shader: &WebGLShader, source: DOMString) {
        self.base.ShaderSource(shader, source)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn GetShaderSource(&self, shader: &WebGLShader) -> Option<DOMString> {
        self.base.GetShaderSource(shader)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform1f(&self, location: Option<&WebGLUniformLocation>, val: f32) {
        self.base.Uniform1f(location, val)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform1i(&self, location: Option<&WebGLUniformLocation>, val: i32) {
        self.base.Uniform1i(location, val)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform1iv(&self, location: Option<&WebGLUniformLocation>, v: Int32ArrayOrLongSequence) {
        self.base.Uniform1iv(location, v)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform1fv(
        &self,
        location: Option<&WebGLUniformLocation>,
        v: Float32ArrayOrUnrestrictedFloatSequence,
    ) {
        self.base.Uniform1fv(location, v);
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform2f(&self, location: Option<&WebGLUniformLocation>, x: f32, y: f32) {
        self.base.Uniform2f(location, x, y)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform2fv(
        &self,
        location: Option<&WebGLUniformLocation>,
        v: Float32ArrayOrUnrestrictedFloatSequence,
    ) {
        self.base.Uniform2fv(location, v);
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform2i(&self, location: Option<&WebGLUniformLocation>, x: i32, y: i32) {
        self.base.Uniform2i(location, x, y)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform2iv(&self, location: Option<&WebGLUniformLocation>, v: Int32ArrayOrLongSequence) {
        self.base.Uniform2iv(location, v)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform3f(&self, location: Option<&WebGLUniformLocation>, x: f32, y: f32, z: f32) {
        self.base.Uniform3f(location, x, y, z)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform3fv(
        &self,
        location: Option<&WebGLUniformLocation>,
        v: Float32ArrayOrUnrestrictedFloatSequence,
    ) {
        self.base.Uniform3fv(location, v);
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform3i(&self, location: Option<&WebGLUniformLocation>, x: i32, y: i32, z: i32) {
        self.base.Uniform3i(location, x, y, z)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform3iv(&self, location: Option<&WebGLUniformLocation>, v: Int32ArrayOrLongSequence) {
        self.base.Uniform3iv(location, v)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform4i(&self, location: Option<&WebGLUniformLocation>, x: i32, y: i32, z: i32, w: i32) {
        self.base.Uniform4i(location, x, y, z, w)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform4iv(&self, location: Option<&WebGLUniformLocation>, v: Int32ArrayOrLongSequence) {
        self.base.Uniform4iv(location, v)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform4f(&self, location: Option<&WebGLUniformLocation>, x: f32, y: f32, z: f32, w: f32) {
        self.base.Uniform4f(location, x, y, z, w)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform4fv(
        &self,
        location: Option<&WebGLUniformLocation>,
        v: Float32ArrayOrUnrestrictedFloatSequence,
    ) {
        self.base.Uniform4fv(location, v);
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn UniformMatrix2fv(
        &self,
        location: Option<&WebGLUniformLocation>,
        transpose: bool,
        v: Float32ArrayOrUnrestrictedFloatSequence,
    ) {
        self.base.UniformMatrix2fv(location, transpose, v)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn UniformMatrix3fv(
        &self,
        location: Option<&WebGLUniformLocation>,
        transpose: bool,
        v: Float32ArrayOrUnrestrictedFloatSequence,
    ) {
        self.base.UniformMatrix3fv(location, transpose, v)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn UniformMatrix4fv(
        &self,
        location: Option<&WebGLUniformLocation>,
        transpose: bool,
        v: Float32ArrayOrUnrestrictedFloatSequence,
    ) {
        self.base.UniformMatrix4fv(location, transpose, v)
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn GetUniform(
        &self,
        cx: JSContext,
        program: &WebGLProgram,
        location: &WebGLUniformLocation,
    ) -> JSVal {
        self.base.GetUniform(cx, program, location)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn UseProgram(&self, program: Option<&WebGLProgram>) {
        self.base.UseProgram(program)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn ValidateProgram(&self, program: &WebGLProgram) {
        self.base.ValidateProgram(program)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn VertexAttrib1f(&self, indx: u32, x: f32) {
        self.base.VertexAttrib1f(indx, x)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn VertexAttrib1fv(&self, indx: u32, v: Float32ArrayOrUnrestrictedFloatSequence) {
        self.base.VertexAttrib1fv(indx, v)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn VertexAttrib2f(&self, indx: u32, x: f32, y: f32) {
        self.base.VertexAttrib2f(indx, x, y)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn VertexAttrib2fv(&self, indx: u32, v: Float32ArrayOrUnrestrictedFloatSequence) {
        self.base.VertexAttrib2fv(indx, v)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn VertexAttrib3f(&self, indx: u32, x: f32, y: f32, z: f32) {
        self.base.VertexAttrib3f(indx, x, y, z)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn VertexAttrib3fv(&self, indx: u32, v: Float32ArrayOrUnrestrictedFloatSequence) {
        self.base.VertexAttrib3fv(indx, v)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn VertexAttrib4f(&self, indx: u32, x: f32, y: f32, z: f32, w: f32) {
        self.base.VertexAttrib4f(indx, x, y, z, w)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn VertexAttrib4fv(&self, indx: u32, v: Float32ArrayOrUnrestrictedFloatSequence) {
        self.base.VertexAttrib4fv(indx, v)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn VertexAttribPointer(
        &self,
        attrib_id: u32,
        size: i32,
        data_type: u32,
        normalized: bool,
        stride: i32,
        offset: i64,
    ) {
        self.base
            .VertexAttribPointer(attrib_id, size, data_type, normalized, stride, offset)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.4
    fn Viewport(&self, x: i32, y: i32, width: i32, height: i32) {
        self.base.Viewport(x, y, width, height)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn TexImage2D(
        &self,
        target: u32,
        level: i32,
        internal_format: u32,
        width: i32,
        height: i32,
        border: i32,
        format: u32,
        data_type: u32,
        pixels: CustomAutoRooterGuard<Option<ArrayBufferView>>,
    ) -> Fallible<()> {
        self.base.TexImage2D(
            target,
            level,
            internal_format,
            width,
            height,
            border,
            format,
            data_type,
            pixels,
        )
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn TexImage2D_(
        &self,
        target: u32,
        level: i32,
        internal_format: u32,
        format: u32,
        data_type: u32,
        source: ImageDataOrHTMLImageElementOrHTMLCanvasElementOrHTMLVideoElement,
    ) -> ErrorResult {
        self.base
            .TexImage2D_(target, level, internal_format, format, data_type, source)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn TexImageDOM(
        &self,
        target: u32,
        level: i32,
        internal_format: u32,
        width: i32,
        height: i32,
        format: u32,
        data_type: u32,
        source: &HTMLIFrameElement,
    ) -> Fallible<()> {
        self.base.TexImageDOM(
            target,
            level,
            internal_format,
            width,
            height,
            format,
            data_type,
            source,
        )
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn TexSubImage2D(
        &self,
        target: u32,
        level: i32,
        xoffset: i32,
        yoffset: i32,
        width: i32,
        height: i32,
        format: u32,
        data_type: u32,
        pixels: CustomAutoRooterGuard<Option<ArrayBufferView>>,
    ) -> Fallible<()> {
        self.base.TexSubImage2D(
            target, level, xoffset, yoffset, width, height, format, data_type, pixels,
        )
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn TexSubImage2D_(
        &self,
        target: u32,
        level: i32,
        xoffset: i32,
        yoffset: i32,
        format: u32,
        data_type: u32,
        source: ImageDataOrHTMLImageElementOrHTMLCanvasElementOrHTMLVideoElement,
    ) -> ErrorResult {
        self.base
            .TexSubImage2D_(target, level, xoffset, yoffset, format, data_type, source)
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
    fn RenderbufferStorage(&self, target: u32, internal_format: u32, width: i32, height: i32) {
        self.base
            .RenderbufferStorage(target, internal_format, width, height)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.6
    fn FramebufferRenderbuffer(
        &self,
        target: u32,
        attachment: u32,
        renderbuffertarget: u32,
        rb: Option<&WebGLRenderbuffer>,
    ) {
        self.base
            .FramebufferRenderbuffer(target, attachment, renderbuffertarget, rb)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.6
    fn FramebufferTexture2D(
        &self,
        target: u32,
        attachment: u32,
        textarget: u32,
        texture: Option<&WebGLTexture>,
        level: i32,
    ) {
        self.base
            .FramebufferTexture2D(target, attachment, textarget, texture, level)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn GetAttachedShaders(&self, program: &WebGLProgram) -> Option<Vec<DomRoot<WebGLShader>>> {
        self.base.GetAttachedShaders(program)
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.9
    fn DrawArraysInstanced(&self, mode: u32, first: i32, count: i32, primcount: i32) {
        handle_potential_webgl_error!(
            self.base,
            self.base
                .draw_arrays_instanced(mode, first, count, primcount)
        )
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.9
    fn DrawElementsInstanced(
        &self,
        mode: u32,
        count: i32,
        type_: u32,
        offset: i64,
        primcount: i32,
    ) {
        handle_potential_webgl_error!(
            self.base,
            self.base
                .draw_elements_instanced(mode, count, type_, offset, primcount)
        )
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.9
    fn VertexAttribDivisor(&self, index: u32, divisor: u32) {
        self.base.vertex_attrib_divisor(index, divisor);
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.12
    fn CreateQuery(&self) -> Option<DomRoot<WebGLQuery>> {
        Some(WebGLQuery::new(&self.base))
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.12
    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn DeleteQuery(&self, query: Option<&WebGLQuery>) {
        if let Some(query) = query {
            handle_potential_webgl_error!(self.base, self.base.validate_ownership(query), return);

            if let Some(query_target) = query.target() {
                let slot = match query_target {
                    constants::ANY_SAMPLES_PASSED |
                    constants::ANY_SAMPLES_PASSED_CONSERVATIVE => {
                        &self.occlusion_query
                    },
                    constants::TRANSFORM_FEEDBACK_PRIMITIVES_WRITTEN => {
                        &self.primitives_query
                    },
                    _ => unreachable!(),
                };
                if let Some(stored_query) = slot.get() {
                    if stored_query.target() == query.target() {
                        slot.set(None);
                    }
                }
            }

            query.delete(false);
        }
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.12
    fn IsQuery(&self, query: Option<&WebGLQuery>) -> bool {
        match query {
            Some(query) => self.base.validate_ownership(query).is_ok() && query.is_valid(),
            None => false,
        }
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.13
    fn CreateSampler(&self) -> Option<DomRoot<WebGLSampler>> {
        Some(WebGLSampler::new(&self.base))
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.13
    fn DeleteSampler(&self, sampler: Option<&WebGLSampler>) {
        if let Some(sampler) = sampler {
            handle_potential_webgl_error!(self.base, self.base.validate_ownership(sampler), return);
            for slot in self.samplers.iter() {
                if slot.get().map_or(false, |s| sampler == &*s) {
                    slot.set(None);
                }
            }
            sampler.delete(false);
        }
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.13
    fn IsSampler(&self, sampler: Option<&WebGLSampler>) -> bool {
        match sampler {
            Some(sampler) => self.base.validate_ownership(sampler).is_ok() && sampler.is_valid(),
            None => false,
        }
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.12
    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn BeginQuery(&self, target: u32, query: &WebGLQuery) {
        handle_potential_webgl_error!(self.base, self.base.validate_ownership(query), return);

        let active_query = match target {
            constants::ANY_SAMPLES_PASSED |
            constants::ANY_SAMPLES_PASSED_CONSERVATIVE => {
                &self.occlusion_query
            },
            constants::TRANSFORM_FEEDBACK_PRIMITIVES_WRITTEN => {
                &self.primitives_query
            },
            _ => {
                self.base.webgl_error(InvalidEnum);
                return;
            },
        };
        if active_query.get().is_some() {
            self.base.webgl_error(InvalidOperation);
            return;
        }
        let result = query.begin(&self.base, target);
        match result {
            Ok(_) => active_query.set(Some(query)),
            Err(error) => self.base.webgl_error(error),
        }
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.12
    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn EndQuery(&self, target: u32) {
        let active_query = match target {
            constants::ANY_SAMPLES_PASSED |
            constants::ANY_SAMPLES_PASSED_CONSERVATIVE => {
                self.occlusion_query.take()
            },
            constants::TRANSFORM_FEEDBACK_PRIMITIVES_WRITTEN => {
                self.primitives_query.take()
            },
            _ => {
                self.base.webgl_error(InvalidEnum);
                return;
            },
        };
        match active_query {
            None => self.base.webgl_error(InvalidOperation),
            Some(query) => {
                let result = query.end(&self.base, target);
                if let Err(error) = result {
                    self.base.webgl_error(error);
                }
            },
        }
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.12
    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn GetQuery(&self, target: u32, pname: u32) -> Option<DomRoot<WebGLQuery>> {
        if pname != constants::CURRENT_QUERY {
            self.base.webgl_error(InvalidEnum);
            return None;
        }
        let active_query = match target {
            constants::ANY_SAMPLES_PASSED |
            constants::ANY_SAMPLES_PASSED_CONSERVATIVE => {
                self.occlusion_query.get()
            },
            constants::TRANSFORM_FEEDBACK_PRIMITIVES_WRITTEN => {
                self.primitives_query.get()
            },
            _ => {
                self.base.webgl_error(InvalidEnum);
                None
            },
        };
        if let Some(query) = active_query.as_ref() {
            if query.target() != Some(target) {
                return None;
            }
        }
        active_query
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.12
    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn GetQueryParameter(&self, _cx: JSContext, query: &WebGLQuery, pname: u32) -> JSVal {
        handle_potential_webgl_error!(
            self.base,
            self.base.validate_ownership(query),
            return NullValue()
        );
        match query.get_parameter(&self.base, pname) {
            Ok(value) => match pname {
                constants::QUERY_RESULT => UInt32Value(value),
                constants::QUERY_RESULT_AVAILABLE => BooleanValue(value != 0),
                _ => unreachable!(),
            },
            Err(error) => {
                self.base.webgl_error(error);
                NullValue()
            },
        }
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.14
    fn FenceSync(&self, condition: u32, flags: u32) -> Option<DomRoot<WebGLSync>> {
        if flags != 0 {
            self.base.webgl_error(InvalidValue);
            return None;
        }
        if condition != constants::SYNC_GPU_COMMANDS_COMPLETE {
            self.base.webgl_error(InvalidEnum);
            return None;
        }

        Some(WebGLSync::new(&self.base))
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.14
    fn IsSync(&self, sync: Option<&WebGLSync>) -> bool {
        match sync {
            Some(sync) => {
                if !sync.is_valid() {
                    return false;
                }
                handle_potential_webgl_error!(
                    self.base,
                    self.base.validate_ownership(sync),
                    return false
                );
                let (sender, receiver) = webgl_channel().unwrap();
                self.base
                    .send_command(WebGLCommand::IsSync(sync.id(), sender));
                receiver.recv().unwrap()
            },
            None => false,
        }
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.14
    fn ClientWaitSync(&self, sync: &WebGLSync, flags: u32, timeout: u64) -> u32 {
        if !sync.is_valid() {
            self.base.webgl_error(InvalidOperation);
            return constants::WAIT_FAILED;
        }
        handle_potential_webgl_error!(
            self.base,
            self.base.validate_ownership(sync),
            return constants::WAIT_FAILED
        );
        if flags != 0 && flags != constants::SYNC_FLUSH_COMMANDS_BIT {
            self.base.webgl_error(InvalidValue);
            return constants::WAIT_FAILED;
        }
        if timeout > self.base.limits().max_client_wait_timeout_webgl.as_nanos() as u64 {
            self.base.webgl_error(InvalidOperation);
            return constants::WAIT_FAILED;
        }

        match sync.client_wait_sync(&self.base, flags, timeout) {
            Some(status) => status,
            None => constants::WAIT_FAILED,
        }
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.14
    fn WaitSync(&self, sync: &WebGLSync, flags: u32, timeout: i64) {
        if !sync.is_valid() {
            self.base.webgl_error(InvalidOperation);
            return;
        }
        handle_potential_webgl_error!(self.base, self.base.validate_ownership(sync), return);
        if flags != 0 {
            self.base.webgl_error(InvalidValue);
            return;
        }
        if timeout != constants::TIMEOUT_IGNORED {
            self.base.webgl_error(InvalidValue);
            return;
        }

        self.base
            .send_command(WebGLCommand::WaitSync(sync.id(), flags, timeout));
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.14
    fn GetSyncParameter(&self, _cx: JSContext, sync: &WebGLSync, pname: u32) -> JSVal {
        if !sync.is_valid() {
            self.base.webgl_error(InvalidOperation);
            return NullValue();
        }
        handle_potential_webgl_error!(
            self.base,
            self.base.validate_ownership(sync),
            return NullValue()
        );
        match pname {
            constants::OBJECT_TYPE | constants::SYNC_CONDITION | constants::SYNC_FLAGS => {
                let (sender, receiver) = webgl_channel().unwrap();
                self.base
                    .send_command(WebGLCommand::GetSyncParameter(sync.id(), pname, sender));
                UInt32Value(receiver.recv().unwrap())
            },
            constants::SYNC_STATUS => match sync.get_sync_status(pname, &self.base) {
                Some(status) => UInt32Value(status),
                None => UInt32Value(constants::UNSIGNALED),
            },
            _ => {
                self.base.webgl_error(InvalidEnum);
                NullValue()
            },
        }
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.14
    fn DeleteSync(&self, sync: Option<&WebGLSync>) {
        if let Some(sync) = sync {
            handle_potential_webgl_error!(self.base, self.base.validate_ownership(sync), return);
            sync.delete(false);
        }
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.13
    fn BindSampler(&self, unit: u32, sampler: Option<&WebGLSampler>) {
        if let Some(sampler) = sampler {
            handle_potential_webgl_error!(self.base, self.base.validate_ownership(sampler), return);

            if unit as usize >= self.samplers.len() {
                self.base.webgl_error(InvalidValue);
                return;
            }

            let result = sampler.bind(&self.base, unit);
            match result {
                Ok(_) => self.samplers[unit as usize].set(Some(sampler)),
                Err(error) => self.base.webgl_error(error),
            }
        }
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.13
    fn SamplerParameteri(&self, sampler: &WebGLSampler, pname: u32, param: i32) {
        handle_potential_webgl_error!(self.base, self.base.validate_ownership(sampler), return);
        let param = WebGLSamplerValue::GLenum(param as u32);
        let result = sampler.set_parameter(&self.base, pname, param);
        if let Err(error) = result {
            self.base.webgl_error(error);
        }
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.13
    fn SamplerParameterf(&self, sampler: &WebGLSampler, pname: u32, param: f32) {
        handle_potential_webgl_error!(self.base, self.base.validate_ownership(sampler), return);
        let param = WebGLSamplerValue::Float(param);
        let result = sampler.set_parameter(&self.base, pname, param);
        if let Err(error) = result {
            self.base.webgl_error(error);
        }
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.13
    fn GetSamplerParameter(&self, _cx: JSContext, sampler: &WebGLSampler, pname: u32) -> JSVal {
        handle_potential_webgl_error!(
            self.base,
            self.base.validate_ownership(sampler),
            return NullValue()
        );
        match sampler.get_parameter(&self.base, pname) {
            Ok(value) => match value {
                WebGLSamplerValue::GLenum(value) => UInt32Value(value),
                WebGLSamplerValue::Float(value) => DoubleValue(value as f64),
            },
            Err(error) => {
                self.base.webgl_error(error);
                NullValue()
            },
        }
    }
}

impl LayoutCanvasWebGLRenderingContextHelpers for LayoutDom<WebGL2RenderingContext> {
    #[allow(unsafe_code)]
    unsafe fn canvas_data_source(&self) -> HTMLCanvasDataSource {
        let this = &*self.unsafe_get();
        (*this.base.to_layout().unsafe_get()).layout_handle()
    }
}
