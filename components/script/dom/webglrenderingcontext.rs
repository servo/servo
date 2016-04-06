/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use canvas_traits::{CanvasCommonMsg, CanvasMsg};
use dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLRenderingContextConstants as constants;
use dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::{WebGLRenderingContextMethods};
use dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::{self, WebGLContextAttributes};
use dom::bindings::codegen::UnionTypes::ImageDataOrHTMLImageElementOrHTMLCanvasElementOrHTMLVideoElement;
use dom::bindings::conversions::{ToJSValConvertible, array_buffer_view_to_vec_checked, array_buffer_view_to_vec};
use dom::bindings::global::GlobalRef;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, LayoutJS, MutNullableHeap, Root};
use dom::bindings::reflector::{Reflectable, Reflector, reflect_dom_object};
use dom::event::{Event, EventBubbles, EventCancelable};
use dom::htmlcanvaselement::HTMLCanvasElement;
use dom::htmlcanvaselement::utils as canvas_utils;
use dom::node::{Node, NodeDamage, window_from_node};
use dom::webglactiveinfo::WebGLActiveInfo;
use dom::webglbuffer::WebGLBuffer;
use dom::webglcontextevent::WebGLContextEvent;
use dom::webglframebuffer::WebGLFramebuffer;
use dom::webglprogram::WebGLProgram;
use dom::webglrenderbuffer::WebGLRenderbuffer;
use dom::webglshader::WebGLShader;
use dom::webgltexture::{TexParameterValue, WebGLTexture};
use dom::webgluniformlocation::WebGLUniformLocation;
use euclid::size::Size2D;
use ipc_channel::ipc::{self, IpcSender};
use js::jsapi::{JSContext, JSObject, RootedValue};
use js::jsval::{BooleanValue, DoubleValue, Int32Value, JSVal, NullValue, UndefinedValue};
use net_traits::image::base::PixelFormat;
use net_traits::image_cache_thread::ImageResponse;
use offscreen_gl_context::GLContextAttributes;
use script_traits::ScriptMsg as ConstellationMsg;
use std::cell::Cell;
use util::str::DOMString;
use util::vec::byte_swap;
use webrender_traits::WebGLError::*;
use webrender_traits::{WebGLCommand, WebGLError, WebGLFramebufferBindingRequest, WebGLParameter};

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
    };
    ($context:ident, $call:expr) => {
        handle_potential_webgl_error!($context, $call, ());
    };
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
pub struct WebGLRenderingContext {
    reflector_: Reflector,
    #[ignore_heap_size_of = "Defined in ipc-channel"]
    ipc_renderer: IpcSender<CanvasMsg>,
    canvas: JS<HTMLCanvasElement>,
    #[ignore_heap_size_of = "Defined in webrender_traits"]
    last_error: Cell<Option<WebGLError>>,
    texture_unpacking_settings: Cell<TextureUnpacking>,
    bound_texture_2d: MutNullableHeap<JS<WebGLTexture>>,
    bound_texture_cube_map: MutNullableHeap<JS<WebGLTexture>>,
    bound_buffer_array: MutNullableHeap<JS<WebGLBuffer>>,
    bound_buffer_element_array: MutNullableHeap<JS<WebGLBuffer>>,
    current_program: MutNullableHeap<JS<WebGLProgram>>,
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
                          .send(ConstellationMsg::CreateWebGLPaintThread(size, attrs, sender))
                          .unwrap();
        let result = receiver.recv().unwrap();

        result.map(|ipc_renderer| {
            WebGLRenderingContext {
                reflector_: Reflector::new(),
                ipc_renderer: ipc_renderer,
                canvas: JS::from_ref(canvas),
                last_error: Cell::new(None),
                texture_unpacking_settings: Cell::new(CONVERT_COLORSPACE),
                bound_texture_2d: MutNullableHeap::new(None),
                bound_texture_cube_map: MutNullableHeap::new(None),
                bound_buffer_array: MutNullableHeap::new(None),
                bound_buffer_element_array: MutNullableHeap::new(None),
                current_program: MutNullableHeap::new(None),
            }
        })
    }

    #[allow(unrooted_must_root)]
    pub fn new(global: GlobalRef, canvas: &HTMLCanvasElement, size: Size2D<i32>, attrs: GLContextAttributes)
               -> Option<Root<WebGLRenderingContext>> {
        match WebGLRenderingContext::new_inherited(global, canvas, size, attrs) {
            Ok(ctx) => Some(reflect_dom_object(box ctx, global,
                                               WebGLRenderingContextBinding::Wrap)),
            Err(msg) => {
                error!("Couldn't create WebGLRenderingContext: {}", msg);
                let event = WebGLContextEvent::new(global,
                                                   atom!("webglcontextcreationerror"),
                                                   EventBubbles::DoesNotBubble,
                                                   EventCancelable::Cancelable,
                                                   DOMString::from(msg));
                event.upcast::<Event>().fire(canvas.upcast());
                None
            }
        }
    }

    pub fn recreate(&self, size: Size2D<i32>) {
        self.ipc_renderer.send(CanvasMsg::Common(CanvasCommonMsg::Recreate(size))).unwrap();
    }

    pub fn ipc_renderer(&self) -> IpcSender<CanvasMsg> {
        self.ipc_renderer.clone()
    }

    pub fn webgl_error(&self, err: WebGLError) {
        // If an error has been detected no further errors must be
        // recorded until `getError` has been called
        if self.last_error.get().is_none() {
            self.last_error.set(Some(err));
        }
    }

    fn tex_parameter(&self, target: u32, name: u32, value: TexParameterValue) {
        let texture = match target {
            constants::TEXTURE_2D => self.bound_texture_2d.get(),
            constants::TEXTURE_CUBE_MAP => self.bound_texture_cube_map.get(),
            _ => return self.webgl_error(InvalidEnum),
        };
        if let Some(texture) = texture {
            handle_potential_webgl_error!(self, texture.tex_parameter(target, name, value));
        } else {
            return self.webgl_error(InvalidOperation);
        }
    }

    fn mark_as_dirty(&self) {
        self.canvas.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
    }

    fn vertex_attrib(&self, indx: u32, x: f32, y: f32, z: f32, w: f32) {
        self.ipc_renderer
            .send(CanvasMsg::WebGL(WebGLCommand::VertexAttrib(indx, x, y, z, w)))
            .unwrap();
    }
}

