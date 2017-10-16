/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use byteorder::{NativeEndian, ReadBytesExt, WriteBytesExt};
use canvas_traits::canvas::{byte_swap, multiply_u8_pixel};
use canvas_traits::webgl::{WebGLContextShareMode, WebGLCommand, WebGLError};
use canvas_traits::webgl::{WebGLFramebufferBindingRequest, WebGLMsg, WebGLMsgSender, WebGLParameter, WebVRCommand};
use canvas_traits::webgl::DOMToTextureCommand;
use canvas_traits::webgl::WebGLError::*;
use canvas_traits::webgl::webgl_channel;
use dom::bindings::cell::DomRefCell;
use dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::{self, WebGLContextAttributes};
use dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLRenderingContextConstants as constants;
use dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLRenderingContextMethods;
use dom::bindings::codegen::UnionTypes::ImageDataOrHTMLImageElementOrHTMLCanvasElementOrHTMLVideoElement;
use dom::bindings::conversions::{ConversionResult, FromJSValConvertible, ToJSValConvertible};
use dom::bindings::error::{Error, Fallible};
use dom::bindings::inheritance::Castable;
use dom::bindings::nonnull::NonNullJSObjectPtr;
use dom::bindings::reflector::{DomObject, Reflector, reflect_dom_object};
use dom::bindings::root::{Dom, DomRoot, LayoutDom, MutNullableDom};
use dom::bindings::str::DOMString;
use dom::event::{Event, EventBubbles, EventCancelable};
use dom::htmlcanvaselement::HTMLCanvasElement;
use dom::htmlcanvaselement::utils as canvas_utils;
use dom::htmliframeelement::HTMLIFrameElement;
use dom::node::{Node, NodeDamage, window_from_node};
use dom::webgl_extensions::WebGLExtensions;
use dom::webgl_validations::WebGLValidator;
use dom::webgl_validations::tex_image_2d::{CommonTexImage2DValidator, CommonTexImage2DValidatorResult};
use dom::webgl_validations::tex_image_2d::{TexImage2DValidator, TexImage2DValidatorResult};
use dom::webgl_validations::types::{TexDataType, TexFormat, TexImageTarget};
use dom::webglactiveinfo::WebGLActiveInfo;
use dom::webglbuffer::WebGLBuffer;
use dom::webglcontextevent::WebGLContextEvent;
use dom::webglframebuffer::WebGLFramebuffer;
use dom::webglprogram::WebGLProgram;
use dom::webglrenderbuffer::WebGLRenderbuffer;
use dom::webglshader::WebGLShader;
use dom::webglshaderprecisionformat::WebGLShaderPrecisionFormat;
use dom::webgltexture::{TexParameterValue, WebGLTexture};
use dom::webgluniformlocation::WebGLUniformLocation;
use dom::window::Window;
use dom_struct::dom_struct;
use euclid::Size2D;
use fnv::FnvHashMap;
use half::f16;
use js::conversions::ConversionBehavior;
use js::jsapi::{JSContext, JSObject, Type, Rooted};
use js::jsval::{BooleanValue, DoubleValue, Int32Value, JSVal, NullValue, UndefinedValue};
use js::typedarray::{TypedArray, TypedArrayElement, Float32, Int32};
use net_traits::image::base::PixelFormat;
use net_traits::image_cache::ImageResponse;
use offscreen_gl_context::{GLContextAttributes, GLLimits};
use script_layout_interface::HTMLCanvasDataSource;
use servo_config::prefs::PREFS;
use std::cell::{Cell, Ref};
use std::iter::FromIterator;
use webrender_api;

type ImagePixelResult = Result<(Vec<u8>, Size2D<i32>, bool), ()>;
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

// From the GLES 2.0.25 spec, page 85:
//
//     "If a texture that is currently bound to one of the targets
//      TEXTURE_2D, or TEXTURE_CUBE_MAP is deleted, it is as though
//      BindTexture had been executed with the same target and texture
//      zero."
//
// and similar text occurs for other object types.
macro_rules! handle_object_deletion {
    ($self_:expr, $binding:expr, $object:ident, $unbind_command:expr) => {
        if let Some(bound_object) = $binding.get() {
            if bound_object.id() == $object.id() {
                $binding.set(None);
            }

            if let Some(command) = $unbind_command {
                $self_.send_command(command);
            }
        }
    };
}

macro_rules! object_binding_to_js_or_null {
    ($cx: expr, $binding:expr) => {
        {
            rooted!(in($cx) let mut rval = NullValue());
            if let Some(bound_object) = $binding.get() {
                bound_object.to_jsval($cx, rval.handle_mut());
            }
            rval.get()
        }
    };
}

macro_rules! optional_root_object_to_js_or_null {
    ($cx: expr, $binding:expr) => {
        {
            rooted!(in($cx) let mut rval = NullValue());
            if let Some(object) = $binding {
                object.to_jsval($cx, rval.handle_mut());
            }
            rval.get()
        }
    };
}

