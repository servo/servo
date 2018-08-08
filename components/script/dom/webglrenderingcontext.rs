/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use byteorder::{NativeEndian, ReadBytesExt, WriteBytesExt};
use canvas_traits::canvas::{byte_swap, multiply_u8_pixel};
use canvas_traits::webgl::{DOMToTextureCommand, Parameter};
use canvas_traits::webgl::{TexParameter, WebGLCommand, WebGLContextShareMode, WebGLError};
use canvas_traits::webgl::{WebGLFramebufferBindingRequest, WebGLMsg, WebGLMsgSender};
use canvas_traits::webgl::{WebGLProgramId, WebGLResult, WebGLSLVersion, WebGLSender};
use canvas_traits::webgl::{WebGLVersion, WebVRCommand, webgl_channel};
use canvas_traits::webgl::WebGLError::*;
use dom::bindings::cell::DomRefCell;
use dom::bindings::codegen::Bindings::ANGLEInstancedArraysBinding::ANGLEInstancedArraysConstants;
use dom::bindings::codegen::Bindings::EXTBlendMinmaxBinding::EXTBlendMinmaxConstants;
use dom::bindings::codegen::Bindings::OESVertexArrayObjectBinding::OESVertexArrayObjectConstants;
use dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::{self, WebGLContextAttributes};
use dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLRenderingContextConstants as constants;
use dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLRenderingContextMethods;
use dom::bindings::codegen::UnionTypes::ArrayBufferViewOrArrayBuffer;
use dom::bindings::codegen::UnionTypes::Float32ArrayOrUnrestrictedFloatSequence;
use dom::bindings::codegen::UnionTypes::ImageDataOrHTMLImageElementOrHTMLCanvasElementOrHTMLVideoElement;
use dom::bindings::codegen::UnionTypes::Int32ArrayOrLongSequence;
use dom::bindings::conversions::{DerivedFrom, ToJSValConvertible};
use dom::bindings::error::{Error, ErrorResult};
use dom::bindings::inheritance::Castable;
use dom::bindings::reflector::{DomObject, Reflector, reflect_dom_object};
use dom::bindings::root::{Dom, DomOnceCell, DomRoot, LayoutDom, MutNullableDom};
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
use dom::webglframebuffer::{WebGLFramebuffer, WebGLFramebufferAttachmentRoot};
use dom::webglobject::WebGLObject;
use dom::webglprogram::WebGLProgram;
use dom::webglrenderbuffer::WebGLRenderbuffer;
use dom::webglshader::WebGLShader;
use dom::webglshaderprecisionformat::WebGLShaderPrecisionFormat;
use dom::webgltexture::{TexParameterValue, WebGLTexture};
use dom::webgluniformlocation::WebGLUniformLocation;
use dom::webglvertexarrayobjectoes::WebGLVertexArrayObjectOES;
use dom::window::Window;
use dom_struct::dom_struct;
use euclid::Size2D;
use fnv::FnvHashMap;
use half::f16;
use js::jsapi::{JSContext, JSObject, Type};
use js::jsval::{BooleanValue, DoubleValue, Int32Value, UInt32Value, JSVal};
use js::jsval::{ObjectValue, NullValue, UndefinedValue};
use js::rust::CustomAutoRooterGuard;
use js::typedarray::{ArrayBufferView, CreateWith, Float32, Float32Array, Int32, Int32Array, Uint32Array};
use js::typedarray::{TypedArray, TypedArrayElementCreator};
use net_traits::image::base::PixelFormat;
use net_traits::image_cache::ImageResponse;
use offscreen_gl_context::{GLContextAttributes, GLLimits};
use script_layout_interface::HTMLCanvasDataSource;
use serde::{Deserialize, Serialize};
use servo_config::prefs::PREFS;
use std::cell::Cell;
use std::cmp;
use std::ptr::{self, NonNull};
use webrender_api;

pub fn is_gles() -> bool {
    // TODO: align this with the actual kind of graphics context in use, rather than
    // making assumptions based on platform
    cfg!(any(target_os = "android", target_os = "ios"))
}

type ImagePixelResult = Result<(Vec<u8>, Size2D<i32>, bool), ()>;
pub const MAX_UNIFORM_AND_ATTRIBUTE_LEN: usize = 256;

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
    #[derive(JSTraceable, MallocSizeOf)]
    struct TextureUnpacking: u8 {
        const FLIP_Y_AXIS = 0x01;
        const PREMULTIPLY_ALPHA = 0x02;
        const CONVERT_COLORSPACE = 0x04;
    }
}

/// Information about the bound textures of a WebGL texture unit.
#[must_root]
#[derive(JSTraceable, MallocSizeOf)]
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
    #[ignore_malloc_size_of = "Channels are hard"]
    webgl_sender: WebGLMsgSender,
    #[ignore_malloc_size_of = "Defined in webrender"]
    webrender_image: Cell<Option<webrender_api::ImageKey>>,
    share_mode: WebGLContextShareMode,
    webgl_version: WebGLVersion,
    glsl_version: WebGLSLVersion,
    #[ignore_malloc_size_of = "Defined in offscreen_gl_context"]
    limits: GLLimits,
    canvas: Dom<HTMLCanvasElement>,
    #[ignore_malloc_size_of = "Defined in canvas_traits"]
    last_error: Cell<Option<WebGLError>>,
    texture_unpacking_settings: Cell<TextureUnpacking>,
    texture_unpacking_alignment: Cell<u32>,
    bound_framebuffer: MutNullableDom<WebGLFramebuffer>,
    bound_renderbuffer: MutNullableDom<WebGLRenderbuffer>,
    bound_textures: DomRefCell<FnvHashMap<u32, TextureUnitBindings>>,
    bound_texture_unit: Cell<u32>,
    bound_buffer_array: MutNullableDom<WebGLBuffer>,
    current_program: MutNullableDom<WebGLProgram>,
    /// https://www.khronos.org/webgl/wiki/WebGL_and_OpenGL_Differences#Vertex_Attribute_0
    #[ignore_malloc_size_of = "Because it's small"]
    current_vertex_attrib_0: Cell<(f32, f32, f32, f32)>,
    #[ignore_malloc_size_of = "Because it's small"]
    current_scissor: Cell<(i32, i32, i32, i32)>,
    #[ignore_malloc_size_of = "Because it's small"]
    current_clear_color: Cell<(f32, f32, f32, f32)>,
    extension_manager: WebGLExtensions,
    capabilities: Capabilities,
    default_vao: DomOnceCell<WebGLVertexArrayObjectOES>,
    current_vao: MutNullableDom<WebGLVertexArrayObjectOES>,
}

impl WebGLRenderingContext {
    pub fn new_inherited(
        window: &Window,
        canvas: &HTMLCanvasElement,
        webgl_version: WebGLVersion,
        size: Size2D<i32>,
        attrs: GLContextAttributes
    ) -> Result<WebGLRenderingContext, String> {
        if let Some(true) = PREFS.get("webgl.testing.context_creation_error").as_boolean() {
            return Err("WebGL context creation error forced by pref `webgl.testing.context_creation_error`".into());
        }

        let webgl_chan = match window.webgl_chan() {
            Some(chan) => chan,
            None => return Err("WebGL initialization failed early on".into()),
        };

        let (sender, receiver) = webgl_channel().unwrap();
        webgl_chan.send(WebGLMsg::CreateContext(webgl_version, size, attrs, sender))
                  .unwrap();
        let result = receiver.recv().unwrap();

        result.map(|ctx_data| {
            WebGLRenderingContext {
                reflector_: Reflector::new(),
                webgl_sender: ctx_data.sender,
                webrender_image: Cell::new(None),
                share_mode: ctx_data.share_mode,
                webgl_version,
                glsl_version: ctx_data.glsl_version,
                limits: ctx_data.limits,
                canvas: Dom::from_ref(canvas),
                last_error: Cell::new(None),
                texture_unpacking_settings: Cell::new(TextureUnpacking::CONVERT_COLORSPACE),
                texture_unpacking_alignment: Cell::new(4),
                bound_framebuffer: MutNullableDom::new(None),
                bound_textures: DomRefCell::new(Default::default()),
                bound_texture_unit: Cell::new(constants::TEXTURE0),
                bound_buffer_array: MutNullableDom::new(None),
                bound_renderbuffer: MutNullableDom::new(None),
                current_program: MutNullableDom::new(None),
                current_vertex_attrib_0: Cell::new((0f32, 0f32, 0f32, 1f32)),
                current_scissor: Cell::new((0, 0, size.width, size.height)),
                current_clear_color: Cell::new((0.0, 0.0, 0.0, 0.0)),
                extension_manager: WebGLExtensions::new(webgl_version),
                capabilities: Default::default(),
                default_vao: Default::default(),
                current_vao: Default::default(),
            }
        })
    }

