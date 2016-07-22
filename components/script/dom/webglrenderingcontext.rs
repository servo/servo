/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use canvas_traits::{CanvasCommonMsg, CanvasMsg, byte_swap};
use dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLRenderingContextConstants as constants;
use dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLRenderingContextMethods;
use dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::{self, WebGLContextAttributes};
use dom::bindings::codegen::UnionTypes::ImageDataOrHTMLImageElementOrHTMLCanvasElementOrHTMLVideoElement;
use dom::bindings::conversions::{ToJSValConvertible, array_buffer_view_data, array_buffer_view_data_checked};
use dom::bindings::conversions::{array_buffer_view_to_vec_checked, array_buffer_view_to_vec};
use dom::bindings::global::GlobalRef;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, LayoutJS, MutNullableHeap, Root};
use dom::bindings::reflector::{Reflectable, Reflector, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::event::{Event, EventBubbles, EventCancelable};
use dom::htmlcanvaselement::HTMLCanvasElement;
use dom::htmlcanvaselement::utils as canvas_utils;
use dom::node::{Node, NodeDamage, window_from_node};
use dom::webgl_validations::WebGLValidator;
use dom::webgl_validations::tex_image_2d::{CommonTexImage2DValidator, CommonTexImage2DValidatorResult};
use dom::webgl_validations::tex_image_2d::{TexImage2DValidator, TexImage2DValidatorResult};
use dom::webgl_validations::types::{TexFormat, TexImageTarget, TexDataType};
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
use js::jsapi::{JSContext, JS_GetArrayBufferViewType, JSObject, Type};
use js::jsval::{BooleanValue, DoubleValue, Int32Value, JSVal, NullValue, UndefinedValue};
use net_traits::image::base::PixelFormat;
use net_traits::image_cache_thread::ImageResponse;
use offscreen_gl_context::{GLContextAttributes, GLLimits};
use script_traits::ScriptMsg as ConstellationMsg;
use std::cell::Cell;
use webrender_traits::WebGLError::*;
use webrender_traits::{WebGLCommand, WebGLError, WebGLFramebufferBindingRequest, WebGLParameter};

type ImagePixelResult = Result<(Vec<u8>, Size2D<i32>), ()>;
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
    #[ignore_heap_size_of = "Defined in offscreen_gl_context"]
    limits: GLLimits,
    canvas: JS<HTMLCanvasElement>,
    #[ignore_heap_size_of = "Defined in webrender_traits"]
    last_error: Cell<Option<WebGLError>>,
    texture_unpacking_settings: Cell<TextureUnpacking>,
    bound_texture_2d: MutNullableHeap<JS<WebGLTexture>>,
    bound_texture_cube_map: MutNullableHeap<JS<WebGLTexture>>,
    bound_buffer_array: MutNullableHeap<JS<WebGLBuffer>>,
    bound_buffer_element_array: MutNullableHeap<JS<WebGLBuffer>>,
    current_program: MutNullableHeap<JS<WebGLProgram>>,
    #[ignore_heap_size_of = "Because it's small"]
    current_vertex_attrib_0: Cell<(f32, f32, f32, f32)>,
}