impl Drop for WebGLRenderingContext {
    fn drop(&mut self) {
        self.ipc_renderer.send(CanvasMsg::Common(CanvasCommonMsg::Close)).unwrap();
    }
}

impl WebGLRenderingContextMethods for WebGLRenderingContext {
    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.1
    fn Canvas(&self) -> Root<HTMLCanvasElement> {
        Root::from_ref(&*self.canvas)
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.11
    fn Flush(&self) {
        self.ipc_renderer
            .send(CanvasMsg::WebGL(WebGLCommand::Flush))
            .unwrap();
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.11
    fn Finish(&self) {
        let (sender, receiver) = ipc::channel().unwrap();
        self.ipc_renderer
            .send(CanvasMsg::WebGL(WebGLCommand::Finish(sender)))
            .unwrap();
        receiver.recv().unwrap()
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.1
    fn DrawingBufferWidth(&self) -> i32 {
        let (sender, receiver) = ipc::channel().unwrap();
        self.ipc_renderer
            .send(CanvasMsg::WebGL(WebGLCommand::DrawingBufferWidth(sender)))
            .unwrap();
        receiver.recv().unwrap()
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.1
    fn DrawingBufferHeight(&self) -> i32 {
        let (sender, receiver) = ipc::channel().unwrap();
        self.ipc_renderer
            .send(CanvasMsg::WebGL(WebGLCommand::DrawingBufferHeight(sender)))
            .unwrap();
        receiver.recv().unwrap()
    }

    #[allow(unsafe_code)]
    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.5
    fn GetBufferParameter(&self, _cx: *mut JSContext, target: u32, parameter: u32) -> JSVal {
        let (sender, receiver) = ipc::channel().unwrap();
        self.ipc_renderer
            .send(CanvasMsg::WebGL(WebGLCommand::GetBufferParameter(target, parameter, sender)))
            .unwrap();
        match handle_potential_webgl_error!(self, receiver.recv().unwrap(), WebGLParameter::Invalid) {
            WebGLParameter::Int(val) => Int32Value(val),
            WebGLParameter::Bool(_) => panic!("Buffer parameter should not be bool"),
            WebGLParameter::Float(_) => panic!("Buffer parameter should not be float"),
            WebGLParameter::String(_) => panic!("Buffer parameter should not be string"),
            WebGLParameter::Invalid => NullValue(),
        }
    }

    #[allow(unsafe_code)]
    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn GetParameter(&self, cx: *mut JSContext, parameter: u32) -> JSVal {
        let (sender, receiver) = ipc::channel().unwrap();
        self.ipc_renderer
            .send(CanvasMsg::WebGL(WebGLCommand::GetParameter(parameter, sender)))
            .unwrap();
        match handle_potential_webgl_error!(self, receiver.recv().unwrap(), WebGLParameter::Invalid) {
            WebGLParameter::Int(val) => Int32Value(val),
            WebGLParameter::Bool(val) => BooleanValue(val),
            WebGLParameter::Float(val) => DoubleValue(val as f64),
            WebGLParameter::String(val) => {
                let mut rval = RootedValue::new(cx, UndefinedValue());
                unsafe {
                    val.to_jsval(cx, rval.handle_mut());
                }
                rval.ptr
            }
            WebGLParameter::Invalid => NullValue(),
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn GetError(&self) -> u32 {
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
    fn GetContextAttributes(&self) -> Option<WebGLContextAttributes> {
        let (sender, receiver) = ipc::channel().unwrap();

        // If the send does not succeed, assume context lost
        if let Err(_) = self.ipc_renderer
                            .send(CanvasMsg::WebGL(WebGLCommand::GetContextAttributes(sender))) {
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
    fn GetSupportedExtensions(&self) -> Option<Vec<DOMString>> {
        Some(vec![])
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.14
    fn GetExtension(&self, _cx: *mut JSContext, _name: DOMString) -> *mut JSObject {
        0 as *mut JSObject
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn ActiveTexture(&self, texture: u32) {
        self.ipc_renderer.send(CanvasMsg::WebGL(WebGLCommand::ActiveTexture(texture))).unwrap();
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn BlendColor(&self, r: f32, g: f32, b: f32, a: f32) {
        self.ipc_renderer.send(CanvasMsg::WebGL(WebGLCommand::BlendColor(r, g, b, a))).unwrap();
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn BlendEquation(&self, mode: u32) {
        self.ipc_renderer.send(CanvasMsg::WebGL(WebGLCommand::BlendEquation(mode))).unwrap();
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn BlendEquationSeparate(&self, mode_rgb: u32, mode_alpha: u32) {
        self.ipc_renderer
            .send(CanvasMsg::WebGL(WebGLCommand::BlendEquationSeparate(mode_rgb, mode_alpha)))
            .unwrap();
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn BlendFunc(&self, src_factor: u32, dest_factor: u32) {
        self.ipc_renderer
            .send(CanvasMsg::WebGL(WebGLCommand::BlendFunc(src_factor, dest_factor)))
            .unwrap();
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn BlendFuncSeparate(&self, src_rgb: u32, dest_rgb: u32, src_alpha: u32, dest_alpha: u32) {
        self.ipc_renderer.send(
            CanvasMsg::WebGL(WebGLCommand::BlendFuncSeparate(src_rgb, dest_rgb, src_alpha, dest_alpha))).unwrap();
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn AttachShader(&self, program: Option<&WebGLProgram>, shader: Option<&WebGLShader>) {
        if let Some(program) = program {
            if let Some(shader) = shader {
                handle_potential_webgl_error!(self, program.attach_shader(shader));
            }
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn DetachShader(&self, program: Option<&WebGLProgram>, shader: Option<&WebGLShader>) {
        if let Some(program) = program {
            if let Some(shader) = shader {
                handle_potential_webgl_error!(self, program.detach_shader(shader));
            }
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn BindAttribLocation(&self, program: Option<&WebGLProgram>,
                          index: u32, name: DOMString) {
        if let Some(program) = program {
            handle_potential_webgl_error!(self, program.bind_attrib_location(index, name));
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.5
    fn BindBuffer(&self, target: u32, buffer: Option<&WebGLBuffer>) {
        let slot = match target {
            constants::ARRAY_BUFFER => &self.bound_buffer_array,
            constants::ELEMENT_ARRAY_BUFFER => &self.bound_buffer_element_array,

            _ => return self.webgl_error(InvalidEnum),
        };

        if let Some(buffer) = buffer {
            match buffer.bind(target) {
                Ok(_) => slot.set(Some(buffer)),
                Err(e) => return self.webgl_error(e),
            }
        } else {
            // Unbind the current buffer
            self.ipc_renderer
                .send(CanvasMsg::WebGL(WebGLCommand::BindBuffer(target, 0)))
                .unwrap()
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.6
    fn BindFramebuffer(&self, target: u32, framebuffer: Option<&WebGLFramebuffer>) {
        if target != constants::FRAMEBUFFER {
            return self.webgl_error(InvalidOperation);
        }

        if let Some(framebuffer) = framebuffer {
            framebuffer.bind(target)
        } else {
            // Bind the default framebuffer
            let cmd = WebGLCommand::BindFramebuffer(target, WebGLFramebufferBindingRequest::Default);
            self.ipc_renderer.send(CanvasMsg::WebGL(cmd)).unwrap();
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.7
    fn BindRenderbuffer(&self, target: u32, renderbuffer: Option<&WebGLRenderbuffer>) {
        if target != constants::RENDERBUFFER {
            return self.webgl_error(InvalidEnum);
        }

        if let Some(renderbuffer) = renderbuffer {
            renderbuffer.bind(target)
        } else {
            // Unbind the currently bound renderbuffer
            self.ipc_renderer
                .send(CanvasMsg::WebGL(WebGLCommand::BindRenderbuffer(target, 0)))
                .unwrap()
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn BindTexture(&self, target: u32, texture: Option<&WebGLTexture>) {
        let slot = match target {
            constants::TEXTURE_2D => &self.bound_texture_2d,
            constants::TEXTURE_CUBE_MAP => &self.bound_texture_cube_map,

            _ => return self.webgl_error(InvalidEnum),
        };

        if let Some(texture) = texture {
            match texture.bind(target) {
                Ok(_) => slot.set(Some(texture)),
                Err(err) => return self.webgl_error(err),
            }
        } else {
            // Unbind the currently bound texture
            self.ipc_renderer
                .send(CanvasMsg::WebGL(WebGLCommand::BindTexture(target, 0)))
                .unwrap()
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn GenerateMipmap(&self, target: u32) {
        let slot = match target {
            constants::TEXTURE_2D => &self.bound_texture_2d,
            constants::TEXTURE_CUBE_MAP => &self.bound_texture_cube_map,

            _ => return self.webgl_error(InvalidEnum),
        };

        match slot.get() {
            Some(texture) => handle_potential_webgl_error!(self, texture.generate_mipmap()),
            None => self.webgl_error(InvalidOperation)
        }
    }

    #[allow(unsafe_code)]
    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.5
    fn BufferData(&self, _cx: *mut JSContext, target: u32, data: Option<*mut JSObject>, usage: u32) {
        let bound_buffer = match target {
            constants::ARRAY_BUFFER => self.bound_buffer_array.get(),
            constants::ELEMENT_ARRAY_BUFFER => self.bound_buffer_element_array.get(),
            _ => return self.webgl_error(InvalidEnum),
        };
        let bound_buffer = match bound_buffer {
            Some(bound_buffer) => bound_buffer,
            None => return self.webgl_error(InvalidValue),
        };
        match usage {
            constants::STREAM_DRAW |
            constants::STATIC_DRAW |
            constants::DYNAMIC_DRAW => (),
            _ => return self.webgl_error(InvalidEnum),
        }
        let data = match data {
            Some(data) => data,
            None => return self.webgl_error(InvalidValue),
        };
        if let Some(data_vec) = array_buffer_view_to_vec::<u8>(data) {
            handle_potential_webgl_error!(self, bound_buffer.buffer_data(target, &data_vec, usage));
        } else {
            // NB: array_buffer_view_to_vec should never fail when
            // we have WebIDL support for Float32Array etc.
            self.webgl_error(InvalidValue);
        }
    }

    #[allow(unsafe_code)]
    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.5
    fn BufferSubData(&self, _cx: *mut JSContext, target: u32, offset: i64, data: Option<*mut JSObject>) {
        let bound_buffer = match target {
            constants::ARRAY_BUFFER => self.bound_buffer_array.get(),
            constants::ELEMENT_ARRAY_BUFFER => self.bound_buffer_element_array.get(),
            _ => return self.webgl_error(InvalidEnum),
        };
        let bound_buffer = match bound_buffer {
            Some(bound_buffer) => bound_buffer,
            None => return self.webgl_error(InvalidOperation),
        };
        let data = match data {
            Some(data) => data,
            None => return self.webgl_error(InvalidValue),
        };

        if offset < 0 {
            return self.webgl_error(InvalidValue);
        }
        if let Some(data_vec) = array_buffer_view_to_vec::<u8>(data) {
            if (offset as usize) + data_vec.len() > bound_buffer.capacity() {
                return self.webgl_error(InvalidValue);
            }
            self.ipc_renderer
                .send(CanvasMsg::WebGL(WebGLCommand::BufferSubData(target, offset as isize, data_vec)))
                .unwrap()
        } else {
            self.webgl_error(InvalidValue);
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn CompressedTexImage2D(&self, _cx: *mut JSContext, _target: u32, _level: i32, _internal_format: u32,
                            _width: i32, _height: i32, _border: i32, _pixels: *mut JSObject) {
        // FIXME: No compressed texture format is currently supported, so error out as per
        // https://www.khronos.org/registry/webgl/specs/latest/1.0/#COMPRESSED_TEXTURE_SUPPORT
        self.webgl_error(InvalidEnum)
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn CompressedTexSubImage2D(&self, _cx: *mut JSContext, _target: u32, _level: i32,
                               _xoffset: i32, _yoffset: i32, _width: i32, _height: i32,
                               _format: u32, _pixels: *mut JSObject) {
        // FIXME: No compressed texture format is currently supported, so error out as per
        // https://www.khronos.org/registry/webgl/specs/latest/1.0/#COMPRESSED_TEXTURE_SUPPORT
        self.webgl_error(InvalidEnum)
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.11
    fn Clear(&self, mask: u32) {
        self.ipc_renderer.send(CanvasMsg::WebGL(WebGLCommand::Clear(mask))).unwrap();
        self.mark_as_dirty();
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn ClearColor(&self, red: f32, green: f32, blue: f32, alpha: f32) {
        self.ipc_renderer
            .send(CanvasMsg::WebGL(WebGLCommand::ClearColor(red, green, blue, alpha)))
            .unwrap()
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn ClearDepth(&self, depth: f32) {
        self.ipc_renderer
            .send(CanvasMsg::WebGL(WebGLCommand::ClearDepth(depth as f64)))
            .unwrap()
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn ClearStencil(&self, stencil: i32) {
        self.ipc_renderer
            .send(CanvasMsg::WebGL(WebGLCommand::ClearStencil(stencil)))
            .unwrap()
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn ColorMask(&self, r: bool, g: bool, b: bool, a: bool) {
        self.ipc_renderer
            .send(CanvasMsg::WebGL(WebGLCommand::ColorMask(r, g, b, a)))
            .unwrap()
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn CullFace(&self, mode: u32) {
        match mode {
            constants::FRONT | constants::BACK | constants::FRONT_AND_BACK =>
                self.ipc_renderer
                    .send(CanvasMsg::WebGL(WebGLCommand::CullFace(mode)))
                    .unwrap(),
            _ => self.webgl_error(InvalidEnum),
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn FrontFace(&self, mode: u32) {
        match mode {
            constants::CW | constants::CCW =>
                self.ipc_renderer
                    .send(CanvasMsg::WebGL(WebGLCommand::FrontFace(mode)))
                    .unwrap(),
            _ => self.webgl_error(InvalidEnum),
        }
    }
    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn DepthFunc(&self, func: u32) {
        match func {
            constants::NEVER | constants::LESS |
            constants::EQUAL | constants::LEQUAL |
            constants::GREATER | constants::NOTEQUAL |
            constants::GEQUAL | constants::ALWAYS =>
                self.ipc_renderer
                    .send(CanvasMsg::WebGL(WebGLCommand::DepthFunc(func)))
                    .unwrap(),
            _ => self.webgl_error(InvalidEnum),
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn DepthMask(&self, flag: bool) {
        self.ipc_renderer
            .send(CanvasMsg::WebGL(WebGLCommand::DepthMask(flag)))
            .unwrap()
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn DepthRange(&self, near: f32, far: f32) {
        self.ipc_renderer
            .send(CanvasMsg::WebGL(WebGLCommand::DepthRange(near as f64, far as f64)))
            .unwrap()
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn Enable(&self, cap: u32) {
        match cap {
            constants::BLEND | constants::CULL_FACE | constants::DEPTH_TEST | constants::DITHER |
            constants::POLYGON_OFFSET_FILL | constants::SAMPLE_ALPHA_TO_COVERAGE | constants::SAMPLE_COVERAGE |
            constants::SAMPLE_COVERAGE_INVERT | constants::SCISSOR_TEST =>
                self.ipc_renderer
                    .send(CanvasMsg::WebGL(WebGLCommand::Enable(cap)))
                    .unwrap(),
            _ => self.webgl_error(InvalidEnum),
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn Disable(&self, cap: u32) {
        match cap {
            constants::BLEND | constants::CULL_FACE | constants::DEPTH_TEST | constants::DITHER |
            constants::POLYGON_OFFSET_FILL | constants::SAMPLE_ALPHA_TO_COVERAGE | constants::SAMPLE_COVERAGE |
            constants::SAMPLE_COVERAGE_INVERT | constants::SCISSOR_TEST =>
                self.ipc_renderer
                    .send(CanvasMsg::WebGL(WebGLCommand::Disable(cap)))
                    .unwrap(),
            _ => self.webgl_error(InvalidEnum),
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn CompileShader(&self, shader: Option<&WebGLShader>) {
        if let Some(shader) = shader {
            shader.compile()
        }
    }

    // TODO(emilio): Probably in the future we should keep track of the
    // generated objects, either here or in the webgl thread
    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.5
    fn CreateBuffer(&self) -> Option<Root<WebGLBuffer>> {
        WebGLBuffer::maybe_new(self.global().r(), self.ipc_renderer.clone())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.6
    fn CreateFramebuffer(&self) -> Option<Root<WebGLFramebuffer>> {
        WebGLFramebuffer::maybe_new(self.global().r(), self.ipc_renderer.clone())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.7
    fn CreateRenderbuffer(&self) -> Option<Root<WebGLRenderbuffer>> {
        WebGLRenderbuffer::maybe_new(self.global().r(), self.ipc_renderer.clone())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn CreateTexture(&self) -> Option<Root<WebGLTexture>> {
        WebGLTexture::maybe_new(self.global().r(), self.ipc_renderer.clone())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn CreateProgram(&self) -> Option<Root<WebGLProgram>> {
        WebGLProgram::maybe_new(self.global().r(), self.ipc_renderer.clone())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn CreateShader(&self, shader_type: u32) -> Option<Root<WebGLShader>> {
        WebGLShader::maybe_new(self.global().r(), self.ipc_renderer.clone(), shader_type)
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.5
    fn DeleteBuffer(&self, buffer: Option<&WebGLBuffer>) {
        if let Some(buffer) = buffer {
            buffer.delete()
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.6
    fn DeleteFramebuffer(&self, framebuffer: Option<&WebGLFramebuffer>) {
        if let Some(framebuffer) = framebuffer {
            framebuffer.delete()
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.7
    fn DeleteRenderbuffer(&self, renderbuffer: Option<&WebGLRenderbuffer>) {
        if let Some(renderbuffer) = renderbuffer {
            renderbuffer.delete()
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn DeleteTexture(&self, texture: Option<&WebGLTexture>) {
        if let Some(texture) = texture {
            texture.delete()
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn DeleteProgram(&self, program: Option<&WebGLProgram>) {
        if let Some(program) = program {
            program.delete()
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn DeleteShader(&self, shader: Option<&WebGLShader>) {
        if let Some(shader) = shader {
            shader.delete()
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.11
    fn DrawArrays(&self, mode: u32, first: i32, count: i32) {
        match mode {
            constants::POINTS | constants::LINE_STRIP |
            constants::LINE_LOOP | constants::LINES |
            constants::TRIANGLE_STRIP | constants::TRIANGLE_FAN |
            constants::TRIANGLES => {
                if self.current_program.get().is_none() {
                    return self.webgl_error(InvalidOperation);
                }

                if first < 0 || count < 0 {
                    self.webgl_error(InvalidValue);
                } else {
                    self.ipc_renderer
                        .send(CanvasMsg::WebGL(WebGLCommand::DrawArrays(mode, first, count)))
                        .unwrap();
                    self.mark_as_dirty();
                }
            },
            _ => self.webgl_error(InvalidEnum),
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.11
    fn DrawElements(&self, mode: u32, count: i32, type_: u32, offset: i64) {
        let type_size = match type_ {
            constants::BYTE | constants::UNSIGNED_BYTE => 1,
            constants::SHORT | constants::UNSIGNED_SHORT => 2,
            constants::INT | constants::UNSIGNED_INT | constants::FLOAT => 4,
            _ => return self.webgl_error(InvalidEnum),
        };

        if offset % type_size != 0 {
            return self.webgl_error(InvalidOperation);
        }

        if count <= 0 {
            return self.webgl_error(InvalidOperation);
        }

        if offset < 0 {
            return self.webgl_error(InvalidValue);
        }

        if self.current_program.get().is_none() || self.bound_buffer_element_array.get().is_none() {
            return self.webgl_error(InvalidOperation);
        }

        match mode {
            constants::POINTS | constants::LINE_STRIP |
            constants::LINE_LOOP | constants::LINES |
            constants::TRIANGLE_STRIP | constants::TRIANGLE_FAN |
            constants::TRIANGLES => {
                self.ipc_renderer
                    .send(CanvasMsg::WebGL(WebGLCommand::DrawElements(mode, count, type_, offset)))
                    .unwrap();
                self.mark_as_dirty();
            },
            _ => self.webgl_error(InvalidEnum),
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn EnableVertexAttribArray(&self, attrib_id: u32) {
        self.ipc_renderer
            .send(CanvasMsg::WebGL(WebGLCommand::EnableVertexAttribArray(attrib_id)))
            .unwrap()
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn GetActiveUniform(&self, program: Option<&WebGLProgram>, index: u32) -> Option<Root<WebGLActiveInfo>> {
        program.and_then(|p| match p.get_active_uniform(index) {
            Ok(ret) => Some(ret),
            Err(error) => {
                self.webgl_error(error);
                None
            },
        })
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn GetAttribLocation(&self, program: Option<&WebGLProgram>, name: DOMString) -> i32 {
        if let Some(program) = program {
            handle_potential_webgl_error!(self, program.get_attrib_location(name), None).unwrap_or(-1)
        } else {
            -1
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn GetProgramParameter(&self, _: *mut JSContext, program: Option<&WebGLProgram>, param_id: u32) -> JSVal {
        if let Some(program) = program {
            match handle_potential_webgl_error!(self, program.parameter(param_id), WebGLParameter::Invalid) {
                WebGLParameter::Int(val) => Int32Value(val),
                WebGLParameter::Bool(val) => BooleanValue(val),
                WebGLParameter::String(_) => panic!("Program parameter should not be string"),
                WebGLParameter::Float(_) => panic!("Program parameter should not be float"),
                WebGLParameter::Invalid => NullValue(),
            }
        } else {
            NullValue()
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn GetShaderInfoLog(&self, shader: Option<&WebGLShader>) -> Option<DOMString> {
        shader.and_then(|s| s.info_log()).map(DOMString::from)
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn GetShaderParameter(&self, _: *mut JSContext, shader: Option<&WebGLShader>, param_id: u32) -> JSVal {
        if let Some(shader) = shader {
            match handle_potential_webgl_error!(self, shader.parameter(param_id), WebGLParameter::Invalid) {
                WebGLParameter::Int(val) => Int32Value(val),
                WebGLParameter::Bool(val) => BooleanValue(val),
                WebGLParameter::String(_) => panic!("Shader parameter should not be string"),
                WebGLParameter::Float(_) => panic!("Shader parameter should not be float"),
                WebGLParameter::Invalid => NullValue(),
            }
        } else {
            NullValue()
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn GetUniformLocation(&self,
                          program: Option<&WebGLProgram>,
                          name: DOMString) -> Option<Root<WebGLUniformLocation>> {
        if let Some(program) = program {
            handle_potential_webgl_error!(self, program.get_uniform_location(name), None)
                .map(|location| WebGLUniformLocation::new(self.global().r(), location, program.id()))
        } else {
            None
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn Hint(&self, target: u32, mode: u32) {
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
            .send(CanvasMsg::WebGL(WebGLCommand::Hint(target, mode)))
            .unwrap()
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn LineWidth(&self, width: f32) {
        if width.is_nan() || width <= 0f32 {
            return self.webgl_error(InvalidValue);
        }

        self.ipc_renderer
            .send(CanvasMsg::WebGL(WebGLCommand::LineWidth(width)))
            .unwrap()
    }

    // NOTE: Usage of this function could affect rendering while we keep using
    //   readback to render to the page.
    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn PixelStorei(&self, param_name: u32, param_value: i32) {
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
            .send(CanvasMsg::WebGL(WebGLCommand::PixelStorei(param_name, param_value)))
            .unwrap()
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn PolygonOffset(&self, factor: f32, units: f32) {
        self.ipc_renderer
            .send(CanvasMsg::WebGL(WebGLCommand::PolygonOffset(factor, units)))
            .unwrap()
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.4
    fn Scissor(&self, x: i32, y: i32, width: i32, height: i32) {
        self.ipc_renderer
            .send(CanvasMsg::WebGL(WebGLCommand::Scissor(x, y, width, height)))
            .unwrap()
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn LinkProgram(&self, program: Option<&WebGLProgram>) {
        if let Some(program) = program {
            program.link()
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn ShaderSource(&self, shader: Option<&WebGLShader>, source: DOMString) {
        if let Some(shader) = shader {
            shader.set_source(source)
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn GetShaderSource(&self, shader: Option<&WebGLShader>) -> Option<DOMString> {
        if let Some(shader) = shader {
            shader.source()
        } else {
            None
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform1f(&self,
                  uniform: Option<&WebGLUniformLocation>,
                  val: f32) {
        let uniform = match uniform {
            Some(uniform) => uniform,
            None => return,
        };

        match self.current_program.get() {
            Some(ref program) if program.id() == uniform.program_id() => {},
            _ => return self.webgl_error(InvalidOperation),
        };

        self.ipc_renderer
            .send(CanvasMsg::WebGL(WebGLCommand::Uniform1f(uniform.id(), val)))
            .unwrap()
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform1i(&self,
                  uniform: Option<&WebGLUniformLocation>,
                  val: i32) {
        let uniform = match uniform {
            Some(uniform) => uniform,
            None => return,
        };

        match self.current_program.get() {
            Some(ref program) if program.id() == uniform.program_id() => {},
            _ => return self.webgl_error(InvalidOperation),
        };

        self.ipc_renderer
            .send(CanvasMsg::WebGL(WebGLCommand::Uniform1i(uniform.id(), val)))
            .unwrap()
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform1iv(&self,
                  _cx: *mut JSContext,
                  uniform: Option<&WebGLUniformLocation>,
                  data: Option<*mut JSObject>) {
        let data = match data {
            Some(data) => data,
            None => return self.webgl_error(InvalidValue),
        };

        if let Some(data) = array_buffer_view_to_vec_checked::<i32>(data) {
            if data.len() < 1 {
                return self.webgl_error(InvalidOperation);
            }

            self.Uniform1i(uniform, data[0]);
        } else {
            self.webgl_error(InvalidValue);
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform1fv(&self,
                  uniform: Option<&WebGLUniformLocation>,
                  data: Vec<f32>) {
        if data.is_empty() {
            return self.webgl_error(InvalidValue);
        }

        self.Uniform1f(uniform, data[0]);
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform2f(&self,
                  uniform: Option<&WebGLUniformLocation>,
                  x: f32, y: f32) {
        let uniform = match uniform {
            Some(uniform) => uniform,
            None => return,
        };

        match self.current_program.get() {
            Some(ref program) if program.id() == uniform.program_id() => {},
            _ => return self.webgl_error(InvalidOperation),
        };

        self.ipc_renderer
            .send(CanvasMsg::WebGL(WebGLCommand::Uniform2f(uniform.id(), x, y)))
            .unwrap()
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform2fv(&self,
                  _cx: *mut JSContext,
                  uniform: Option<&WebGLUniformLocation>,
                  data: Option<*mut JSObject>) {
        let data = match data {
            Some(data) => data,
            None => return self.webgl_error(InvalidValue),
        };

        if let Some(data) = array_buffer_view_to_vec_checked::<f32>(data) {
            if data.len() < 2 {
                return self.webgl_error(InvalidOperation);
            }

            self.Uniform2f(uniform, data[0], data[1]);
        } else {
            self.webgl_error(InvalidValue);
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform4i(&self,
                  uniform: Option<&WebGLUniformLocation>,
                  x: i32, y: i32, z: i32, w: i32) {
        let uniform = match uniform {
            Some(uniform) => uniform,
            None => return,
        };

        match self.current_program.get() {
            Some(ref program) if program.id() == uniform.program_id() => {},
            _ => return self.webgl_error(InvalidOperation),
        };

        self.ipc_renderer
            .send(CanvasMsg::WebGL(WebGLCommand::Uniform4i(uniform.id(), x, y, z, w)))
            .unwrap()
    }


    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform4iv(&self,
                  _cx: *mut JSContext,
                  uniform: Option<&WebGLUniformLocation>,
                  data: Option<*mut JSObject>) {
        let data = match data {
            Some(data) => data,
            None => return self.webgl_error(InvalidValue),
        };

        if let Some(data) = array_buffer_view_to_vec_checked::<i32>(data) {
            if data.len() < 4 {
                return self.webgl_error(InvalidOperation);
            }

            self.Uniform4i(uniform, data[0], data[1], data[2], data[3]);
        } else {
            self.webgl_error(InvalidValue);
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform4f(&self,
                  uniform: Option<&WebGLUniformLocation>,
                  x: f32, y: f32, z: f32, w: f32) {
        let uniform = match uniform {
            Some(uniform) => uniform,
            None => return,
        };

        match self.current_program.get() {
            Some(ref program) if program.id() == uniform.program_id() => {},
            _ => return self.webgl_error(InvalidOperation),
        };

        self.ipc_renderer
            .send(CanvasMsg::WebGL(WebGLCommand::Uniform4f(uniform.id(), x, y, z, w)))
            .unwrap()
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform4fv(&self,
                  _cx: *mut JSContext,
                  uniform: Option<&WebGLUniformLocation>,
                  data: Option<*mut JSObject>) {
        let data = match data {
            Some(data) => data,
            None => return self.webgl_error(InvalidValue),
        };

        if let Some(data) = array_buffer_view_to_vec_checked::<f32>(data) {
            if data.len() < 4 {
                return self.webgl_error(InvalidOperation);
            }

            self.Uniform4f(uniform, data[0], data[1], data[2], data[3]);
        } else {
            self.webgl_error(InvalidValue);
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn UseProgram(&self, program: Option<&WebGLProgram>) {
        if let Some(program) = program {
            match program.use_program() {
                Ok(()) => self.current_program.set(Some(program)),
                Err(e) => self.webgl_error(e),
            }
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn VertexAttrib1f(&self, indx: u32, x: f32) {
        self.vertex_attrib(indx, x, 0f32, 0f32, 1f32)
    }

    #[allow(unsafe_code)]
    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn VertexAttrib1fv(&self, _cx: *mut JSContext, indx: u32, data: *mut JSObject) {
        if let Some(data_vec) = array_buffer_view_to_vec_checked::<f32>(data) {
            if data_vec.len() < 4 {
                return self.webgl_error(InvalidOperation);
            }
            self.vertex_attrib(indx, data_vec[0], 0f32, 0f32, 1f32)
        } else {
            self.webgl_error(InvalidValue);
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn VertexAttrib2f(&self, indx: u32, x: f32, y: f32) {
        self.vertex_attrib(indx, x, y, 0f32, 1f32)
    }

    #[allow(unsafe_code)]
    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn VertexAttrib2fv(&self, _cx: *mut JSContext, indx: u32, data: *mut JSObject) {
        if let Some(data_vec) = array_buffer_view_to_vec_checked::<f32>(data) {
            if data_vec.len() < 2 {
                return self.webgl_error(InvalidOperation);
            }
            self.vertex_attrib(indx, data_vec[0], data_vec[1], 0f32, 1f32)
        } else {
            self.webgl_error(InvalidValue);
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn VertexAttrib3f(&self, indx: u32, x: f32, y: f32, z: f32) {
        self.vertex_attrib(indx, x, y, z, 1f32)
    }

    #[allow(unsafe_code)]
    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn VertexAttrib3fv(&self, _cx: *mut JSContext, indx: u32, data: *mut JSObject) {
        if let Some(data_vec) = array_buffer_view_to_vec_checked::<f32>(data) {
            if data_vec.len() < 3 {
                return self.webgl_error(InvalidOperation);
            }
            self.vertex_attrib(indx, data_vec[0], data_vec[1], data_vec[2], 1f32)
        } else {
            self.webgl_error(InvalidValue);
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn VertexAttrib4f(&self, indx: u32, x: f32, y: f32, z: f32, w: f32) {
        self.vertex_attrib(indx, x, y, z, w)
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn VertexAttrib4fv(&self, _cx: *mut JSContext, indx: u32, data: *mut JSObject) {
        if let Some(data_vec) = array_buffer_view_to_vec_checked::<f32>(data) {
            if data_vec.len() < 4 {
                return self.webgl_error(InvalidOperation);
            }
            self.vertex_attrib(indx, data_vec[0], data_vec[1], data_vec[2], data_vec[3])
        } else {
            self.webgl_error(InvalidValue);
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn VertexAttribPointer(&self, attrib_id: u32, size: i32, data_type: u32,
                           normalized: bool, stride: i32, offset: i64) {
        if let constants::FLOAT = data_type {
           let msg = CanvasMsg::WebGL(
               WebGLCommand::VertexAttribPointer2f(attrib_id, size, normalized, stride, offset as u32));
            self.ipc_renderer.send(msg).unwrap()
        } else {
            panic!("VertexAttribPointer: Data Type not supported")
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.4
    fn Viewport(&self, x: i32, y: i32, width: i32, height: i32) {
        self.ipc_renderer
            .send(CanvasMsg::WebGL(WebGLCommand::Viewport(x, y, width, height)))
            .unwrap()
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn TexImage2D(&self,
                  target: u32,
                  level: i32,
                  internal_format: u32,
                  format: u32,
                  data_type: u32,
                  source: Option<ImageDataOrHTMLImageElementOrHTMLCanvasElementOrHTMLVideoElement>) {
        let texture = match target {
            constants::TEXTURE_2D => self.bound_texture_2d.get(),
            constants::TEXTURE_CUBE_MAP => self.bound_texture_cube_map.get(),
            _ => return self.webgl_error(InvalidEnum),
        };
        if texture.is_none() {
            return self.webgl_error(InvalidOperation);
        }
        // TODO(emilio): Validate more parameters
        let source = match source {
            Some(s) => s,
            None => return,
        };

        let (pixels, size) = match source {
            ImageDataOrHTMLImageElementOrHTMLCanvasElementOrHTMLVideoElement::ImageData(image_data) => {
                let global = self.global();
                (image_data.get_data_array(&global.r()), image_data.get_size())
            },
            ImageDataOrHTMLImageElementOrHTMLCanvasElementOrHTMLVideoElement::HTMLImageElement(image) => {
                let img_url = match image.get_url() {
                    Some(url) => url,
                    None => return,
                };

                let window = window_from_node(&*self.canvas);

                let img = match canvas_utils::request_image_from_cache(window.r(), img_url) {
                    ImageResponse::Loaded(img) => img,
                    ImageResponse::PlaceholderLoaded(_) | ImageResponse::None |
                    ImageResponse::MetadataLoaded(_)
                        => return,
                };

                let size = Size2D::new(img.width as i32, img.height as i32);
                // TODO(emilio): Validate that the format argument is coherent with the image.
                // RGB8 should be easy to support too
                let mut data = match img.format {
                    PixelFormat::RGBA8 => img.bytes.to_vec(),
                    _ => unimplemented!(),
                };

                byte_swap(&mut data);

                (data, size)
            },
            // TODO(emilio): Getting canvas data is implemented in CanvasRenderingContext2D, but
            // we need to refactor it moving it to `HTMLCanvasElement` and supporting WebGLContext
            ImageDataOrHTMLImageElementOrHTMLCanvasElementOrHTMLVideoElement::HTMLCanvasElement(canvas) => {
                let canvas = canvas.r();
                if let Some((mut data, size)) = canvas.fetch_all_data() {
                    byte_swap(&mut data);
                    (data, size)
                } else {
                    return
                }
            },
            ImageDataOrHTMLImageElementOrHTMLCanvasElementOrHTMLVideoElement::HTMLVideoElement(_rooted_video)
                => unimplemented!(),
        };

        if size.width < 0 || size.height < 0 || level < 0 {
            self.webgl_error(WebGLError::InvalidOperation);
        }

        // TODO(emilio): Invert axis, convert colorspace, premultiply alpha if requested
        let msg = WebGLCommand::TexImage2D(target, level, internal_format as i32,
                                             size.width, size.height,
                                             format, data_type, pixels);

        // depth is always 1 when coming from html elements
        handle_potential_webgl_error!(self, texture.unwrap().initialize(size.width as u32,
                                                                        size.height as u32,
                                                                        1,
                                                                        internal_format,
                                                                        level as u32));

        self.ipc_renderer
            .send(CanvasMsg::WebGL(msg))
            .unwrap()
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn TexParameterf(&self, target: u32, name: u32, value: f32) {
        self.tex_parameter(target, name, TexParameterValue::Float(value))
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn TexParameteri(&self, target: u32, name: u32, value: i32) {
        self.tex_parameter(target, name, TexParameterValue::Int(value))
    }
}

pub trait LayoutCanvasWebGLRenderingContextHelpers {
    #[allow(unsafe_code)]
    unsafe fn get_ipc_renderer(&self) -> IpcSender<CanvasMsg>;
}

impl LayoutCanvasWebGLRenderingContextHelpers for LayoutJS<WebGLRenderingContext> {
    #[allow(unsafe_code)]
    unsafe fn get_ipc_renderer(&self) -> IpcSender<CanvasMsg> {
        (*self.unsafe_get()).ipc_renderer.clone()
    }
}
