/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use canvas_traits::
            {CanvasMsg, CanvasWebGLMsg, CanvasCommonMsg, WebGLError,
             WebGLShaderParameter, WebGLFramebufferBindingRequest};
use canvas_traits::WebGLError::*;
use dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::
            {self, WebGLContextAttributes, WebGLRenderingContextMethods};
use dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLRenderingContextConstants as constants;
use dom::bindings::codegen::InheritTypes::NodeCast;
use dom::bindings::codegen::UnionTypes::ImageDataOrHTMLImageElementOrHTMLCanvasElementOrHTMLVideoElement;

use dom::bindings::conversions::ToJSValConvertible;
use dom::bindings::global::{GlobalRef, GlobalField};
use dom::bindings::js::{JS, LayoutJS, Root};
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::htmlcanvaselement::HTMLCanvasElement;
use dom::htmlcanvaselement::utils as canvas_utils;
use dom::htmlimageelement::HTMLImageElementHelpers;
use dom::imagedata::ImageDataHelpers;
use dom::node::{window_from_node, NodeHelpers, NodeDamage};
use dom::webglbuffer::{WebGLBuffer, WebGLBufferHelpers};
use dom::webglframebuffer::{WebGLFramebuffer, WebGLFramebufferHelpers};
use dom::webglprogram::{WebGLProgram, WebGLProgramHelpers};
use dom::webglrenderbuffer::{WebGLRenderbuffer, WebGLRenderbufferHelpers};
use dom::webglshader::{WebGLShader, WebGLShaderHelpers};
use dom::webgltexture::{TexParameterValue, WebGLTexture, WebGLTextureHelpers};
use dom::webgluniformlocation::{WebGLUniformLocation, WebGLUniformLocationHelpers};
use euclid::size::Size2D;
use ipc_channel::ipc::{self, IpcSender};
use js::jsapi::{JSContext, JSObject, RootedValue};
use js::jsapi::{JS_GetFloat32ArrayData, JS_GetObjectAsArrayBufferView};
use js::jsval::{JSVal, UndefinedValue, NullValue, Int32Value, BooleanValue};

use msg::constellation_msg::Msg as ConstellationMsg;
use net_traits::image::base::PixelFormat;
use net_traits::image_cache_task::ImageResponse;

use std::cell::Cell;
use std::mem;
use std::ptr;
use std::slice;
use std::sync::mpsc::channel;
use util::str::DOMString;
use util::vec::byte_swap;

use offscreen_gl_context::GLContextAttributes;

pub const MAX_UNIFORM_AND_ATTRIBUTE_LEN: usize = 256;

macro_rules! handle_potential_webgl_error {
    ($context:ident, $call:expr, $return_on_error:expr) => {
        match $call {
            Ok(ret) => ret,
            Err(error) => {
                $context.webgl_error(error);
                $return_on_error
            }
        }
    }
}

/// Set of bitflags for texture unpacking (texImage2d, etc...)
bitflags! {
    #[derive(HeapSizeOf, JSTraceable)]
    flags TextureUnpacking: u8 {
        const FLIP_Y_AXIS = 0x01,
        const PREMULTIPLY_ALPHA = 0x02,
        const CONVERT_COLORSPACE = 0x04,
    }
}

#[dom_struct]
#[derive(HeapSizeOf)]
pub struct WebGLRenderingContext {
    reflector_: Reflector,
    global: GlobalField,
    renderer_id: usize,
    #[ignore_heap_size_of = "Defined in ipc-channel"]
    ipc_renderer: IpcSender<CanvasMsg>,
    canvas: JS<HTMLCanvasElement>,
    last_error: Cell<Option<WebGLError>>,
    texture_unpacking_settings: Cell<TextureUnpacking>,
    bound_texture_2d: Cell<Option<JS<WebGLTexture>>>,
    bound_texture_cube_map: Cell<Option<JS<WebGLTexture>>>,
}