impl WebGLRenderingContext {
    fn new_inherited(global: GlobalRef,
                     canvas: &HTMLCanvasElement,
                     size: Size2D<i32>,
                     attrs: GLContextAttributes)
                     -> Result<WebGLRenderingContext, String> {
        let (sender, receiver) = ipc::channel().unwrap();
        let constellation_chan = global.constellation_chan();
        constellation_chan.send(ConstellationMsg::CreateWebGLPaintThread(size, attrs, sender))
                          .unwrap();
        let result = receiver.recv().unwrap();

        result.map(|(ipc_renderer, context_limits)| {
            WebGLRenderingContext {
                reflector_: Reflector::new(),
                ipc_renderer: ipc_renderer,
                limits: context_limits,
                canvas: JS::from_ref(canvas),
                last_error: Cell::new(None),
                texture_unpacking_settings: Cell::new(CONVERT_COLORSPACE),
                bound_texture_2d: MutNullableHeap::new(None),
                bound_texture_cube_map: MutNullableHeap::new(None),
                bound_buffer_array: MutNullableHeap::new(None),
                bound_buffer_element_array: MutNullableHeap::new(None),
                current_program: MutNullableHeap::new(None),
                current_vertex_attrib_0: Cell::new((0f32, 0f32, 0f32, 1f32)),
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

    pub fn limits(&self) -> &GLLimits {
        &self.limits
    }

    pub fn bound_texture_for_target(&self, target: &TexImageTarget) -> Option<Root<WebGLTexture>> {
        match *target {
            TexImageTarget::Texture2D => self.bound_texture_2d.get(),
            TexImageTarget::CubeMapPositiveX |
            TexImageTarget::CubeMapNegativeX |
            TexImageTarget::CubeMapPositiveY |
            TexImageTarget::CubeMapNegativeY |
            TexImageTarget::CubeMapPositiveZ |
            TexImageTarget::CubeMapNegativeZ => self.bound_texture_cube_map.get(),
        }
    }

    pub fn recreate(&self, size: Size2D<i32>) {
        self.ipc_renderer.send(CanvasMsg::Common(CanvasCommonMsg::Recreate(size))).unwrap();
    }

    pub fn ipc_renderer(&self) -> IpcSender<CanvasMsg> {
        self.ipc_renderer.clone()
    }

    pub fn webgl_error(&self, err: WebGLError) {
        // TODO(emilio): Add useful debug messages to this
        warn!("WebGL error: {:?}, previous error was {:?}", err, self.last_error.get());

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
            self.webgl_error(InvalidOperation)
        }
    }

    fn mark_as_dirty(&self) {
        self.canvas.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
    }

    fn vertex_attrib(&self, indx: u32, x: f32, y: f32, z: f32, w: f32) {
        if indx > self.limits.max_vertex_attribs {
            return self.webgl_error(InvalidValue);
        }

        if indx == 0 {
            self.current_vertex_attrib_0.set((x, y, z, w))
        }

        self.ipc_renderer
            .send(CanvasMsg::WebGL(WebGLCommand::VertexAttrib(indx, x, y, z, w)))
            .unwrap();
    }

    fn validate_stencil_actions(&self, action: u32) -> bool {
        match action {
            0 | constants::KEEP | constants::REPLACE | constants::INCR | constants::DECR |
            constants::INVERT | constants::INCR_WRAP | constants::DECR_WRAP => true,
            _ => false,
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    // https://www.khronos.org/opengles/sdk/docs/man/xhtml/glUniform.xml
    // https://www.khronos.org/registry/gles/specs/2.0/es_full_spec_2.0.25.pdf#nameddest=section-2.10.4
    fn validate_uniform_parameters<T>(&self,
                                      uniform: Option<&WebGLUniformLocation>,
                                      uniform_type: UniformSetterType,
                                      data: Option<&[T]>) -> bool {
        let uniform = match uniform {
            Some(uniform) => uniform,
            None => return false,
        };

        let program = self.current_program.get();
        match program {
            Some(ref program) if program.id() == uniform.program_id() => {},
            _ => {
                self.webgl_error(InvalidOperation);
                return false;
            },
        };

        let data = match data {
            Some(data) => data,
            None => {
                self.webgl_error(InvalidOperation);
                return false;
            },
        };

        // TODO(emilio): Get more complex uniform info from ANGLE, and use it to
        // properly validate that the uniform setter type is compatible with the
        // uniform type, and that the uniform size matches.
        if data.len() % uniform_type.element_count() != 0 {
            self.webgl_error(InvalidOperation);
            return false;
        }

        true
    }

    fn get_image_pixels(&self,
                        source: Option<ImageDataOrHTMLImageElementOrHTMLCanvasElementOrHTMLVideoElement>)
                        -> ImagePixelResult {
        let source = match source {
            Some(s) => s,
            None => return Err(()),
        };

        // NOTE: Getting the pixels probably can be short-circuited if some
        // parameter is invalid.
        //
        // Nontheless, since it's the error case, I'm not totally sure the
        // complexity is worth it.
        let (pixels, size) = match source {
            ImageDataOrHTMLImageElementOrHTMLCanvasElementOrHTMLVideoElement::ImageData(image_data) => {
                let global = self.global();
                (image_data.get_data_array(&global.r()), image_data.get_size())
            },
            ImageDataOrHTMLImageElementOrHTMLCanvasElementOrHTMLVideoElement::HTMLImageElement(image) => {
                let img_url = match image.get_url() {
                    Some(url) => url,
                    None => return Err(()),
                };

                let window = window_from_node(&*self.canvas);

                let img = match canvas_utils::request_image_from_cache(window.r(), img_url) {
                    ImageResponse::Loaded(img) => img,
                    ImageResponse::PlaceholderLoaded(_) | ImageResponse::None |
                    ImageResponse::MetadataLoaded(_)
                        => return Err(()),
                };

                let size = Size2D::new(img.width as i32, img.height as i32);

                // TODO(emilio): Validate that the format argument
                // is coherent with the image.
                //
                // RGB8 should be easy to support too
                let mut data = match img.format {
                    PixelFormat::RGBA8 => img.bytes.to_vec(),
                    _ => unimplemented!(),
                };

                byte_swap(&mut data);

                (data, size)
            },
            // TODO(emilio): Getting canvas data is implemented in CanvasRenderingContext2D,
            // but we need to refactor it moving it to `HTMLCanvasElement` and support
            // WebGLContext (probably via GetPixels()).
            ImageDataOrHTMLImageElementOrHTMLCanvasElementOrHTMLVideoElement::HTMLCanvasElement(canvas) => {
                let canvas = canvas.r();
                if let Some((mut data, size)) = canvas.fetch_all_data() {
                    byte_swap(&mut data);
                    (data, size)
                } else {
                    return Err(());
                }
            },
            ImageDataOrHTMLImageElementOrHTMLCanvasElementOrHTMLVideoElement::HTMLVideoElement(_rooted_video)
                => unimplemented!(),
        };

        return Ok((pixels, size));
    }

    // TODO(emilio): Move this logic to a validator.
    #[allow(unsafe_code)]
    fn validate_tex_image_2d_data(&self,
                                  width: u32,
                                  height: u32,
                                  format: TexFormat,
                                  data_type: TexDataType,
                                  data: Option<*mut JSObject>)
                                  -> Result<u32, ()> {
        let element_size = data_type.element_size();
        let components_per_element = data_type.components_per_element();
        let components = format.components();

        // If data is non-null, the type of pixels must match the type of the
        // data to be read.
        // If it is UNSIGNED_BYTE, a Uint8Array must be supplied;
        // if it is UNSIGNED_SHORT_5_6_5, UNSIGNED_SHORT_4_4_4_4,
        // or UNSIGNED_SHORT_5_5_5_1, a Uint16Array must be supplied.
        // If the types do not match, an INVALID_OPERATION error is generated.
        let received_size = if let Some(data) = data {
            if unsafe { array_buffer_view_data_checked::<u16>(data).is_some() } {
                2
            } else if unsafe { array_buffer_view_data_checked::<u8>(data).is_some() } {
                1
            } else {
                self.webgl_error(InvalidOperation);
                return Err(());
            }
        } else {
            element_size
        };

        if received_size != element_size {
            self.webgl_error(InvalidOperation);
            return Err(());
        }

        // NOTE: width and height are positive or zero due to validate()
        let expected_byte_length = width * height * element_size * components / components_per_element;
        return Ok(expected_byte_length);
    }

    fn tex_image_2d(&self,
                    texture: Root<WebGLTexture>,
                    target: TexImageTarget,
                    data_type: TexDataType,
                    internal_format: TexFormat,
                    level: u32,
                    width: u32,
                    height: u32,
                    _border: u32,
                    pixels: Vec<u8>) { // NB: pixels should NOT be premultipied
        if internal_format == TexFormat::RGBA &&
           data_type == TexDataType::UnsignedByte &&
           self.texture_unpacking_settings.get().contains(PREMULTIPLY_ALPHA) {
            // TODO(emilio): premultiply here.
        }

        // TODO(emilio): Flip Y axis if necessary here

        // TexImage2D depth is always equal to 1
        handle_potential_webgl_error!(self, texture.initialize(target,
                                                               width,
                                                               height, 1,
                                                               internal_format,
                                                               level,
                                                               Some(data_type)));

        // TODO(emilio): Invert axis, convert colorspace, premultiply alpha if requested
        let msg = WebGLCommand::TexImage2D(target.as_gl_constant(), level as i32,
                                           internal_format.as_gl_constant() as i32,
                                           width as i32, height as i32,
                                           internal_format.as_gl_constant(),
                                           data_type.as_gl_constant(), pixels);

        self.ipc_renderer
            .send(CanvasMsg::WebGL(msg))
            .unwrap()
    }

    fn tex_sub_image_2d(&self,
                        texture: Root<WebGLTexture>,
                        target: TexImageTarget,
                        level: u32,
                        xoffset: i32,
                        yoffset: i32,
                        width: u32,
                        height: u32,
                        format: TexFormat,
                        data_type: TexDataType,
                        pixels: Vec<u8>) {  // NB: pixels should NOT be premultipied
        // We have already validated level
        let image_info = texture.image_info_for_target(&target, level);

        // GL_INVALID_VALUE is generated if:
        //   - xoffset or yoffset is less than 0
        //   - x offset plus the width is greater than the texture width
        //   - y offset plus the height is greater than the texture height
        if xoffset < 0 || (xoffset as u32 + width) > image_info.width() ||
            yoffset < 0 || (yoffset as u32 + height) > image_info.height() {
            return self.webgl_error(InvalidValue);
        }

        // NB: format and internal_format must match.
        if format != image_info.internal_format().unwrap() ||
            data_type != image_info.data_type().unwrap() {
            return self.webgl_error(InvalidOperation);
        }

        // TODO(emilio): Flip Y axis if necessary here

        // TODO(emilio): Invert axis, convert colorspace, premultiply alpha if requested
        let msg = WebGLCommand::TexSubImage2D(target.as_gl_constant(),
                                              level as i32, xoffset, yoffset,
                                              width as i32, height as i32,
                                              format.as_gl_constant(),
                                              data_type.as_gl_constant(), pixels);

        self.ipc_renderer
            .send(CanvasMsg::WebGL(msg))
            .unwrap()
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
            WebGLParameter::FloatArray(_) => panic!("Buffer parameter should not be float array"),
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
            WebGLParameter::FloatArray(_) => panic!("Parameter should not be float array"),
            WebGLParameter::String(val) => {
                rooted!(in(cx) let mut rval = UndefinedValue());
                unsafe {
                    val.to_jsval(cx, rval.handle_mut());
                }
                rval.get()
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
            slot.set(None);
            // Unbind the current buffer
            self.ipc_renderer
                .send(CanvasMsg::WebGL(WebGLCommand::BindBuffer(target, None)))
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
                .send(CanvasMsg::WebGL(WebGLCommand::BindRenderbuffer(target, None)))
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
                .send(CanvasMsg::WebGL(WebGLCommand::BindTexture(target, None)))
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

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn CopyTexImage2D(&self, target: u32, level: i32, internal_format: u32,
                      x: i32, y: i32, width: i32, height: i32, border: i32) {
        let validator = CommonTexImage2DValidator::new(self, target, level,
                                                       internal_format, width,
                                                       height, border);
        let CommonTexImage2DValidatorResult {
            texture,
            target,
            level,
            internal_format,
            width,
            height,
            border,
        } = match validator.validate() {
            Ok(result) => result,
            Err(_) => return,
        };

        let image_info = texture.image_info_for_target(&target, level);

        // The color buffer components can be dropped during the conversion to
        // the internal_format, but new components cannot be added.
        //
        // Note that this only applies if we're copying to an already
        // initialized texture.
        //
        // GL_INVALID_OPERATION is generated if the color buffer cannot be
        // converted to the internal_format.
        if let Some(old_internal_format) = image_info.internal_format() {
            if old_internal_format.components() > internal_format.components() {
                return self.webgl_error(InvalidOperation);
            }
        }

        // NB: TexImage2D depth is always equal to 1
        handle_potential_webgl_error!(self, texture.initialize(target,
                                                               width as u32,
                                                               height as u32, 1,
                                                               internal_format,
                                                               level as u32,
                                                               None));

        let msg = WebGLCommand::CopyTexImage2D(target.as_gl_constant(),
                                               level as i32,
                                               internal_format.as_gl_constant(),
                                               x, y,
                                               width as i32, height as i32,
                                               border as i32);

        self.ipc_renderer.send(CanvasMsg::WebGL(msg)).unwrap()
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn CopyTexSubImage2D(&self, target: u32, level: i32, xoffset: i32, yoffset: i32,
                         x: i32, y: i32, width: i32, height: i32) {
        // NB: We use a dummy (valid) format and border in order to reuse the
        // common validations, but this should have its own validator.
        let validator = CommonTexImage2DValidator::new(self, target, level,
                                                       TexFormat::RGBA.as_gl_constant(),
                                                       width, height, 0);
        let CommonTexImage2DValidatorResult {
            texture,
            target,
            level,
            width,
            height,
            ..
        } = match validator.validate() {
            Ok(result) => result,
            Err(_) => return,
        };

        let image_info = texture.image_info_for_target(&target, level);

        // GL_INVALID_VALUE is generated if:
        //   - xoffset or yoffset is less than 0
        //   - x offset plus the width is greater than the texture width
        //   - y offset plus the height is greater than the texture height
        if xoffset < 0 || (xoffset as u32 + width) > image_info.width() ||
            yoffset < 0 || (yoffset as u32 + height) > image_info.height() {
                self.webgl_error(InvalidValue);
                return;
        }

        let msg = WebGLCommand::CopyTexSubImage2D(target.as_gl_constant(),
                                                  level as i32, xoffset, yoffset,
                                                  x, y,
                                                  width as i32, height as i32);

        self.ipc_renderer.send(CanvasMsg::WebGL(msg)).unwrap();
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
        match shader_type {
            constants::VERTEX_SHADER | constants::FRAGMENT_SHADER => {},
            _ => {
                self.webgl_error(InvalidEnum);
                return None;
            }
        }
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
                    return self.webgl_error(InvalidValue);
                }

                self.ipc_renderer
                    .send(CanvasMsg::WebGL(WebGLCommand::DrawArrays(mode, first, count)))
                    .unwrap();
                self.mark_as_dirty();
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
        if attrib_id > self.limits.max_vertex_attribs {
            return self.webgl_error(InvalidValue);
        }

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
    fn GetActiveAttrib(&self, program: Option<&WebGLProgram>, index: u32) -> Option<Root<WebGLActiveInfo>> {
        program.and_then(|p| match p.get_active_attrib(index) {
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
                WebGLParameter::FloatArray(_) => {
                    panic!("Program paramenter should not be float array")
                }
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
                WebGLParameter::FloatArray(_) => {
                    panic!("Shader paramenter should not be float array")
                }
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
        program.and_then(|p| {
            handle_potential_webgl_error!(self, p.get_uniform_location(name), None)
                .map(|location| WebGLUniformLocation::new(self.global().r(), location, p.id()))
        })
    }

    #[allow(unsafe_code)]
    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn GetVertexAttrib(&self, cx: *mut JSContext, index: u32, pname: u32) -> JSVal {
        if index == 0 && pname == constants::CURRENT_VERTEX_ATTRIB {
            rooted!(in(cx) let mut result = UndefinedValue());
            let (x, y, z, w) = self.current_vertex_attrib_0.get();
            let attrib = vec![x, y, z, w];
            unsafe {
                attrib.to_jsval(cx, result.handle_mut());
            }
            return result.get()
        }

        let (sender, receiver) = ipc::channel().unwrap();
        self.ipc_renderer.send(CanvasMsg::WebGL(WebGLCommand::GetVertexAttrib(index, pname, sender))).unwrap();

        match handle_potential_webgl_error!(self, receiver.recv().unwrap(), WebGLParameter::Invalid) {
            WebGLParameter::Int(val) => Int32Value(val),
            WebGLParameter::Bool(val) => BooleanValue(val),
            WebGLParameter::String(_) => panic!("Vertex attrib should not be string"),
            WebGLParameter::Float(_) => panic!("Vertex attrib should not be float"),
            WebGLParameter::FloatArray(val) => {
                rooted!(in(cx) let mut result = UndefinedValue());
                unsafe {
                    val.to_jsval(cx, result.handle_mut());
                }
                result.get()
            }
            WebGLParameter::Invalid => NullValue(),
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

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.5
    fn IsBuffer(&self, buffer: Option<&WebGLBuffer>) -> bool {
        buffer.map_or(false, |buf| buf.target().is_some() && !buf.is_deleted())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.6
    fn IsFramebuffer(&self, frame_buffer: Option<&WebGLFramebuffer>) -> bool {
        frame_buffer.map_or(false, |buf| buf.target().is_some() && !buf.is_deleted())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.7
    fn IsRenderbuffer(&self, render_buffer: Option<&WebGLRenderbuffer>) -> bool {
        render_buffer.map_or(false, |buf| buf.ever_bound() && !buf.is_deleted())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn IsShader(&self, shader: Option<&WebGLShader>) -> bool {
        shader.map_or(false, |s| !s.is_deleted() || s.is_attached())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn IsTexture(&self, texture: Option<&WebGLTexture>) -> bool {
        texture.map_or(false, |tex| tex.target().is_some() && !tex.is_deleted())
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

    #[allow(unsafe_code)]
    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.12
    fn ReadPixels(&self, _cx: *mut JSContext, x: i32, y: i32, width: i32, height: i32,
                  format: u32, pixel_type: u32, pixels: *mut JSObject) {
        let mut data = match unsafe { array_buffer_view_data::<u8>(pixels) } {
            Some(data) => data,
            None => return self.webgl_error(InvalidValue),
        };

        match unsafe { JS_GetArrayBufferViewType(pixels) } {
            Type::Uint8 => (),
            _ => return self.webgl_error(InvalidOperation)
        }

        let (sender, receiver) = ipc::channel().unwrap();
        self.ipc_renderer
            .send(CanvasMsg::WebGL(WebGLCommand::ReadPixels(x, y, width, height, format, pixel_type, sender)))
            .unwrap();

        let result = receiver.recv().unwrap();

        if result.len() > data.len() {
            return self.webgl_error(InvalidOperation)
        }

        for i in 0..result.len() {
            data[i] = result[i]
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn SampleCoverage(&self, value: f32, invert: bool) {
        self.ipc_renderer.send(CanvasMsg::WebGL(WebGLCommand::SampleCoverage(value, invert))).unwrap();
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.4
    fn Scissor(&self, x: i32, y: i32, width: i32, height: i32) {
        self.ipc_renderer
            .send(CanvasMsg::WebGL(WebGLCommand::Scissor(x, y, width, height)))
            .unwrap()
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn StencilFunc(&self, func: u32, ref_: i32, mask: u32) {
        match func {
            constants::NEVER | constants::LESS | constants::EQUAL | constants::LEQUAL |
            constants::GREATER | constants::NOTEQUAL | constants::GEQUAL | constants::ALWAYS =>
                self.ipc_renderer
                    .send(CanvasMsg::WebGL(WebGLCommand::StencilFunc(func, ref_, mask)))
                    .unwrap(),
            _ => self.webgl_error(InvalidEnum),
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn StencilFuncSeparate(&self, face: u32, func: u32, ref_: i32, mask: u32) {
        match face {
            constants::FRONT | constants::BACK | constants::FRONT_AND_BACK => (),
            _ => return self.webgl_error(InvalidEnum),
        }

        match func {
            constants::NEVER | constants::LESS | constants::EQUAL | constants::LEQUAL |
            constants::GREATER | constants::NOTEQUAL | constants::GEQUAL | constants::ALWAYS =>
                self.ipc_renderer
                    .send(CanvasMsg::WebGL(WebGLCommand::StencilFuncSeparate(face, func, ref_, mask)))
                    .unwrap(),
            _ => self.webgl_error(InvalidEnum),
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn StencilMask(&self, mask: u32) {
        self.ipc_renderer
            .send(CanvasMsg::WebGL(WebGLCommand::StencilMask(mask)))
            .unwrap()
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn StencilMaskSeparate(&self, face: u32, mask: u32) {
        match face {
            constants::FRONT | constants::BACK | constants::FRONT_AND_BACK =>
                self.ipc_renderer
                    .send(CanvasMsg::WebGL(WebGLCommand::StencilMaskSeparate(face, mask)))
                    .unwrap(),
            _ => return self.webgl_error(InvalidEnum),
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn StencilOp(&self, fail: u32, zfail: u32, zpass: u32) {
        if self.validate_stencil_actions(fail) && self.validate_stencil_actions(zfail) &&
           self.validate_stencil_actions(zpass) {
                self.ipc_renderer
                    .send(CanvasMsg::WebGL(WebGLCommand::StencilOp(fail, zfail, zpass)))
                    .unwrap()
        } else {
            self.webgl_error(InvalidEnum)
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn StencilOpSeparate(&self, face: u32, fail: u32, zfail: u32, zpass: u32) {
        match face {
            constants::FRONT | constants::BACK | constants::FRONT_AND_BACK => (),
            _ => return self.webgl_error(InvalidEnum),
        }

        if self.validate_stencil_actions(fail) && self.validate_stencil_actions(zfail) &&
           self.validate_stencil_actions(zpass) {
                self.ipc_renderer
                    .send(CanvasMsg::WebGL(WebGLCommand::StencilOpSeparate(face, fail, zfail, zpass)))
                    .unwrap()
        } else {
            self.webgl_error(InvalidEnum)
        }
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
        shader.and_then(|s| s.source())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform1f(&self,
                  uniform: Option<&WebGLUniformLocation>,
                  val: f32) {
        if self.validate_uniform_parameters(uniform, UniformSetterType::Float, Some(&[val])) {
            self.ipc_renderer
                .send(CanvasMsg::WebGL(WebGLCommand::Uniform1f(uniform.unwrap().id(), val)))
                .unwrap()
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform1i(&self,
                  uniform: Option<&WebGLUniformLocation>,
                  val: i32) {
        if self.validate_uniform_parameters(uniform, UniformSetterType::Int, Some(&[val])) {
            self.ipc_renderer
                .send(CanvasMsg::WebGL(WebGLCommand::Uniform1i(uniform.unwrap().id(), val)))
                .unwrap()
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform1iv(&self,
                  _cx: *mut JSContext,
                  uniform: Option<&WebGLUniformLocation>,
                  data: Option<*mut JSObject>) {
        let data_vec = data.and_then(|d| array_buffer_view_to_vec::<i32>(d));
        if self.validate_uniform_parameters(uniform, UniformSetterType::Int, data_vec.as_ref().map(Vec::as_slice)) {
            self.ipc_renderer
                .send(CanvasMsg::WebGL(WebGLCommand::Uniform1iv(uniform.unwrap().id(), data_vec.unwrap())))
                .unwrap()
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform1fv(&self,
                  _cx: *mut JSContext,
                  uniform: Option<&WebGLUniformLocation>,
                  data: Option<*mut JSObject>) {
        let data_vec = data.and_then(|d| array_buffer_view_to_vec::<f32>(d));
        if self.validate_uniform_parameters(uniform, UniformSetterType::Float, data_vec.as_ref().map(Vec::as_slice)) {
            self.ipc_renderer
                .send(CanvasMsg::WebGL(WebGLCommand::Uniform1fv(uniform.unwrap().id(), data_vec.unwrap())))
                .unwrap()
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform2f(&self,
                  uniform: Option<&WebGLUniformLocation>,
                  x: f32, y: f32) {
        if self.validate_uniform_parameters(uniform, UniformSetterType::FloatVec2, Some(&[x, y])) {
            self.ipc_renderer
                .send(CanvasMsg::WebGL(WebGLCommand::Uniform2f(uniform.unwrap().id(), x, y)))
                .unwrap()
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform2fv(&self,
                  _cx: *mut JSContext,
                  uniform: Option<&WebGLUniformLocation>,
                  data: Option<*mut JSObject>) {
        let data_vec = data.and_then(|d| array_buffer_view_to_vec::<f32>(d));
        if self.validate_uniform_parameters(uniform,
                                            UniformSetterType::FloatVec2,
                                            data_vec.as_ref().map(Vec::as_slice)) {
            self.ipc_renderer
                .send(CanvasMsg::WebGL(WebGLCommand::Uniform2fv(uniform.unwrap().id(), data_vec.unwrap())))
                .unwrap()
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform2i(&self,
                  uniform: Option<&WebGLUniformLocation>,
                  x: i32, y: i32) {
        if self.validate_uniform_parameters(uniform,
                                            UniformSetterType::IntVec2,
                                            Some(&[x, y])) {
            self.ipc_renderer
                .send(CanvasMsg::WebGL(WebGLCommand::Uniform2i(uniform.unwrap().id(), x, y)))
                .unwrap()
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform2iv(&self,
                  _cx: *mut JSContext,
                  uniform: Option<&WebGLUniformLocation>,
                  data: Option<*mut JSObject>) {
        let data_vec = data.and_then(|d| array_buffer_view_to_vec::<i32>(d));
        if self.validate_uniform_parameters(uniform,
                                            UniformSetterType::IntVec2,
                                            data_vec.as_ref().map(Vec::as_slice)) {
            self.ipc_renderer
                .send(CanvasMsg::WebGL(WebGLCommand::Uniform2iv(uniform.unwrap().id(), data_vec.unwrap())))
                .unwrap()
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform3f(&self,
                  uniform: Option<&WebGLUniformLocation>,
                  x: f32, y: f32, z: f32) {
        if self.validate_uniform_parameters(uniform,
                                            UniformSetterType::FloatVec3,
                                            Some(&[x, y, z])) {
            self.ipc_renderer
                .send(CanvasMsg::WebGL(WebGLCommand::Uniform3f(uniform.unwrap().id(), x, y, z)))
                .unwrap()
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform3fv(&self,
                  _cx: *mut JSContext,
                  uniform: Option<&WebGLUniformLocation>,
                  data: Option<*mut JSObject>) {
        let data_vec = data.and_then(|d| array_buffer_view_to_vec::<f32>(d));
        if self.validate_uniform_parameters(uniform,
                                            UniformSetterType::FloatVec3,
                                            data_vec.as_ref().map(Vec::as_slice)) {
            self.ipc_renderer
                .send(CanvasMsg::WebGL(WebGLCommand::Uniform3fv(uniform.unwrap().id(), data_vec.unwrap())))
                .unwrap()
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform3i(&self,
                  uniform: Option<&WebGLUniformLocation>,
                  x: i32, y: i32, z: i32) {
        if self.validate_uniform_parameters(uniform,
                                            UniformSetterType::IntVec3,
                                            Some(&[x, y, z])) {
            self.ipc_renderer
                .send(CanvasMsg::WebGL(WebGLCommand::Uniform3i(uniform.unwrap().id(), x, y, z)))
                .unwrap()
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform3iv(&self,
                  _cx: *mut JSContext,
                  uniform: Option<&WebGLUniformLocation>,
                  data: Option<*mut JSObject>) {
        let data_vec = data.and_then(|d| array_buffer_view_to_vec::<i32>(d));
        if self.validate_uniform_parameters(uniform,
                                            UniformSetterType::IntVec3,
                                            data_vec.as_ref().map(Vec::as_slice)) {
            self.ipc_renderer
                .send(CanvasMsg::WebGL(WebGLCommand::Uniform3iv(uniform.unwrap().id(), data_vec.unwrap())))
                .unwrap()
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform4i(&self,
                  uniform: Option<&WebGLUniformLocation>,
                  x: i32, y: i32, z: i32, w: i32) {
        if self.validate_uniform_parameters(uniform,
                                            UniformSetterType::IntVec4,
                                            Some(&[x, y, z, w])) {
            self.ipc_renderer
                .send(CanvasMsg::WebGL(WebGLCommand::Uniform4i(uniform.unwrap().id(), x, y, z, w)))
                .unwrap()
        }
    }


    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform4iv(&self,
                  _cx: *mut JSContext,
                  uniform: Option<&WebGLUniformLocation>,
                  data: Option<*mut JSObject>) {
        let data_vec = data.and_then(|d| array_buffer_view_to_vec::<i32>(d));
        if self.validate_uniform_parameters(uniform,
                                            UniformSetterType::IntVec4,
                                            data_vec.as_ref().map(Vec::as_slice)) {
            self.ipc_renderer
                .send(CanvasMsg::WebGL(WebGLCommand::Uniform4iv(uniform.unwrap().id(), data_vec.unwrap())))
                .unwrap()
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform4f(&self,
                  uniform: Option<&WebGLUniformLocation>,
                  x: f32, y: f32, z: f32, w: f32) {
        if self.validate_uniform_parameters(uniform,
                                            UniformSetterType::FloatVec4,
                                            Some(&[x, y, z, w])) {
            self.ipc_renderer
                .send(CanvasMsg::WebGL(WebGLCommand::Uniform4f(uniform.unwrap().id(), x, y, z, w)))
                .unwrap()
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform4fv(&self,
                  _cx: *mut JSContext,
                  uniform: Option<&WebGLUniformLocation>,
                  data: Option<*mut JSObject>) {
        let data_vec = data.and_then(|d| array_buffer_view_to_vec::<f32>(d));
        if self.validate_uniform_parameters(uniform,
                                            UniformSetterType::FloatVec4,
                                            data_vec.as_ref().map(Vec::as_slice)) {
            self.ipc_renderer
                .send(CanvasMsg::WebGL(WebGLCommand::Uniform4fv(uniform.unwrap().id(), data_vec.unwrap())))
                .unwrap()
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
        if attrib_id > self.limits.max_vertex_attribs {
            return self.webgl_error(InvalidValue);
        }

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
                  _cx: *mut JSContext,
                  target: u32,
                  level: i32,
                  internal_format: u32,
                  width: i32,
                  height: i32,
                  border: i32,
                  format: u32,
                  data_type: u32,
                  data: Option<*mut JSObject>) {
        let validator = TexImage2DValidator::new(self, target, level,
                                                 internal_format, width, height,
                                                 border, format, data_type);

        let TexImage2DValidatorResult {
            texture,
            target,
            width,
            height,
            level,
            border,
            format,
            data_type,
        } = match validator.validate() {
            Ok(result) => result,
            Err(_) => return, // NB: The validator sets the correct error for us.
        };

        let expected_byte_length = match self.validate_tex_image_2d_data(width,
                                                                         height,
                                                                         format,
                                                                         data_type,
                                                                         data) {
            Ok(byte_length) => byte_length,
            Err(_) => return,
        };

        // If data is null, a buffer of sufficient size
        // initialized to 0 is passed.
        let buff = if let Some(data) = data {
            array_buffer_view_to_vec::<u8>(data)
                .expect("Can't reach here without being an ArrayBufferView!")
        } else {
            vec![0u8; expected_byte_length as usize]
        };

        if buff.len() != expected_byte_length as usize {
            return self.webgl_error(InvalidOperation);
        }

        self.tex_image_2d(texture, target, data_type, format,
                          level, width, height, border, buff)
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn TexImage2D_(&self,
                   target: u32,
                   level: i32,
                   internal_format: u32,
                   format: u32,
                   data_type: u32,
                   source: Option<ImageDataOrHTMLImageElementOrHTMLCanvasElementOrHTMLVideoElement>) {
        // Get pixels from image source
        let (pixels, size) = match self.get_image_pixels(source) {
            Ok((pixels, size)) => (pixels, size),
            Err(_) => return,
        };


        let validator = TexImage2DValidator::new(self,
                                                 target, level, internal_format,
                                                 size.width, size.height,
                                                 0, format, data_type);

        let TexImage2DValidatorResult {
            texture,
            target,
            width,
            height,
            level,
            border,
            format,
            data_type,
        } = match validator.validate() {
            Ok(result) => result,
            Err(_) => return, // NB: The validator sets the correct error for us.
        };

        self.tex_image_2d(texture, target, data_type, format,
                          level, width, height, border, pixels);
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn TexSubImage2D(&self,
                     _cx: *mut JSContext,
                     target: u32,
                     level: i32,
                     xoffset: i32,
                     yoffset: i32,
                     width: i32,
                     height: i32,
                     format: u32,
                     data_type: u32,
                     data: Option<*mut JSObject>) {
        let validator = TexImage2DValidator::new(self, target, level,
                                                 format, width, height,
                                                 0, format, data_type);
        let TexImage2DValidatorResult {
            texture,
            target,
            width,
            height,
            level,
            format,
            data_type,
            ..
        } = match validator.validate() {
            Ok(result) => result,
            Err(_) => return, // NB: The validator sets the correct error for us.
        };

        let expected_byte_length = match self.validate_tex_image_2d_data(width,
                                                                         height,
                                                                         format,
                                                                         data_type,
                                                                         data) {
            Ok(byte_length) => byte_length,
            Err(()) => return,
        };

        // If data is null, a buffer of sufficient size
        // initialized to 0 is passed.
        let buff = if let Some(data) = data {
            array_buffer_view_to_vec::<u8>(data)
                .expect("Can't reach here without being an ArrayBufferView!")
        } else {
            vec![0u8; expected_byte_length as usize]
        };

        if expected_byte_length != 0 &&
            buff.len() != expected_byte_length as usize {
            return self.webgl_error(InvalidOperation);
        }

        self.tex_sub_image_2d(texture, target, level, xoffset, yoffset,
                              width, height, format, data_type, buff);
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn TexSubImage2D_(&self,
                      target: u32,
                      level: i32,
                      xoffset: i32,
                      yoffset: i32,
                      format: u32,
                      data_type: u32,
                      source: Option<ImageDataOrHTMLImageElementOrHTMLCanvasElementOrHTMLVideoElement>) {
        let (pixels, size) = match self.get_image_pixels(source) {
            Ok((pixels, size)) => (pixels, size),
            Err(_) => return,
        };

        let validator = TexImage2DValidator::new(self, target, level, format,
                                                 size.width, size.height,
                                                 0, format, data_type);
        let TexImage2DValidatorResult {
            texture,
            target,
            width,
            height,
            level,
            format,
            data_type,
            ..
        } = match validator.validate() {
            Ok(result) => result,
            Err(_) => return, // NB: The validator sets the correct error for us.
        };

        self.tex_sub_image_2d(texture, target, level, xoffset, yoffset,
                              width, height, format, data_type, pixels);
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

#[derive(Debug, PartialEq)]
pub enum UniformSetterType {
    Int,
    IntVec2,
    IntVec3,
    IntVec4,
    Float,
    FloatVec2,
    FloatVec3,
    FloatVec4,
}

impl UniformSetterType {
    pub fn element_count(&self) -> usize {
        match *self {
            UniformSetterType::Int => 1,
            UniformSetterType::IntVec2 => 2,
            UniformSetterType::IntVec3 => 3,
            UniformSetterType::IntVec4 => 4,
            UniformSetterType::Float => 1,
            UniformSetterType::FloatVec2 => 2,
            UniformSetterType::FloatVec3 => 3,
            UniformSetterType::FloatVec4 => 4,
        }
    }

    pub fn is_compatible_with(&self, gl_type: u32) -> bool {
        gl_type == self.as_gl_constant() || match *self {
            // Sampler uniform variables have an index value (the index of the
            // texture), and as such they have to be set as ints
            UniformSetterType::Int => gl_type == constants::SAMPLER_2D ||
                                gl_type == constants::SAMPLER_CUBE,
            // Don't ask me why, but it seems we must allow setting bool
            // uniforms with uniform1f.
            //
            // See the WebGL conformance test
            //   conformance/uniforms/gl-uniform-bool.html
            UniformSetterType::Float => gl_type == constants::BOOL,
            UniformSetterType::FloatVec2 => gl_type == constants::BOOL_VEC2,
            UniformSetterType::FloatVec3 => gl_type == constants::BOOL_VEC3,
            UniformSetterType::FloatVec4 => gl_type == constants::BOOL_VEC4,
            _ => false,
        }
    }

    fn as_gl_constant(&self) -> u32 {
        match *self {
            UniformSetterType::Int => constants::INT,
            UniformSetterType::IntVec2 => constants::INT_VEC2,
            UniformSetterType::IntVec3 => constants::INT_VEC3,
            UniformSetterType::IntVec4 => constants::INT_VEC4,
            UniformSetterType::Float => constants::FLOAT,
            UniformSetterType::FloatVec2 => constants::FLOAT_VEC2,
            UniformSetterType::FloatVec3 => constants::FLOAT_VEC3,
            UniformSetterType::FloatVec4 => constants::FLOAT_VEC4,
        }
    }
}