    #[allow(unrooted_must_root)]
    pub fn new(
        window: &Window,
        canvas: &HTMLCanvasElement,
        webgl_version: WebGLVersion,
        size: Size2D<i32>,
        attrs: GLContextAttributes
    ) -> Option<DomRoot<WebGLRenderingContext>> {
        match WebGLRenderingContext::new_inherited(window, canvas, webgl_version, size, attrs) {
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

    fn current_vao(&self) -> DomRoot<WebGLVertexArrayObjectOES> {
        self.current_vao.or_init(|| {
            DomRoot::from_ref(
                self.default_vao.init_once(|| WebGLVertexArrayObjectOES::new(self, None)),
            )
        })
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

    fn validate_ownership<T>(&self, object: &T) -> WebGLResult<()>
    where
        T: DerivedFrom<WebGLObject>,
    {
        if self != object.upcast().context() {
            return Err(InvalidOperation);
        }
        Ok(())
    }

    fn with_location<F>(&self, location: Option<&WebGLUniformLocation>, f: F)
    where
        F: FnOnce(&WebGLUniformLocation) -> WebGLResult<()>,
    {
        let location = match location {
            Some(loc) => loc,
            None => return,
        };
        match self.current_program.get() {
            Some(ref program) if
                program.id() == location.program_id() &&
                program.link_generation() == location.link_generation()
            => {}
            _ => return self.webgl_error(InvalidOperation),
        }
        handle_potential_webgl_error!(self, f(location));
    }

    fn tex_parameter(&self, target: u32, param: u32, value: TexParameterValue) {
        let texture = match target {
            constants::TEXTURE_2D |
            constants::TEXTURE_CUBE_MAP => self.bound_texture(target),
            _ => return self.webgl_error(InvalidEnum),
        };

        let texture = match texture {
            Some(tex) => tex,
            None => return self.webgl_error(InvalidOperation),
        };

        if !self.extension_manager.is_get_tex_parameter_name_enabled(param) {
            return self.webgl_error(InvalidEnum);
        }

        handle_potential_webgl_error!(self, texture.tex_parameter(param, value), return);

        // Validate non filterable TEXTURE_2D data_types
        if target != constants::TEXTURE_2D {
            return;
        }

        let target = TexImageTarget::Texture2D;
        let info = texture.image_info_for_target(&target, 0);
        if info.is_initialized() {
            self.validate_filterable_texture(
                &texture,
                target,
                0,
                info.internal_format().unwrap_or(TexFormat::RGBA),
                info.width(),
                info.height(),
                info.data_type().unwrap_or(TexDataType::UnsignedByte),
            );
        }
    }

    fn mark_as_dirty(&self) {
        self.canvas.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
    }

    fn vertex_attrib(&self, indx: u32, x: f32, y: f32, z: f32, w: f32) {
        if indx >= self.limits.max_vertex_attribs {
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

    fn get_image_pixels(
        &self,
        source: ImageDataOrHTMLImageElementOrHTMLCanvasElementOrHTMLVideoElement,
    ) -> ImagePixelResult {
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
    fn validate_tex_image_2d_data(&self,
                                  width: u32,
                                  height: u32,
                                  format: TexFormat,
                                  data_type: TexDataType,
                                  unpacking_alignment: u32,
                                  data: &Option<ArrayBufferView>)
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
        let received_size = match *data {
            None => element_size,
            Some(ref buffer) => match buffer.get_array_type() {
                Type::Uint8 => 1,
                Type::Uint16 => 2,
                Type::Float32 => 4,
                _ => {
                    self.webgl_error(InvalidOperation);
                    return Err(());
                }
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
        if !self.texture_unpacking_settings.get().contains(TextureUnpacking::FLIP_Y_AXIS) {
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
        if !self.texture_unpacking_settings.get().contains(TextureUnpacking::PREMULTIPLY_ALPHA) {
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
        let dest_premultiply = self.texture_unpacking_settings.get().contains(TextureUnpacking::PREMULTIPLY_ALPHA);
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
        let msg = WebGLCommand::TexImage2D(
            target.as_gl_constant(),
            level as i32,
            internal_format as i32,
            width as i32, height as i32,
            format,
            data_type,
            pixels.into(),
        );

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
        let msg = WebGLCommand::TexSubImage2D(
            target.as_gl_constant(),
            level as i32,
            xoffset,
            yoffset,
            width as i32,
            height as i32,
            format.as_gl_constant(),
            data_type.as_gl_constant(),
            pixels.into(),
        );

        self.send_command(msg);
    }

    fn get_gl_extensions(&self) -> String {
        let (sender, receiver) = webgl_channel().unwrap();
        self.send_command(WebGLCommand::GetExtensions(sender));
        receiver.recv().unwrap()
    }

    pub fn layout_handle(&self) -> webrender_api::ImageKey {
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

    // https://www.khronos.org/registry/webgl/extensions/ANGLE_instanced_arrays/
    pub fn draw_arrays_instanced(
        &self,
        mode: u32,
        first: i32,
        count: i32,
        primcount: i32,
    ) {
        match mode {
            constants::POINTS | constants::LINE_STRIP |
            constants::LINE_LOOP | constants::LINES |
            constants::TRIANGLE_STRIP | constants::TRIANGLE_FAN |
            constants::TRIANGLES => {},
            _ => {
                return self.webgl_error(InvalidEnum);
            }
        }
        if first < 0 || count < 0 || primcount < 0 {
            return self.webgl_error(InvalidValue);
        }

        let current_program = handle_potential_webgl_error!(
            self,
            self.current_program.get().ok_or(InvalidOperation),
            return
        );

        let required_len = if count > 0 {
            handle_potential_webgl_error!(
                self,
                first.checked_add(count).map(|len| len as u32).ok_or(InvalidOperation),
                return
            )
        } else {
            0
        };

        handle_potential_webgl_error!(
            self,
            self.current_vao().validate_for_draw(required_len, primcount as u32, &current_program.active_attribs()),
            return
        );

        if !self.validate_framebuffer_complete() {
            return;
        }

        if count == 0 || primcount == 0 {
            return;
        }

        self.send_command(if primcount == 1 {
            WebGLCommand::DrawArrays { mode, first, count }
        } else {
            WebGLCommand::DrawArraysInstanced { mode, first, count, primcount }
        });
        self.mark_as_dirty();
    }

    // https://www.khronos.org/registry/webgl/extensions/ANGLE_instanced_arrays/
    pub fn draw_elements_instanced(
        &self,
        mode: u32,
        count: i32,
        type_: u32,
        offset: i64,
        primcount: i32,
    ) {
        match mode {
            constants::POINTS | constants::LINE_STRIP |
            constants::LINE_LOOP | constants::LINES |
            constants::TRIANGLE_STRIP | constants::TRIANGLE_FAN |
            constants::TRIANGLES => {},
            _ => {
                return self.webgl_error(InvalidEnum);
            }
        }
        if count < 0 || offset < 0 || primcount < 0 {
            return self.webgl_error(InvalidValue);
        }
        let type_size = match type_ {
            constants::UNSIGNED_BYTE => 1,
            constants::UNSIGNED_SHORT => 2,
            constants::UNSIGNED_INT if self.extension_manager.is_element_index_uint_enabled() => 4,
            _ => return self.webgl_error(InvalidEnum),
        };
        if offset % type_size != 0 {
            return self.webgl_error(InvalidOperation);
        }

        let current_program = handle_potential_webgl_error!(
            self,
            self.current_program.get().ok_or(InvalidOperation),
            return
        );

        if count > 0 && primcount > 0 {
            if let Some(array_buffer) = self.current_vao().element_array_buffer().get() {
                // This operation cannot overflow in u64 and we know all those values are nonnegative.
                let val = offset as u64 + (count as u64 * type_size as u64);
                if val > array_buffer.capacity() as u64 {
                    return self.webgl_error(InvalidOperation);
                }
            } else {
                return self.webgl_error(InvalidOperation);
            }
        }

        // TODO(nox): Pass the correct number of vertices required.
        handle_potential_webgl_error!(
            self,
            self.current_vao().validate_for_draw(0, primcount as u32, &current_program.active_attribs()),
            return
        );

        if !self.validate_framebuffer_complete() {
            return;
        }

        if count == 0 || primcount == 0 {
            return;
        }

        let offset = offset as u32;
        self.send_command(if primcount == 1 {
            WebGLCommand::DrawElements { mode, count, type_, offset }
        } else {
            WebGLCommand::DrawElementsInstanced { mode, count, type_, offset, primcount }
        });
        self.mark_as_dirty();
    }

    pub fn vertex_attrib_divisor(&self, index: u32, divisor: u32) {
        if index >= self.limits.max_vertex_attribs {
            return self.webgl_error(InvalidValue);
        }

        self.current_vao().vertex_attrib_divisor(index, divisor);
        self.send_command(WebGLCommand::VertexAttribDivisor { index, divisor });
    }

    // Used by HTMLCanvasElement.toDataURL
    //
    // This emits errors quite liberally, but the spec says that this operation
    // can fail and that it is UB what happens in that case.
    //
    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#2.2
    pub fn get_image_data(&self, mut width: u32, mut height: u32) -> Option<Vec<u8>> {
        if !self.validate_framebuffer_complete() {
            return None;
        }

        if let Some((fb_width, fb_height)) = self.get_current_framebuffer_size() {
            width = cmp::min(width, fb_width as u32);
            height = cmp::min(height, fb_height as u32);
        } else {
            self.webgl_error(InvalidOperation);
            return None;
        }

        let (sender, receiver) = webgl_channel().unwrap();
        self.send_command(WebGLCommand::ReadPixels(
            0,
            0,
            width as i32,
            height as i32,
            constants::RGBA,
            constants::UNSIGNED_BYTE,
            sender,
        ));
        Some(receiver.recv().unwrap().into())
    }

    pub fn array_buffer(&self) -> Option<DomRoot<WebGLBuffer>> {
        self.bound_buffer_array.get()
    }

    pub fn bound_buffer(&self, target: u32) -> WebGLResult<Option<DomRoot<WebGLBuffer>>> {
        match target {
            constants::ARRAY_BUFFER => Ok(self.bound_buffer_array.get()),
            constants::ELEMENT_ARRAY_BUFFER => Ok(self.current_vao().element_array_buffer().get()),
            _ => Err(WebGLError::InvalidEnum),
        }
    }

    pub fn create_vertex_array(&self) -> Option<DomRoot<WebGLVertexArrayObjectOES>> {
        let (sender, receiver) = webgl_channel().unwrap();
        self.send_command(WebGLCommand::CreateVertexArray(sender));
        receiver.recv().unwrap().map(|id| WebGLVertexArrayObjectOES::new(self, Some(id)))
    }

    pub fn delete_vertex_array(&self, vao: Option<&WebGLVertexArrayObjectOES>) {
        if let Some(vao) = vao {
            handle_potential_webgl_error!(self, self.validate_ownership(vao), return);
            // The default vertex array has no id and should never be passed around.
            assert!(vao.id().is_some());
            if vao.is_deleted() {
                return;
            }
            if vao == &*self.current_vao() {
                // Setting it to None will make self.current_vao() reset it to the default one
                // next time it is called.
                self.current_vao.set(None);
                self.send_command(WebGLCommand::BindVertexArray(None));
            }
            vao.delete();
        }
    }

    pub fn is_vertex_array(&self, vao: Option<&WebGLVertexArrayObjectOES>) -> bool {
        vao.map_or(false, |vao| {
            // The default vertex array has no id and should never be passed around.
            assert!(vao.id().is_some());
            self.validate_ownership(vao).is_ok() && vao.ever_bound() && !vao.is_deleted()
        })
    }

    pub fn bind_vertex_array(&self, vao: Option<&WebGLVertexArrayObjectOES>) {
        if let Some(vao) = vao {
            // The default vertex array has no id and should never be passed around.
            assert!(vao.id().is_some());
            handle_potential_webgl_error!(self, self.validate_ownership(vao), return);
            if vao.is_deleted() {
                return self.webgl_error(InvalidOperation);
            }
            vao.set_ever_bound();
        }
        self.send_command(WebGLCommand::BindVertexArray(vao.and_then(|vao| vao.id())));
        // Setting it to None will make self.current_vao() reset it to the default one
        // next time it is called.
        self.current_vao.set(vao);
    }

    fn validate_blend_mode(&self, mode: u32) -> WebGLResult<()> {
        match mode {
            constants::FUNC_ADD |
            constants::FUNC_SUBTRACT |
            constants::FUNC_REVERSE_SUBTRACT => {
                Ok(())
            }
            EXTBlendMinmaxConstants::MIN_EXT |
            EXTBlendMinmaxConstants::MAX_EXT if self.extension_manager.is_blend_minmax_enabled() => {
                Ok(())
            }
            _ => Err(InvalidEnum),
        }
    }
}

impl Drop for WebGLRenderingContext {
    fn drop(&mut self) {
        let _ = self.webgl_sender.send_remove();
    }
}

#[allow(unsafe_code)]
unsafe fn fallible_array_buffer_view_to_vec(
    cx: *mut JSContext,
    abv: *mut JSObject,
) -> Result<Vec<u8>, Error> {
    assert!(!abv.is_null());
    typedarray!(in(cx) let array_buffer_view: ArrayBufferView = abv);
    match array_buffer_view {
        Ok(v) => Ok(v.to_vec()),
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
    unsafe fn GetBufferParameter(
        &self,
        _cx: *mut JSContext,
        target: u32,
        parameter: u32,
    ) -> JSVal {
        let buffer = handle_potential_webgl_error!(
            self,
            self.bound_buffer(target).and_then(|buf| buf.ok_or(InvalidOperation)),
            return NullValue()
        );

        match parameter {
            constants::BUFFER_SIZE => Int32Value(buffer.capacity() as i32),
            constants::BUFFER_USAGE => Int32Value(buffer.usage() as i32),
            _ => {
                self.webgl_error(InvalidEnum);
                NullValue()
            }
        }
    }

    #[allow(unsafe_code)]
    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    unsafe fn GetParameter(&self, cx: *mut JSContext, parameter: u32) -> JSVal {
        if !self.extension_manager.is_get_parameter_name_enabled(parameter) {
            self.webgl_error(WebGLError::InvalidEnum);
            return NullValue();
        }

        match parameter {
            constants::ARRAY_BUFFER_BINDING => {
                return optional_root_object_to_js_or_null!(cx, &self.bound_buffer_array.get());
            }
            constants::CURRENT_PROGRAM => {
                return optional_root_object_to_js_or_null!(cx, &self.current_program.get());
            }
            constants::ELEMENT_ARRAY_BUFFER_BINDING => {
                let buffer = self.current_vao().element_array_buffer().get();
                return optional_root_object_to_js_or_null!(cx, buffer);
            }
            constants::FRAMEBUFFER_BINDING => {
                return optional_root_object_to_js_or_null!(cx, &self.bound_framebuffer.get());
            }
            constants::RENDERBUFFER_BINDING => {
                return optional_root_object_to_js_or_null!(cx, &self.bound_renderbuffer.get());
            }
            constants::TEXTURE_BINDING_2D => {
                let texture = self.bound_texture(constants::TEXTURE_2D);
                return optional_root_object_to_js_or_null!(cx, texture);
            }
            constants::TEXTURE_BINDING_CUBE_MAP => {
                let texture = self.bound_texture(constants::TEXTURE_CUBE_MAP);
                return optional_root_object_to_js_or_null!(cx, texture);
            }
            OESVertexArrayObjectConstants::VERTEX_ARRAY_BINDING_OES => {
                let vao = self.current_vao.get().filter(|vao| vao.id().is_some());
                return optional_root_object_to_js_or_null!(cx, vao);
            }
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
            constants::COMPRESSED_TEXTURE_FORMATS => {
                // FIXME(nox): https://github.com/servo/servo/issues/20594
                rooted!(in(cx) let mut rval = ptr::null_mut::<JSObject>());
                let _ = Uint32Array::create(
                    cx,
                    CreateWith::Slice(&[]),
                    rval.handle_mut(),
                ).unwrap();
                return ObjectValue(rval.get());
            }
            constants::VERSION => {
                rooted!(in(cx) let mut rval = UndefinedValue());
                "WebGL 1.0".to_jsval(cx, rval.handle_mut());
                return rval.get();
            }
            constants::RENDERER | constants::VENDOR => {
                rooted!(in(cx) let mut rval = UndefinedValue());
                "Mozilla/Servo".to_jsval(cx, rval.handle_mut());
                return rval.get();
            }
            constants::SHADING_LANGUAGE_VERSION => {
                rooted!(in(cx) let mut rval = UndefinedValue());
                "WebGL GLSL ES 1.0".to_jsval(cx, rval.handle_mut());
                return rval.get();
            }
            constants::UNPACK_FLIP_Y_WEBGL => {
                let unpack = self.texture_unpacking_settings.get();
                return BooleanValue(unpack.contains(TextureUnpacking::FLIP_Y_AXIS));
            }
            constants::UNPACK_PREMULTIPLY_ALPHA_WEBGL => {
                let unpack = self.texture_unpacking_settings.get();
                return BooleanValue(unpack.contains(TextureUnpacking::PREMULTIPLY_ALPHA));
            }
            _ => {}
        }

        // Handle any MAX_ parameters by retrieving the limits that were stored
        // when this context was created.
        let limit = match parameter {
            constants::MAX_VERTEX_ATTRIBS =>
                Some(self.limits.max_vertex_attribs),
            constants::MAX_TEXTURE_SIZE =>
                Some(self.limits.max_tex_size),
            constants::MAX_CUBE_MAP_TEXTURE_SIZE =>
                Some(self.limits.max_cube_map_tex_size),
            constants::MAX_COMBINED_TEXTURE_IMAGE_UNITS =>
                Some(self.limits.max_combined_texture_image_units),
            constants::MAX_FRAGMENT_UNIFORM_VECTORS =>
                Some(self.limits.max_fragment_uniform_vectors),
            constants::MAX_RENDERBUFFER_SIZE =>
                Some(self.limits.max_renderbuffer_size),
            constants::MAX_TEXTURE_IMAGE_UNITS =>
                Some(self.limits.max_texture_image_units),
            constants::MAX_VARYING_VECTORS =>
                Some(self.limits.max_varying_vectors),
            constants::MAX_VERTEX_TEXTURE_IMAGE_UNITS =>
                Some(self.limits.max_vertex_texture_image_units),
            constants::MAX_VERTEX_UNIFORM_VECTORS =>
                Some(self.limits.max_vertex_uniform_vectors),
            _ => None,
        };
        if let Some(limit) = limit {
            return UInt32Value(limit);
        }

        if let Ok(value) = self.capabilities.is_enabled(parameter) {
            return BooleanValue(value);
        }

        match handle_potential_webgl_error!(self, Parameter::from_u32(parameter), return NullValue()) {
            Parameter::Bool(param) => {
                let (sender, receiver) = webgl_channel().unwrap();
                self.send_command(WebGLCommand::GetParameterBool(param, sender));
                BooleanValue(receiver.recv().unwrap())
            }
            Parameter::Bool4(param) => {
                let (sender, receiver) = webgl_channel().unwrap();
                self.send_command(WebGLCommand::GetParameterBool4(param, sender));
                rooted!(in(cx) let mut rval = UndefinedValue());
                receiver.recv().unwrap().to_jsval(cx, rval.handle_mut());
                rval.get()
            }
            Parameter::Int(param) => {
                let (sender, receiver) = webgl_channel().unwrap();
                self.send_command(WebGLCommand::GetParameterInt(param, sender));
                Int32Value(receiver.recv().unwrap())
            }
            Parameter::Int2(param) => {
                let (sender, receiver) = webgl_channel().unwrap();
                self.send_command(WebGLCommand::GetParameterInt2(param, sender));
                rooted!(in(cx) let mut rval = ptr::null_mut::<JSObject>());
                let _ = Int32Array::create(
                    cx,
                    CreateWith::Slice(&receiver.recv().unwrap()),
                    rval.handle_mut(),
                ).unwrap();
                ObjectValue(rval.get())
            }
            Parameter::Int4(param) => {
                let (sender, receiver) = webgl_channel().unwrap();
                self.send_command(WebGLCommand::GetParameterInt4(param, sender));
                rooted!(in(cx) let mut rval = ptr::null_mut::<JSObject>());
                let _ = Int32Array::create(
                    cx,
                    CreateWith::Slice(&receiver.recv().unwrap()),
                    rval.handle_mut(),
                ).unwrap();
                ObjectValue(rval.get())
            }
            Parameter::Float(param) => {
                let (sender, receiver) = webgl_channel().unwrap();
                self.send_command(WebGLCommand::GetParameterFloat(param, sender));
                DoubleValue(receiver.recv().unwrap() as f64)
            }
            Parameter::Float2(param) => {
                let (sender, receiver) = webgl_channel().unwrap();
                self.send_command(WebGLCommand::GetParameterFloat2(param, sender));
                rooted!(in(cx) let mut rval = ptr::null_mut::<JSObject>());
                let _ = Float32Array::create(
                    cx,
                    CreateWith::Slice(&receiver.recv().unwrap()),
                    rval.handle_mut(),
                ).unwrap();
                ObjectValue(rval.get())
            }
            Parameter::Float4(param) => {
                let (sender, receiver) = webgl_channel().unwrap();
                self.send_command(WebGLCommand::GetParameterFloat4(param, sender));
                rooted!(in(cx) let mut rval = ptr::null_mut::<JSObject>());
                let _ = Float32Array::create(
                    cx,
                    CreateWith::Slice(&receiver.recv().unwrap()),
                    rval.handle_mut(),
                ).unwrap();
                ObjectValue(rval.get())
            }
        }
    }

    #[allow(unsafe_code)]
    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    unsafe fn GetTexParameter(&self, _cx: *mut JSContext, target: u32, pname: u32) -> JSVal {
        match target {
            constants::TEXTURE_2D |
            constants::TEXTURE_CUBE_MAP => {},
            _ => {
                self.webgl_error(InvalidEnum);
                return NullValue();
            }
        };

        if !self.extension_manager.is_get_tex_parameter_name_enabled(pname) {
            self.webgl_error(InvalidEnum);
            return NullValue();
        }

        let texture = match self.bound_texture(target) {
            Some(texture) => texture,
            None => {
                self.webgl_error(InvalidOperation);
                return NullValue();
            }
        };

        match pname {
            constants::TEXTURE_MAG_FILTER => return UInt32Value(texture.mag_filter()),
            constants::TEXTURE_MIN_FILTER => return UInt32Value(texture.min_filter()),
            _ => {}
        }

        match handle_potential_webgl_error!(self, TexParameter::from_u32(pname), return NullValue()) {
            TexParameter::Float(param) => {
                let (sender, receiver) = webgl_channel().unwrap();
                self.send_command(WebGLCommand::GetTexParameterFloat(target, param, sender));
                DoubleValue(receiver.recv().unwrap() as f64)
            }
            TexParameter::Int(param) => {
                let (sender, receiver) = webgl_channel().unwrap();
                self.send_command(WebGLCommand::GetTexParameterInt(target, param, sender));
                Int32Value(receiver.recv().unwrap())
            }
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
                    -> Option<NonNull<JSObject>> {
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
        handle_potential_webgl_error!(self, self.validate_blend_mode(mode), return);
        self.send_command(WebGLCommand::BlendEquation(mode))
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn BlendEquationSeparate(&self, mode_rgb: u32, mode_alpha: u32) {
        handle_potential_webgl_error!(self, self.validate_blend_mode(mode_rgb), return);
        handle_potential_webgl_error!(self, self.validate_blend_mode(mode_alpha), return);
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
    fn AttachShader(&self, program: &WebGLProgram, shader: &WebGLShader) {
        handle_potential_webgl_error!(self, self.validate_ownership(program), return);
        handle_potential_webgl_error!(self, self.validate_ownership(shader), return);
        handle_potential_webgl_error!(self, program.attach_shader(shader));
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn DetachShader(&self, program: &WebGLProgram, shader: &WebGLShader) {
        handle_potential_webgl_error!(self, self.validate_ownership(program), return);
        handle_potential_webgl_error!(self, self.validate_ownership(shader), return);
        handle_potential_webgl_error!(self, program.detach_shader(shader));
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn BindAttribLocation(&self, program: &WebGLProgram, index: u32, name: DOMString) {
        handle_potential_webgl_error!(self, self.validate_ownership(program), return);
        handle_potential_webgl_error!(self, program.bind_attrib_location(index, name));
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.5
    fn BindBuffer(&self, target: u32, buffer: Option<&WebGLBuffer>) {
        if let Some(buffer) = buffer {
            handle_potential_webgl_error!(self, self.validate_ownership(buffer), return);
        }

        let current_vao;
        let slot = match target {
            constants::ARRAY_BUFFER => {
                &self.bound_buffer_array
            }
            constants::ELEMENT_ARRAY_BUFFER => {
                current_vao = self.current_vao();
                current_vao.element_array_buffer()
            }
            _ => return self.webgl_error(InvalidEnum),
        };

        if let Some(buffer) = buffer {
            if buffer.is_marked_for_deletion() {
                return self.webgl_error(InvalidOperation);
            }
            handle_potential_webgl_error!(self, buffer.set_target(target), return);
            buffer.increment_attached_counter();
        }
        self.send_command(WebGLCommand::BindBuffer(target, buffer.map(|b| b.id())));
        if let Some(old) = slot.get() {
            old.decrement_attached_counter();
        }
        slot.set(buffer);
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.6
    fn BindFramebuffer(&self, target: u32, framebuffer: Option<&WebGLFramebuffer>) {
        if let Some(fb) = framebuffer {
            handle_potential_webgl_error!(self, self.validate_ownership(fb), return);
        }

        if target != constants::FRAMEBUFFER {
            return self.webgl_error(InvalidEnum);
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
        if let Some(rb) = renderbuffer {
            handle_potential_webgl_error!(self, self.validate_ownership(rb), return);
        }

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
        if let Some(texture) = texture {
            handle_potential_webgl_error!(self, self.validate_ownership(texture), return);
        }

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
    unsafe fn BufferData(
        &self,
        cx: *mut JSContext,
        target: u32,
        data: *mut JSObject,
        usage: u32,
    ) -> ErrorResult {
        if data.is_null() {
            return Ok(self.webgl_error(InvalidValue));
        }

        typedarray!(in(cx) let array_buffer: ArrayBuffer = data);
        let data_vec = match array_buffer {
            Ok(mut data) => data.to_vec(),
            Err(_) => fallible_array_buffer_view_to_vec(cx, data)?,
        };

        let bound_buffer = handle_potential_webgl_error!(self, self.bound_buffer(target), return Ok(()));
        let bound_buffer = match bound_buffer {
            Some(bound_buffer) => bound_buffer,
            None => return Ok(self.webgl_error(InvalidOperation)),
        };

        handle_potential_webgl_error!(self, bound_buffer.buffer_data(target, data_vec, usage));
        Ok(())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.5
    fn BufferData_(&self, target: u32, size: i64, usage: u32) -> ErrorResult {
        let bound_buffer = handle_potential_webgl_error!(self, self.bound_buffer(target), return Ok(()));
        let bound_buffer = match bound_buffer {
            Some(bound_buffer) => bound_buffer,
            None => return Ok(self.webgl_error(InvalidOperation)),
        };

        if size < 0 {
            return Ok(self.webgl_error(InvalidValue));
        }

        // FIXME: Allocating a buffer based on user-requested size is
        // not great, but we don't have a fallible allocation to try.
        let data = vec![0u8; size as usize];
        handle_potential_webgl_error!(self, bound_buffer.buffer_data(target, data, usage));
        Ok(())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.5
    fn BufferSubData(&self, target: u32, offset: i64, data: ArrayBufferViewOrArrayBuffer) {
        let data_vec = match data {
            // Typed array is rooted, so we can safely temporarily retrieve its slice
            ArrayBufferViewOrArrayBuffer::ArrayBuffer(mut inner) => inner.to_vec(),
            ArrayBufferViewOrArrayBuffer::ArrayBufferView(mut inner) => inner.to_vec(),
        };

        let bound_buffer = handle_potential_webgl_error!(self, self.bound_buffer(target), return);
        let bound_buffer = match bound_buffer {
            Some(bound_buffer) => bound_buffer,
            None => return self.webgl_error(InvalidOperation),
        };

        if offset < 0 {
            return self.webgl_error(InvalidValue);
        }

        if (offset as usize) + data_vec.len() > bound_buffer.capacity() {
            return self.webgl_error(InvalidValue);
        }
        self.send_command(WebGLCommand::BufferSubData(
            target,
            offset as isize,
            data_vec.into(),
        ));
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn CompressedTexImage2D(&self, _target: u32, _level: i32, _internal_format: u32,
                            _width: i32, _height: i32, _border: i32,
                            _data: CustomAutoRooterGuard<ArrayBufferView>) {
        // FIXME: No compressed texture format is currently supported, so error out as per
        // https://www.khronos.org/registry/webgl/specs/latest/1.0/#COMPRESSED_TEXTURE_SUPPORT
        self.webgl_error(InvalidEnum);
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn CompressedTexSubImage2D(&self, _target: u32, _level: i32, _xoffset: i32,
                                     _yoffset: i32, _width: i32, _height: i32, _format: u32,
                               _data: CustomAutoRooterGuard<ArrayBufferView>) {
        // FIXME: No compressed texture format is currently supported, so error out as per
        // https://www.khronos.org/registry/webgl/specs/latest/1.0/#COMPRESSED_TEXTURE_SUPPORT
        self.webgl_error(InvalidEnum);
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
        if mask & !(constants::DEPTH_BUFFER_BIT | constants::STENCIL_BUFFER_BIT | constants::COLOR_BUFFER_BIT) != 0 {
            return self.webgl_error(InvalidValue);
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
        self.send_command(WebGLCommand::ClearDepth(depth))
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
        // https://www.khronos.org/registry/webgl/specs/latest/1.0/#VIEWPORT_DEPTH_RANGE
        if near > far {
            return self.webgl_error(InvalidOperation);
        }
        self.send_command(WebGLCommand::DepthRange(near, far))
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn Enable(&self, cap: u32) {
        if handle_potential_webgl_error!(self, self.capabilities.set(cap, true), return) {
            self.send_command(WebGLCommand::Enable(cap));
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn Disable(&self, cap: u32) {
        if handle_potential_webgl_error!(self, self.capabilities.set(cap, false), return) {
            self.send_command(WebGLCommand::Disable(cap));
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn CompileShader(&self, shader: &WebGLShader) {
        handle_potential_webgl_error!(self, self.validate_ownership(shader), return);
        handle_potential_webgl_error!(
            self,
            shader.compile(
                self.webgl_version,
                self.glsl_version,
                &self.limits,
                &self.extension_manager,
            )
        )
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.5
    fn CreateBuffer(&self) -> Option<DomRoot<WebGLBuffer>> {
        WebGLBuffer::maybe_new(self)
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.6
    fn CreateFramebuffer(&self) -> Option<DomRoot<WebGLFramebuffer>> {
        WebGLFramebuffer::maybe_new(self)
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.7
    fn CreateRenderbuffer(&self) -> Option<DomRoot<WebGLRenderbuffer>> {
        WebGLRenderbuffer::maybe_new(self)
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn CreateTexture(&self) -> Option<DomRoot<WebGLTexture>> {
        WebGLTexture::maybe_new(self)
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn CreateProgram(&self) -> Option<DomRoot<WebGLProgram>> {
        WebGLProgram::maybe_new(self)
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
        WebGLShader::maybe_new(self, shader_type)
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.5
    fn DeleteBuffer(&self, buffer: Option<&WebGLBuffer>) {
        let buffer = match buffer {
            Some(buffer) => buffer,
            None => return,
        };
        handle_potential_webgl_error!(self, self.validate_ownership(buffer), return);
        if buffer.is_marked_for_deletion() {
            return;
        }
        self.current_vao().unbind_buffer(buffer);
        if self.bound_buffer_array.get().map_or(false, |b| buffer == &*b) {
            self.bound_buffer_array.set(None);
            buffer.decrement_attached_counter();
        }
        buffer.mark_for_deletion();
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.6
    fn DeleteFramebuffer(&self, framebuffer: Option<&WebGLFramebuffer>) {
        if let Some(framebuffer) = framebuffer {
            handle_potential_webgl_error!(self, self.validate_ownership(framebuffer), return);
            handle_object_deletion!(self, self.bound_framebuffer, framebuffer,
                                    Some(WebGLCommand::BindFramebuffer(constants::FRAMEBUFFER,
                                                                       WebGLFramebufferBindingRequest::Default)));
            framebuffer.delete()
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.7
    fn DeleteRenderbuffer(&self, renderbuffer: Option<&WebGLRenderbuffer>) {
        if let Some(renderbuffer) = renderbuffer {
            handle_potential_webgl_error!(self, self.validate_ownership(renderbuffer), return);
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
            handle_potential_webgl_error!(self, self.validate_ownership(texture), return);

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
            handle_potential_webgl_error!(self, self.validate_ownership(program), return);
            program.mark_for_deletion()
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn DeleteShader(&self, shader: Option<&WebGLShader>) {
        if let Some(shader) = shader {
            handle_potential_webgl_error!(self, self.validate_ownership(shader), return);
            shader.mark_for_deletion()
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.11
    fn DrawArrays(&self, mode: u32, first: i32, count: i32) {
        self.draw_arrays_instanced(mode, first, count, 1);
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.11
    fn DrawElements(&self, mode: u32, count: i32, type_: u32, offset: i64) {
        self.draw_elements_instanced(mode, count, type_, offset, 1);
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn EnableVertexAttribArray(&self, attrib_id: u32) {
        if attrib_id >= self.limits.max_vertex_attribs {
            return self.webgl_error(InvalidValue);
        }

        self.current_vao().enabled_vertex_attrib_array(attrib_id, true);
        self.send_command(WebGLCommand::EnableVertexAttribArray(attrib_id));
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn DisableVertexAttribArray(&self, attrib_id: u32) {
        if attrib_id >= self.limits.max_vertex_attribs {
            return self.webgl_error(InvalidValue);
        }

        self.current_vao().enabled_vertex_attrib_array(attrib_id, false);
        self.send_command(WebGLCommand::DisableVertexAttribArray(attrib_id));
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn GetActiveUniform(&self, program: &WebGLProgram, index: u32) -> Option<DomRoot<WebGLActiveInfo>> {
        handle_potential_webgl_error!(self, self.validate_ownership(program), return None);
        match program.get_active_uniform(index) {
            Ok(ret) => Some(ret),
            Err(e) => {
                self.webgl_error(e);
                return None;
            }
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn GetActiveAttrib(&self, program: &WebGLProgram, index: u32) -> Option<DomRoot<WebGLActiveInfo>> {
        handle_potential_webgl_error!(self, self.validate_ownership(program), return None);
        handle_potential_webgl_error!(self, program.get_active_attrib(index).map(Some), None)
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn GetAttribLocation(&self, program: &WebGLProgram, name: DOMString) -> i32 {
        handle_potential_webgl_error!(self, self.validate_ownership(program), return -1);
        handle_potential_webgl_error!(self, program.get_attrib_location(name), -1)
    }

    #[allow(unsafe_code)]
    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.6
    unsafe fn GetFramebufferAttachmentParameter(
        &self,
        cx: *mut JSContext,
        target: u32,
        attachment: u32,
        pname: u32
    ) -> JSVal {
        // Check if currently bound framebuffer is non-zero as per spec.
        if self.bound_framebuffer.get().is_none() {
            self.webgl_error(InvalidOperation);
            return NullValue();
        }

        // Note: commented out stuff is for the WebGL2 standard.
        let target_matches = match target {
            // constants::READ_FRAMEBUFFER |
            // constants::DRAW_FRAMEBUFFER => true,
            constants::FRAMEBUFFER => true,
            _ => false
        };
        let attachment_matches = match attachment {
            // constants::MAX_COLOR_ATTACHMENTS ... gl::COLOR_ATTACHMENT0 |
            // constants::BACK |
            constants::COLOR_ATTACHMENT0 |
            constants::DEPTH_STENCIL_ATTACHMENT |
            constants::DEPTH_ATTACHMENT |
            constants::STENCIL_ATTACHMENT => true,
            _ => false,
        };
        let pname_matches = match pname {
            // constants::FRAMEBUFFER_ATTACHMENT_ALPHA_SIZE |
            // constants::FRAMEBUFFER_ATTACHMENT_BLUE_SIZE |
            // constants::FRAMEBUFFER_ATTACHMENT_COLOR_ENCODING |
            // constants::FRAMEBUFFER_ATTACHMENT_COMPONENT_TYPE |
            // constants::FRAMEBUFFER_ATTACHMENT_DEPTH_SIZE |
            // constants::FRAMEBUFFER_ATTACHMENT_GREEN_SIZE |
            // constants::FRAMEBUFFER_ATTACHMENT_RED_SIZE |
            // constants::FRAMEBUFFER_ATTACHMENT_STENCIL_SIZE |
            // constants::FRAMEBUFFER_ATTACHMENT_TEXTURE_LAYER |
            constants::FRAMEBUFFER_ATTACHMENT_OBJECT_NAME |
            constants::FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE |
            constants::FRAMEBUFFER_ATTACHMENT_TEXTURE_CUBE_MAP_FACE |
            constants::FRAMEBUFFER_ATTACHMENT_TEXTURE_LEVEL => true,
            _ => false
        };

        let bound_attachment_matches = match self.bound_framebuffer.get().unwrap().attachment(attachment) {
            Some(attachment_root) => {
                match attachment_root {
                    WebGLFramebufferAttachmentRoot::Renderbuffer(_) => {
                        match pname {
                            constants::FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE |
                            constants::FRAMEBUFFER_ATTACHMENT_OBJECT_NAME => true,
                            _ => false
                        }
                    },
                    WebGLFramebufferAttachmentRoot::Texture(_) => {
                        match pname {
                            constants::FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE |
                            constants::FRAMEBUFFER_ATTACHMENT_OBJECT_NAME |
                            constants::FRAMEBUFFER_ATTACHMENT_TEXTURE_LEVEL |
                            constants::FRAMEBUFFER_ATTACHMENT_TEXTURE_CUBE_MAP_FACE => true,
                            _ => false
                        }
                    }
                }
            },
            _ => {
                match pname {
                    constants::FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE => true,
                    _ => false
                }
            }
        };

        if !target_matches || !attachment_matches || !pname_matches || !bound_attachment_matches {
            self.webgl_error(InvalidEnum);
            return NullValue();
        }

        // From the GLES2 spec:
        //
        //     If the value of FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE is NONE,
        //     then querying any other pname will generate INVALID_ENUM.
        //
        // otherwise, return `WebGLRenderbuffer` or `WebGLTexture` dom object
        if pname == constants::FRAMEBUFFER_ATTACHMENT_OBJECT_NAME {
            // if fb is None, an INVALID_OPERATION is returned
            // at the beggining of the function, so `.unwrap()` will never panic
            let fb = self.bound_framebuffer.get().unwrap();
            if let Some(webgl_attachment) = fb.attachment(attachment) {
                match webgl_attachment {
                    WebGLFramebufferAttachmentRoot::Renderbuffer(rb) => {
                        rooted!(in(cx) let mut rval = NullValue());
                        rb.to_jsval(cx, rval.handle_mut());
                        return rval.get();
                    },
                    WebGLFramebufferAttachmentRoot::Texture(texture) => {
                        rooted!(in(cx) let mut rval = NullValue());
                        texture.to_jsval(cx, rval.handle_mut());
                        return rval.get();
                    },
                }
            }
            self.webgl_error(InvalidEnum);
            return NullValue();
        }

        let (sender, receiver) = webgl_channel().unwrap();
        self.send_command(WebGLCommand::GetFramebufferAttachmentParameter(target, attachment, pname, sender));

        Int32Value(receiver.recv().unwrap())
    }

    #[allow(unsafe_code)]
    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.7
    unsafe fn GetRenderbufferParameter(
        &self,
        _cx: *mut JSContext,
        target: u32,
        pname: u32
    ) -> JSVal {
        let target_matches = target == constants::RENDERBUFFER;

        let pname_matches = match pname {
            constants::RENDERBUFFER_WIDTH |
            constants::RENDERBUFFER_HEIGHT |
            constants::RENDERBUFFER_INTERNAL_FORMAT |
            constants::RENDERBUFFER_RED_SIZE |
            constants::RENDERBUFFER_GREEN_SIZE |
            constants::RENDERBUFFER_BLUE_SIZE |
            constants::RENDERBUFFER_ALPHA_SIZE |
            constants::RENDERBUFFER_DEPTH_SIZE |
            constants::RENDERBUFFER_STENCIL_SIZE => true,
            _ => false,
        };

        if !target_matches || !pname_matches {
            self.webgl_error(InvalidEnum);
            return NullValue();
        }

        if self.bound_renderbuffer.get().is_none() {
            self.webgl_error(InvalidOperation);
            return NullValue();
        }

        let result = if pname == constants::RENDERBUFFER_INTERNAL_FORMAT {
            let rb = self.bound_renderbuffer.get().unwrap();
            rb.internal_format() as i32
        } else {
            let (sender, receiver) = webgl_channel().unwrap();
            self.send_command(WebGLCommand::GetRenderbufferParameter(target, pname, sender));
            receiver.recv().unwrap()
        };


        Int32Value(result)
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn GetProgramInfoLog(&self, program: &WebGLProgram) -> Option<DOMString> {
        handle_potential_webgl_error!(self, self.validate_ownership(program), return None);
        match program.get_info_log() {
            Ok(value) => Some(DOMString::from(value)),
            Err(e) => {
                self.webgl_error(e);
                None
            }
        }
    }

    #[allow(unsafe_code)]
    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    unsafe fn GetProgramParameter(&self, _: *mut JSContext, program: &WebGLProgram, param: u32) -> JSVal {
        handle_potential_webgl_error!(self, self.validate_ownership(program), return NullValue());
        if program.is_deleted() {
            self.webgl_error(InvalidOperation);
            return NullValue();
        }
        match param {
            constants::DELETE_STATUS => BooleanValue(program.is_marked_for_deletion()),
            constants::LINK_STATUS => BooleanValue(program.is_linked()),
            constants::VALIDATE_STATUS => {
                // FIXME(nox): This could be cached on the DOM side when we call validateProgram
                // but I'm not sure when the value should be reset.
                let (sender, receiver) = webgl_channel().unwrap();
                self.send_command(WebGLCommand::GetProgramValidateStatus(program.id(), sender));
                BooleanValue(receiver.recv().unwrap())
            }
            constants::ATTACHED_SHADERS => {
                // FIXME(nox): This allocates a vector and roots a couple of shaders for nothing.
                Int32Value(program.attached_shaders().map(|shaders| shaders.len() as i32).unwrap_or(0))
            }
            constants::ACTIVE_ATTRIBUTES => Int32Value(program.active_attribs().len() as i32),
            constants::ACTIVE_UNIFORMS => Int32Value(program.active_uniforms().len() as i32),
            _ => {
                self.webgl_error(InvalidEnum);
                NullValue()
            }
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn GetShaderInfoLog(&self, shader: &WebGLShader) -> Option<DOMString> {
        handle_potential_webgl_error!(self, self.validate_ownership(shader), return None);
        Some(shader.info_log())
    }

    #[allow(unsafe_code)]
    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    unsafe fn GetShaderParameter(&self, _: *mut JSContext, shader: &WebGLShader, param: u32) -> JSVal {
        handle_potential_webgl_error!(self, self.validate_ownership(shader), return NullValue());
        if shader.is_deleted() {
            self.webgl_error(InvalidValue);
            return NullValue();
        }
        match param {
            constants::DELETE_STATUS => BooleanValue(shader.is_marked_for_deletion()),
            constants::COMPILE_STATUS => BooleanValue(shader.successfully_compiled()),
            constants::SHADER_TYPE => UInt32Value(shader.gl_type()),
            _ => {
                self.webgl_error(InvalidEnum);
                NullValue()
            }
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn GetShaderPrecisionFormat(
        &self,
        shader_type: u32,
        precision_type: u32
    ) -> Option<DomRoot<WebGLShaderPrecisionFormat>> {
        match precision_type {
            constants::LOW_FLOAT |
            constants::MEDIUM_FLOAT |
            constants::HIGH_FLOAT |
            constants::LOW_INT |
            constants::MEDIUM_INT |
            constants::HIGH_INT => (),
            _ => {
                self.webgl_error(InvalidEnum);
                return None;
            },
        }

        let (sender, receiver) = webgl_channel().unwrap();
        self.send_command(WebGLCommand::GetShaderPrecisionFormat(shader_type,
                                                                 precision_type,
                                                                 sender));

        let (range_min, range_max, precision) = receiver.recv().unwrap();
        Some(WebGLShaderPrecisionFormat::new(self.global().as_window(), range_min, range_max, precision))
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn GetUniformLocation(
        &self,
        program: &WebGLProgram,
        name: DOMString,
    ) -> Option<DomRoot<WebGLUniformLocation>> {
        handle_potential_webgl_error!(self, self.validate_ownership(program), return None);
        handle_potential_webgl_error!(self, program.get_uniform_location(name), None)
    }

    #[allow(unsafe_code)]
    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    unsafe fn GetVertexAttrib(&self, cx: *mut JSContext, index: u32, param: u32) -> JSVal {
        let current_vao = self.current_vao();
        let data = handle_potential_webgl_error!(
            self,
            current_vao.get_vertex_attrib(index).ok_or(InvalidValue),
            return NullValue()
        );
        if param == constants::CURRENT_VERTEX_ATTRIB {
            let value = if index == 0 {
                let (x, y, z, w) = self.current_vertex_attrib_0.get();
                [x, y, z, w]
            } else {
                let (sender, receiver) = webgl_channel().unwrap();
                self.send_command(WebGLCommand::GetCurrentVertexAttrib(index, sender));
                receiver.recv().unwrap()
            };
            rooted!(in(cx) let mut result = ptr::null_mut::<JSObject>());
            let _ = Float32Array::create(
                cx,
                CreateWith::Slice(&value),
                result.handle_mut(),
            ).unwrap();
            return ObjectValue(result.get());
        }

        if !self.extension_manager.is_get_vertex_attrib_name_enabled(param) {
            self.webgl_error(WebGLError::InvalidEnum);
            return NullValue();
        }

        match param {
            constants::VERTEX_ATTRIB_ARRAY_ENABLED => BooleanValue(data.enabled_as_array),
            constants::VERTEX_ATTRIB_ARRAY_SIZE => Int32Value(data.size as i32),
            constants::VERTEX_ATTRIB_ARRAY_TYPE => Int32Value(data.type_ as i32),
            constants::VERTEX_ATTRIB_ARRAY_NORMALIZED => BooleanValue(data.normalized),
            constants::VERTEX_ATTRIB_ARRAY_STRIDE => Int32Value(data.stride as i32),
            constants::VERTEX_ATTRIB_ARRAY_BUFFER_BINDING => {
                rooted!(in(cx) let mut jsval = NullValue());
                if let Some(buffer) = data.buffer() {
                    buffer.to_jsval(cx, jsval.handle_mut());
                }
                jsval.get()
            }
            ANGLEInstancedArraysConstants::VERTEX_ATTRIB_ARRAY_DIVISOR_ANGLE => {
                UInt32Value(data.divisor)
            }
            _ => {
                self.webgl_error(InvalidEnum);
                NullValue()
            }
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn GetVertexAttribOffset(&self, index: u32, pname: u32) -> i64 {
        if pname != constants::VERTEX_ATTRIB_ARRAY_POINTER {
            self.webgl_error(InvalidEnum);
            return 0;
        }
        let vao = self.current_vao();
        let data = handle_potential_webgl_error!(
            self,
            vao.get_vertex_attrib(index).ok_or(InvalidValue),
            return 0
        );
        data.offset as i64
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
        buffer.map_or(false, |buf| {
            self.validate_ownership(buf).is_ok() && buf.target().is_some() && !buf.is_marked_for_deletion()
        })
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn IsEnabled(&self, cap: u32) -> bool {
        handle_potential_webgl_error!(self, self.capabilities.is_enabled(cap), false)
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.6
    fn IsFramebuffer(&self, frame_buffer: Option<&WebGLFramebuffer>) -> bool {
        frame_buffer.map_or(false, |buf| {
            self.validate_ownership(buf).is_ok() && buf.target().is_some() && !buf.is_deleted()
        })
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn IsProgram(&self, program: Option<&WebGLProgram>) -> bool {
        program.map_or(false, |p| self.validate_ownership(p).is_ok() && !p.is_deleted())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.7
    fn IsRenderbuffer(&self, render_buffer: Option<&WebGLRenderbuffer>) -> bool {
        render_buffer.map_or(false, |buf| {
            self.validate_ownership(buf).is_ok() && buf.ever_bound() && !buf.is_deleted()
        })
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn IsShader(&self, shader: Option<&WebGLShader>) -> bool {
        shader.map_or(false, |s| self.validate_ownership(s).is_ok() && !s.is_deleted())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn IsTexture(&self, texture: Option<&WebGLTexture>) -> bool {
        texture.map_or(false, |tex| {
            self.validate_ownership(tex).is_ok() && tex.target().is_some() && !tex.is_deleted()
        })
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
                    texture_settings.insert(TextureUnpacking::FLIP_Y_AXIS)
                } else {
                    texture_settings.remove(TextureUnpacking::FLIP_Y_AXIS)
                }

                self.texture_unpacking_settings.set(texture_settings);
                return;
            },
            constants::UNPACK_PREMULTIPLY_ALPHA_WEBGL => {
                if param_value != 0 {
                    texture_settings.insert(TextureUnpacking::PREMULTIPLY_ALPHA)
                } else {
                    texture_settings.remove(TextureUnpacking::PREMULTIPLY_ALPHA)
                }

                self.texture_unpacking_settings.set(texture_settings);
                return;
            },
            constants::UNPACK_COLORSPACE_CONVERSION_WEBGL => {
                match param_value as u32 {
                    constants::BROWSER_DEFAULT_WEBGL
                        => texture_settings.insert(TextureUnpacking::CONVERT_COLORSPACE),
                    constants::NONE
                        => texture_settings.remove(TextureUnpacking::CONVERT_COLORSPACE),
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

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.12
    #[allow(unsafe_code)]
    fn ReadPixels(&self, x: i32, y: i32, width: i32, height: i32, format: u32, pixel_type: u32,
                  mut pixels: CustomAutoRooterGuard<Option<ArrayBufferView>>) {
        let (array_type, data) = match *pixels {
            // Spec: If data is null then an INVALID_VALUE error is generated.
            None => return self.webgl_error(InvalidValue),
            // The typed array is rooted and we should have a unique reference to it,
            // so retrieving its mutable slice is safe here
            Some(ref mut data) => (data.get_array_type(), unsafe { data.as_mut_slice() }),
        };

        if !self.validate_framebuffer_complete() {
            return;
        }

        match array_type {
            Type::Uint8 => (),
            _ => return self.webgl_error(InvalidOperation),
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
            return self.webgl_error(InvalidOperation);
        }
        let cpp = 4;

        //     "If pixels is non-null, but is not large enough to
        //      retrieve all of the pixels in the specified rectangle
        //      taking into account pixel store modes, an
        //      INVALID_OPERATION error is generated."
        let stride = match width.checked_mul(cpp) {
            Some(stride) => stride,
            _ => return self.webgl_error(InvalidOperation),
        };

        match height.checked_mul(stride) {
            Some(size) if size <= data.len() as i32 => {}
            _ => return self.webgl_error(InvalidOperation),
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
            return self.webgl_error(InvalidValue);
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
            _ => return self.webgl_error(InvalidOperation),
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
    fn LinkProgram(&self, program: &WebGLProgram) {
        handle_potential_webgl_error!(self, self.validate_ownership(program), return);
        if program.is_deleted() {
            return self.webgl_error(InvalidValue);
        }
        handle_potential_webgl_error!(self, program.link());
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn ShaderSource(&self, shader: &WebGLShader, source: DOMString) {
        handle_potential_webgl_error!(self, self.validate_ownership(shader), return);
        shader.set_source(source)
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn GetShaderSource(&self, shader: &WebGLShader) -> Option<DOMString> {
        handle_potential_webgl_error!(self, self.validate_ownership(shader), return None);
        Some(shader.source())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform1f(
        &self,
        location: Option<&WebGLUniformLocation>,
        val: f32,
    ) {
        self.with_location(location, |location| {
            match location.type_() {
                constants::BOOL | constants::FLOAT => {}
                _ => return Err(InvalidOperation),
            }
            self.send_command(WebGLCommand::Uniform1f(location.id(), val));
            Ok(())
        });
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform1i(
        &self,
        location: Option<&WebGLUniformLocation>,
        val: i32,
    ) {
        self.with_location(location, |location| {
            match location.type_() {
                constants::BOOL | constants::INT => {}
                constants::SAMPLER_2D | constants::SAMPLER_CUBE => {
                    if val < 0 || val as u32 >= self.limits.max_combined_texture_image_units {
                        return Err(InvalidValue);
                    }
                }
                _ => return Err(InvalidOperation),
            }
            self.send_command(WebGLCommand::Uniform1i(location.id(), val));
            Ok(())
        });
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform1iv(
        &self,
        location: Option<&WebGLUniformLocation>,
        val: Int32ArrayOrLongSequence,
    ) {
        self.with_location(location, |location| {
            match location.type_() {
                constants::BOOL | constants::INT | constants::SAMPLER_2D | constants::SAMPLER_CUBE => {}
                _ => return Err(InvalidOperation),
            }
            let val = match val {
                Int32ArrayOrLongSequence::Int32Array(v) => v.to_vec(),
                Int32ArrayOrLongSequence::LongSequence(v) => v,
            };
            if val.is_empty() {
                return Err(InvalidValue);
            }
            if location.size().is_none() && val.len() != 1 {
                return Err(InvalidOperation);
            }
            match location.type_() {
                constants::SAMPLER_2D | constants::SAMPLER_CUBE => {
                    for &v in val.iter().take(cmp::min(location.size().unwrap_or(1) as usize, val.len())) {
                        if v < 0 || v as u32 >= self.limits.max_combined_texture_image_units {
                            return Err(InvalidValue);
                        }
                    }
                }
                _ => {}
            }
            self.send_command(WebGLCommand::Uniform1iv(location.id(), val));
            Ok(())
        });
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform1fv(
        &self,
        location: Option<&WebGLUniformLocation>,
        val: Float32ArrayOrUnrestrictedFloatSequence,
    ) {
        self.with_location(location, |location| {
            match location.type_() {
                constants::BOOL | constants::FLOAT => {}
                _ => return Err(InvalidOperation),
            }
            let val = match val {
                Float32ArrayOrUnrestrictedFloatSequence::Float32Array(v) => v.to_vec(),
                Float32ArrayOrUnrestrictedFloatSequence::UnrestrictedFloatSequence(v) => v,
            };
            if val.is_empty() {
                return Err(InvalidValue);
            }
            if location.size().is_none() && val.len() != 1 {
                return Err(InvalidOperation);
            }
            self.send_command(WebGLCommand::Uniform1fv(location.id(), val));
            Ok(())
        });
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform2f(
        &self,
        location: Option<&WebGLUniformLocation>,
        x: f32,
        y: f32,
    ) {
        self.with_location(location, |location| {
            match location.type_() {
                constants::BOOL_VEC2 | constants::FLOAT_VEC2 => {}
                _ => return Err(InvalidOperation),
            }
            self.send_command(WebGLCommand::Uniform2f(location.id(), x, y));
            Ok(())
        });
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform2fv(
        &self,
        location: Option<&WebGLUniformLocation>,
        val: Float32ArrayOrUnrestrictedFloatSequence,
    ) {
        self.with_location(location, |location| {
            match location.type_() {
                constants::BOOL_VEC2 | constants::FLOAT_VEC2 => {}
                _ => return Err(InvalidOperation),
            }
            let val = match val {
                Float32ArrayOrUnrestrictedFloatSequence::Float32Array(v) => v.to_vec(),
                Float32ArrayOrUnrestrictedFloatSequence::UnrestrictedFloatSequence(v) => v,
            };
            if val.len() < 2 || val.len() % 2 != 0 {
                return Err(InvalidValue);
            }
            if location.size().is_none() && val.len() != 2 {
                return Err(InvalidOperation);
            }
            self.send_command(WebGLCommand::Uniform2fv(location.id(), val));
            Ok(())
        });
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform2i(
        &self,
        location: Option<&WebGLUniformLocation>,
        x: i32,
        y: i32,
    ) {
        self.with_location(location, |location| {
            match location.type_() {
                constants::BOOL_VEC2 | constants::INT_VEC2 => {}
                _ => return Err(InvalidOperation),
            }
            self.send_command(WebGLCommand::Uniform2i(location.id(), x, y));
            Ok(())
        });
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform2iv(
        &self,
        location: Option<&WebGLUniformLocation>,
        val: Int32ArrayOrLongSequence,
    ) {
        self.with_location(location, |location| {
            match location.type_() {
                constants::BOOL_VEC2 | constants::INT_VEC2 => {}
                _ => return Err(InvalidOperation),
            }
            let val = match val {
                Int32ArrayOrLongSequence::Int32Array(v) => v.to_vec(),
                Int32ArrayOrLongSequence::LongSequence(v) => v,
            };
            if val.len() < 2 || val.len() % 2 != 0 {
                return Err(InvalidValue);
            }
            if location.size().is_none() && val.len() != 2 {
                return Err(InvalidOperation);
            }
            self.send_command(WebGLCommand::Uniform2iv(location.id(), val));
            Ok(())
        });
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform3f(
        &self,
        location: Option<&WebGLUniformLocation>,
        x: f32,
        y: f32,
        z: f32,
    ) {
        self.with_location(location, |location| {
            match location.type_() {
                constants::BOOL_VEC3 | constants::FLOAT_VEC3 => {}
                _ => return Err(InvalidOperation),
            }
            self.send_command(WebGLCommand::Uniform3f(location.id(), x, y, z));
            Ok(())
        });
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform3fv(
        &self,
        location: Option<&WebGLUniformLocation>,
        val: Float32ArrayOrUnrestrictedFloatSequence,
    ) {
        self.with_location(location, |location| {
            match location.type_() {
                constants::BOOL_VEC3 | constants::FLOAT_VEC3 => {}
                _ => return Err(InvalidOperation),
            }
            let val = match val {
                Float32ArrayOrUnrestrictedFloatSequence::Float32Array(v) => v.to_vec(),
                Float32ArrayOrUnrestrictedFloatSequence::UnrestrictedFloatSequence(v) => v,
            };
            if val.len() < 3 || val.len() % 3 != 0 {
                return Err(InvalidValue);
            }
            if location.size().is_none() && val.len() != 3 {
                return Err(InvalidOperation);
            }
            self.send_command(WebGLCommand::Uniform3fv(location.id(), val));
            Ok(())
        });
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform3i(
        &self,
        location: Option<&WebGLUniformLocation>,
        x: i32,
        y: i32,
        z: i32,
    ) {
        self.with_location(location, |location| {
            match location.type_() {
                constants::BOOL_VEC3 | constants::INT_VEC3 => {}
                _ => return Err(InvalidOperation),
            }
            self.send_command(WebGLCommand::Uniform3i(location.id(), x, y, z));
            Ok(())
        });
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform3iv(
        &self,
        location: Option<&WebGLUniformLocation>,
        val: Int32ArrayOrLongSequence,
    ) {
        self.with_location(location, |location| {
            match location.type_() {
                constants::BOOL_VEC3 | constants::INT_VEC3 => {}
                _ => return Err(InvalidOperation),
            }
            let val = match val {
                Int32ArrayOrLongSequence::Int32Array(v) => v.to_vec(),
                Int32ArrayOrLongSequence::LongSequence(v) => v,
            };
            if val.len() < 3 || val.len() % 3 != 0 {
                return Err(InvalidValue);
            }
            if location.size().is_none() && val.len() != 3 {
                return Err(InvalidOperation);
            }
            self.send_command(WebGLCommand::Uniform3iv(location.id(), val));
            Ok(())
        });
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform4i(
        &self,
        location: Option<&WebGLUniformLocation>,
        x: i32,
        y: i32,
        z: i32,
        w: i32,
    ) {
        self.with_location(location, |location| {
            match location.type_() {
                constants::BOOL_VEC4 | constants::INT_VEC4 => {}
                _ => return Err(InvalidOperation),
            }
            self.send_command(WebGLCommand::Uniform4i(location.id(), x, y, z, w));
            Ok(())
        });
    }


    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform4iv(
        &self,
        location: Option<&WebGLUniformLocation>,
        val: Int32ArrayOrLongSequence,
    ) {
        self.with_location(location, |location| {
            match location.type_() {
                constants::BOOL_VEC4 | constants::INT_VEC4 => {}
                _ => return Err(InvalidOperation),
            }
            let val = match val {
                Int32ArrayOrLongSequence::Int32Array(v) => v.to_vec(),
                Int32ArrayOrLongSequence::LongSequence(v) => v,
            };
            if val.len() < 4 || val.len() % 4 != 0 {
                return Err(InvalidValue);
            }
            if location.size().is_none() && val.len() != 4 {
                return Err(InvalidOperation);
            }
            self.send_command(WebGLCommand::Uniform4iv(location.id(), val));
            Ok(())
        });
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform4f(
        &self,
        location: Option<&WebGLUniformLocation>,
        x: f32,
        y: f32,
        z: f32,
        w: f32,
    ) {
        self.with_location(location, |location| {
            match location.type_() {
                constants::BOOL_VEC4 | constants::FLOAT_VEC4 => {}
                _ => return Err(InvalidOperation),
            }
            self.send_command(WebGLCommand::Uniform4f(location.id(), x, y, z, w));
            Ok(())
        });
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform4fv(
        &self,
        location: Option<&WebGLUniformLocation>,
        val: Float32ArrayOrUnrestrictedFloatSequence,
    ) {
        self.with_location(location, |location| {
            match location.type_() {
                constants::BOOL_VEC4 | constants::FLOAT_VEC4 => {}
                _ => return Err(InvalidOperation),
            }
            let val = match val {
                Float32ArrayOrUnrestrictedFloatSequence::Float32Array(v) => v.to_vec(),
                Float32ArrayOrUnrestrictedFloatSequence::UnrestrictedFloatSequence(v) => v,
            };
            if val.len() < 4 || val.len() % 4 != 0 {
                return Err(InvalidValue);
            }
            if location.size().is_none() && val.len() != 4 {
                return Err(InvalidOperation);
            }
            self.send_command(WebGLCommand::Uniform4fv(location.id(), val));
            Ok(())
        });
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn UniformMatrix2fv(
        &self,
        location: Option<&WebGLUniformLocation>,
        transpose: bool,
        val: Float32ArrayOrUnrestrictedFloatSequence,
    ) {
        self.with_location(location, |location| {
            match location.type_() {
                constants::FLOAT_MAT2 => {}
                _ => return Err(InvalidOperation),
            }
            let val = match val {
                Float32ArrayOrUnrestrictedFloatSequence::Float32Array(v) => v.to_vec(),
                Float32ArrayOrUnrestrictedFloatSequence::UnrestrictedFloatSequence(v) => v,
            };
            if transpose {
                return Err(InvalidValue);
            }
            if val.len() < 4 || val.len() % 4 != 0 {
                return Err(InvalidValue);
            }
            if location.size().is_none() && val.len() != 4 {
                return Err(InvalidOperation);
            }
            self.send_command(WebGLCommand::UniformMatrix2fv(location.id(), val));
            Ok(())
        });
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn UniformMatrix3fv(
        &self,
        location: Option<&WebGLUniformLocation>,
        transpose: bool,
        val: Float32ArrayOrUnrestrictedFloatSequence,
    ) {
        self.with_location(location, |location| {
            match location.type_() {
                constants::FLOAT_MAT3 => {}
                _ => return Err(InvalidOperation),
            }
            let val = match val {
                Float32ArrayOrUnrestrictedFloatSequence::Float32Array(v) => v.to_vec(),
                Float32ArrayOrUnrestrictedFloatSequence::UnrestrictedFloatSequence(v) => v,
            };
            if transpose {
                return Err(InvalidValue);
            }
            if val.len() < 9 || val.len() % 9 != 0 {
                return Err(InvalidValue);
            }
            if location.size().is_none() && val.len() != 9 {
                return Err(InvalidOperation);
            }
            self.send_command(WebGLCommand::UniformMatrix3fv(location.id(), val));
            Ok(())
        });
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn UniformMatrix4fv(
        &self,
        location: Option<&WebGLUniformLocation>,
        transpose: bool,
        val: Float32ArrayOrUnrestrictedFloatSequence,
    ) {
        self.with_location(location, |location| {
            match location.type_() {
                constants::FLOAT_MAT4 => {}
                _ => return Err(InvalidOperation),
            }
            let val = match val {
                Float32ArrayOrUnrestrictedFloatSequence::Float32Array(v) => v.to_vec(),
                Float32ArrayOrUnrestrictedFloatSequence::UnrestrictedFloatSequence(v) => v,
            };
            if transpose {
                return Err(InvalidValue);
            }
            if val.len() < 16 || val.len() % 16 != 0 {
                return Err(InvalidValue);
            }
            if location.size().is_none() && val.len() != 16 {
                return Err(InvalidOperation);
            }
            self.send_command(WebGLCommand::UniformMatrix4fv(location.id(), val));
            Ok(())
        });
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    #[allow(unsafe_code)]
    unsafe fn GetUniform(
        &self,
        cx: *mut JSContext,
        program: &WebGLProgram,
        location: &WebGLUniformLocation,
    ) -> JSVal {
        handle_potential_webgl_error!(self, self.validate_ownership(program), return NullValue());

        if
            program.is_deleted() ||
            !program.is_linked() ||
            program.id() != location.program_id() ||
            program.link_generation() != location.link_generation()
        {
            self.webgl_error(InvalidOperation);
            return NullValue();
        }

        fn get<T, F>(
            triple: (&WebGLRenderingContext, WebGLProgramId, i32),
            f: F,
        ) -> T
        where
            F: FnOnce(WebGLProgramId, i32, WebGLSender<T>) -> WebGLCommand,
            T: for<'de> Deserialize<'de> + Serialize,
        {
            let (sender, receiver) = webgl_channel().unwrap();
            triple.0.send_command(f(triple.1, triple.2, sender));
            receiver.recv().unwrap()
        }

        let triple = (self, program.id(), location.id());

        unsafe fn typed<T>(cx: *mut JSContext, value: &[T::Element]) -> JSVal
        where
            T: TypedArrayElementCreator,
        {
            rooted!(in(cx) let mut rval = ptr::null_mut::<JSObject>());
            <TypedArray<T, *mut JSObject>>::create(cx, CreateWith::Slice(&value), rval.handle_mut()).unwrap();
            ObjectValue(rval.get())
        }

        match location.type_() {
            constants::BOOL => BooleanValue(get(triple, WebGLCommand::GetUniformBool)),
            constants::BOOL_VEC2 => {
                rooted!(in(cx) let mut rval = NullValue());
                get(triple, WebGLCommand::GetUniformBool2).to_jsval(cx, rval.handle_mut());
                rval.get()
            }
            constants::BOOL_VEC3 => {
                rooted!(in(cx) let mut rval = NullValue());
                get(triple, WebGLCommand::GetUniformBool3).to_jsval(cx, rval.handle_mut());
                rval.get()
            }
            constants::BOOL_VEC4 => {
                rooted!(in(cx) let mut rval = NullValue());
                get(triple, WebGLCommand::GetUniformBool4).to_jsval(cx, rval.handle_mut());
                rval.get()
            }
            constants::INT | constants::SAMPLER_2D | constants::SAMPLER_CUBE => {
                Int32Value(get(triple, WebGLCommand::GetUniformInt))
            }
            constants::INT_VEC2 => typed::<Int32>(cx, &get(triple, WebGLCommand::GetUniformInt2)),
            constants::INT_VEC3 => typed::<Int32>(cx, &get(triple, WebGLCommand::GetUniformInt3)),
            constants::INT_VEC4 => typed::<Int32>(cx, &get(triple, WebGLCommand::GetUniformInt4)),
            constants::FLOAT => DoubleValue(get(triple, WebGLCommand::GetUniformFloat) as f64),
            constants::FLOAT_VEC2 => typed::<Float32>(cx, &get(triple, WebGLCommand::GetUniformFloat2)),
            constants::FLOAT_VEC3 => typed::<Float32>(cx, &get(triple, WebGLCommand::GetUniformFloat3)),
            constants::FLOAT_VEC4 | constants::FLOAT_MAT2 => {
                typed::<Float32>(cx, &get(triple, WebGLCommand::GetUniformFloat4))
            }
            constants::FLOAT_MAT3 => typed::<Float32>(cx, &get(triple, WebGLCommand::GetUniformFloat9)),
            constants::FLOAT_MAT4 => typed::<Float32>(cx, &get(triple, WebGLCommand::GetUniformFloat16)),
            _ => panic!("wrong uniform type"),
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn UseProgram(&self, program: Option<&WebGLProgram>) {
        if let Some(program) = program {
            handle_potential_webgl_error!(self, self.validate_ownership(program), return);
            if program.is_deleted() || !program.is_linked() {
                return self.webgl_error(InvalidOperation);
            }
            if program.is_in_use() {
                return;
            }
            program.in_use(true);
        }
        match self.current_program.get() {
            Some(ref current) if program != Some(&**current) => current.in_use(false),
            _ => {}
        }
        self.send_command(WebGLCommand::UseProgram(program.map(|p| p.id())));
        self.current_program.set(program);
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn ValidateProgram(&self, program: &WebGLProgram) {
        handle_potential_webgl_error!(self, self.validate_ownership(program), return);
        if let Err(e) = program.validate() {
            self.webgl_error(e);
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn VertexAttrib1f(&self, indx: u32, x: f32) {
        self.vertex_attrib(indx, x, 0f32, 0f32, 1f32)
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn VertexAttrib1fv(&self, indx: u32, v: Float32ArrayOrUnrestrictedFloatSequence) {
        let values = match v {
            Float32ArrayOrUnrestrictedFloatSequence::Float32Array(v) => v.to_vec(),
            Float32ArrayOrUnrestrictedFloatSequence::UnrestrictedFloatSequence(v) => v,
        };
        if values.len() < 1 {
            return self.webgl_error(InvalidOperation);
        }
        self.vertex_attrib(indx, values[0], 0f32, 0f32, 1f32);
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn VertexAttrib2f(&self, indx: u32, x: f32, y: f32) {
        self.vertex_attrib(indx, x, y, 0f32, 1f32)
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn VertexAttrib2fv(&self, indx: u32, v: Float32ArrayOrUnrestrictedFloatSequence) {
        let values = match v {
            Float32ArrayOrUnrestrictedFloatSequence::Float32Array(v) => v.to_vec(),
            Float32ArrayOrUnrestrictedFloatSequence::UnrestrictedFloatSequence(v) => v,
        };
        if values.len() < 2 {
            return self.webgl_error(InvalidOperation);
        }
        self.vertex_attrib(indx, values[0], values[1], 0f32, 1f32);
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn VertexAttrib3f(&self, indx: u32, x: f32, y: f32, z: f32) {
        self.vertex_attrib(indx, x, y, z, 1f32)
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn VertexAttrib3fv(&self, indx: u32, v: Float32ArrayOrUnrestrictedFloatSequence) {
        let values = match v {
            Float32ArrayOrUnrestrictedFloatSequence::Float32Array(v) => v.to_vec(),
            Float32ArrayOrUnrestrictedFloatSequence::UnrestrictedFloatSequence(v) => v,
        };
        if values.len() < 3 {
            return self.webgl_error(InvalidOperation);
        }
        self.vertex_attrib(indx, values[0], values[1], values[2], 1f32);
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn VertexAttrib4f(&self, indx: u32, x: f32, y: f32, z: f32, w: f32) {
        self.vertex_attrib(indx, x, y, z, w)
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn VertexAttrib4fv(&self, indx: u32, v: Float32ArrayOrUnrestrictedFloatSequence) {
        let values = match v {
            Float32ArrayOrUnrestrictedFloatSequence::Float32Array(v) => v.to_vec(),
            Float32ArrayOrUnrestrictedFloatSequence::UnrestrictedFloatSequence(v) => v,
        };
        if values.len() < 4 {
            return self.webgl_error(InvalidOperation);
        }
        self.vertex_attrib(indx, values[0], values[1], values[2], values[3]);
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn VertexAttribPointer(
        &self,
        index: u32,
        size: i32,
        type_: u32,
        normalized: bool,
        stride: i32,
        offset: i64,
    ) {
        handle_potential_webgl_error!(
            self,
            self.current_vao().vertex_attrib_pointer(
                index,
                size,
                type_,
                normalized,
                stride,
                offset,
            )
        );
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.4
    fn Viewport(&self, x: i32, y: i32, width: i32, height: i32) {
        if width < 0 || height < 0 {
            return self.webgl_error(InvalidValue)
        }

        self.send_command(WebGLCommand::SetViewport(x, y, width, height))
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
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
        mut pixels: CustomAutoRooterGuard<Option<ArrayBufferView>>,
    ) -> ErrorResult {
        if !self.extension_manager.is_tex_type_enabled(data_type) {
            return Ok(self.webgl_error(InvalidEnum));
        }

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
                                                    unpacking_alignment, &*pixels) } {
                Ok(byte_length) => byte_length,
                Err(()) => return Ok(()),
            };

        // If data is null, a buffer of sufficient size
        // initialized to 0 is passed.
        let buff = match *pixels {
            None => vec![0u8; expected_byte_length as usize],
            Some(ref mut data) => data.to_vec(),
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
    fn TexImage2D_(
        &self,
        target: u32,
        level: i32,
        internal_format: u32,
        format: u32,
        data_type: u32,
        source: ImageDataOrHTMLImageElementOrHTMLCanvasElementOrHTMLVideoElement,
    ) -> ErrorResult {
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
                   source: &HTMLIFrameElement) -> ErrorResult {
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
        mut pixels: CustomAutoRooterGuard<Option<ArrayBufferView>>,
    ) -> ErrorResult {
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
                                                    unpacking_alignment, &*pixels) } {
                Ok(byte_length) => byte_length,
                Err(()) => return Ok(()),
            };

        // If data is null, a buffer of sufficient size
        // initialized to 0 is passed.
        let buff = match *pixels {
            None => vec![0u8; expected_byte_length as usize],
            Some(ref mut data) => data.to_vec(),
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
    fn RenderbufferStorage(&self, target: u32, internal_format: u32, width: i32, height: i32) {
        if target != constants::RENDERBUFFER {
            return self.webgl_error(InvalidEnum);
        }

        let max = self.limits.max_renderbuffer_size;

        if width < 0 || width as u32 > max || height < 0 || height as u32 > max {
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
    fn FramebufferRenderbuffer(
        &self,
        target: u32,
        attachment: u32,
        renderbuffertarget: u32,
        rb: Option<&WebGLRenderbuffer>,
    ) {
        if let Some(rb) = rb {
            handle_potential_webgl_error!(self, self.validate_ownership(rb), return);
        }

        if target != constants::FRAMEBUFFER || renderbuffertarget != constants::RENDERBUFFER {
            return self.webgl_error(InvalidEnum);
        }

        match self.bound_framebuffer.get() {
            Some(fb) => handle_potential_webgl_error!(self, fb.renderbuffer(attachment, rb)),
            None => self.webgl_error(InvalidOperation),
        };
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.6
    fn FramebufferTexture2D(
        &self,
        target: u32,
        attachment: u32,
        textarget: u32,
        texture: Option<&WebGLTexture>,
        level: i32,
    ) {
        if let Some(texture) = texture {
            handle_potential_webgl_error!(self, self.validate_ownership(texture), return);
        }

        if target != constants::FRAMEBUFFER {
            return self.webgl_error(InvalidEnum);
        }

        match self.bound_framebuffer.get() {
            Some(fb) => handle_potential_webgl_error!(self, fb.texture2d(attachment, textarget, texture, level)),
            None => self.webgl_error(InvalidOperation),
        };
    }

    /// https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn GetAttachedShaders(&self, program: &WebGLProgram) -> Option<Vec<DomRoot<WebGLShader>>> {
        handle_potential_webgl_error!(self, self.validate_ownership(program), return None);
        handle_potential_webgl_error!(self, program.attached_shaders().map(Some), None)
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

#[derive(Default, JSTraceable, MallocSizeOf)]
struct Capabilities {
    value: Cell<CapFlags>,
}

impl Capabilities {
    fn set(&self, cap: u32, set: bool) -> WebGLResult<bool> {
        let cap = CapFlags::from_enum(cap)?;
        let mut value = self.value.get();
        if value.contains(cap) == set {
            return Ok(false);
        }
        value.set(cap, set);
        self.value.set(value);
        Ok(true)
    }

    fn is_enabled(&self, cap: u32) -> WebGLResult<bool> {
        Ok(self.value.get().contains(CapFlags::from_enum(cap)?))
    }
}

impl Default for CapFlags {
    fn default() -> Self {
        CapFlags::DITHER
    }
}

macro_rules! capabilities {
    ($name:ident, $next:ident, $($rest:ident,)*) => {
        capabilities!($name, $next, $($rest,)* [$name = 1;]);
    };
    ($prev:ident, $name:ident, $($rest:ident,)* [$($tt:tt)*]) => {
        capabilities!($name, $($rest,)* [$($tt)* $name = Self::$prev.bits << 1;]);
    };
    ($prev:ident, [$($name:ident = $value:expr;)*]) => {
        bitflags! {
            #[derive(JSTraceable, MallocSizeOf)]
            struct CapFlags: u16 {
                $(const $name = $value;)*
            }
        }

        impl CapFlags {
            fn from_enum(cap: u32) -> WebGLResult<Self> {
                match cap {
                    $(constants::$name => Ok(Self::$name),)*
                    _ => Err(InvalidEnum),
                }
            }
        }
    };
}

capabilities! {
    BLEND,
    CULL_FACE,
    DEPTH_TEST,
    DITHER,
    POLYGON_OFFSET_FILL,
    SAMPLE_ALPHA_TO_COVERAGE,
    SAMPLE_COVERAGE,
    SCISSOR_TEST,
    STENCIL_TEST,
}