impl WebGLRenderingContext {
    fn new_inherited(global: GlobalRef,
                     canvas: &HTMLCanvasElement,
                     size: Size2D<i32>,
                     attrs: GLContextAttributes)
                     -> Result<WebGLRenderingContext, String> {
        let (sender, receiver) = ipc::channel().unwrap();
        let constellation_chan = global.constellation_chan();
        constellation_chan.0
                          .send(ConstellationMsg::CreateWebGLPaintTask(size, attrs, sender))
                          .unwrap();
        let result = receiver.recv().unwrap();

        result.map(|(ipc_renderer, renderer_id)| {
            WebGLRenderingContext {
                reflector_: Reflector::new(),
                global: GlobalField::from_rooted(&global),
                renderer_id: renderer_id,
                ipc_renderer: ipc_renderer,
                canvas: JS::from_ref(canvas),
                last_error: Cell::new(None),
                texture_unpacking_settings: Cell::new(CONVERT_COLORSPACE),
                bound_texture_2d: Cell::new(None),
                bound_texture_cube_map: Cell::new(None),
            }
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
        self.ipc_renderer.send(CanvasMsg::Common(CanvasCommonMsg::Recreate(size))).unwrap();
    }

    fn mark_as_dirty(&self) {
        let canvas = self.canvas.root();
        let node = NodeCast::from_ref(canvas.r());
        node.dirty(NodeDamage::OtherNodeDamage);
    }
}

impl Drop for WebGLRenderingContext {
    fn drop(&mut self) {
        self.ipc_renderer.send(CanvasMsg::Common(CanvasCommonMsg::Close)).unwrap();
    }
}

impl<'a> WebGLRenderingContextMethods for &'a WebGLRenderingContext {
    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.1
    fn Canvas(self) -> Root<HTMLCanvasElement> {
        self.canvas.root()
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.1
    fn DrawingBufferWidth(self) -> i32 {
        let (sender, receiver) = ipc::channel().unwrap();
        self.ipc_renderer
            .send(CanvasMsg::WebGL(CanvasWebGLMsg::DrawingBufferWidth(sender)))
            .unwrap();
        receiver.recv().unwrap()
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.1
    fn DrawingBufferHeight(self) -> i32 {
        let (sender, receiver) = ipc::channel().unwrap();
        self.ipc_renderer
            .send(CanvasMsg::WebGL(CanvasWebGLMsg::DrawingBufferHeight(sender)))
            .unwrap();
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
        let (sender, receiver) = ipc::channel().unwrap();

        // If the send does not succeed, assume context lost
        if let Err(_) = self.ipc_renderer
                            .send(CanvasMsg::WebGL(CanvasWebGLMsg::GetContextAttributes(sender))) {
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
        self.ipc_renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::ActiveTexture(texture))).unwrap();
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn BlendColor(self, r: f32, g: f32, b: f32, a: f32) {
        self.ipc_renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::BlendColor(r, g, b, a))).unwrap();
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn BlendEquation(self, mode: u32) {
        self.ipc_renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::BlendEquation(mode))).unwrap();
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn BlendEquationSeparate(self, mode_rgb: u32, mode_alpha: u32) {
        self.ipc_renderer
            .send(CanvasMsg::WebGL(CanvasWebGLMsg::BlendEquationSeparate(mode_rgb, mode_alpha)))
            .unwrap();
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn BlendFunc(self, src_factor: u32, dest_factor: u32) {
        self.ipc_renderer
            .send(CanvasMsg::WebGL(CanvasWebGLMsg::BlendFunc(src_factor, dest_factor)))
            .unwrap();
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn BlendFuncSeparate(self, src_rgb: u32, dest_rgb: u32, src_alpha: u32, dest_alpha: u32) {
        self.ipc_renderer.send(
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
        match target {
            constants::ARRAY_BUFFER |
            constants::ELEMENT_ARRAY_BUFFER => (),

            _ => return self.webgl_error(InvalidEnum),
        }

        if let Some(buffer) = buffer {
            handle_potential_webgl_error!(self, buffer.bind(target), ())
        } else {
            // Unbind the current buffer
            self.ipc_renderer
                .send(CanvasMsg::WebGL(CanvasWebGLMsg::BindBuffer(target, 0)))
                .unwrap()
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.6
    fn BindFramebuffer(self, target: u32, framebuffer: Option<&WebGLFramebuffer>) {
        if target != constants::FRAMEBUFFER {
            return self.webgl_error(InvalidOperation);
        }

        if let Some(framebuffer) = framebuffer {
            framebuffer.bind(target)
        } else {
            // Bind the default framebuffer
            let cmd = CanvasWebGLMsg::BindFramebuffer(target, WebGLFramebufferBindingRequest::Default);
            self.ipc_renderer.send(CanvasMsg::WebGL(cmd)).unwrap();
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.7
    fn BindRenderbuffer(self, target: u32, renderbuffer: Option<&WebGLRenderbuffer>) {
        if target != constants::RENDERBUFFER {
            return self.webgl_error(InvalidEnum);
        }

        if let Some(renderbuffer) = renderbuffer {
            renderbuffer.bind(target)
        } else {
            // Unbind the currently bound renderbuffer
            self.ipc_renderer
                .send(CanvasMsg::WebGL(CanvasWebGLMsg::BindRenderbuffer(target, 0)))
                .unwrap()
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn BindTexture(self, target: u32, texture: Option<&WebGLTexture>) {
        let slot = match target {
            constants::TEXTURE_2D => &self.bound_texture_2d,
            constants::TEXTURE_CUBE_MAP => &self.bound_texture_cube_map,

            _ => return self.webgl_error(InvalidEnum),
        };

        if let Some(texture) = texture {
            match texture.bind(target) {
                Ok(_) => slot.set(Some(JS::from_ref(texture))),
                Err(err) => return self.webgl_error(err),
            }
        } else {
            // Unbind the currently bound texture
            self.ipc_renderer
                .send(CanvasMsg::WebGL(CanvasWebGLMsg::BindTexture(target, 0)))
                .unwrap()
        }
    }

    #[allow(unsafe_code)]
    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.5
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
        self.ipc_renderer
            .send(CanvasMsg::WebGL(CanvasWebGLMsg::BufferData(target, data_vec, usage)))
            .unwrap()
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.11
    fn Clear(self, mask: u32) {
        self.ipc_renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::Clear(mask))).unwrap();
        self.mark_as_dirty();
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn ClearColor(self, red: f32, green: f32, blue: f32, alpha: f32) {
        self.ipc_renderer
            .send(CanvasMsg::WebGL(CanvasWebGLMsg::ClearColor(red, green, blue, alpha)))
            .unwrap()
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn ClearDepth(self, depth: f32) {
        self.ipc_renderer
            .send(CanvasMsg::WebGL(CanvasWebGLMsg::ClearDepth(depth as f64)))
            .unwrap()
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn ClearStencil(self, stencil: i32) {
        self.ipc_renderer
            .send(CanvasMsg::WebGL(CanvasWebGLMsg::ClearStencil(stencil)))
            .unwrap()
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn ColorMask(self, r: bool, g: bool, b: bool, a: bool) {
        self.ipc_renderer
            .send(CanvasMsg::WebGL(CanvasWebGLMsg::ColorMask(r, g, b, a)))
            .unwrap()
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn CullFace(self, mode: u32) {
        match mode {
            constants::FRONT | constants::BACK | constants::FRONT_AND_BACK =>
                self.ipc_renderer
                    .send(CanvasMsg::WebGL(CanvasWebGLMsg::CullFace(mode)))
                    .unwrap(),
            _ => self.webgl_error(InvalidEnum),
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn FrontFace(self, mode: u32) {
        match mode {
            constants::CW | constants::CCW =>
                self.ipc_renderer
                    .send(CanvasMsg::WebGL(CanvasWebGLMsg::FrontFace(mode)))
                    .unwrap(),
            _ => self.webgl_error(InvalidEnum),
        }
    }
    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn DepthFunc(self, func: u32) {
        match func {
            constants::NEVER | constants::LESS |
            constants::EQUAL | constants::LEQUAL |
            constants::GREATER | constants::NOTEQUAL |
            constants::GEQUAL | constants::ALWAYS =>
                self.ipc_renderer
                    .send(CanvasMsg::WebGL(CanvasWebGLMsg::DepthFunc(func)))
                    .unwrap(),
            _ => self.webgl_error(InvalidEnum),
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn DepthMask(self, flag: bool) {
        self.ipc_renderer
            .send(CanvasMsg::WebGL(CanvasWebGLMsg::DepthMask(flag)))
            .unwrap()
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn DepthRange(self, near: f32, far: f32) {
        self.ipc_renderer
            .send(CanvasMsg::WebGL(CanvasWebGLMsg::DepthRange(near as f64, far as f64)))
            .unwrap()
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn Enable(self, cap: u32) {
        match cap {
            constants::BLEND | constants::CULL_FACE | constants::DEPTH_TEST | constants::DITHER |
            constants::POLYGON_OFFSET_FILL | constants::SAMPLE_ALPHA_TO_COVERAGE | constants::SAMPLE_COVERAGE |
            constants::SAMPLE_COVERAGE_INVERT | constants::SCISSOR_TEST =>
                self.ipc_renderer
                    .send(CanvasMsg::WebGL(CanvasWebGLMsg::Enable(cap)))
                    .unwrap(),
            _ => self.webgl_error(InvalidEnum),
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn Disable(self, cap: u32) {
        match cap {
            constants::BLEND | constants::CULL_FACE | constants::DEPTH_TEST | constants::DITHER |
            constants::POLYGON_OFFSET_FILL | constants::SAMPLE_ALPHA_TO_COVERAGE | constants::SAMPLE_COVERAGE |
            constants::SAMPLE_COVERAGE_INVERT | constants::SCISSOR_TEST =>
                self.ipc_renderer
                    .send(CanvasMsg::WebGL(CanvasWebGLMsg::Disable(cap)))
                    .unwrap(),
            _ => self.webgl_error(InvalidEnum),
        }
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
        WebGLBuffer::maybe_new(self.global.root().r(), self.ipc_renderer.clone())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.6
    fn CreateFramebuffer(self) -> Option<Root<WebGLFramebuffer>> {
        WebGLFramebuffer::maybe_new(self.global.root().r(), self.ipc_renderer.clone())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.7
    fn CreateRenderbuffer(self) -> Option<Root<WebGLRenderbuffer>> {
        WebGLRenderbuffer::maybe_new(self.global.root().r(), self.ipc_renderer.clone())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn CreateTexture(self) -> Option<Root<WebGLTexture>> {
        WebGLTexture::maybe_new(self.global.root().r(), self.ipc_renderer.clone())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn CreateProgram(self) -> Option<Root<WebGLProgram>> {
        WebGLProgram::maybe_new(self.global.root().r(), self.ipc_renderer.clone())
    }

    // TODO(ecoal95): Check if constants are cross-platform or if we must make a translation
    // between WebGL constants and native ones.
    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn CreateShader(self, shader_type: u32) -> Option<Root<WebGLShader>> {
        WebGLShader::maybe_new(self.global.root().r(), self.ipc_renderer.clone(), shader_type)
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
        match mode {
            constants::POINTS | constants::LINE_STRIP |
            constants::LINE_LOOP | constants::LINES |
            constants::TRIANGLE_STRIP | constants::TRIANGLE_FAN |
            constants::TRIANGLES => {
                // TODO(ecoal95): Check the CURRENT_PROGRAM when we keep track of it, and if it's
                // null generate an InvalidOperation error
                if first < 0 || count < 0 {
                    self.webgl_error(InvalidValue);
                } else {
                    self.ipc_renderer
                        .send(CanvasMsg::WebGL(CanvasWebGLMsg::DrawArrays(mode, first, count)))
                        .unwrap();
                    self.mark_as_dirty();
                }
            },
            _ => self.webgl_error(InvalidEnum),
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn EnableVertexAttribArray(self, attrib_id: u32) {
        self.ipc_renderer
            .send(CanvasMsg::WebGL(CanvasWebGLMsg::EnableVertexAttribArray(attrib_id)))
            .unwrap()
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

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn Hint(self, target: u32, mode: u32) {
        if target != constants::GENERATE_MIPMAP_HINT {
            return self.webgl_error(InvalidEnum);
        }

        match mode {
            constants::FASTEST |
            constants::NICEST |
            constants::DONT_CARE => (),

            _ => return self.webgl_error(InvalidEnum),
        }

        self.ipc_renderer
            .send(CanvasMsg::WebGL(CanvasWebGLMsg::Hint(target, mode)))
            .unwrap()
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn LineWidth(self, width: f32) {
        if width.is_nan() || width <= 0f32 {
            return self.webgl_error(InvalidValue);
        }

        self.ipc_renderer
            .send(CanvasMsg::WebGL(CanvasWebGLMsg::LineWidth(width)))
            .unwrap()
    }

    // NOTE: Usage of this function could affect rendering while we keep using
    //   readback to render to the page.
    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn PixelStorei(self, param_name: u32, param_value: i32) {
        let mut texture_settings = self.texture_unpacking_settings.get();
        match param_name {
            constants::UNPACK_FLIP_Y_WEBGL => {
               if param_value != 0 {
                    texture_settings.insert(FLIP_Y_AXIS)
                } else {
                    texture_settings.remove(FLIP_Y_AXIS)
                }

                self.texture_unpacking_settings.set(texture_settings);
                return;
            },
            constants::UNPACK_PREMULTIPLY_ALPHA_WEBGL => {
                if param_value != 0 {
                    texture_settings.insert(PREMULTIPLY_ALPHA)
                } else {
                    texture_settings.remove(PREMULTIPLY_ALPHA)
                }

                self.texture_unpacking_settings.set(texture_settings);
                return;
            },
            constants::UNPACK_COLORSPACE_CONVERSION_WEBGL => {
                match param_value as u32 {
                    constants::BROWSER_DEFAULT_WEBGL
                        => texture_settings.insert(CONVERT_COLORSPACE),
                    constants::NONE
                        => texture_settings.remove(CONVERT_COLORSPACE),
                    _ => return self.webgl_error(InvalidEnum),
                }

                self.texture_unpacking_settings.set(texture_settings);
                return;
            },
            constants::UNPACK_ALIGNMENT |
            constants::PACK_ALIGNMENT => {
                match param_value {
                    1 | 2 | 4 | 8 => (),
                    _ => return self.webgl_error(InvalidValue),
                }
            },
            _ => return self.webgl_error(InvalidEnum),
        }

        self.ipc_renderer
            .send(CanvasMsg::WebGL(CanvasWebGLMsg::PixelStorei(param_name, param_value)))
            .unwrap()
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn PolygonOffset(self, factor: f32, units: f32) {
        self.ipc_renderer
            .send(CanvasMsg::WebGL(CanvasWebGLMsg::PolygonOffset(factor, units)))
            .unwrap()
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

    #[allow(unsafe_code)]
    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
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
        self.ipc_renderer
            .send(CanvasMsg::WebGL(CanvasWebGLMsg::Uniform4fv(uniform_id, data_vec)))
            .unwrap()
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
        if let constants::FLOAT = data_type {
           let msg = CanvasMsg::WebGL(
               CanvasWebGLMsg::VertexAttribPointer2f(attrib_id, size, normalized, stride, offset as u32));
            self.ipc_renderer.send(msg).unwrap()
        } else {
            panic!("VertexAttribPointer: Data Type not supported")
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.4
    fn Viewport(self, x: i32, y: i32, width: i32, height: i32) {
        self.ipc_renderer
            .send(CanvasMsg::WebGL(CanvasWebGLMsg::Viewport(x, y, width, height)))
            .unwrap()
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn TexImage2D(self,
                  target: u32,
                  level: i32,
                  internal_format: u32,
                  format: u32,
                  data_type: u32,
                  source: Option<ImageDataOrHTMLImageElementOrHTMLCanvasElementOrHTMLVideoElement >) {
        // TODO(ecoal95): Check for bound WebGLTexture, and validate more parameters
        match target {
            constants::TEXTURE_2D |
            constants::TEXTURE_CUBE_MAP => (),

            _ => return self.webgl_error(InvalidEnum),
        }

        let source = match source {
            Some(s) => s,
            None => return,
        };

        let (pixels, size) = match source {
            ImageDataOrHTMLImageElementOrHTMLCanvasElementOrHTMLVideoElement::eImageData(image_data) => {
                let global = self.global.root();
                (image_data.get_data_array(&global.r()), image_data.get_size())
            },
            ImageDataOrHTMLImageElementOrHTMLCanvasElementOrHTMLVideoElement::eHTMLImageElement(image) => {
                let img_url = match image.r().get_url() {
                    Some(url) => url,
                    None => return,
                };

                let canvas = self.canvas.root();
                let window = window_from_node(canvas.r());

                let img = match canvas_utils::request_image_from_cache(window.r(), img_url) {
                    ImageResponse::Loaded(img) => img,
                    ImageResponse::PlaceholderLoaded(_) | ImageResponse::None
                        => return,
                };

                let size = Size2D::new(img.width as i32, img.height as i32);
                // TODO(ecoal95): Validate that the format argument is coherent with the image.
                // RGB8 should be easy to support too
                let mut data = match img.format {
                    PixelFormat::RGBA8 => img.bytes.to_vec(),
                    _ => unimplemented!(),
                };

                byte_swap(&mut data);

                (data, size)
            },
            // TODO(ecoal95): Getting canvas data is implemented in CanvasRenderingContext2D, but
            // we need to refactor it moving it to `HTMLCanvasElement` and supporting WebGLContext
            ImageDataOrHTMLImageElementOrHTMLCanvasElementOrHTMLVideoElement::eHTMLCanvasElement(_rooted_canvas)
                => unimplemented!(),
            ImageDataOrHTMLImageElementOrHTMLCanvasElementOrHTMLVideoElement::eHTMLVideoElement(_rooted_video)
                => unimplemented!(),
        };

        // TODO(ecoal95): Invert axis, convert colorspace, premultiply alpha if requested
        let msg = CanvasWebGLMsg::TexImage2D(target, level, internal_format as i32,
                                             size.width, size.height,
                                             format, data_type, pixels);

        self.ipc_renderer
            .send(CanvasMsg::WebGL(msg))
            .unwrap()
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn TexParameterf(self, target: u32, name: u32, value: f32) {
        match target {
            constants::TEXTURE_2D |
            constants::TEXTURE_CUBE_MAP => {
                if let Some(texture) = self.bound_texture_for(target) {
                    let texture = texture.root();
                    let result = texture.r().tex_parameter(target, name, TexParameterValue::Float(value));
                    handle_potential_webgl_error!(self, result, ());
                } else {
                    return self.webgl_error(InvalidOperation);
                }
            },

            _ => return self.webgl_error(InvalidEnum),
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn TexParameteri(self, target: u32, name: u32, value: i32) {
        match target {
            constants::TEXTURE_2D |
            constants::TEXTURE_CUBE_MAP => {
                if let Some(texture) = self.bound_texture_for(target) {
                    let texture = texture.root();
                    let result = texture.r().tex_parameter(target, name, TexParameterValue::Int(value));
                    handle_potential_webgl_error!(self, result, ());
                } else {
                    return self.webgl_error(InvalidOperation);
                }
            },

            _ => return self.webgl_error(InvalidEnum),
        }
    }
}

pub trait WebGLRenderingContextHelpers {
    fn webgl_error(&self, err: WebGLError);
    fn bound_texture_for(&self, target: u32) -> Option<JS<WebGLTexture>>;
}

impl<'a> WebGLRenderingContextHelpers for &'a WebGLRenderingContext {
    fn webgl_error(&self, err: WebGLError) {
        // If an error has been detected no further errors must be
        // recorded until `getError` has been called
        if self.last_error.get().is_none() {
            self.last_error.set(Some(err));
        }
    }

    fn bound_texture_for(&self, target: u32) -> Option<JS<WebGLTexture>> {
        match target {
            constants::TEXTURE_2D => self.bound_texture_2d.get(),
            constants::TEXTURE_CUBE_MAP => self.bound_texture_cube_map.get(),

            _ => unreachable!(),
        }
    }
}

pub trait LayoutCanvasWebGLRenderingContextHelpers {
    #[allow(unsafe_code)]
    unsafe fn get_renderer_id(&self) -> usize;
    #[allow(unsafe_code)]
    unsafe fn get_ipc_renderer(&self) -> IpcSender<CanvasMsg>;
}

impl LayoutCanvasWebGLRenderingContextHelpers for LayoutJS<WebGLRenderingContext> {
    #[allow(unsafe_code)]
    unsafe fn get_renderer_id(&self) -> usize {
        (*self.unsafe_get()).renderer_id
    }
    #[allow(unsafe_code)]
    unsafe fn get_ipc_renderer(&self) -> IpcSender<CanvasMsg> {
        (*self.unsafe_get()).ipc_renderer.clone()
    }
}