fn has_invalid_blend_constants(arg1: u32, arg2: u32) -> bool {
    match (arg1, arg2) {
        (constants::CONSTANT_COLOR, constants::CONSTANT_ALPHA) => true,
        (constants::ONE_MINUS_CONSTANT_COLOR, constants::ONE_MINUS_CONSTANT_ALPHA) => true,
        (constants::ONE_MINUS_CONSTANT_COLOR, constants::CONSTANT_ALPHA) => true,
        (constants::CONSTANT_COLOR, constants::ONE_MINUS_CONSTANT_ALPHA) => true,
        (_, _) => false
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

/// Information about the bound textures of a WebGL texture unit.
#[must_root]
#[derive(HeapSizeOf, JSTraceable)]
struct TextureUnitBindings {
    bound_texture_2d: MutNullableDom<WebGLTexture>,
    bound_texture_cube_map: MutNullableDom<WebGLTexture>,
}

impl TextureUnitBindings {
    fn new() -> Self {
        Self {
            bound_texture_2d: MutNullableDom::new(None),
            bound_texture_cube_map: MutNullableDom::new(None),
        }
    }

    /// Clears the slot associated to the given texture.
    /// Returns the GL target of the cleared slot, if any.
    fn clear_slot(&self, texture: &WebGLTexture) -> Option<u32> {
        let fields = [(&self.bound_texture_2d, constants::TEXTURE_2D),
                      (&self.bound_texture_cube_map, constants::TEXTURE_CUBE_MAP)];

        fields.iter().find(|field| {
            match field.0.get() {
                Some(t) => t.id() == texture.id(),
                _ => false,
            }
        }).and_then(|field| {
            field.0.set(None);
            Some(field.1)
        })
    }
}


#[dom_struct]
pub struct WebGLRenderingContext {
    reflector_: Reflector,
    #[ignore_heap_size_of = "Channels are hard"]
    webgl_sender: WebGLMsgSender,
    #[ignore_heap_size_of = "Defined in webrender"]
    webrender_image: Cell<Option<webrender_api::ImageKey>>,
    share_mode: WebGLContextShareMode,
    #[ignore_heap_size_of = "Defined in offscreen_gl_context"]
    limits: GLLimits,
    canvas: Dom<HTMLCanvasElement>,
    #[ignore_heap_size_of = "Defined in canvas_traits"]
    last_error: Cell<Option<WebGLError>>,
    texture_unpacking_settings: Cell<TextureUnpacking>,
    texture_unpacking_alignment: Cell<u32>,
    bound_framebuffer: MutNullableDom<WebGLFramebuffer>,
    bound_renderbuffer: MutNullableDom<WebGLRenderbuffer>,
    bound_textures: DomRefCell<FnvHashMap<u32, TextureUnitBindings>>,
    bound_texture_unit: Cell<u32>,
    bound_buffer_array: MutNullableDom<WebGLBuffer>,
    bound_buffer_element_array: MutNullableDom<WebGLBuffer>,
    bound_attrib_buffers: DomRefCell<FnvHashMap<u32, Dom<WebGLBuffer>>>,
    current_program: MutNullableDom<WebGLProgram>,
    #[ignore_heap_size_of = "Because it's small"]
    current_vertex_attrib_0: Cell<(f32, f32, f32, f32)>,
    #[ignore_heap_size_of = "Because it's small"]
    current_scissor: Cell<(i32, i32, i32, i32)>,
    #[ignore_heap_size_of = "Because it's small"]
    current_clear_color: Cell<(f32, f32, f32, f32)>,
    extension_manager: WebGLExtensions,
}

impl WebGLRenderingContext {
    fn new_inherited(window: &Window,
                     canvas: &HTMLCanvasElement,
                     size: Size2D<i32>,
                     attrs: GLContextAttributes)
                     -> Result<WebGLRenderingContext, String> {
        if let Some(true) = PREFS.get("webgl.testing.context_creation_error").as_boolean() {
            return Err("WebGL context creation error forced by pref `webgl.testing.context_creation_error`".into());
        }

        let (sender, receiver) = webgl_channel().unwrap();
        let webgl_chan = window.webgl_chan();
        webgl_chan.send(WebGLMsg::CreateContext(size, attrs, sender))
                  .unwrap();
        let result = receiver.recv().unwrap();

        result.map(|ctx_data| {
            WebGLRenderingContext {
                reflector_: Reflector::new(),
                webgl_sender: ctx_data.sender,
                webrender_image: Cell::new(None),
                share_mode: ctx_data.share_mode,
                limits: ctx_data.limits,
                canvas: Dom::from_ref(canvas),
                last_error: Cell::new(None),
                texture_unpacking_settings: Cell::new(CONVERT_COLORSPACE),
                texture_unpacking_alignment: Cell::new(4),
                bound_framebuffer: MutNullableDom::new(None),
                bound_textures: DomRefCell::new(Default::default()),
                bound_texture_unit: Cell::new(constants::TEXTURE0),
                bound_buffer_array: MutNullableDom::new(None),
                bound_buffer_element_array: MutNullableDom::new(None),
                bound_attrib_buffers: DomRefCell::new(Default::default()),
                bound_renderbuffer: MutNullableDom::new(None),
                current_program: MutNullableDom::new(None),
                current_vertex_attrib_0: Cell::new((0f32, 0f32, 0f32, 1f32)),
                current_scissor: Cell::new((0, 0, size.width, size.height)),
                current_clear_color: Cell::new((0.0, 0.0, 0.0, 0.0)),
                extension_manager: WebGLExtensions::new()
            }
        })
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window, canvas: &HTMLCanvasElement, size: Size2D<i32>, attrs: GLContextAttributes)
               -> Option<DomRoot<WebGLRenderingContext>> {
        match WebGLRenderingContext::new_inherited(window, canvas, size, attrs) {
            Ok(ctx) => Some(reflect_dom_object(Box::new(ctx), window, WebGLRenderingContextBinding::Wrap)),
            Err(msg) => {
                error!("Couldn't create WebGLRenderingContext: {}", msg);
                let event = WebGLContextEvent::new(window,
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

    fn bound_texture(&self, target: u32) -> Option<DomRoot<WebGLTexture>> {
        match target {
            constants::TEXTURE_2D => {
                self.bound_textures.borrow().get(&self.bound_texture_unit.get()).and_then(|t| {
                    t.bound_texture_2d.get()
                })
            },
            constants::TEXTURE_CUBE_MAP => {
                self.bound_textures.borrow().get(&self.bound_texture_unit.get()).and_then(|t| {
                    t.bound_texture_cube_map.get()
                })
            },
            _ => None,
        }
    }

    pub fn bound_texture_for_target(&self, target: &TexImageTarget) -> Option<DomRoot<WebGLTexture>> {
        self.bound_textures.borrow().get(&self.bound_texture_unit.get()).and_then(|binding| {
            match *target {
                TexImageTarget::Texture2D => binding.bound_texture_2d.get(),
                TexImageTarget::CubeMapPositiveX |
                TexImageTarget::CubeMapNegativeX |
                TexImageTarget::CubeMapPositiveY |
                TexImageTarget::CubeMapNegativeY |
                TexImageTarget::CubeMapPositiveZ |
                TexImageTarget::CubeMapNegativeZ => binding.bound_texture_cube_map.get(),
            }
        })
    }

    pub fn borrow_bound_attrib_buffers(&self) -> Ref<FnvHashMap<u32, Dom<WebGLBuffer>>> {
        self.bound_attrib_buffers.borrow()
    }

    pub fn set_bound_attrib_buffers<'a, T>(&self, iter: T) where T: Iterator<Item=(u32, &'a WebGLBuffer)> {
        *self.bound_attrib_buffers.borrow_mut() = FnvHashMap::from_iter(iter.map(|(k,v)| (k, Dom::from_ref(v))));
    }

    pub fn bound_buffer_element_array(&self) -> Option<DomRoot<WebGLBuffer>> {
        self.bound_buffer_element_array.get()
    }

    pub fn set_bound_buffer_element_array(&self, buffer: Option<&WebGLBuffer>) {
        self.bound_buffer_element_array.set(buffer);
    }

    pub fn recreate(&self, size: Size2D<i32>) {
        let (sender, receiver) = webgl_channel().unwrap();
        self.webgl_sender.send_resize(size, sender).unwrap();

        if let Err(msg) = receiver.recv().unwrap() {
            error!("Error resizing WebGLContext: {}", msg);
            return;
        };

        // ClearColor needs to be restored because after a resize the GLContext is recreated
        // and the framebuffer is cleared using the default black transparent color.
        let color = self.current_clear_color.get();
        self.send_command(WebGLCommand::ClearColor(color.0, color.1, color.2, color.3));

        // WebGL Spec: Scissor rect must not change if the canvas is resized.
        // See: webgl/conformance-1.0.3/conformance/rendering/gl-scissor-canvas-dimensions.html
        // NativeContext handling library changes the scissor after a resize, so we need to reset the
        // default scissor when the canvas was created or the last scissor that the user set.
        let rect = self.current_scissor.get();
        self.send_command(WebGLCommand::Scissor(rect.0, rect.1, rect.2, rect.3));

        // Bound texture must not change when the canvas is resized.
        // Right now offscreen_gl_context generates a new FBO and the bound texture is changed
        // in order to create a new render to texture attachment.
        // Send a command to re-bind the TEXTURE_2D, if any.
        if let Some(texture) = self.bound_texture(constants::TEXTURE_2D) {
            self.send_command(WebGLCommand::BindTexture(constants::TEXTURE_2D, Some(texture.id())));
        }

        // Bound framebuffer must not change when the canvas is resized.
        // Right now offscreen_gl_context generates a new FBO on resize.
        // Send a command to re-bind the framebuffer, if any.
        if let Some(fbo) = self.bound_framebuffer.get() {
            let id = WebGLFramebufferBindingRequest::Explicit(fbo.id());
            self.send_command(WebGLCommand::BindFramebuffer(constants::FRAMEBUFFER, id));
        }
    }

    pub fn webgl_sender(&self) -> WebGLMsgSender {
        self.webgl_sender.clone()
    }

    #[inline]
    pub fn send_command(&self, command: WebGLCommand) {
        self.webgl_sender.send(command).unwrap();
    }

    #[inline]
    pub fn send_vr_command(&self, command: WebVRCommand) {
        self.webgl_sender.send_vr(command).unwrap();
    }

    pub fn get_extension_manager<'a>(&'a self) -> &'a WebGLExtensions {
        &self.extension_manager
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

    // Helper function for validating framebuffer completeness in
    // calls touching the framebuffer.  From the GLES 2.0.25 spec,
    // page 119:
    //
    //    "Effects of Framebuffer Completeness on Framebuffer
    //     Operations
    //
    //     If the currently bound framebuffer is not framebuffer
    //     complete, then it is an error to attempt to use the
    //     framebuffer for writing or reading. This means that
    //     rendering commands such as DrawArrays and DrawElements, as
    //     well as commands that read the framebuffer such as
    //     ReadPixels and CopyTexSubImage, will generate the error
    //     INVALID_FRAMEBUFFER_OPERATION if called while the
    //     framebuffer is not framebuffer complete."
    //
    // The WebGL spec mentions a couple more operations that trigger
    // this: clear() and getParameter(IMPLEMENTATION_COLOR_READ_*).
    fn validate_framebuffer_complete(&self) -> bool {
        match self.bound_framebuffer.get() {
            Some(fb) => match fb.check_status() {
                constants::FRAMEBUFFER_COMPLETE => return true,
                _ => {
                    self.webgl_error(InvalidFramebufferOperation);
                    return false;
                }
            },
            // The default framebuffer is always complete.
            None => return true,
        }
    }

    fn tex_parameter(&self, target: u32, name: u32, value: TexParameterValue) {
        let texture = match target {
            constants::TEXTURE_2D |
            constants::TEXTURE_CUBE_MAP => self.bound_texture(target),
            _ => return self.webgl_error(InvalidEnum),
        };
        if let Some(texture) = texture {
            handle_potential_webgl_error!(self, texture.tex_parameter(target, name, value));

            // Validate non filterable TEXTURE_2D data_types
            if target != constants::TEXTURE_2D {
                return;
            }

            let target = TexImageTarget::Texture2D;
            let info = texture.image_info_for_target(&target, 0);
            if info.is_initialized() {
                self.validate_filterable_texture(&texture,
                                                 target,
                                                 0,
                                                 info.internal_format().unwrap_or(TexFormat::RGBA),
                                                 info.width(),
                                                 info.height(),
                                                 info.data_type().unwrap_or(TexDataType::UnsignedByte));
            }
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

        self.send_command(WebGLCommand::VertexAttrib(indx, x, y, z, w));
    }

    fn get_current_framebuffer_size(&self) -> Option<(i32, i32)> {
        match self.bound_framebuffer.get() {
            Some(fb) => return fb.size(),

            // The window system framebuffer is bound
            None => return Some((self.DrawingBufferWidth(),
                                 self.DrawingBufferHeight())),
        }
    }

    // LINEAR filtering may be forbidden when using WebGL extensions.
    // https://www.khronos.org/registry/webgl/extensions/OES_texture_float_linear/
    fn validate_filterable_texture(&self,
                                   texture: &WebGLTexture,
                                   target: TexImageTarget,
                                   level: u32,
                                   format: TexFormat,
                                   width: u32,
                                   height: u32,
                                   data_type: TexDataType)
                                   -> bool
    {
        if self.extension_manager.is_filterable(data_type.as_gl_constant()) || !texture.is_using_linear_filtering() {
            return true;
        }

        // Handle validation failed: LINEAR filtering not valid for this texture
        // WebGL Conformance tests expect to fallback to [0, 0, 0, 255] RGBA UNSIGNED_BYTE
        let data_type = TexDataType::UnsignedByte;
        let expected_byte_length = width * height * 4;
        let mut pixels = vec![0u8; expected_byte_length as usize];
        for rgba8 in pixels.chunks_mut(4) {
            rgba8[3] = 255u8;
        }

        let pixels = self.prepare_pixels(format, data_type, width, height, 1, true, true, pixels);
        self.tex_image_2d(texture, target, data_type, format, level, width, height, 0, 1, pixels);

        false
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
                                      data: &[T]) -> bool {
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

        // TODO(emilio): Get more complex uniform info from ANGLE, and use it to
        // properly validate that the uniform setter type is compatible with the
        // uniform type, and that the uniform size matches.
        if data.len() % uniform_type.element_count() != 0 {
            self.webgl_error(InvalidOperation);
            return false;
        }

        true
    }

    // https://en.wikipedia.org/wiki/Relative_luminance
    #[inline]
    fn luminance(r: u8, g: u8, b: u8) -> u8 {
        (0.2126 * (r as f32) +
         0.7152 * (g as f32) +
         0.0722 * (b as f32)) as u8
    }

    /// Translates an image in rgba8 (red in the first byte) format to
    /// the format that was requested of TexImage.
    ///
    /// From the WebGL 1.0 spec, 5.14.8:
    ///
    ///     "The source image data is conceptually first converted to
    ///      the data type and format specified by the format and type
    ///      arguments, and then transferred to the WebGL
    ///      implementation. If a packed pixel format is specified
    ///      which would imply loss of bits of precision from the image
    ///      data, this loss of precision must occur."
    fn rgba8_image_to_tex_image_data(&self,
                                     format: TexFormat,
                                     data_type: TexDataType,
                                     pixels: Vec<u8>) -> Vec<u8> {
        // hint for vector allocation sizing.
        let pixel_count = pixels.len() / 4;

        match (format, data_type) {
            (TexFormat::RGBA, TexDataType::UnsignedByte) => pixels,
            (TexFormat::RGB, TexDataType::UnsignedByte) => {
                // Remove alpha channel
                let mut rgb8 = Vec::<u8>::with_capacity(pixel_count * 3);
                for rgba8 in pixels.chunks(4) {
                    rgb8.push(rgba8[0]);
                    rgb8.push(rgba8[1]);
                    rgb8.push(rgba8[2]);
                }
                rgb8
            },

            (TexFormat::Alpha, TexDataType::UnsignedByte) => {
                let mut alpha = Vec::<u8>::with_capacity(pixel_count);
                for rgba8 in pixels.chunks(4) {
                    alpha.push(rgba8[3]);
                }
                alpha
            },

            (TexFormat::Luminance, TexDataType::UnsignedByte) => {
                let mut luminance = Vec::<u8>::with_capacity(pixel_count);
                for rgba8 in pixels.chunks(4) {
                    luminance.push(Self::luminance(rgba8[0], rgba8[1], rgba8[2]));
                }
                luminance
            },

            (TexFormat::LuminanceAlpha, TexDataType::UnsignedByte) => {
                let mut data = Vec::<u8>::with_capacity(pixel_count * 2);
                for rgba8 in pixels.chunks(4) {
                    data.push(Self::luminance(rgba8[0], rgba8[1], rgba8[2]));
                    data.push(rgba8[3]);
                }
                data
            },

            (TexFormat::RGBA, TexDataType::UnsignedShort4444) => {
                let mut rgba4 = Vec::<u8>::with_capacity(pixel_count * 2);
                for rgba8 in pixels.chunks(4) {
                    rgba4.write_u16::<NativeEndian>((rgba8[0] as u16 & 0xf0) << 8 |
                                                    (rgba8[1] as u16 & 0xf0) << 4 |
                                                    (rgba8[2] as u16 & 0xf0) |
                                                    (rgba8[3] as u16 & 0xf0) >> 4).unwrap();
                }
                rgba4
            }

            (TexFormat::RGBA, TexDataType::UnsignedShort5551) => {
                let mut rgba5551 = Vec::<u8>::with_capacity(pixel_count * 2);
                for rgba8 in pixels.chunks(4) {
                    rgba5551.write_u16::<NativeEndian>((rgba8[0] as u16 & 0xf8) << 8 |
                                                       (rgba8[1] as u16 & 0xf8) << 3 |
                                                       (rgba8[2] as u16 & 0xf8) >> 2 |
                                                       (rgba8[3] as u16) >> 7).unwrap();
                }
                rgba5551
            }

            (TexFormat::RGB, TexDataType::UnsignedShort565) => {
                let mut rgb565 = Vec::<u8>::with_capacity(pixel_count * 2);
                for rgba8 in pixels.chunks(4) {
                    rgb565.write_u16::<NativeEndian>((rgba8[0] as u16 & 0xf8) << 8 |
                                                     (rgba8[1] as u16 & 0xfc) << 3 |
                                                     (rgba8[2] as u16 & 0xf8) >> 3).unwrap();
                }
                rgb565
            }


            (TexFormat::RGBA, TexDataType::Float) => {
                let mut rgbaf32 = Vec::<u8>::with_capacity(pixel_count * 16);
                for rgba8 in pixels.chunks(4) {
                    rgbaf32.write_f32::<NativeEndian>(rgba8[0] as f32).unwrap();
                    rgbaf32.write_f32::<NativeEndian>(rgba8[1] as f32).unwrap();
                    rgbaf32.write_f32::<NativeEndian>(rgba8[2] as f32).unwrap();
                    rgbaf32.write_f32::<NativeEndian>(rgba8[3] as f32).unwrap();
                }
                rgbaf32
            }

            (TexFormat::RGB, TexDataType::Float) => {
                let mut rgbf32 = Vec::<u8>::with_capacity(pixel_count * 12);
                for rgba8 in pixels.chunks(4) {
                    rgbf32.write_f32::<NativeEndian>(rgba8[0] as f32).unwrap();
                    rgbf32.write_f32::<NativeEndian>(rgba8[1] as f32).unwrap();
                    rgbf32.write_f32::<NativeEndian>(rgba8[2] as f32).unwrap();
                }
                rgbf32
            }

            (TexFormat::Alpha, TexDataType::Float) => {
                let mut alpha = Vec::<u8>::with_capacity(pixel_count * 4);
                for rgba8 in pixels.chunks(4) {
                    alpha.write_f32::<NativeEndian>(rgba8[0] as f32).unwrap();
                }
                alpha
            },

            (TexFormat::Luminance, TexDataType::Float) => {
                let mut luminance = Vec::<u8>::with_capacity(pixel_count * 4);
                for rgba8 in pixels.chunks(4) {
                    let p = Self::luminance(rgba8[0], rgba8[1], rgba8[2]);
                    luminance.write_f32::<NativeEndian>(p as f32).unwrap();
                }
                luminance
            },

            (TexFormat::LuminanceAlpha, TexDataType::Float) => {
                let mut data = Vec::<u8>::with_capacity(pixel_count * 8);
                for rgba8 in pixels.chunks(4) {
                    let p = Self::luminance(rgba8[0], rgba8[1], rgba8[2]);
                    data.write_f32::<NativeEndian>(p as f32).unwrap();
                    data.write_f32::<NativeEndian>(rgba8[3] as f32).unwrap();
                }
                data
            },

            (TexFormat::RGBA, TexDataType::HalfFloat) => {
                let mut rgbaf16 = Vec::<u8>::with_capacity(pixel_count * 8);
                for rgba8 in pixels.chunks(4) {
                    rgbaf16.write_u16::<NativeEndian>(f16::from_f32(rgba8[0] as f32).as_bits()).unwrap();
                    rgbaf16.write_u16::<NativeEndian>(f16::from_f32(rgba8[1] as f32).as_bits()).unwrap();
                    rgbaf16.write_u16::<NativeEndian>(f16::from_f32(rgba8[2] as f32).as_bits()).unwrap();
                    rgbaf16.write_u16::<NativeEndian>(f16::from_f32(rgba8[3] as f32).as_bits()).unwrap();
                }
                rgbaf16
            },

            (TexFormat::RGB, TexDataType::HalfFloat) => {
                let mut rgbf16 = Vec::<u8>::with_capacity(pixel_count * 6);
                for rgba8 in pixels.chunks(4) {
                    rgbf16.write_u16::<NativeEndian>(f16::from_f32(rgba8[0] as f32).as_bits()).unwrap();
                    rgbf16.write_u16::<NativeEndian>(f16::from_f32(rgba8[1] as f32).as_bits()).unwrap();
                    rgbf16.write_u16::<NativeEndian>(f16::from_f32(rgba8[2] as f32).as_bits()).unwrap();
                }
                rgbf16
            },

            (TexFormat::Alpha, TexDataType::HalfFloat) => {
                let mut alpha = Vec::<u8>::with_capacity(pixel_count * 2);
                for rgba8 in pixels.chunks(4) {
                    alpha.write_u16::<NativeEndian>(f16::from_f32(rgba8[3] as f32).as_bits()).unwrap();
                }
                alpha
            },

            (TexFormat::Luminance, TexDataType::HalfFloat) => {
                let mut luminance = Vec::<u8>::with_capacity(pixel_count * 4);
                for rgba8 in pixels.chunks(4) {
                    let p = Self::luminance(rgba8[0], rgba8[1], rgba8[2]);
                    luminance.write_u16::<NativeEndian>(f16::from_f32(p as f32).as_bits()).unwrap();
                }
                luminance
            },

            (TexFormat::LuminanceAlpha, TexDataType::HalfFloat) => {
                let mut data = Vec::<u8>::with_capacity(pixel_count * 8);
                for rgba8 in pixels.chunks(4) {
                    let p = Self::luminance(rgba8[0], rgba8[1], rgba8[2]);
                    data.write_u16::<NativeEndian>(f16::from_f32(p as f32).as_bits()).unwrap();
                    data.write_u16::<NativeEndian>(f16::from_f32(rgba8[3] as f32).as_bits()).unwrap();
                }
                data
            },

            // Validation should have ensured that we only hit the
            // above cases, but we haven't turned the (format, type)
            // into an enum yet so there's a default case here.
            _ => unreachable!("Unsupported formats {:?} {:?}", format, data_type)
        }
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
        let (pixels, size, premultiplied) = match source {
            ImageDataOrHTMLImageElementOrHTMLCanvasElementOrHTMLVideoElement::ImageData(image_data) => {
                (image_data.get_data_array(), image_data.get_size(), false)
            },
            ImageDataOrHTMLImageElementOrHTMLCanvasElementOrHTMLVideoElement::HTMLImageElement(image) => {
                let img_url = match image.get_url() {
                    Some(url) => url,
                    None => return Err(()),
                };

                let window = window_from_node(&*self.canvas);

                let img = match canvas_utils::request_image_from_cache(&window, img_url) {
                    ImageResponse::Loaded(img, _) => img,
                    ImageResponse::PlaceholderLoaded(_, _) | ImageResponse::None |
                    ImageResponse::MetadataLoaded(_)
                        => return Err(()),
                };

                let size = Size2D::new(img.width as i32, img.height as i32);

                // For now Servo's images are all stored as BGRA8 internally.
                let mut data = match img.format {
                    PixelFormat::BGRA8 => img.bytes.to_vec(),
                    _ => unimplemented!(),
                };

                byte_swap(&mut data);

                (data, size, false)
            },
            // TODO(emilio): Getting canvas data is implemented in CanvasRenderingContext2D,
            // but we need to refactor it moving it to `HTMLCanvasElement` and support
            // WebGLContext (probably via GetPixels()).
            ImageDataOrHTMLImageElementOrHTMLCanvasElementOrHTMLVideoElement::HTMLCanvasElement(canvas) => {
                if let Some((mut data, size)) = canvas.fetch_all_data() {
                    // Pixels got from Canvas have already alpha premultiplied
                    byte_swap(&mut data);
                    (data, size, true)
                } else {
                    return Err(());
                }
            },
            ImageDataOrHTMLImageElementOrHTMLCanvasElementOrHTMLVideoElement::HTMLVideoElement(_rooted_video)
                => unimplemented!(),
        };

        return Ok((pixels, size, premultiplied));
    }

    // TODO(emilio): Move this logic to a validator.
    #[allow(unsafe_code)]
    unsafe fn validate_tex_image_2d_data(&self,
                                         width: u32,
                                         height: u32,
                                         format: TexFormat,
                                         data_type: TexDataType,
                                         unpacking_alignment: u32,
                                         data: *mut JSObject,
                                         cx: *mut JSContext)
                                         -> Result<u32, ()> {
        let element_size = data_type.element_size();
        let components_per_element = data_type.components_per_element();
        let components = format.components();

        // If data is non-null, the type of pixels must match the type of the
        // data to be read.
        // If it is UNSIGNED_BYTE, a Uint8Array must be supplied;
        // if it is UNSIGNED_SHORT_5_6_5, UNSIGNED_SHORT_4_4_4_4,
        // or UNSIGNED_SHORT_5_5_5_1, a Uint16Array must be supplied.
        // or FLOAT, a Float32Array must be supplied.
        // If the types do not match, an INVALID_OPERATION error is generated.
        typedarray!(in(cx) let typedarray_u8: Uint8Array = data);
        typedarray!(in(cx) let typedarray_u16: Uint16Array = data);
        typedarray!(in(cx) let typedarray_f32: Float32Array = data);
        let received_size = if data.is_null() {
            element_size
        } else {
            if typedarray_u16.is_ok() {
                2
            } else if typedarray_u8.is_ok() {
                1
            } else if typedarray_f32.is_ok() {
                4
            } else {
                self.webgl_error(InvalidOperation);
                return Err(());
            }
        };

        if received_size != element_size {
            self.webgl_error(InvalidOperation);
            return Err(());
        }

        // NOTE: width and height are positive or zero due to validate()
        if height == 0 {
            return Ok(0);
        } else {
            // We need to be careful here to not count unpack
            // alignment at the end of the image, otherwise (for
            // example) passing a single byte for uploading a 1x1
            // GL_ALPHA/GL_UNSIGNED_BYTE texture would throw an error.
            let cpp = element_size * components / components_per_element;
            let stride = (width * cpp + unpacking_alignment - 1) & !(unpacking_alignment - 1);
            return Ok(stride * (height - 1) + width * cpp);
        }
    }

    /// Flips the pixels in the Vec on the Y axis if
    /// UNPACK_FLIP_Y_WEBGL is currently enabled.
    fn flip_teximage_y(&self,
                       pixels: Vec<u8>,
                       internal_format: TexFormat,
                       data_type: TexDataType,
                       width: usize,
                       height: usize,
                       unpacking_alignment: usize) -> Vec<u8> {
        if !self.texture_unpacking_settings.get().contains(FLIP_Y_AXIS) {
            return pixels;
        }

        let cpp = (data_type.element_size() *
                   internal_format.components() / data_type.components_per_element()) as usize;

        let stride = (width * cpp + unpacking_alignment - 1) & !(unpacking_alignment - 1);

        let mut flipped = Vec::<u8>::with_capacity(pixels.len());

        for y in 0..height {
            let flipped_y = height - 1 - y;
            let start = flipped_y * stride;

            flipped.extend_from_slice(&pixels[start..(start + width * cpp)]);
            flipped.extend(vec![0u8; stride - width * cpp]);
        }

        flipped
    }

    /// Performs premultiplication of the pixels if
    /// UNPACK_PREMULTIPLY_ALPHA_WEBGL is currently enabled.
    fn premultiply_pixels(&self,
                          format: TexFormat,
                          data_type: TexDataType,
                          pixels: Vec<u8>) -> Vec<u8> {
        if !self.texture_unpacking_settings.get().contains(PREMULTIPLY_ALPHA) {
            return pixels;
        }

        match (format, data_type) {
            (TexFormat::RGBA, TexDataType::UnsignedByte) => {
                let mut premul = Vec::<u8>::with_capacity(pixels.len());
                for rgba in pixels.chunks(4) {
                    premul.push(multiply_u8_pixel(rgba[0], rgba[3]));
                    premul.push(multiply_u8_pixel(rgba[1], rgba[3]));
                    premul.push(multiply_u8_pixel(rgba[2], rgba[3]));
                    premul.push(rgba[3]);
                }
                premul
            }
            (TexFormat::LuminanceAlpha, TexDataType::UnsignedByte) => {
                let mut premul = Vec::<u8>::with_capacity(pixels.len());
                for la in pixels.chunks(2) {
                    premul.push(multiply_u8_pixel(la[0], la[1]));
                    premul.push(la[1]);
                }
                premul
            }

            (TexFormat::RGBA, TexDataType::UnsignedShort5551) => {
                let mut premul = Vec::<u8>::with_capacity(pixels.len());
                for mut rgba in pixels.chunks(2) {
                    let pix = rgba.read_u16::<NativeEndian>().unwrap();
                    if pix & (1 << 15) != 0 {
                        premul.write_u16::<NativeEndian>(pix).unwrap();
                    } else {
                        premul.write_u16::<NativeEndian>(0).unwrap();
                    }
                }
                premul
            }

            (TexFormat::RGBA, TexDataType::UnsignedShort4444) => {
                let mut premul = Vec::<u8>::with_capacity(pixels.len());
                for mut rgba in pixels.chunks(2) {
                    let pix = rgba.read_u16::<NativeEndian>().unwrap();
                    let extend_to_8_bits = |val| { (val | val << 4) as u8 };
                    let r = extend_to_8_bits(pix & 0x000f);
                    let g = extend_to_8_bits((pix & 0x00f0) >> 4);
                    let b = extend_to_8_bits((pix & 0x0f00) >> 8);
                    let a = extend_to_8_bits((pix & 0xf000) >> 12);

                    premul.write_u16::<NativeEndian>((multiply_u8_pixel(r, a) & 0xf0) as u16 >> 4 |
                                                     (multiply_u8_pixel(g, a) & 0xf0) as u16 |
                                                     ((multiply_u8_pixel(b, a) & 0xf0) as u16) << 4 |
                                                     pix & 0xf000).unwrap();
                }
                premul
            }

            // Other formats don't have alpha, so return their data untouched.
            _ => pixels
        }
    }

    // Remove premultiplied alpha.
    // This is only called when texImage2D is called using a canvas2d source and
    // UNPACK_PREMULTIPLY_ALPHA_WEBGL is disabled. Pixels got from a canvas2D source
    // are always RGBA8 with premultiplied alpha, so we don't have to worry about
    // additional formats as happens in the premultiply_pixels method.
    fn remove_premultiplied_alpha(&self, mut pixels: Vec<u8>) -> Vec<u8> {
        for rgba in pixels.chunks_mut(4) {
            let a = (rgba[3] as f32) / 255.0;
            rgba[0] = (rgba[0] as f32 / a) as u8;
            rgba[1] = (rgba[1] as f32 / a) as u8;
            rgba[2] = (rgba[2] as f32 / a) as u8;
        }
        pixels
    }

    fn prepare_pixels(&self,
                      internal_format: TexFormat,
                      data_type: TexDataType,
                      width: u32,
                      height: u32,
                      unpacking_alignment: u32,
                      source_premultiplied: bool,
                      source_from_image_or_canvas: bool,
                      mut pixels: Vec<u8>) -> Vec<u8> {
        let dest_premultiply = self.texture_unpacking_settings.get().contains(PREMULTIPLY_ALPHA);
        if !source_premultiplied && dest_premultiply {
            if source_from_image_or_canvas {
                // When the pixels come from image or canvas or imagedata, use RGBA8 format
                pixels = self.premultiply_pixels(TexFormat::RGBA, TexDataType::UnsignedByte, pixels);
            } else {
                pixels = self.premultiply_pixels(internal_format, data_type, pixels);
            }
        } else if source_premultiplied && !dest_premultiply {
            pixels = self.remove_premultiplied_alpha(pixels);
        }

        if source_from_image_or_canvas {
            pixels = self.rgba8_image_to_tex_image_data(internal_format, data_type, pixels);
        }

        // FINISHME: Consider doing premultiply and flip in a single mutable Vec.
        self.flip_teximage_y(pixels, internal_format, data_type,
                             width as usize, height as usize, unpacking_alignment as usize)
    }

    fn tex_image_2d(&self,
                    texture: &WebGLTexture,
                    target: TexImageTarget,
                    data_type: TexDataType,
                    internal_format: TexFormat,
                    level: u32,
                    width: u32,
                    height: u32,
                    _border: u32,
                    unpacking_alignment: u32,
                    pixels: Vec<u8>) { // NB: pixels should NOT be premultipied

        // TexImage2D depth is always equal to 1
        handle_potential_webgl_error!(self, texture.initialize(target,
                                                               width,
                                                               height, 1,
                                                               internal_format,
                                                               level,
                                                               Some(data_type)));

        // Set the unpack alignment.  For textures coming from arrays,
        // this will be the current value of the context's
        // GL_UNPACK_ALIGNMENT, while for textures from images or
        // canvas (produced by rgba8_image_to_tex_image_data()), it
        // will be 1.
        self.send_command(WebGLCommand::PixelStorei(constants::UNPACK_ALIGNMENT, unpacking_alignment as i32));

        let format = internal_format.as_gl_constant();
        let data_type = data_type.as_gl_constant();
        let internal_format = self.extension_manager.get_effective_tex_internal_format(format, data_type);

        // TODO(emilio): convert colorspace if requested
        let msg = WebGLCommand::TexImage2D(target.as_gl_constant(), level as i32,
                                           internal_format as i32,
                                           width as i32, height as i32,
                                           format,
                                           data_type,
                                           pixels);

        self.send_command(msg);

        if let Some(fb) = self.bound_framebuffer.get() {
            fb.invalidate_texture(&*texture);
        }
    }

    fn tex_sub_image_2d(&self,
                        texture: DomRoot<WebGLTexture>,
                        target: TexImageTarget,
                        level: u32,
                        xoffset: i32,
                        yoffset: i32,
                        width: u32,
                        height: u32,
                        format: TexFormat,
                        data_type: TexDataType,
                        unpacking_alignment: u32,
                        pixels: Vec<u8>) {
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

        // Set the unpack alignment.  For textures coming from arrays,
        // this will be the current value of the context's
        // GL_UNPACK_ALIGNMENT, while for textures from images or
        // canvas (produced by rgba8_image_to_tex_image_data()), it
        // will be 1.
        self.send_command(WebGLCommand::PixelStorei(constants::UNPACK_ALIGNMENT, unpacking_alignment as i32));

        // TODO(emilio): convert colorspace if requested
        let msg = WebGLCommand::TexSubImage2D(target.as_gl_constant(),
                                              level as i32, xoffset, yoffset,
                                              width as i32, height as i32,
                                              format.as_gl_constant(),
                                              data_type.as_gl_constant(), pixels);

        self.send_command(msg);
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14
    fn validate_feature_enum(&self, cap: u32) -> bool {
        match cap {
            constants::BLEND | constants::CULL_FACE | constants::DEPTH_TEST | constants::DITHER |
            constants::POLYGON_OFFSET_FILL | constants::SAMPLE_ALPHA_TO_COVERAGE | constants::SAMPLE_COVERAGE |
            constants::SAMPLE_COVERAGE_INVERT | constants::SCISSOR_TEST | constants::STENCIL_TEST => true,
            _ => {
                self.webgl_error(InvalidEnum);
                false
            },
        }
    }

    fn get_gl_extensions(&self) -> String {
        let (sender, receiver) = webgl_channel().unwrap();
        self.send_command(WebGLCommand::GetExtensions(sender));
        receiver.recv().unwrap()
    }

    fn layout_handle(&self) -> webrender_api::ImageKey {
        match self.share_mode {
            WebGLContextShareMode::SharedTexture => {
                // WR using ExternalTexture requires a single update message.
                self.webrender_image.get().unwrap_or_else(|| {
                    let (sender, receiver) = webgl_channel().unwrap();
                    self.webgl_sender.send_update_wr_image(sender).unwrap();
                    let image_key = receiver.recv().unwrap();
                    self.webrender_image.set(Some(image_key));

                    image_key
                })
            },
            WebGLContextShareMode::Readback => {
                // WR using Readback requires to update WR image every frame
                // in order to send the new raw pixels.
                let (sender, receiver) = webgl_channel().unwrap();
                self.webgl_sender.send_update_wr_image(sender).unwrap();
                receiver.recv().unwrap()
            }
        }
    }
}

impl Drop for WebGLRenderingContext {
    fn drop(&mut self) {
        self.webgl_sender.send_remove().unwrap();
    }
}

// FIXME: After [1] lands and the relevant Servo and codegen PR too, we should
// convert all our raw JSObject pointers to proper types.
//
// [1]: https://github.com/servo/rust-mozjs/pull/304
#[allow(unsafe_code)]
unsafe fn typed_array_or_sequence_to_vec<T>(cx: *mut JSContext,
                                            sequence_or_abv: *mut JSObject,
                                            config: <T::Element as FromJSValConvertible>::Config)
                                            -> Result<Vec<T::Element>, Error>
    where T: TypedArrayElement,
          T::Element: FromJSValConvertible + Clone,
          <T::Element as FromJSValConvertible>::Config: Clone,
{
    // TODO(servo/rust-mozjs#330): replace this with a macro that supports generic types.
    let mut typed_array_root = Rooted::new_unrooted();
    let typed_array: Option<TypedArray<T>> =
          TypedArray::from(cx, &mut typed_array_root, sequence_or_abv).ok();
    if let Some(mut typed_array) = typed_array {
        return Ok(typed_array.as_slice().to_vec());
    }
    assert!(!sequence_or_abv.is_null());
    rooted!(in(cx) let mut val = UndefinedValue());
    sequence_or_abv.to_jsval(cx, val.handle_mut());

    match Vec::<T::Element>::from_jsval(cx, val.handle(), config) {
        Ok(ConversionResult::Success(v)) => Ok(v),
        Ok(ConversionResult::Failure(error)) => Err(Error::Type(error.into_owned())),
        // FIXME: What to do here? Generated code only aborts the execution of
        // the script.
        Err(err) => panic!("unexpected conversion error: {:?}", err),
    }
}

#[allow(unsafe_code)]
unsafe fn fallible_array_buffer_view_to_vec(cx: *mut JSContext, abv: *mut JSObject) -> Result<Vec<u8>, Error>
{
    assert!(!abv.is_null());
    typedarray!(in(cx) let array_buffer_view: ArrayBufferView = abv);
    match array_buffer_view {
        Ok(mut v) => Ok(v.as_slice().to_vec()),
        Err(_) => Err(Error::Type("Not an ArrayBufferView".to_owned())),
    }
}

impl WebGLRenderingContextMethods for WebGLRenderingContext {
    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.1
    fn Canvas(&self) -> DomRoot<HTMLCanvasElement> {
        DomRoot::from_ref(&*self.canvas)
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.11
    fn Flush(&self) {
        self.send_command(WebGLCommand::Flush);
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.11
    fn Finish(&self) {
        let (sender, receiver) = webgl_channel().unwrap();
        self.send_command(WebGLCommand::Finish(sender));
        receiver.recv().unwrap()
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.1
    fn DrawingBufferWidth(&self) -> i32 {
        let (sender, receiver) = webgl_channel().unwrap();
        self.send_command(WebGLCommand::DrawingBufferWidth(sender));
        receiver.recv().unwrap()
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.1
    fn DrawingBufferHeight(&self) -> i32 {
        let (sender, receiver) = webgl_channel().unwrap();
        self.send_command(WebGLCommand::DrawingBufferHeight(sender));
        receiver.recv().unwrap()
    }

    #[allow(unsafe_code)]
    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.5
    unsafe fn GetBufferParameter(&self, _cx: *mut JSContext, target: u32, parameter: u32) -> JSVal {
        let (sender, receiver) = webgl_channel().unwrap();
        self.send_command(WebGLCommand::GetBufferParameter(target, parameter, sender));

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
    unsafe fn GetParameter(&self, cx: *mut JSContext, parameter: u32) -> JSVal {
        // Handle the GL_*_BINDING without going all the way
        // to the GL, since we would just need to map back from GL's
        // returned ID to the WebGL* object we're tracking.
        match parameter {
            constants::ARRAY_BUFFER_BINDING =>
                return object_binding_to_js_or_null!(cx, &self.bound_buffer_array),
            constants::ELEMENT_ARRAY_BUFFER_BINDING =>
                return object_binding_to_js_or_null!(cx, &self.bound_buffer_element_array),
            constants::FRAMEBUFFER_BINDING =>
                return object_binding_to_js_or_null!(cx, &self.bound_framebuffer),
            constants::RENDERBUFFER_BINDING =>
                return object_binding_to_js_or_null!(cx, &self.bound_renderbuffer),
            constants::TEXTURE_BINDING_2D => {
                let texture = self.bound_texture(constants::TEXTURE_2D);
                return optional_root_object_to_js_or_null!(cx, texture)
            },
            constants::TEXTURE_BINDING_CUBE_MAP => {
                let texture = self.bound_texture(constants::TEXTURE_CUBE_MAP);
                return optional_root_object_to_js_or_null!(cx, texture)
            },
            // In readPixels we currently support RGBA/UBYTE only.  If
            // we wanted to support other formats, we could ask the
            // driver, but we would need to check for
            // GL_OES_read_format support (assuming an underlying GLES
            // driver. Desktop is happy to format convert for us).
            constants::IMPLEMENTATION_COLOR_READ_FORMAT => {
                if !self.validate_framebuffer_complete() {
                    return NullValue();
                } else {
                    return Int32Value(constants::RGBA as i32);
                }
            }
            constants::IMPLEMENTATION_COLOR_READ_TYPE => {
                if !self.validate_framebuffer_complete() {
                    return NullValue();
                } else {
                    return Int32Value(constants::UNSIGNED_BYTE as i32);
                }
            }
            _ => {
                if !self.extension_manager.is_get_parameter_name_enabled(parameter) {
                    self.webgl_error(WebGLError::InvalidEnum);
                    return NullValue();
                }
            }
        }

        // Handle GetParameter getters injected via WebGL extensions
        if let Some(query_handler) = self.extension_manager.get_query_parameter_handler(parameter) {
            match query_handler(cx, &self) {
                Ok(value) => {
                    return value;
                },
                Err(error) => {
                    self.webgl_error(error);
                    return NullValue();
                }
            }
        }

        let (sender, receiver) = webgl_channel().unwrap();
        self.send_command(WebGLCommand::GetParameter(parameter, sender));

        match handle_potential_webgl_error!(self, receiver.recv().unwrap(), WebGLParameter::Invalid) {
            WebGLParameter::Int(val) => Int32Value(val),
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

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn GetError(&self) -> u32 {
        let error_code = if let Some(error) = self.last_error.get() {
            match error {
                WebGLError::InvalidEnum => constants::INVALID_ENUM,
                WebGLError::InvalidFramebufferOperation => constants::INVALID_FRAMEBUFFER_OPERATION,
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
        let (sender, receiver) = webgl_channel().unwrap();

        // If the send does not succeed, assume context lost
        if self.webgl_sender.send(WebGLCommand::GetContextAttributes(sender)).is_err() {
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
        self.extension_manager.init_once(|| {
            self.get_gl_extensions()
        });
        let extensions = self.extension_manager.get_suported_extensions();
        Some(extensions.iter().map(|name| DOMString::from(*name)).collect())
    }

    #[allow(unsafe_code)]
    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.14
    unsafe fn GetExtension(&self, _cx: *mut JSContext, name: DOMString)
                    -> Option<NonNullJSObjectPtr> {
        self.extension_manager.init_once(|| {
            self.get_gl_extensions()
        });
        self.extension_manager.get_or_init_extension(&name, self)
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn ActiveTexture(&self, texture: u32) {
        self.bound_texture_unit.set(texture);
        self.send_command(WebGLCommand::ActiveTexture(texture));
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn BlendColor(&self, r: f32, g: f32, b: f32, a: f32) {
        self.send_command(WebGLCommand::BlendColor(r, g, b, a));
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn BlendEquation(&self, mode: u32) {
        if mode != constants::FUNC_ADD {
            return self.webgl_error(InvalidEnum);
        }

        self.send_command(WebGLCommand::BlendEquation(mode));
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn BlendEquationSeparate(&self, mode_rgb: u32, mode_alpha: u32) {
        if mode_rgb != constants::FUNC_ADD || mode_alpha != constants::FUNC_ADD {
            return self.webgl_error(InvalidEnum);
        }

        self.send_command(WebGLCommand::BlendEquationSeparate(mode_rgb, mode_alpha));
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn BlendFunc(&self, src_factor: u32, dest_factor: u32) {
        // From the WebGL 1.0 spec, 6.13: Viewport Depth Range:
        //
        //     A call to blendFunc will generate an INVALID_OPERATION error if one of the two
        //     factors is set to CONSTANT_COLOR or ONE_MINUS_CONSTANT_COLOR and the other to
        //     CONSTANT_ALPHA or ONE_MINUS_CONSTANT_ALPHA.
        if has_invalid_blend_constants(src_factor, dest_factor) {
            return self.webgl_error(InvalidOperation);
        }
        if has_invalid_blend_constants(dest_factor, src_factor) {
            return self.webgl_error(InvalidOperation);
        }

        self.send_command(WebGLCommand::BlendFunc(src_factor, dest_factor));
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn BlendFuncSeparate(&self, src_rgb: u32, dest_rgb: u32, src_alpha: u32, dest_alpha: u32) {
        // From the WebGL 1.0 spec, 6.13: Viewport Depth Range:
        //
        //     A call to blendFuncSeparate will generate an INVALID_OPERATION error if srcRGB is
        //     set to CONSTANT_COLOR or ONE_MINUS_CONSTANT_COLOR and dstRGB is set to
        //     CONSTANT_ALPHA or ONE_MINUS_CONSTANT_ALPHA or vice versa.
        if has_invalid_blend_constants(src_rgb, dest_rgb) {
            return self.webgl_error(InvalidOperation);
        }
        if has_invalid_blend_constants(dest_rgb, src_rgb) {
            return self.webgl_error(InvalidOperation);
        }

        self.send_command(WebGLCommand::BlendFuncSeparate(src_rgb, dest_rgb, src_alpha, dest_alpha));
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
            self.send_command(WebGLCommand::BindBuffer(target, None));
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.6
    fn BindFramebuffer(&self, target: u32, framebuffer: Option<&WebGLFramebuffer>) {
        if target != constants::FRAMEBUFFER {
            return self.webgl_error(InvalidOperation);
        }

        if let Some(framebuffer) = framebuffer {
            if framebuffer.is_deleted() {
                // From the WebGL spec:
                //
                //     "An attempt to bind a deleted framebuffer will
                //      generate an INVALID_OPERATION error, and the
                //      current binding will remain untouched."
                return self.webgl_error(InvalidOperation);
            } else {
                framebuffer.bind(target);
                self.bound_framebuffer.set(Some(framebuffer));
            }
        } else {
            // Bind the default framebuffer
            let cmd = WebGLCommand::BindFramebuffer(target, WebGLFramebufferBindingRequest::Default);
            self.send_command(cmd);
            self.bound_framebuffer.set(framebuffer);
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.7
    fn BindRenderbuffer(&self, target: u32, renderbuffer: Option<&WebGLRenderbuffer>) {
        if target != constants::RENDERBUFFER {
            return self.webgl_error(InvalidEnum);
        }

        match renderbuffer {
            // Implementations differ on what to do in the deleted
            // case: Chromium currently unbinds, and Gecko silently
            // returns.  The conformance tests don't cover this case.
            Some(renderbuffer) if !renderbuffer.is_deleted() => {
                self.bound_renderbuffer.set(Some(renderbuffer));
                renderbuffer.bind(target);
            }
            _ => {
                self.bound_renderbuffer.set(None);
                // Unbind the currently bound renderbuffer
                self.send_command(WebGLCommand::BindRenderbuffer(target, None));
            }
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn BindTexture(&self, target: u32, texture: Option<&WebGLTexture>) {
        let mut bound_textures = self.bound_textures.borrow_mut();
        let binding = bound_textures.entry(self.bound_texture_unit.get())
                                    .or_insert(TextureUnitBindings::new());
        let slot = match target {
            constants::TEXTURE_2D => &binding.bound_texture_2d,
            constants::TEXTURE_CUBE_MAP => &binding.bound_texture_cube_map,
            _ => return self.webgl_error(InvalidEnum),
        };

        if let Some(texture) = texture {
            match texture.bind(target) {
                Ok(_) => slot.set(Some(texture)),
                Err(err) => return self.webgl_error(err),
            }
        } else {
            slot.set(None);
            // Unbind the currently bound texture
            self.send_command(WebGLCommand::BindTexture(target, None));
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn GenerateMipmap(&self, target: u32) {
        let texture = match target {
            constants::TEXTURE_2D |
            constants::TEXTURE_CUBE_MAP => self.bound_texture(target),
            _ => return self.webgl_error(InvalidEnum),
        };

        match texture {
            Some(texture) => handle_potential_webgl_error!(self, texture.generate_mipmap()),
            None => self.webgl_error(InvalidOperation)
        }
    }

    #[allow(unsafe_code)]
    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.5
    unsafe fn BufferData(&self, cx: *mut JSContext, target: u32, data: *mut JSObject, usage: u32) -> Fallible<()> {
        if data.is_null() {
            return Ok(self.webgl_error(InvalidValue));
        }

        typedarray!(in(cx) let array_buffer: ArrayBuffer = data);
        let data_vec = match array_buffer {
            Ok(mut data) => data.as_slice().to_vec(),
            Err(_) => fallible_array_buffer_view_to_vec(cx, data)?,
        };

        let bound_buffer = match target {
            constants::ARRAY_BUFFER => self.bound_buffer_array.get(),
            constants::ELEMENT_ARRAY_BUFFER => self.bound_buffer_element_array.get(),
            _ => return Ok(self.webgl_error(InvalidEnum)),
        };

        let bound_buffer = match bound_buffer {
            Some(bound_buffer) => bound_buffer,
            None => return Ok(self.webgl_error(InvalidValue)),
        };

        match usage {
            constants::STREAM_DRAW |
            constants::STATIC_DRAW |
            constants::DYNAMIC_DRAW => (),
            _ => return Ok(self.webgl_error(InvalidEnum)),
        }

        handle_potential_webgl_error!(self, bound_buffer.buffer_data(target, &data_vec, usage));

        Ok(())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.5
    fn BufferData_(&self, target: u32, size: i64, usage: u32) -> Fallible<()> {
        let bound_buffer = match target {
            constants::ARRAY_BUFFER => self.bound_buffer_array.get(),
            constants::ELEMENT_ARRAY_BUFFER => self.bound_buffer_element_array.get(),
            _ => return Ok(self.webgl_error(InvalidEnum)),
        };

        let bound_buffer = match bound_buffer {
            Some(bound_buffer) => bound_buffer,
            None => return Ok(self.webgl_error(InvalidValue)),
        };

        if size < 0 {
            return Ok(self.webgl_error(InvalidValue));
        }

        match usage {
            constants::STREAM_DRAW |
            constants::STATIC_DRAW |
            constants::DYNAMIC_DRAW => (),
            _ => return Ok(self.webgl_error(InvalidEnum)),
        }

        // FIXME: Allocating a buffer based on user-requested size is
        // not great, but we don't have a fallible allocation to try.
        let data = vec![0u8; size as usize];
        handle_potential_webgl_error!(self, bound_buffer.buffer_data(target, &data, usage));

        Ok(())
    }

    #[allow(unsafe_code)]
    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.5
    unsafe fn BufferSubData(&self, cx: *mut JSContext, target: u32, offset: i64, data: *mut JSObject) -> Fallible<()> {
        if data.is_null() {
            return Ok(self.webgl_error(InvalidValue));
        }

        typedarray!(in(cx) let array_buffer: ArrayBuffer = data);
        let data_vec = match array_buffer {
            Ok(mut data) => data.as_slice().to_vec(),
            Err(_) => fallible_array_buffer_view_to_vec(cx, data)?,
        };

        let bound_buffer = match target {
            constants::ARRAY_BUFFER => self.bound_buffer_array.get(),
            constants::ELEMENT_ARRAY_BUFFER => self.bound_buffer_element_array.get(),
            _ => return Ok(self.webgl_error(InvalidEnum)),
        };

        let bound_buffer = match bound_buffer {
            Some(bound_buffer) => bound_buffer,
            None => return Ok(self.webgl_error(InvalidOperation)),
        };

        if offset < 0 {
            return Ok(self.webgl_error(InvalidValue));
        }

        if (offset as usize) + data_vec.len() > bound_buffer.capacity() {
            return Ok(self.webgl_error(InvalidValue));
        }
        self.send_command(WebGLCommand::BufferSubData(target, offset as isize, data_vec));

        Ok(())
    }

    #[allow(unsafe_code)]
    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    unsafe fn CompressedTexImage2D(&self, cx: *mut JSContext, _target: u32, _level: i32, _internal_format: u32,
                            _width: i32, _height: i32, _border: i32, pixels: *mut JSObject) -> Fallible<()> {
        let _data = fallible_array_buffer_view_to_vec(cx, pixels)?;
        // FIXME: No compressed texture format is currently supported, so error out as per
        // https://www.khronos.org/registry/webgl/specs/latest/1.0/#COMPRESSED_TEXTURE_SUPPORT
        self.webgl_error(InvalidEnum);
        Ok(())
    }

    #[allow(unsafe_code)]
    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    unsafe fn CompressedTexSubImage2D(&self, cx: *mut JSContext, _target: u32, _level: i32,
                               _xoffset: i32, _yoffset: i32, _width: i32, _height: i32,
                               _format: u32, pixels: *mut JSObject) -> Fallible<()> {
        let _data = fallible_array_buffer_view_to_vec(cx, pixels)?;
        // FIXME: No compressed texture format is currently supported, so error out as per
        // https://www.khronos.org/registry/webgl/specs/latest/1.0/#COMPRESSED_TEXTURE_SUPPORT
        self.webgl_error(InvalidEnum);

        Ok(())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn CopyTexImage2D(&self, target: u32, level: i32, internal_format: u32,
                      x: i32, y: i32, width: i32, height: i32, border: i32) {
        if !self.validate_framebuffer_complete() {
            return;
        }

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

        self.send_command(msg);
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn CopyTexSubImage2D(&self, target: u32, level: i32, xoffset: i32, yoffset: i32,
                         x: i32, y: i32, width: i32, height: i32) {
        if !self.validate_framebuffer_complete() {
            return;
        }

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

        self.send_command(msg);
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.11
    fn Clear(&self, mask: u32) {
        if !self.validate_framebuffer_complete() {
            return;
        }

        self.send_command(WebGLCommand::Clear(mask));
        self.mark_as_dirty();
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn ClearColor(&self, red: f32, green: f32, blue: f32, alpha: f32) {
        self.current_clear_color.set((red, green, blue, alpha));
        self.send_command(WebGLCommand::ClearColor(red, green, blue, alpha));
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn ClearDepth(&self, depth: f32) {
        self.send_command(WebGLCommand::ClearDepth(depth as f64))
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn ClearStencil(&self, stencil: i32) {
        self.send_command(WebGLCommand::ClearStencil(stencil))
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn ColorMask(&self, r: bool, g: bool, b: bool, a: bool) {
        self.send_command(WebGLCommand::ColorMask(r, g, b, a))
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn CullFace(&self, mode: u32) {
        match mode {
            constants::FRONT | constants::BACK | constants::FRONT_AND_BACK =>
                self.send_command(WebGLCommand::CullFace(mode)),
            _ => self.webgl_error(InvalidEnum),
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn FrontFace(&self, mode: u32) {
        match mode {
            constants::CW | constants::CCW =>
                self.send_command(WebGLCommand::FrontFace(mode)),
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
                self.send_command(WebGLCommand::DepthFunc(func)),
            _ => self.webgl_error(InvalidEnum),
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn DepthMask(&self, flag: bool) {
        self.send_command(WebGLCommand::DepthMask(flag))
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn DepthRange(&self, near: f32, far: f32) {
        // From the WebGL 1.0 spec, 6.12: Viewport Depth Range:
        //
        //     "A call to depthRange will generate an
        //      INVALID_OPERATION error if zNear is greater than
        //      zFar."
        if near > far {
            return self.webgl_error(InvalidOperation);
        }

        self.send_command(WebGLCommand::DepthRange(near as f64, far as f64))
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn Enable(&self, cap: u32) {
        if self.validate_feature_enum(cap) {
            self.send_command(WebGLCommand::Enable(cap));
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn Disable(&self, cap: u32) {
        if self.validate_feature_enum(cap) {
            self.send_command(WebGLCommand::Disable(cap));
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn CompileShader(&self, shader: Option<&WebGLShader>) {
        if let Some(shader) = shader {
            shader.compile(&self.extension_manager)
        }
    }

    // TODO(emilio): Probably in the future we should keep track of the
    // generated objects, either here or in the webgl thread
    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.5
    fn CreateBuffer(&self) -> Option<DomRoot<WebGLBuffer>> {
        WebGLBuffer::maybe_new(self.global().as_window(), self.webgl_sender.clone())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.6
    fn CreateFramebuffer(&self) -> Option<DomRoot<WebGLFramebuffer>> {
        WebGLFramebuffer::maybe_new(self.global().as_window(), self.webgl_sender.clone())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.7
    fn CreateRenderbuffer(&self) -> Option<DomRoot<WebGLRenderbuffer>> {
        WebGLRenderbuffer::maybe_new(self.global().as_window(), self.webgl_sender.clone())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn CreateTexture(&self) -> Option<DomRoot<WebGLTexture>> {
        WebGLTexture::maybe_new(self.global().as_window(), self.webgl_sender.clone())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn CreateProgram(&self) -> Option<DomRoot<WebGLProgram>> {
        WebGLProgram::maybe_new(self.global().as_window(), self.webgl_sender.clone())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn CreateShader(&self, shader_type: u32) -> Option<DomRoot<WebGLShader>> {
        match shader_type {
            constants::VERTEX_SHADER | constants::FRAGMENT_SHADER => {},
            _ => {
                self.webgl_error(InvalidEnum);
                return None;
            }
        }
        WebGLShader::maybe_new(self.global().as_window(), self.webgl_sender.clone(), shader_type)
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.5
    fn DeleteBuffer(&self, buffer: Option<&WebGLBuffer>) {
        if let Some(buffer) = buffer {
            if buffer.is_attached_to_vao() {
                // WebGL spec: The buffers attached to VAOs should still not be deleted.
                // They are deleted after the VAO is deleted.
                buffer.set_pending_delete();
                return;
            }

            // Remove deleted buffer from bound attrib buffers.
            let attrib_ids: Vec<_> = self.bound_attrib_buffers.borrow().iter()
                                                              .filter(|&(_, v)| v.id() == buffer.id())
                                                              .map(|(&k, _)| k)
                                                              .collect();
            for id in attrib_ids {
                self.bound_attrib_buffers.borrow_mut().remove(&id);
            }

            // Delete buffer.
            handle_object_deletion!(self, self.bound_buffer_array, buffer,
                                    Some(WebGLCommand::BindBuffer(constants::ARRAY_BUFFER, None)));
            handle_object_deletion!(self, self.bound_buffer_element_array, buffer,
                                    Some(WebGLCommand::BindBuffer(constants::ELEMENT_ARRAY_BUFFER, None)));
            buffer.delete()
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.6
    fn DeleteFramebuffer(&self, framebuffer: Option<&WebGLFramebuffer>) {
        if let Some(framebuffer) = framebuffer {
            handle_object_deletion!(self, self.bound_framebuffer, framebuffer,
                                    Some(WebGLCommand::BindFramebuffer(constants::FRAMEBUFFER,
                                                                       WebGLFramebufferBindingRequest::Default)));
            framebuffer.delete()
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.7
    fn DeleteRenderbuffer(&self, renderbuffer: Option<&WebGLRenderbuffer>) {
        if let Some(renderbuffer) = renderbuffer {
            handle_object_deletion!(self, self.bound_renderbuffer, renderbuffer,
                                    Some(WebGLCommand::BindRenderbuffer(constants::RENDERBUFFER, None)));
            // From the GLES 2.0.25 spec, page 113:
            //
            //     "If a renderbuffer object is deleted while its
            //     image is attached to the currently bound
            //     framebuffer, then it is as if
            //     FramebufferRenderbuffer had been called, with a
            //     renderbuffer of 0, for each attachment point to
            //     which this image was attached in the currently
            //     bound framebuffer."
            //
            if let Some(fb) = self.bound_framebuffer.get() {
                fb.detach_renderbuffer(renderbuffer);
            }

            renderbuffer.delete()
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn DeleteTexture(&self, texture: Option<&WebGLTexture>) {
        if let Some(texture) = texture {
            // From the GLES 2.0.25 spec, page 85:
            //
            //     "If a texture that is currently bound to one of the targets
            //      TEXTURE_2D, or TEXTURE_CUBE_MAP is deleted, it is as though
            //      BindTexture had been executed with the same target and texture
            //      zero."
            //
            // The same texture may be bound to multiple texture units.
            let mut bound_unit = self.bound_texture_unit.get();
            for (texture_unit, binding) in self.bound_textures.borrow().iter() {
                if let Some(target) = binding.clear_slot(texture) {
                    if *texture_unit != bound_unit {
                        self.send_command(WebGLCommand::ActiveTexture(*texture_unit));
                        bound_unit = *texture_unit;
                    }
                    self.send_command(WebGLCommand::BindTexture(target, None));
                }
            }

            // Restore bound texture unit if it has been changed.
            if self.bound_texture_unit.get() != bound_unit {
                self.send_command(WebGLCommand::ActiveTexture(self.bound_texture_unit.get()));
            }

            // From the GLES 2.0.25 spec, page 113:
            //
            //     "If a texture object is deleted while its image is
            //      attached to the currently bound framebuffer, then
            //      it is as if FramebufferTexture2D had been called,
            //      with a texture of 0, for each attachment point to
            //      which this image was attached in the currently
            //      bound framebuffer."
            if let Some(fb) = self.bound_framebuffer.get() {
                fb.detach_texture(texture);
            }
            texture.delete()
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn DeleteProgram(&self, program: Option<&WebGLProgram>) {
        if let Some(program) = program {
            // FIXME: We should call glUseProgram(0), but
            // WebGLCommand::UseProgram() doesn't take an Option
            // currently.  This is also a problem for useProgram(null)
            handle_object_deletion!(self, self.current_program, program, None);
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

                if !self.validate_framebuffer_complete() {
                    return;
                }

                self.send_command(WebGLCommand::DrawArrays(mode, first, count));
                self.mark_as_dirty();
            },
            _ => self.webgl_error(InvalidEnum),
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.11
    fn DrawElements(&self, mode: u32, count: i32, type_: u32, offset: i64) {
        // From the GLES 2.0.25 spec, page 21:
        //
        //     "type must be one of UNSIGNED_BYTE or UNSIGNED_SHORT"
        let type_size = match type_ {
            constants::UNSIGNED_BYTE => 1,
            constants::UNSIGNED_SHORT => 2,
            _ => return self.webgl_error(InvalidEnum),
        };

        if offset % type_size != 0 {
            return self.webgl_error(InvalidOperation);
        }

        if count < 0 {
            return self.webgl_error(InvalidValue);
        }

        if offset < 0 {
            return self.webgl_error(InvalidValue);
        }

        if self.current_program.get().is_none() {
            // From the WebGL spec
            //
            //     If the CURRENT_PROGRAM is null, an INVALID_OPERATION error will be generated.
            //     WebGL performs additional error checking beyond that specified
            //     in OpenGL ES 2.0 during calls to drawArrays and drawElements.
            //
            return self.webgl_error(InvalidOperation);
        }

        if let Some(array_buffer) = self.bound_buffer_element_array.get() {
            // WebGL Spec: check buffer overflows, must be a valid multiple of the size.
            let val = offset as u64 + (count as u64 * type_size as u64);
            if val > array_buffer.capacity() as u64 {
                return self.webgl_error(InvalidOperation);
            }
        } else {
            // From the WebGL spec
            //
            //      a non-null WebGLBuffer must be bound to the ELEMENT_ARRAY_BUFFER binding point
            //      or an INVALID_OPERATION error will be generated.
            //
            return self.webgl_error(InvalidOperation);
        }

        if !self.validate_framebuffer_complete() {
            return;
        }

        match mode {
            constants::POINTS | constants::LINE_STRIP |
            constants::LINE_LOOP | constants::LINES |
            constants::TRIANGLE_STRIP | constants::TRIANGLE_FAN |
            constants::TRIANGLES => {
                self.send_command(WebGLCommand::DrawElements(mode, count, type_, offset));
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

        self.send_command(WebGLCommand::EnableVertexAttribArray(attrib_id));
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn DisableVertexAttribArray(&self, attrib_id: u32) {
        if attrib_id > self.limits.max_vertex_attribs {
            return self.webgl_error(InvalidValue);
        }

        self.send_command(WebGLCommand::DisableVertexAttribArray(attrib_id));
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn GetActiveUniform(&self, program: Option<&WebGLProgram>, index: u32) -> Option<DomRoot<WebGLActiveInfo>> {
        let program = match program {
            Some(program) => program,
            None => {
                // Reasons to generate InvalidValue error
                // From the GLES 2.0 spec
                //
                //     "INVALID_VALUE is generated if index is greater than or equal
                //      to the number of active uniform variables in program"
                //
                // A null program has no uniforms so any index is always greater than the active uniforms
                // WebGl conformance expects error with null programs. Check tests in get-active-test.html
                self.webgl_error(InvalidValue);
                return None;
            }
        };

        match program.get_active_uniform(index) {
            Ok(ret) => Some(ret),
            Err(e) => {
                self.webgl_error(e);
                return None;
            }
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn GetActiveAttrib(&self, program: Option<&WebGLProgram>, index: u32) -> Option<DomRoot<WebGLActiveInfo>> {
        let program = match program {
            Some(program) => program,
            None => {
                // Reasons to generate InvalidValue error
                // From the GLES 2.0 spec
                //
                //     "INVALID_VALUE is generated if index is greater than or equal
                //      to the number of active attribute variables in program"
                //
                // A null program has no attributes so any index is always greater than the active uniforms
                // WebGl conformance expects error with null programs. Check tests in get-active-test.html
                self.webgl_error(InvalidValue);
                return None;
            }
        };

        match program.get_active_attrib(index) {
            Ok(ret) => Some(ret),
            Err(e) => {
                self.webgl_error(e);
                return None;
            }
        }
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
    fn GetProgramInfoLog(&self, program: Option<&WebGLProgram>) -> Option<DOMString> {
        if let Some(program) = program {
            match program.get_info_log() {
                Ok(value) => Some(DOMString::from(value)),
                Err(e) => {
                    self.webgl_error(e);
                    None
                }
            }
        } else {
            self.webgl_error(WebGLError::InvalidValue);
            None
        }
    }

    #[allow(unsafe_code)]
    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    unsafe fn GetProgramParameter(&self, _: *mut JSContext, program: Option<&WebGLProgram>, param_id: u32) -> JSVal {
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

    #[allow(unsafe_code)]
    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    unsafe fn GetShaderParameter(&self, _: *mut JSContext, shader: Option<&WebGLShader>, param_id: u32) -> JSVal {
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

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn GetShaderPrecisionFormat(&self,
                                shader_type: u32,
                                precision_type: u32)
                                -> Option<DomRoot<WebGLShaderPrecisionFormat>> {
        let (sender, receiver) = webgl_channel().unwrap();
        self.send_command(WebGLCommand::GetShaderPrecisionFormat(shader_type,
                                                                 precision_type,
                                                                 sender));

        match receiver.recv().unwrap() {
            Ok((range_min, range_max, precision)) => {
                Some(WebGLShaderPrecisionFormat::new(self.global().as_window(), range_min, range_max, precision))
            },
            Err(error) => {
                self.webgl_error(error);
                None
            }
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn GetUniformLocation(&self,
                          program: Option<&WebGLProgram>,
                          name: DOMString) -> Option<DomRoot<WebGLUniformLocation>> {
        program.and_then(|p| {
            handle_potential_webgl_error!(self, p.get_uniform_location(name), None)
                .map(|location| WebGLUniformLocation::new(self.global().as_window(), location, p.id()))
        })
    }

    #[allow(unsafe_code)]
    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    unsafe fn GetVertexAttrib(&self, cx: *mut JSContext, index: u32, pname: u32) -> JSVal {
        if index == 0 && pname == constants::CURRENT_VERTEX_ATTRIB {
            rooted!(in(cx) let mut result = UndefinedValue());
            let (x, y, z, w) = self.current_vertex_attrib_0.get();
            let attrib = vec![x, y, z, w];
            attrib.to_jsval(cx, result.handle_mut());
            return result.get()
        }

        if pname == constants::VERTEX_ATTRIB_ARRAY_BUFFER_BINDING {
            rooted!(in(cx) let mut jsval = NullValue());
            if let Some(buffer) =  self.bound_attrib_buffers.borrow().get(&index) {
                buffer.to_jsval(cx, jsval.handle_mut());
            }
            return jsval.get();
        }

        let (sender, receiver) = webgl_channel().unwrap();
        self.send_command(WebGLCommand::GetVertexAttrib(index, pname, sender));

        match handle_potential_webgl_error!(self, receiver.recv().unwrap(), WebGLParameter::Invalid) {
            WebGLParameter::Int(val) => Int32Value(val),
            WebGLParameter::Bool(val) => BooleanValue(val),
            WebGLParameter::String(_) => panic!("Vertex attrib should not be string"),
            WebGLParameter::Float(_) => panic!("Vertex attrib should not be float"),
            WebGLParameter::FloatArray(val) => {
                rooted!(in(cx) let mut result = UndefinedValue());
                val.to_jsval(cx, result.handle_mut());
                result.get()
            }
            WebGLParameter::Invalid => NullValue(),
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn GetVertexAttribOffset(&self, index: u32, pname: u32) -> i64 {
        let (sender, receiver) = webgl_channel().unwrap();
        self.send_command(WebGLCommand::GetVertexAttribOffset(index, pname, sender));

        handle_potential_webgl_error!(self, receiver.recv().unwrap(), 0) as i64
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn Hint(&self, target: u32, mode: u32) {
        if target != constants::GENERATE_MIPMAP_HINT && !self.extension_manager.is_hint_target_enabled(target) {
            return self.webgl_error(InvalidEnum);
        }

        match mode {
            constants::FASTEST |
            constants::NICEST |
            constants::DONT_CARE => (),

            _ => return self.webgl_error(InvalidEnum),
        }

        self.send_command(WebGLCommand::Hint(target, mode));
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.5
    fn IsBuffer(&self, buffer: Option<&WebGLBuffer>) -> bool {
        buffer.map_or(false, |buf| buf.target().is_some() && !buf.is_deleted())
    }

    // TODO: We could write this without IPC, recording the calls to `enable` and `disable`.
    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn IsEnabled(&self, cap: u32) -> bool {
        if self.validate_feature_enum(cap) {
            let (sender, receiver) = webgl_channel().unwrap();
            self.send_command(WebGLCommand::IsEnabled(cap, sender));
            return receiver.recv().unwrap();
        }

        false
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.6
    fn IsFramebuffer(&self, frame_buffer: Option<&WebGLFramebuffer>) -> bool {
        frame_buffer.map_or(false, |buf| buf.target().is_some() && !buf.is_deleted())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn IsProgram(&self, program: Option<&WebGLProgram>) -> bool {
        program.map_or(false, |p| !p.is_deleted())
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

        self.send_command(WebGLCommand::LineWidth(width))
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
                self.texture_unpacking_alignment.set(param_value as u32);
                return;
            },
            _ => return self.webgl_error(InvalidEnum),
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn PolygonOffset(&self, factor: f32, units: f32) {
        self.send_command(WebGLCommand::PolygonOffset(factor, units))
    }

    #[allow(unsafe_code)]
    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.12
    unsafe fn ReadPixels(&self, cx: *mut JSContext, x: i32, y: i32, width: i32, height: i32,
                  format: u32, pixel_type: u32, pixels: *mut JSObject) -> Fallible<()> {
        if pixels.is_null() {
            return Ok(self.webgl_error(InvalidValue));
        }

        typedarray!(in(cx) let mut pixels_data: ArrayBufferView = pixels);
        let (array_type, data) = match { pixels_data.as_mut() } {
            Ok(data) => (data.get_array_type(), data.as_mut_slice()),
            Err(_) => return Err(Error::Type("Not an ArrayBufferView".to_owned())),
        };

        if !self.validate_framebuffer_complete() {
            return Ok(());
        }

        match array_type {
            Type::Uint8 => (),
            _ => return Ok(self.webgl_error(InvalidOperation)),
        }

        // From the WebGL specification, 5.14.12 Reading back pixels
        //
        //     "Only two combinations of format and type are
        //      accepted. The first is format RGBA and type
        //      UNSIGNED_BYTE. The second is an implementation-chosen
        //      format. The values of format and type for this format
        //      may be determined by calling getParameter with the
        //      symbolic constants IMPLEMENTATION_COLOR_READ_FORMAT
        //      and IMPLEMENTATION_COLOR_READ_TYPE, respectively. The
        //      implementation-chosen format may vary depending on the
        //      format of the currently bound rendering
        //      surface. Unsupported combinations of format and type
        //      will generate an INVALID_OPERATION error."
        //
        // To avoid having to support general format packing math, we
        // always report RGBA/UNSIGNED_BYTE as our only supported
        // format.
        if format != constants::RGBA || pixel_type != constants::UNSIGNED_BYTE {
            return Ok(self.webgl_error(InvalidOperation));
        }
        let cpp = 4;

        //     "If pixels is non-null, but is not large enough to
        //      retrieve all of the pixels in the specified rectangle
        //      taking into account pixel store modes, an
        //      INVALID_OPERATION error is generated."
        let stride = match width.checked_mul(cpp) {
            Some(stride) => stride,
            _ => return Ok(self.webgl_error(InvalidOperation)),
        };

        match height.checked_mul(stride) {
            Some(size) if size <= data.len() as i32 => {}
            _ => return Ok(self.webgl_error(InvalidOperation)),
        }

        //     "For any pixel lying outside the frame buffer, the
        //      corresponding destination buffer range remains
        //      untouched; see Reading Pixels Outside the
        //      Framebuffer."
        let mut x = x;
        let mut y = y;
        let mut width = width;
        let mut height = height;
        let mut dst_offset = 0;

        if x < 0 {
            dst_offset += cpp * -x;
            width += x;
            x = 0;
        }

        if y < 0 {
            dst_offset += stride * -y;
            height += y;
            y = 0;
        }

        if width < 0 || height < 0 {
            return Ok(self.webgl_error(InvalidValue));
        }

        match self.get_current_framebuffer_size() {
            Some((fb_width, fb_height)) => {
                if x + width > fb_width {
                    width = fb_width - x;
                }
                if y + height > fb_height {
                    height = fb_height - y;
                }
            }
            _ => return Ok(self.webgl_error(InvalidOperation)),
        };

        let (sender, receiver) = webgl_channel().unwrap();
        self.send_command(WebGLCommand::ReadPixels(x, y, width, height, format, pixel_type, sender));

        let result = receiver.recv().unwrap();

        for i in 0..height {
            for j in 0..(width * cpp) {
                data[(dst_offset + i * stride + j) as usize] =
                    result[(i * width * cpp + j) as usize];
            }
        }

        Ok(())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn SampleCoverage(&self, value: f32, invert: bool) {
        self.send_command(WebGLCommand::SampleCoverage(value, invert));
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.4
    fn Scissor(&self, x: i32, y: i32, width: i32, height: i32) {
        if width < 0 || height < 0 {
            return self.webgl_error(InvalidValue)
        }

        self.current_scissor.set((x, y, width, height));
        self.send_command(WebGLCommand::Scissor(x, y, width, height));
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn StencilFunc(&self, func: u32, ref_: i32, mask: u32) {
        match func {
            constants::NEVER | constants::LESS | constants::EQUAL | constants::LEQUAL |
            constants::GREATER | constants::NOTEQUAL | constants::GEQUAL | constants::ALWAYS =>
                self.send_command(WebGLCommand::StencilFunc(func, ref_, mask)),
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
                self.send_command(WebGLCommand::StencilFuncSeparate(face, func, ref_, mask)),
            _ => self.webgl_error(InvalidEnum),
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn StencilMask(&self, mask: u32) {
        self.send_command(WebGLCommand::StencilMask(mask))
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn StencilMaskSeparate(&self, face: u32, mask: u32) {
        match face {
            constants::FRONT | constants::BACK | constants::FRONT_AND_BACK =>
                self.send_command(WebGLCommand::StencilMaskSeparate(face, mask)),
            _ => return self.webgl_error(InvalidEnum),
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn StencilOp(&self, fail: u32, zfail: u32, zpass: u32) {
        if self.validate_stencil_actions(fail) && self.validate_stencil_actions(zfail) &&
           self.validate_stencil_actions(zpass) {
                self.send_command(WebGLCommand::StencilOp(fail, zfail, zpass));
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
                self.send_command(WebGLCommand::StencilOpSeparate(face, fail, zfail, zpass))
        } else {
            self.webgl_error(InvalidEnum)
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn LinkProgram(&self, program: Option<&WebGLProgram>) {
        if let Some(program) = program {
            if let Err(e) = program.link() {
                self.webgl_error(e);
            }
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
        if self.validate_uniform_parameters(uniform, UniformSetterType::Float, &[val]) {
            self.send_command(WebGLCommand::Uniform1f(uniform.unwrap().id(), val))
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform1i(&self,
                  uniform: Option<&WebGLUniformLocation>,
                  val: i32) {
        if self.validate_uniform_parameters(uniform, UniformSetterType::Int, &[val]) {
            self.send_command(WebGLCommand::Uniform1i(uniform.unwrap().id(), val))
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    #[allow(unsafe_code)]
    unsafe fn Uniform1iv(&self,
                  cx: *mut JSContext,
                  uniform: Option<&WebGLUniformLocation>,
                  data: *mut JSObject) -> Fallible<()> {
        assert!(!data.is_null());
        let data_vec = typed_array_or_sequence_to_vec::<Int32>(cx, data, ConversionBehavior::Default)?;

        if self.validate_uniform_parameters(uniform, UniformSetterType::Int, &data_vec) {
            self.send_command(WebGLCommand::Uniform1iv(uniform.unwrap().id(), data_vec))
        }

        Ok(())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    #[allow(unsafe_code)]
    unsafe fn Uniform1fv(&self,
                  cx: *mut JSContext,
                  uniform: Option<&WebGLUniformLocation>,
                  data: *mut JSObject) -> Fallible<()> {
        assert!(!data.is_null());
        let data_vec = typed_array_or_sequence_to_vec::<Float32>(cx, data, ())?;

        if self.validate_uniform_parameters(uniform, UniformSetterType::Float, &data_vec) {
            self.send_command(WebGLCommand::Uniform1fv(uniform.unwrap().id(), data_vec));
        }

        Ok(())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform2f(&self,
                  uniform: Option<&WebGLUniformLocation>,
                  x: f32, y: f32) {
        if self.validate_uniform_parameters(uniform, UniformSetterType::FloatVec2, &[x, y]) {
            self.send_command(WebGLCommand::Uniform2f(uniform.unwrap().id(), x, y));
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    #[allow(unsafe_code)]
    unsafe fn Uniform2fv(&self,
                  cx: *mut JSContext,
                  uniform: Option<&WebGLUniformLocation>,
                  data: *mut JSObject) -> Fallible<()> {
        assert!(!data.is_null());
        let data_vec = typed_array_or_sequence_to_vec::<Float32>(cx, data, ())?;

        if self.validate_uniform_parameters(uniform,
                                            UniformSetterType::FloatVec2,
                                            &data_vec) {
            self.send_command(WebGLCommand::Uniform2fv(uniform.unwrap().id(), data_vec));
        }

        Ok(())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform2i(&self,
                  uniform: Option<&WebGLUniformLocation>,
                  x: i32, y: i32) {
        if self.validate_uniform_parameters(uniform,
                                            UniformSetterType::IntVec2,
                                            &[x, y]) {
            self.send_command(WebGLCommand::Uniform2i(uniform.unwrap().id(), x, y));
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    #[allow(unsafe_code)]
    unsafe fn Uniform2iv(&self,
                  cx: *mut JSContext,
                  uniform: Option<&WebGLUniformLocation>,
                  data: *mut JSObject) -> Fallible<()> {
        assert!(!data.is_null());
        let data_vec = typed_array_or_sequence_to_vec::<Int32>(cx, data, ConversionBehavior::Default)?;

        if self.validate_uniform_parameters(uniform,
                                            UniformSetterType::IntVec2,
                                            &data_vec) {
            self.send_command(WebGLCommand::Uniform2iv(uniform.unwrap().id(), data_vec));
        }

        Ok(())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform3f(&self,
                  uniform: Option<&WebGLUniformLocation>,
                  x: f32, y: f32, z: f32) {
        if self.validate_uniform_parameters(uniform,
                                            UniformSetterType::FloatVec3,
                                            &[x, y, z]) {
            self.send_command(WebGLCommand::Uniform3f(uniform.unwrap().id(), x, y, z));
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    #[allow(unsafe_code)]
    unsafe fn Uniform3fv(&self,
                  cx: *mut JSContext,
                  uniform: Option<&WebGLUniformLocation>,
                  data: *mut JSObject) -> Fallible<()> {
        assert!(!data.is_null());
        let data_vec = typed_array_or_sequence_to_vec::<Float32>(cx, data, ())?;

        if self.validate_uniform_parameters(uniform,
                                            UniformSetterType::FloatVec3,
                                            &data_vec) {
            self.send_command(WebGLCommand::Uniform3fv(uniform.unwrap().id(), data_vec))
        }

        Ok(())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform3i(&self,
                  uniform: Option<&WebGLUniformLocation>,
                  x: i32, y: i32, z: i32) {
        if self.validate_uniform_parameters(uniform,
                                            UniformSetterType::IntVec3,
                                            &[x, y, z]) {
            self.send_command(WebGLCommand::Uniform3i(uniform.unwrap().id(), x, y, z))
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    #[allow(unsafe_code)]
    unsafe fn Uniform3iv(&self,
                  cx: *mut JSContext,
                  uniform: Option<&WebGLUniformLocation>,
                  data: *mut JSObject) -> Fallible<()> {
        assert!(!data.is_null());
        let data_vec = typed_array_or_sequence_to_vec::<Int32>(cx, data, ConversionBehavior::Default)?;

        if self.validate_uniform_parameters(uniform,
                                            UniformSetterType::IntVec3,
                                            &data_vec) {
            self.send_command(WebGLCommand::Uniform3iv(uniform.unwrap().id(), data_vec))
        }

        Ok(())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform4i(&self,
                  uniform: Option<&WebGLUniformLocation>,
                  x: i32, y: i32, z: i32, w: i32) {
        if self.validate_uniform_parameters(uniform,
                                            UniformSetterType::IntVec4,
                                            &[x, y, z, w]) {
            self.send_command(WebGLCommand::Uniform4i(uniform.unwrap().id(), x, y, z, w))
        }
    }


    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    #[allow(unsafe_code)]
    unsafe fn Uniform4iv(&self,
                  cx: *mut JSContext,
                  uniform: Option<&WebGLUniformLocation>,
                  data: *mut JSObject) -> Fallible<()> {
        assert!(!data.is_null());
        let data_vec = typed_array_or_sequence_to_vec::<Int32>(cx, data, ConversionBehavior::Default)?;

        if self.validate_uniform_parameters(uniform,
                                            UniformSetterType::IntVec4,
                                            &data_vec) {
            self.send_command(WebGLCommand::Uniform4iv(uniform.unwrap().id(), data_vec))
        }

        Ok(())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform4f(&self,
                  uniform: Option<&WebGLUniformLocation>,
                  x: f32, y: f32, z: f32, w: f32) {
        if self.validate_uniform_parameters(uniform,
                                            UniformSetterType::FloatVec4,
                                            &[x, y, z, w]) {
            self.send_command(WebGLCommand::Uniform4f(uniform.unwrap().id(), x, y, z, w))
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    #[allow(unsafe_code)]
    unsafe fn Uniform4fv(&self,
                  cx: *mut JSContext,
                  uniform: Option<&WebGLUniformLocation>,
                  data: *mut JSObject) -> Fallible<()> {
        assert!(!data.is_null());
        let data_vec = typed_array_or_sequence_to_vec::<Float32>(cx, data, ())?;

        if self.validate_uniform_parameters(uniform,
                                            UniformSetterType::FloatVec4,
                                            &data_vec) {
            self.send_command(WebGLCommand::Uniform4fv(uniform.unwrap().id(), data_vec))
        }

        Ok(())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    #[allow(unsafe_code)]
    unsafe fn UniformMatrix2fv(&self,
                        cx: *mut JSContext,
                        uniform: Option<&WebGLUniformLocation>,
                        transpose: bool,
                        data: *mut JSObject) -> Fallible<()> {
        assert!(!data.is_null());
        let data_vec = typed_array_or_sequence_to_vec::<Float32>(cx, data, ())?;
        if self.validate_uniform_parameters(uniform,
                                            UniformSetterType::FloatMat2,
                                            &data_vec) {
            self.send_command(WebGLCommand::UniformMatrix2fv(uniform.unwrap().id(), transpose, data_vec));
        }

        Ok(())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    #[allow(unsafe_code)]
    unsafe fn UniformMatrix3fv(&self,
                        cx: *mut JSContext,
                        uniform: Option<&WebGLUniformLocation>,
                        transpose: bool,
                        data: *mut JSObject) -> Fallible<()> {
        assert!(!data.is_null());
        let data_vec = typed_array_or_sequence_to_vec::<Float32>(cx, data, ())?;
        if self.validate_uniform_parameters(uniform,
                                            UniformSetterType::FloatMat3,
                                            &data_vec) {
            self.send_command(WebGLCommand::UniformMatrix3fv(uniform.unwrap().id(), transpose, data_vec));
        }

        Ok(())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    #[allow(unsafe_code)]
    unsafe fn UniformMatrix4fv(&self,
                        cx: *mut JSContext,
                        uniform: Option<&WebGLUniformLocation>,
                        transpose: bool,
                        data: *mut JSObject) -> Fallible<()> {
        assert!(!data.is_null());
        let data_vec = typed_array_or_sequence_to_vec::<Float32>(cx, data, ())?;
        if self.validate_uniform_parameters(uniform,
                                            UniformSetterType::FloatMat4,
                                            &data_vec) {
            self.send_command(WebGLCommand::UniformMatrix4fv(uniform.unwrap().id(), transpose, data_vec));
        }

        Ok(())
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

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn ValidateProgram(&self, program: Option<&WebGLProgram>) {
        if let Some(program) = program {
            if let Err(e) = program.validate() {
                self.webgl_error(e);
            }
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn VertexAttrib1f(&self, indx: u32, x: f32) {
        self.vertex_attrib(indx, x, 0f32, 0f32, 1f32)
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    #[allow(unsafe_code)]
    unsafe fn VertexAttrib1fv(&self, cx: *mut JSContext, indx: u32, data: *mut JSObject) -> Fallible<()> {
        assert!(!data.is_null());
        let data_vec = typed_array_or_sequence_to_vec::<Float32>(cx, data, ())?;
        if data_vec.len() < 1 {
            return Ok(self.webgl_error(InvalidOperation));
        }
        self.vertex_attrib(indx, data_vec[0], 0f32, 0f32, 1f32);
        Ok(())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn VertexAttrib2f(&self, indx: u32, x: f32, y: f32) {
        self.vertex_attrib(indx, x, y, 0f32, 1f32)
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    #[allow(unsafe_code)]
    unsafe fn VertexAttrib2fv(&self, cx: *mut JSContext, indx: u32, data: *mut JSObject) -> Fallible<()> {
        assert!(!data.is_null());
        let data_vec = typed_array_or_sequence_to_vec::<Float32>(cx, data, ())?;
        if data_vec.len() < 2 {
            return Ok(self.webgl_error(InvalidOperation));
        }
        self.vertex_attrib(indx, data_vec[0], data_vec[1], 0f32, 1f32);
        Ok(())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn VertexAttrib3f(&self, indx: u32, x: f32, y: f32, z: f32) {
        self.vertex_attrib(indx, x, y, z, 1f32)
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    #[allow(unsafe_code)]
    unsafe fn VertexAttrib3fv(&self, cx: *mut JSContext, indx: u32, data: *mut JSObject) -> Fallible<()> {
        assert!(!data.is_null());
        let data_vec = typed_array_or_sequence_to_vec::<Float32>(cx, data, ())?;
        if data_vec.len() < 3 {
            return Ok(self.webgl_error(InvalidOperation));
        }
        self.vertex_attrib(indx, data_vec[0], data_vec[1], data_vec[2], 1f32);
        Ok(())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn VertexAttrib4f(&self, indx: u32, x: f32, y: f32, z: f32, w: f32) {
        self.vertex_attrib(indx, x, y, z, w)
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    #[allow(unsafe_code)]
    unsafe fn VertexAttrib4fv(&self, cx: *mut JSContext, indx: u32, data: *mut JSObject) -> Fallible<()> {
        assert!(!data.is_null());
        let data_vec = typed_array_or_sequence_to_vec::<Float32>(cx, data, ())?;
        if data_vec.len() < 4 {
            return Ok(self.webgl_error(InvalidOperation));
        }

        self.vertex_attrib(indx, data_vec[0], data_vec[1], data_vec[2], data_vec[3]);
        Ok(())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn VertexAttribPointer(&self, attrib_id: u32, size: i32, data_type: u32,
                           normalized: bool, stride: i32, offset: i64) {
        if attrib_id > self.limits.max_vertex_attribs {
            return self.webgl_error(InvalidValue);
        }

        // GLES spec: If offset or stride  is negative, an INVALID_VALUE error will be generated
        // WebGL spec: the maximum supported stride is 255
        if stride < 0 || stride > 255 || offset < 0 {
            return self.webgl_error(InvalidValue);
        }
        if size < 1 || size > 4 {
            return self.webgl_error(InvalidValue);
        }

        let buffer_array = match self.bound_buffer_array.get() {
            Some(buffer) => buffer,
            None => {
                return self.webgl_error(InvalidOperation);
            }
        };

        // stride and offset must be multiple of data_type
        match data_type {
            constants::BYTE | constants::UNSIGNED_BYTE => {},
            constants::SHORT | constants::UNSIGNED_SHORT => {
                if offset % 2 > 0 || stride % 2 > 0 {
                    return self.webgl_error(InvalidOperation);
                }
            },
            constants::FLOAT => {
                if offset % 4 > 0 || stride % 4 > 0 {
                    return self.webgl_error(InvalidOperation);
                }
            },
            _ => return self.webgl_error(InvalidEnum),

        }

        self.bound_attrib_buffers.borrow_mut().insert(attrib_id, Dom::from_ref(&*buffer_array));

        let msg = WebGLCommand::VertexAttribPointer(attrib_id, size, data_type, normalized, stride, offset as u32);
        self.send_command(msg);
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.4
    fn Viewport(&self, x: i32, y: i32, width: i32, height: i32) {
        if width < 0 || height < 0 {
            return self.webgl_error(InvalidValue)
        }

        self.send_command(WebGLCommand::Viewport(x, y, width, height))
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
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
        if !self.extension_manager.is_tex_type_enabled(data_type) {
            return Ok(self.webgl_error(InvalidEnum));
        }

        let data = if data_ptr.is_null() {
            None
        } else {
            Some(fallible_array_buffer_view_to_vec(cx, data_ptr)?)
        };

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
            Err(_) => return Ok(()), // NB: The validator sets the correct error for us.
        };

        let unpacking_alignment = self.texture_unpacking_alignment.get();

        let expected_byte_length =
            match { self.validate_tex_image_2d_data(width, height,
                                                    format, data_type,
                                                    unpacking_alignment, data_ptr, cx) } {
                Ok(byte_length) => byte_length,
                Err(()) => return Ok(()),
            };

        // If data is null, a buffer of sufficient size
        // initialized to 0 is passed.
        let buff = match data {
            None => vec![0u8; expected_byte_length as usize],
            Some(data) => data,
        };

        // From the WebGL spec:
        //
        //     "If pixels is non-null but its size is less than what
        //      is required by the specified width, height, format,
        //      type, and pixel storage parameters, generates an
        //      INVALID_OPERATION error."
        if buff.len() < expected_byte_length as usize {
            return Ok(self.webgl_error(InvalidOperation));
        }

        if !self.validate_filterable_texture(&texture, target, level, format, width, height, data_type) {
            return Ok(()); // The validator sets the correct error for use
        }

        let pixels = self.prepare_pixels(format, data_type, width, height,
                                         unpacking_alignment, false, false, buff);

        self.tex_image_2d(&texture, target, data_type, format,
                          level, width, height, border, unpacking_alignment, pixels);

        Ok(())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn TexImage2D_(&self,
                   target: u32,
                   level: i32,
                   internal_format: u32,
                   format: u32,
                   data_type: u32,
                   source: Option<ImageDataOrHTMLImageElementOrHTMLCanvasElementOrHTMLVideoElement>) -> Fallible<()> {
        if !self.extension_manager.is_tex_type_enabled(data_type) {
            return Ok(self.webgl_error(InvalidEnum));
        }

        // Get pixels from image source
        let (pixels, size, premultiplied) = match self.get_image_pixels(source) {
            Ok((pixels, size, premultiplied)) => (pixels, size, premultiplied),
            Err(_) => return Ok(()),
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
            Err(_) => return Ok(()), // NB: The validator sets the correct error for us.
        };

        if !self.validate_filterable_texture(&texture, target, level, format, width, height, data_type) {
            return Ok(()); // The validator sets the correct error for use
        }

        let unpacking_alignment = 1;
        let pixels = self.prepare_pixels(format, data_type, width, height,
                                         unpacking_alignment, premultiplied, true, pixels);

        self.tex_image_2d(&texture, target, data_type, format,
                          level, width, height, border, 1, pixels);
        Ok(())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn TexImageDOM(&self,
                   target: u32,
                   level: i32,
                   internal_format: u32,
                   width: i32,
                   height: i32,
                   format: u32,
                   data_type: u32,
                   source: &HTMLIFrameElement) -> Fallible<()> {
        // Currently DOMToTexture only supports TEXTURE_2D, RGBA, UNSIGNED_BYTE and no levels.
        if target != constants::TEXTURE_2D || level != 0 || internal_format != constants::RGBA ||
            format != constants::RGBA || data_type != constants::UNSIGNED_BYTE {
            return Ok(self.webgl_error(InvalidValue));
        }

        // Get bound texture
        let texture = match self.bound_texture(constants::TEXTURE_2D) {
            Some(texture) => texture,
            None => {
                return Ok(self.webgl_error(InvalidOperation));
            }
        };

        let pipeline_id = source.pipeline_id().ok_or(Error::InvalidState)?;
        let document_id  = self.global().downcast::<Window>().ok_or(Error::InvalidState)?.webrender_document();

        texture.set_attached_to_dom();

        let command = DOMToTextureCommand::Attach(self.webgl_sender.context_id(),
                                                  texture.id(),
                                                  document_id,
                                                  pipeline_id.to_webrender(),
                                                  Size2D::new(width, height));
        self.webgl_sender.send_dom_to_texture(command).unwrap();

        Ok(())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
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
        let data = if data_ptr.is_null() {
            None
        } else {
            Some(fallible_array_buffer_view_to_vec(cx, data_ptr)?)
        };

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
            Err(_) => return Ok(()), // NB: The validator sets the correct error for us.
        };

        let unpacking_alignment = self.texture_unpacking_alignment.get();

        let expected_byte_length =
            match { self.validate_tex_image_2d_data(width, height,
                                                    format, data_type,
                                                    unpacking_alignment, data_ptr, cx) } {
                Ok(byte_length) => byte_length,
                Err(()) => return Ok(()),
            };

        // If data is null, a buffer of sufficient size
        // initialized to 0 is passed.
        let buff = match data {
            None => vec![0u8; expected_byte_length as usize],
            Some(data) => data,
        };

        // From the WebGL spec:
        //
        //     "If pixels is non-null but its size is less than what
        //      is required by the specified width, height, format,
        //      type, and pixel storage parameters, generates an
        //      INVALID_OPERATION error."
        if buff.len() < expected_byte_length as usize {
            return Ok(self.webgl_error(InvalidOperation));
        }

        let unpacking_alignment = self.texture_unpacking_alignment.get();
        let pixels = self.prepare_pixels(format, data_type, width, height,
                                         unpacking_alignment, false, false, buff);

        self.tex_sub_image_2d(texture, target, level, xoffset, yoffset,
                              width, height, format, data_type, unpacking_alignment, pixels);
        Ok(())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn TexSubImage2D_(&self,
                      target: u32,
                      level: i32,
                      xoffset: i32,
                      yoffset: i32,
                      format: u32,
                      data_type: u32,
                      source: Option<ImageDataOrHTMLImageElementOrHTMLCanvasElementOrHTMLVideoElement>)
                      -> Fallible<()> {
        let (pixels, size, premultiplied) = match self.get_image_pixels(source) {
            Ok((pixels, size, premultiplied)) => (pixels, size, premultiplied),
            Err(_) => return Ok(()),
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
            Err(_) => return Ok(()), // NB: The validator sets the correct error for us.
        };

        let unpacking_alignment = 1;
        let pixels = self.prepare_pixels(format, data_type, width, height,
                                         unpacking_alignment, premultiplied, true, pixels);

        self.tex_sub_image_2d(texture, target, level, xoffset, yoffset,
                              width, height, format, data_type, 1, pixels);
        Ok(())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn TexParameterf(&self, target: u32, name: u32, value: f32) {
        self.tex_parameter(target, name, TexParameterValue::Float(value))
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn TexParameteri(&self, target: u32, name: u32, value: i32) {
        self.tex_parameter(target, name, TexParameterValue::Int(value))
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.6
    fn CheckFramebufferStatus(&self, target: u32) -> u32 {
        // From the GLES 2.0.25 spec, 4.4 ("Framebuffer Objects"):
        //
        //    "If target is not FRAMEBUFFER, INVALID_ENUM is
        //     generated. If CheckFramebufferStatus generates an
        //     error, 0 is returned."
        if target != constants::FRAMEBUFFER {
            self.webgl_error(InvalidEnum);
            return 0;
        }

        match self.bound_framebuffer.get() {
            Some(fb) => return fb.check_status(),
            None => return constants::FRAMEBUFFER_COMPLETE,
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.7
    fn RenderbufferStorage(&self, target: u32, internal_format: u32,
                           width: i32, height: i32) {
        // From the GLES 2.0.25 spec:
        //
        //    "target must be RENDERBUFFER."
        if target != constants::RENDERBUFFER {
            return self.webgl_error(InvalidOperation)
        }

        // From the GLES 2.0.25 spec:
        //
        //     "If either width or height is greater than the value of
        //      MAX_RENDERBUFFER_SIZE , the error INVALID_VALUE is
        //      generated."
        //
        // and we have to throw out negative-size values as well just
        // like for TexImage.
        //
        // FIXME: Handle max_renderbuffer_size, which doesn't seem to
        // be in limits.
        if width < 0 || height < 0 {
            return self.webgl_error(InvalidValue);
        }

        match self.bound_renderbuffer.get() {
            Some(rb) => {
                handle_potential_webgl_error!(self, rb.storage(internal_format, width, height));
                if let Some(fb) = self.bound_framebuffer.get() {
                    fb.invalidate_renderbuffer(&*rb);
                }
            }
            None => self.webgl_error(InvalidOperation),
        };

        // FIXME: We need to clear the renderbuffer before it can be
        // accessed.  See https://github.com/servo/servo/issues/13710
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.6
    fn FramebufferRenderbuffer(&self, target: u32, attachment: u32,
                               renderbuffertarget: u32,
                               rb: Option<&WebGLRenderbuffer>) {
        if target != constants::FRAMEBUFFER || renderbuffertarget != constants::RENDERBUFFER {
            return self.webgl_error(InvalidEnum);
        }

        match self.bound_framebuffer.get() {
            Some(fb) => handle_potential_webgl_error!(self, fb.renderbuffer(attachment, rb)),
            None => self.webgl_error(InvalidOperation),
        };
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.6
    fn FramebufferTexture2D(&self, target: u32, attachment: u32,
                            textarget: u32, texture: Option<&WebGLTexture>,
                            level: i32) {
        if target != constants::FRAMEBUFFER {
            return self.webgl_error(InvalidEnum);
        }

        match self.bound_framebuffer.get() {
            Some(fb) => handle_potential_webgl_error!(self, fb.texture2d(attachment, textarget, texture, level)),
            None => self.webgl_error(InvalidOperation),
        };
    }
}

pub trait LayoutCanvasWebGLRenderingContextHelpers {
    #[allow(unsafe_code)]
    unsafe fn canvas_data_source(&self) -> HTMLCanvasDataSource;
}

impl LayoutCanvasWebGLRenderingContextHelpers for LayoutDom<WebGLRenderingContext> {
    #[allow(unsafe_code)]
    unsafe fn canvas_data_source(&self) -> HTMLCanvasDataSource {
        HTMLCanvasDataSource::WebGL((*self.unsafe_get()).layout_handle())
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
    FloatMat2,
    FloatMat3,
    FloatMat4,
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
            UniformSetterType::FloatMat2 => 4,
            UniformSetterType::FloatMat3 => 9,
            UniformSetterType::FloatMat4 => 16,
        }
    }
}
