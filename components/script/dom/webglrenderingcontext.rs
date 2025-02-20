/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::cmp;
use std::ptr::{self, NonNull};
#[cfg(feature = "webxr")]
use std::rc::Rc;

#[cfg(feature = "webgl_backtrace")]
use backtrace::Backtrace;
use bitflags::bitflags;
use canvas_traits::webgl::WebGLError::*;
use canvas_traits::webgl::{
    webgl_channel, AlphaTreatment, GLContextAttributes, GLLimits, GlType, Parameter, SizedDataType,
    TexDataType, TexFormat, TexParameter, WebGLChan, WebGLCommand, WebGLCommandBacktrace,
    WebGLContextId, WebGLError, WebGLFramebufferBindingRequest, WebGLMsg, WebGLMsgSender,
    WebGLProgramId, WebGLResult, WebGLSLVersion, WebGLSendResult, WebGLSender, WebGLVersion,
    YAxisTreatment,
};
use dom_struct::dom_struct;
use euclid::default::{Point2D, Rect, Size2D};
use ipc_channel::ipc::{self, IpcSharedMemory};
use js::jsapi::{JSContext, JSObject, Type};
use js::jsval::{BooleanValue, DoubleValue, Int32Value, NullValue, ObjectValue, UInt32Value};
use js::rust::{CustomAutoRooterGuard, MutableHandleValue};
use js::typedarray::{
    ArrayBufferView, CreateWith, Float32, Float32Array, Int32, Int32Array, TypedArray,
    TypedArrayElementCreator, Uint32Array,
};
use net_traits::image_cache::ImageResponse;
use pixels::{self, PixelFormat};
use script_layout_interface::HTMLCanvasDataSource;
use serde::{Deserialize, Serialize};
use servo_config::pref;
use webrender_api::ImageKey;

use crate::dom::bindings::cell::{DomRefCell, Ref, RefMut};
use crate::dom::bindings::codegen::Bindings::ANGLEInstancedArraysBinding::ANGLEInstancedArraysConstants;
use crate::dom::bindings::codegen::Bindings::EXTBlendMinmaxBinding::EXTBlendMinmaxConstants;
use crate::dom::bindings::codegen::Bindings::OESVertexArrayObjectBinding::OESVertexArrayObjectConstants;
use crate::dom::bindings::codegen::Bindings::WebGL2RenderingContextBinding::WebGL2RenderingContextConstants;
use crate::dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::{
    TexImageSource, WebGLContextAttributes, WebGLRenderingContextConstants as constants,
    WebGLRenderingContextMethods,
};
use crate::dom::bindings::codegen::UnionTypes::{
    ArrayBufferViewOrArrayBuffer, Float32ArrayOrUnrestrictedFloatSequence,
    HTMLCanvasElementOrOffscreenCanvas, Int32ArrayOrLongSequence,
};
use crate::dom::bindings::conversions::{DerivedFrom, ToJSValConvertible};
use crate::dom::bindings::error::{Error, ErrorResult, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal, DomObject, Reflector};
use crate::dom::bindings::root::{DomOnceCell, DomRoot, LayoutDom, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::element::cors_setting_for_element;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::htmlcanvaselement::{utils as canvas_utils, LayoutCanvasRenderingContextHelpers};
use crate::dom::node::{Node, NodeDamage, NodeTraits};
#[cfg(feature = "webxr")]
use crate::dom::promise::Promise;
use crate::dom::vertexarrayobject::VertexAttribData;
use crate::dom::webgl_extensions::WebGLExtensions;
use crate::dom::webgl_validations::tex_image_2d::{
    CommonCompressedTexImage2DValidatorResult, CommonTexImage2DValidator,
    CommonTexImage2DValidatorResult, CompressedTexImage2DValidator,
    CompressedTexSubImage2DValidator, TexImage2DValidator, TexImage2DValidatorResult,
};
use crate::dom::webgl_validations::types::TexImageTarget;
use crate::dom::webgl_validations::WebGLValidator;
use crate::dom::webglactiveinfo::WebGLActiveInfo;
use crate::dom::webglbuffer::WebGLBuffer;
use crate::dom::webglcontextevent::WebGLContextEvent;
use crate::dom::webglframebuffer::{
    CompleteForRendering, WebGLFramebuffer, WebGLFramebufferAttachmentRoot,
};
use crate::dom::webglobject::WebGLObject;
use crate::dom::webglprogram::WebGLProgram;
use crate::dom::webglrenderbuffer::WebGLRenderbuffer;
use crate::dom::webglshader::WebGLShader;
use crate::dom::webglshaderprecisionformat::WebGLShaderPrecisionFormat;
use crate::dom::webgltexture::{TexParameterValue, WebGLTexture};
use crate::dom::webgluniformlocation::WebGLUniformLocation;
use crate::dom::webglvertexarrayobject::WebGLVertexArrayObject;
use crate::dom::webglvertexarrayobjectoes::WebGLVertexArrayObjectOES;
use crate::dom::window::Window;
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

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
                if let Some(command) = $unbind_command {
                    $self_.send_command(command);
                }
            }
        }
    };
}

fn has_invalid_blend_constants(arg1: u32, arg2: u32) -> bool {
    match (arg1, arg2) {
        (constants::CONSTANT_COLOR, constants::CONSTANT_ALPHA) => true,
        (constants::ONE_MINUS_CONSTANT_COLOR, constants::ONE_MINUS_CONSTANT_ALPHA) => true,
        (constants::ONE_MINUS_CONSTANT_COLOR, constants::CONSTANT_ALPHA) => true,
        (constants::CONSTANT_COLOR, constants::ONE_MINUS_CONSTANT_ALPHA) => true,
        (_, _) => false,
    }
}

pub(crate) fn uniform_get<T, F>(triple: (&WebGLRenderingContext, WebGLProgramId, i32), f: F) -> T
where
    F: FnOnce(WebGLProgramId, i32, WebGLSender<T>) -> WebGLCommand,
    T: for<'de> Deserialize<'de> + Serialize,
{
    let (sender, receiver) = webgl_channel().unwrap();
    triple.0.send_command(f(triple.1, triple.2, sender));
    receiver.recv().unwrap()
}

#[allow(unsafe_code)]
pub(crate) unsafe fn uniform_typed<T>(
    cx: *mut JSContext,
    value: &[T::Element],
    mut retval: MutableHandleValue,
) where
    T: TypedArrayElementCreator,
{
    rooted!(in(cx) let mut rval = ptr::null_mut::<JSObject>());
    <TypedArray<T, *mut JSObject>>::create(cx, CreateWith::Slice(value), rval.handle_mut())
        .unwrap();
    retval.set(ObjectValue(rval.get()));
}

/// Set of bitflags for texture unpacking (texImage2d, etc...)
#[derive(Clone, Copy, JSTraceable, MallocSizeOf)]
struct TextureUnpacking(u8);

bitflags! {
    impl TextureUnpacking: u8 {
        const FLIP_Y_AXIS = 0x01;
        const PREMULTIPLY_ALPHA = 0x02;
        const CONVERT_COLORSPACE = 0x04;
    }
}

#[derive(Clone, Copy, Debug, JSTraceable, MallocSizeOf)]
pub(crate) enum VertexAttrib {
    Float(f32, f32, f32, f32),
    Int(i32, i32, i32, i32),
    Uint(u32, u32, u32, u32),
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Operation {
    Fallible,
    Infallible,
}

#[dom_struct]
pub(crate) struct WebGLRenderingContext {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "Channels are hard"]
    webgl_sender: WebGLMessageSender,
    #[ignore_malloc_size_of = "Defined in webrender"]
    #[no_trace]
    webrender_image: ImageKey,
    #[no_trace]
    webgl_version: WebGLVersion,
    #[no_trace]
    glsl_version: WebGLSLVersion,
    #[ignore_malloc_size_of = "Defined in surfman"]
    #[no_trace]
    limits: GLLimits,
    canvas: HTMLCanvasElementOrOffscreenCanvas,
    #[ignore_malloc_size_of = "Defined in canvas_traits"]
    #[no_trace]
    last_error: Cell<Option<WebGLError>>,
    texture_packing_alignment: Cell<u8>,
    texture_unpacking_settings: Cell<TextureUnpacking>,
    // TODO(nox): Should be Cell<u8>.
    texture_unpacking_alignment: Cell<u32>,
    bound_draw_framebuffer: MutNullableDom<WebGLFramebuffer>,
    // TODO(mmatyas): This was introduced in WebGL2, but listed here because it's used by
    // Textures and Renderbuffers, but such WebGLObjects have access only to the GL1 context.
    bound_read_framebuffer: MutNullableDom<WebGLFramebuffer>,
    bound_renderbuffer: MutNullableDom<WebGLRenderbuffer>,
    bound_buffer_array: MutNullableDom<WebGLBuffer>,
    current_program: MutNullableDom<WebGLProgram>,
    current_vertex_attribs: DomRefCell<Box<[VertexAttrib]>>,
    #[ignore_malloc_size_of = "Because it's small"]
    current_scissor: Cell<(i32, i32, u32, u32)>,
    #[ignore_malloc_size_of = "Because it's small"]
    current_clear_color: Cell<(f32, f32, f32, f32)>,
    #[no_trace]
    size: Cell<Size2D<u32>>,
    extension_manager: WebGLExtensions,
    capabilities: Capabilities,
    default_vao: DomOnceCell<WebGLVertexArrayObjectOES>,
    current_vao: MutNullableDom<WebGLVertexArrayObjectOES>,
    default_vao_webgl2: DomOnceCell<WebGLVertexArrayObject>,
    current_vao_webgl2: MutNullableDom<WebGLVertexArrayObject>,
    textures: Textures,
    #[no_trace]
    api_type: GlType,
}

impl WebGLRenderingContext {
    pub(crate) fn new_inherited(
        window: &Window,
        canvas: &HTMLCanvasElementOrOffscreenCanvas,
        webgl_version: WebGLVersion,
        size: Size2D<u32>,
        attrs: GLContextAttributes,
    ) -> Result<WebGLRenderingContext, String> {
        if pref!(webgl_testing_context_creation_error) {
            return Err("WebGL context creation error forced by pref `webgl.testing.context_creation_error`".into());
        }

        let webgl_chan = match window.webgl_chan() {
            Some(chan) => chan,
            None => return Err("WebGL initialization failed early on".into()),
        };

        let (sender, receiver) = webgl_channel().unwrap();
        webgl_chan
            .send(WebGLMsg::CreateContext(webgl_version, size, attrs, sender))
            .unwrap();
        let result = receiver.recv().unwrap();

        result.map(|ctx_data| {
            let max_combined_texture_image_units = ctx_data.limits.max_combined_texture_image_units;
            let max_vertex_attribs = ctx_data.limits.max_vertex_attribs as usize;
            Self {
                reflector_: Reflector::new(),
                webgl_sender: WebGLMessageSender::new(ctx_data.sender),
                webrender_image: ctx_data.image_key,
                webgl_version,
                glsl_version: ctx_data.glsl_version,
                limits: ctx_data.limits,
                canvas: canvas.clone(),
                last_error: Cell::new(None),
                texture_packing_alignment: Cell::new(4),
                texture_unpacking_settings: Cell::new(TextureUnpacking::CONVERT_COLORSPACE),
                texture_unpacking_alignment: Cell::new(4),
                bound_draw_framebuffer: MutNullableDom::new(None),
                bound_read_framebuffer: MutNullableDom::new(None),
                bound_buffer_array: MutNullableDom::new(None),
                bound_renderbuffer: MutNullableDom::new(None),
                current_program: MutNullableDom::new(None),
                current_vertex_attribs: DomRefCell::new(
                    vec![VertexAttrib::Float(0f32, 0f32, 0f32, 1f32); max_vertex_attribs].into(),
                ),
                current_scissor: Cell::new((0, 0, size.width, size.height)),
                // FIXME(#21718) The backend is allowed to choose a size smaller than
                // what was requested
                size: Cell::new(size),
                current_clear_color: Cell::new((0.0, 0.0, 0.0, 0.0)),
                extension_manager: WebGLExtensions::new(
                    webgl_version,
                    ctx_data.api_type,
                    ctx_data.glsl_version,
                ),
                capabilities: Default::default(),
                default_vao: Default::default(),
                current_vao: Default::default(),
                default_vao_webgl2: Default::default(),
                current_vao_webgl2: Default::default(),
                textures: Textures::new(max_combined_texture_image_units),
                api_type: ctx_data.api_type,
            }
        })
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        window: &Window,
        canvas: &HTMLCanvasElementOrOffscreenCanvas,
        webgl_version: WebGLVersion,
        size: Size2D<u32>,
        attrs: GLContextAttributes,
        can_gc: CanGc,
    ) -> Option<DomRoot<WebGLRenderingContext>> {
        match WebGLRenderingContext::new_inherited(window, canvas, webgl_version, size, attrs) {
            Ok(ctx) => Some(reflect_dom_object(Box::new(ctx), window, can_gc)),
            Err(msg) => {
                error!("Couldn't create WebGLRenderingContext: {}", msg);
                let event = WebGLContextEvent::new(
                    window,
                    atom!("webglcontextcreationerror"),
                    EventBubbles::DoesNotBubble,
                    EventCancelable::Cancelable,
                    DOMString::from(msg),
                    can_gc,
                );
                match canvas {
                    HTMLCanvasElementOrOffscreenCanvas::HTMLCanvasElement(canvas) => {
                        event.upcast::<Event>().fire(canvas.upcast(), can_gc);
                    },
                    HTMLCanvasElementOrOffscreenCanvas::OffscreenCanvas(canvas) => {
                        event.upcast::<Event>().fire(canvas.upcast(), can_gc);
                    },
                }
                None
            },
        }
    }

    pub(crate) fn webgl_version(&self) -> WebGLVersion {
        self.webgl_version
    }

    pub(crate) fn limits(&self) -> &GLLimits {
        &self.limits
    }

    pub(crate) fn texture_unpacking_alignment(&self) -> u32 {
        self.texture_unpacking_alignment.get()
    }

    pub(crate) fn current_vao(&self) -> DomRoot<WebGLVertexArrayObjectOES> {
        self.current_vao.or_init(|| {
            DomRoot::from_ref(
                self.default_vao
                    .init_once(|| WebGLVertexArrayObjectOES::new(self, None, CanGc::note())),
            )
        })
    }

    pub(crate) fn current_vao_webgl2(&self) -> DomRoot<WebGLVertexArrayObject> {
        self.current_vao_webgl2.or_init(|| {
            DomRoot::from_ref(
                self.default_vao_webgl2
                    .init_once(|| WebGLVertexArrayObject::new(self, None, CanGc::note())),
            )
        })
    }

    pub(crate) fn current_vertex_attribs(&self) -> RefMut<Box<[VertexAttrib]>> {
        self.current_vertex_attribs.borrow_mut()
    }

    pub(crate) fn recreate(&self, size: Size2D<u32>) {
        let (sender, receiver) = webgl_channel().unwrap();
        self.webgl_sender.send_resize(size, sender).unwrap();
        // FIXME(#21718) The backend is allowed to choose a size smaller than
        // what was requested
        self.size.set(size);

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
        // Right now surfman generates a new FBO and the bound texture is changed
        // in order to create a new render to texture attachment.
        // Send a command to re-bind the TEXTURE_2D, if any.
        if let Some(texture) = self
            .textures
            .active_texture_slot(constants::TEXTURE_2D, self.webgl_version())
            .unwrap()
            .get()
        {
            self.send_command(WebGLCommand::BindTexture(
                constants::TEXTURE_2D,
                Some(texture.id()),
            ));
        }

        // Bound framebuffer must not change when the canvas is resized.
        // Right now surfman generates a new FBO on resize.
        // Send a command to re-bind the framebuffer, if any.
        if let Some(fbo) = self.bound_draw_framebuffer.get() {
            let id = WebGLFramebufferBindingRequest::Explicit(fbo.id());
            self.send_command(WebGLCommand::BindFramebuffer(constants::FRAMEBUFFER, id));
        }
    }

    pub(crate) fn context_id(&self) -> WebGLContextId {
        self.webgl_sender.context_id()
    }

    pub(crate) fn onscreen(&self) -> bool {
        match self.canvas {
            HTMLCanvasElementOrOffscreenCanvas::HTMLCanvasElement(ref canvas) => {
                canvas.upcast::<Node>().is_connected()
            },
            HTMLCanvasElementOrOffscreenCanvas::OffscreenCanvas(_) => false,
        }
    }

    #[inline]
    pub(crate) fn send_command(&self, command: WebGLCommand) {
        self.webgl_sender
            .send(command, capture_webgl_backtrace(self))
            .unwrap();
    }

    pub(crate) fn send_command_ignored(&self, command: WebGLCommand) {
        let _ = self
            .webgl_sender
            .send(command, capture_webgl_backtrace(self));
    }

    pub(crate) fn webgl_error(&self, err: WebGLError) {
        // TODO(emilio): Add useful debug messages to this
        warn!(
            "WebGL error: {:?}, previous error was {:?}",
            err,
            self.last_error.get()
        );

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
    pub(crate) fn validate_framebuffer(&self) -> WebGLResult<()> {
        match self.bound_draw_framebuffer.get() {
            Some(fb) => match fb.check_status_for_rendering() {
                CompleteForRendering::Complete => Ok(()),
                CompleteForRendering::Incomplete => Err(InvalidFramebufferOperation),
                CompleteForRendering::MissingColorAttachment => Err(InvalidOperation),
            },
            None => Ok(()),
        }
    }

    pub(crate) fn validate_ownership<T>(&self, object: &T) -> WebGLResult<()>
    where
        T: DerivedFrom<WebGLObject>,
    {
        if self != object.upcast().context() {
            return Err(InvalidOperation);
        }
        Ok(())
    }

    pub(crate) fn with_location<F>(&self, location: Option<&WebGLUniformLocation>, f: F)
    where
        F: FnOnce(&WebGLUniformLocation) -> WebGLResult<()>,
    {
        let location = match location {
            Some(loc) => loc,
            None => return,
        };
        match self.current_program.get() {
            Some(ref program)
                if program.id() == location.program_id() &&
                    program.link_generation() == location.link_generation() => {},
            _ => return self.webgl_error(InvalidOperation),
        }
        handle_potential_webgl_error!(self, f(location));
    }

    pub(crate) fn textures(&self) -> &Textures {
        &self.textures
    }

    fn tex_parameter(&self, target: u32, param: u32, value: TexParameterValue) {
        let texture_slot = handle_potential_webgl_error!(
            self,
            self.textures
                .active_texture_slot(target, self.webgl_version()),
            return
        );
        let texture =
            handle_potential_webgl_error!(self, texture_slot.get().ok_or(InvalidOperation), return);

        if !self
            .extension_manager
            .is_get_tex_parameter_name_enabled(param)
        {
            return self.webgl_error(InvalidEnum);
        }

        handle_potential_webgl_error!(self, texture.tex_parameter(param, value), return);

        // Validate non filterable TEXTURE_2D data_types
        if target != constants::TEXTURE_2D {
            return;
        }

        let target = TexImageTarget::Texture2D;
        if let Some(info) = texture.image_info_for_target(&target, 0) {
            self.validate_filterable_texture(
                &texture,
                target,
                0,
                info.internal_format(),
                Size2D::new(info.width(), info.height()),
                info.data_type().unwrap_or(TexDataType::UnsignedByte),
            );
        }
    }

    pub(crate) fn mark_as_dirty(&self) {
        // If we have a bound framebuffer, then don't mark the canvas as dirty.
        if self.bound_draw_framebuffer.get().is_some() {
            return;
        }

        // Dirtying the canvas is unnecessary if we're actively displaying immersive
        // XR content right now.
        if self.global().as_window().in_immersive_xr_session() {
            return;
        }

        match self.canvas {
            HTMLCanvasElementOrOffscreenCanvas::HTMLCanvasElement(ref canvas) => {
                canvas.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
                canvas.owner_document().add_dirty_webgl_canvas(self);
            },
            HTMLCanvasElementOrOffscreenCanvas::OffscreenCanvas(_) => {},
        }
    }

    fn vertex_attrib(&self, indx: u32, x: f32, y: f32, z: f32, w: f32) {
        if indx >= self.limits.max_vertex_attribs {
            return self.webgl_error(InvalidValue);
        }

        match self.webgl_version() {
            WebGLVersion::WebGL1 => self
                .current_vao()
                .set_vertex_attrib_type(indx, constants::FLOAT),
            WebGLVersion::WebGL2 => self
                .current_vao_webgl2()
                .set_vertex_attrib_type(indx, constants::FLOAT),
        };
        self.current_vertex_attribs.borrow_mut()[indx as usize] = VertexAttrib::Float(x, y, z, w);

        self.send_command(WebGLCommand::VertexAttrib(indx, x, y, z, w));
    }

    pub(crate) fn get_current_framebuffer_size(&self) -> Option<(i32, i32)> {
        match self.bound_draw_framebuffer.get() {
            Some(fb) => fb.size(),

            // The window system framebuffer is bound
            None => Some((self.DrawingBufferWidth(), self.DrawingBufferHeight())),
        }
    }

    pub(crate) fn get_texture_packing_alignment(&self) -> u8 {
        self.texture_packing_alignment.get()
    }

    // LINEAR filtering may be forbidden when using WebGL extensions.
    // https://www.khronos.org/registry/webgl/extensions/OES_texture_float_linear/
    fn validate_filterable_texture(
        &self,
        texture: &WebGLTexture,
        target: TexImageTarget,
        level: u32,
        internal_format: TexFormat,
        size: Size2D<u32>,
        data_type: TexDataType,
    ) -> bool {
        if self
            .extension_manager
            .is_filterable(data_type.as_gl_constant()) ||
            !texture.is_using_linear_filtering()
        {
            return true;
        }

        // Handle validation failed: LINEAR filtering not valid for this texture
        // WebGL Conformance tests expect to fallback to [0, 0, 0, 255] RGBA UNSIGNED_BYTE
        let data_type = TexDataType::UnsignedByte;
        let expected_byte_length = size.area() * 4;
        let mut pixels = vec![0u8; expected_byte_length as usize];
        for rgba8 in pixels.chunks_mut(4) {
            rgba8[3] = 255u8;
        }

        // TODO(nox): AFAICT here we construct a RGBA8 array and then we
        // convert it to whatever actual format we need, we should probably
        // construct the desired format from the start.
        self.tex_image_2d(
            texture,
            target,
            data_type,
            internal_format,
            internal_format.to_unsized(),
            level,
            0,
            1,
            size,
            TexSource::Pixels(TexPixels::new(
                IpcSharedMemory::from_bytes(&pixels),
                size,
                PixelFormat::RGBA8,
                true,
            )),
        );

        false
    }

    fn validate_stencil_actions(&self, action: u32) -> bool {
        matches!(
            action,
            0 | constants::KEEP |
                constants::REPLACE |
                constants::INCR |
                constants::DECR |
                constants::INVERT |
                constants::INCR_WRAP |
                constants::DECR_WRAP
        )
    }

    pub(crate) fn get_image_pixels(&self, source: TexImageSource) -> Fallible<Option<TexPixels>> {
        Ok(Some(match source {
            TexImageSource::ImageData(image_data) => TexPixels::new(
                image_data.to_shared_memory(),
                image_data.get_size(),
                PixelFormat::RGBA8,
                false,
            ),
            TexImageSource::HTMLImageElement(image) => {
                let document = match self.canvas {
                    HTMLCanvasElementOrOffscreenCanvas::HTMLCanvasElement(ref canvas) => {
                        canvas.owner_document()
                    },
                    HTMLCanvasElementOrOffscreenCanvas::OffscreenCanvas(ref _canvas) => {
                        // TODO: Support retrieving image pixels here for OffscreenCanvas
                        return Ok(None);
                    },
                };
                if !image.same_origin(document.origin()) {
                    return Err(Error::Security);
                }

                let img_url = match image.get_url() {
                    Some(url) => url,
                    None => return Ok(None),
                };

                let window = match self.canvas {
                    HTMLCanvasElementOrOffscreenCanvas::HTMLCanvasElement(ref canvas) => {
                        canvas.owner_window()
                    },
                    // This is marked as unreachable as we should have returned already
                    HTMLCanvasElementOrOffscreenCanvas::OffscreenCanvas(_) => unreachable!(),
                };
                let cors_setting = cors_setting_for_element(image.upcast());

                let img =
                    match canvas_utils::request_image_from_cache(&window, img_url, cors_setting) {
                        ImageResponse::Loaded(img, _) => img,
                        ImageResponse::PlaceholderLoaded(_, _) |
                        ImageResponse::None |
                        ImageResponse::MetadataLoaded(_) => return Ok(None),
                    };

                let size = Size2D::new(img.width, img.height);

                TexPixels::new(img.bytes.clone(), size, img.format, false)
            },
            // TODO(emilio): Getting canvas data is implemented in CanvasRenderingContext2D,
            // but we need to refactor it moving it to `HTMLCanvasElement` and support
            // WebGLContext (probably via GetPixels()).
            TexImageSource::HTMLCanvasElement(canvas) => {
                if !canvas.origin_is_clean() {
                    return Err(Error::Security);
                }
                if let Some((data, size)) = canvas.fetch_all_data() {
                    let data = data.unwrap_or_else(|| {
                        IpcSharedMemory::from_bytes(&vec![0; size.area() as usize * 4])
                    });
                    TexPixels::new(data, size, PixelFormat::BGRA8, true)
                } else {
                    return Ok(None);
                }
            },
            TexImageSource::HTMLVideoElement(video) => match video.get_current_frame_data() {
                Some((data, size)) => {
                    let data = data.unwrap_or_else(|| {
                        IpcSharedMemory::from_bytes(&vec![0; size.area() as usize * 4])
                    });
                    TexPixels::new(data, size, PixelFormat::BGRA8, false)
                },
                None => return Ok(None),
            },
        }))
    }

    // TODO(emilio): Move this logic to a validator.
    pub(crate) fn validate_tex_image_2d_data(
        &self,
        width: u32,
        height: u32,
        format: TexFormat,
        data_type: TexDataType,
        unpacking_alignment: u32,
        data: Option<&ArrayBufferView>,
    ) -> Result<u32, ()> {
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
        let data_type_matches = data.as_ref().map_or(true, |buffer| {
            Some(data_type.sized_data_type()) ==
                array_buffer_type_to_sized_type(buffer.get_array_type()) &&
                data_type.required_webgl_version() <= self.webgl_version()
        });

        if !data_type_matches {
            self.webgl_error(InvalidOperation);
            return Err(());
        }

        // NOTE: width and height are positive or zero due to validate()
        if height == 0 {
            Ok(0)
        } else {
            // We need to be careful here to not count unpack
            // alignment at the end of the image, otherwise (for
            // example) passing a single byte for uploading a 1x1
            // GL_ALPHA/GL_UNSIGNED_BYTE texture would throw an error.
            let cpp = element_size * components / components_per_element;
            let stride = (width * cpp + unpacking_alignment - 1) & !(unpacking_alignment - 1);
            Ok(stride * (height - 1) + width * cpp)
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn tex_image_2d(
        &self,
        texture: &WebGLTexture,
        target: TexImageTarget,
        data_type: TexDataType,
        internal_format: TexFormat,
        format: TexFormat,
        level: u32,
        _border: u32,
        unpacking_alignment: u32,
        size: Size2D<u32>,
        source: TexSource,
    ) {
        // TexImage2D depth is always equal to 1.
        handle_potential_webgl_error!(
            self,
            texture.initialize(
                target,
                size.width,
                size.height,
                1,
                format,
                level,
                Some(data_type)
            )
        );

        let settings = self.texture_unpacking_settings.get();
        let dest_premultiplied = settings.contains(TextureUnpacking::PREMULTIPLY_ALPHA);

        let y_axis_treatment = if settings.contains(TextureUnpacking::FLIP_Y_AXIS) {
            YAxisTreatment::Flipped
        } else {
            YAxisTreatment::AsIs
        };

        let internal_format = self
            .extension_manager
            .get_effective_tex_internal_format(internal_format, data_type.as_gl_constant());

        let effective_data_type = self
            .extension_manager
            .effective_type(data_type.as_gl_constant());

        match source {
            TexSource::Pixels(pixels) => {
                let alpha_treatment = match (pixels.premultiplied, dest_premultiplied) {
                    (true, false) => Some(AlphaTreatment::Unmultiply),
                    (false, true) => Some(AlphaTreatment::Premultiply),
                    _ => None,
                };

                // TODO(emilio): convert colorspace if requested.
                self.send_command(WebGLCommand::TexImage2D {
                    target: target.as_gl_constant(),
                    level,
                    internal_format,
                    size,
                    format,
                    data_type,
                    effective_data_type,
                    unpacking_alignment,
                    alpha_treatment,
                    y_axis_treatment,
                    pixel_format: pixels.pixel_format,
                    data: pixels.data.into(),
                });
            },
            TexSource::BufferOffset(offset) => {
                self.send_command(WebGLCommand::TexImage2DPBO {
                    target: target.as_gl_constant(),
                    level,
                    internal_format,
                    size,
                    format,
                    effective_data_type,
                    unpacking_alignment,
                    offset,
                });
            },
        }

        if let Some(fb) = self.bound_draw_framebuffer.get() {
            fb.invalidate_texture(texture);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn tex_sub_image_2d(
        &self,
        texture: DomRoot<WebGLTexture>,
        target: TexImageTarget,
        level: u32,
        xoffset: i32,
        yoffset: i32,
        format: TexFormat,
        data_type: TexDataType,
        unpacking_alignment: u32,
        pixels: TexPixels,
    ) {
        // We have already validated level
        let image_info = match texture.image_info_for_target(&target, level) {
            Some(info) => info,
            None => return self.webgl_error(InvalidOperation),
        };

        // GL_INVALID_VALUE is generated if:
        //   - xoffset or yoffset is less than 0
        //   - x offset plus the width is greater than the texture width
        //   - y offset plus the height is greater than the texture height
        if xoffset < 0 ||
            (xoffset as u32 + pixels.size().width) > image_info.width() ||
            yoffset < 0 ||
            (yoffset as u32 + pixels.size().height) > image_info.height()
        {
            return self.webgl_error(InvalidValue);
        }

        // The unsized format must be compatible with the sized internal format
        debug_assert!(!format.is_sized());
        if format != image_info.internal_format().to_unsized() {
            return self.webgl_error(InvalidOperation);
        }

        // See https://www.khronos.org/registry/webgl/specs/latest/2.0/#4.1.6
        if self.webgl_version() == WebGLVersion::WebGL1 &&
            data_type != image_info.data_type().unwrap()
        {
            return self.webgl_error(InvalidOperation);
        }

        let settings = self.texture_unpacking_settings.get();
        let dest_premultiplied = settings.contains(TextureUnpacking::PREMULTIPLY_ALPHA);

        let alpha_treatment = match (pixels.premultiplied, dest_premultiplied) {
            (true, false) => Some(AlphaTreatment::Unmultiply),
            (false, true) => Some(AlphaTreatment::Premultiply),
            _ => None,
        };

        let y_axis_treatment = if settings.contains(TextureUnpacking::FLIP_Y_AXIS) {
            YAxisTreatment::Flipped
        } else {
            YAxisTreatment::AsIs
        };

        let effective_data_type = self
            .extension_manager
            .effective_type(data_type.as_gl_constant());

        // TODO(emilio): convert colorspace if requested.
        self.send_command(WebGLCommand::TexSubImage2D {
            target: target.as_gl_constant(),
            level,
            xoffset,
            yoffset,
            size: pixels.size(),
            format,
            data_type,
            effective_data_type,
            unpacking_alignment,
            alpha_treatment,
            y_axis_treatment,
            pixel_format: pixels.pixel_format,
            data: pixels.data.into(),
        });
    }

    fn get_gl_extensions(&self) -> String {
        let (sender, receiver) = webgl_channel().unwrap();
        self.send_command(WebGLCommand::GetExtensions(sender));
        receiver.recv().unwrap()
    }

    pub(crate) fn layout_handle(&self) -> HTMLCanvasDataSource {
        let image_key = self.webrender_image;
        HTMLCanvasDataSource::WebGL(image_key)
    }

    // https://www.khronos.org/registry/webgl/extensions/ANGLE_instanced_arrays/
    pub(crate) fn draw_arrays_instanced(
        &self,
        mode: u32,
        first: i32,
        count: i32,
        primcount: i32,
    ) -> WebGLResult<()> {
        match mode {
            constants::POINTS |
            constants::LINE_STRIP |
            constants::LINE_LOOP |
            constants::LINES |
            constants::TRIANGLE_STRIP |
            constants::TRIANGLE_FAN |
            constants::TRIANGLES => {},
            _ => {
                return Err(InvalidEnum);
            },
        }
        if first < 0 || count < 0 || primcount < 0 {
            return Err(InvalidValue);
        }

        let current_program = self.current_program.get().ok_or(InvalidOperation)?;

        let required_len = if count > 0 {
            first
                .checked_add(count)
                .map(|len| len as u32)
                .ok_or(InvalidOperation)?
        } else {
            0
        };

        match self.webgl_version() {
            WebGLVersion::WebGL1 => self.current_vao().validate_for_draw(
                required_len,
                primcount as u32,
                &current_program.active_attribs(),
            )?,
            WebGLVersion::WebGL2 => self.current_vao_webgl2().validate_for_draw(
                required_len,
                primcount as u32,
                &current_program.active_attribs(),
            )?,
        };

        self.validate_framebuffer()?;

        if count == 0 || primcount == 0 {
            return Ok(());
        }

        self.send_command(if primcount == 1 {
            WebGLCommand::DrawArrays { mode, first, count }
        } else {
            WebGLCommand::DrawArraysInstanced {
                mode,
                first,
                count,
                primcount,
            }
        });
        self.mark_as_dirty();
        Ok(())
    }

    // https://www.khronos.org/registry/webgl/extensions/ANGLE_instanced_arrays/
    pub(crate) fn draw_elements_instanced(
        &self,
        mode: u32,
        count: i32,
        type_: u32,
        offset: i64,
        primcount: i32,
    ) -> WebGLResult<()> {
        match mode {
            constants::POINTS |
            constants::LINE_STRIP |
            constants::LINE_LOOP |
            constants::LINES |
            constants::TRIANGLE_STRIP |
            constants::TRIANGLE_FAN |
            constants::TRIANGLES => {},
            _ => {
                return Err(InvalidEnum);
            },
        }
        if count < 0 || offset < 0 || primcount < 0 {
            return Err(InvalidValue);
        }
        let type_size = match type_ {
            constants::UNSIGNED_BYTE => 1,
            constants::UNSIGNED_SHORT => 2,
            constants::UNSIGNED_INT => match self.webgl_version() {
                WebGLVersion::WebGL1 if self.extension_manager.is_element_index_uint_enabled() => 4,
                WebGLVersion::WebGL2 => 4,
                _ => return Err(InvalidEnum),
            },
            _ => return Err(InvalidEnum),
        };
        if offset % type_size != 0 {
            return Err(InvalidOperation);
        }

        let current_program = self.current_program.get().ok_or(InvalidOperation)?;
        let array_buffer = match self.webgl_version() {
            WebGLVersion::WebGL1 => self.current_vao().element_array_buffer().get(),
            WebGLVersion::WebGL2 => self.current_vao_webgl2().element_array_buffer().get(),
        }
        .ok_or(InvalidOperation)?;

        if count > 0 && primcount > 0 {
            // This operation cannot overflow in u64 and we know all those values are nonnegative.
            let val = offset as u64 + (count as u64 * type_size as u64);
            if val > array_buffer.capacity() as u64 {
                return Err(InvalidOperation);
            }
        }

        // TODO(nox): Pass the correct number of vertices required.
        match self.webgl_version() {
            WebGLVersion::WebGL1 => self.current_vao().validate_for_draw(
                0,
                primcount as u32,
                &current_program.active_attribs(),
            )?,
            WebGLVersion::WebGL2 => self.current_vao_webgl2().validate_for_draw(
                0,
                primcount as u32,
                &current_program.active_attribs(),
            )?,
        };

        self.validate_framebuffer()?;

        if count == 0 || primcount == 0 {
            return Ok(());
        }

        let offset = offset as u32;
        self.send_command(if primcount == 1 {
            WebGLCommand::DrawElements {
                mode,
                count,
                type_,
                offset,
            }
        } else {
            WebGLCommand::DrawElementsInstanced {
                mode,
                count,
                type_,
                offset,
                primcount,
            }
        });
        self.mark_as_dirty();
        Ok(())
    }

    pub(crate) fn vertex_attrib_divisor(&self, index: u32, divisor: u32) {
        if index >= self.limits.max_vertex_attribs {
            return self.webgl_error(InvalidValue);
        }

        match self.webgl_version() {
            WebGLVersion::WebGL1 => self.current_vao().vertex_attrib_divisor(index, divisor),
            WebGLVersion::WebGL2 => self
                .current_vao_webgl2()
                .vertex_attrib_divisor(index, divisor),
        };
        self.send_command(WebGLCommand::VertexAttribDivisor { index, divisor });
    }

    // Used by HTMLCanvasElement.toDataURL
    //
    // This emits errors quite liberally, but the spec says that this operation
    // can fail and that it is UB what happens in that case.
    //
    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#2.2
    pub(crate) fn get_image_data(&self, mut size: Size2D<u32>) -> Option<Vec<u8>> {
        handle_potential_webgl_error!(self, self.validate_framebuffer(), return None);

        let (fb_width, fb_height) = handle_potential_webgl_error!(
            self,
            self.get_current_framebuffer_size().ok_or(InvalidOperation),
            return None
        );
        size.width = cmp::min(size.width, fb_width as u32);
        size.height = cmp::min(size.height, fb_height as u32);

        let (sender, receiver) = ipc::bytes_channel().unwrap();
        self.send_command(WebGLCommand::ReadPixels(
            Rect::from_size(size),
            constants::RGBA,
            constants::UNSIGNED_BYTE,
            sender,
        ));
        Some(receiver.recv().unwrap())
    }

    pub(crate) fn array_buffer(&self) -> Option<DomRoot<WebGLBuffer>> {
        self.bound_buffer_array.get()
    }

    pub(crate) fn array_buffer_slot(&self) -> &MutNullableDom<WebGLBuffer> {
        &self.bound_buffer_array
    }

    pub(crate) fn bound_buffer(&self, target: u32) -> WebGLResult<Option<DomRoot<WebGLBuffer>>> {
        match target {
            constants::ARRAY_BUFFER => Ok(self.bound_buffer_array.get()),
            constants::ELEMENT_ARRAY_BUFFER => Ok(self.current_vao().element_array_buffer().get()),
            _ => Err(WebGLError::InvalidEnum),
        }
    }

    pub(crate) fn buffer_usage(&self, usage: u32) -> WebGLResult<u32> {
        match usage {
            constants::STREAM_DRAW | constants::STATIC_DRAW | constants::DYNAMIC_DRAW => Ok(usage),
            _ => Err(WebGLError::InvalidEnum),
        }
    }

    pub(crate) fn create_vertex_array(&self) -> Option<DomRoot<WebGLVertexArrayObjectOES>> {
        let (sender, receiver) = webgl_channel().unwrap();
        self.send_command(WebGLCommand::CreateVertexArray(sender));
        receiver
            .recv()
            .unwrap()
            .map(|id| WebGLVertexArrayObjectOES::new(self, Some(id), CanGc::note()))
    }

    pub(crate) fn create_vertex_array_webgl2(&self) -> Option<DomRoot<WebGLVertexArrayObject>> {
        let (sender, receiver) = webgl_channel().unwrap();
        self.send_command(WebGLCommand::CreateVertexArray(sender));
        receiver
            .recv()
            .unwrap()
            .map(|id| WebGLVertexArrayObject::new(self, Some(id), CanGc::note()))
    }

    pub(crate) fn delete_vertex_array(&self, vao: Option<&WebGLVertexArrayObjectOES>) {
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
            vao.delete(Operation::Infallible);
        }
    }

    pub(crate) fn delete_vertex_array_webgl2(&self, vao: Option<&WebGLVertexArrayObject>) {
        if let Some(vao) = vao {
            handle_potential_webgl_error!(self, self.validate_ownership(vao), return);
            // The default vertex array has no id and should never be passed around.
            assert!(vao.id().is_some());
            if vao.is_deleted() {
                return;
            }
            if vao == &*self.current_vao_webgl2() {
                // Setting it to None will make self.current_vao() reset it to the default one
                // next time it is called.
                self.current_vao_webgl2.set(None);
                self.send_command(WebGLCommand::BindVertexArray(None));
            }
            vao.delete(Operation::Infallible);
        }
    }

    pub(crate) fn is_vertex_array(&self, vao: Option<&WebGLVertexArrayObjectOES>) -> bool {
        vao.is_some_and(|vao| {
            // The default vertex array has no id and should never be passed around.
            assert!(vao.id().is_some());
            self.validate_ownership(vao).is_ok() && vao.ever_bound() && !vao.is_deleted()
        })
    }

    pub(crate) fn is_vertex_array_webgl2(&self, vao: Option<&WebGLVertexArrayObject>) -> bool {
        vao.is_some_and(|vao| {
            // The default vertex array has no id and should never be passed around.
            assert!(vao.id().is_some());
            self.validate_ownership(vao).is_ok() && vao.ever_bound() && !vao.is_deleted()
        })
    }

    pub(crate) fn bind_vertex_array(&self, vao: Option<&WebGLVertexArrayObjectOES>) {
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

    pub(crate) fn bind_vertex_array_webgl2(&self, vao: Option<&WebGLVertexArrayObject>) {
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
        self.current_vao_webgl2.set(vao);
    }

    fn validate_blend_mode(&self, mode: u32) -> WebGLResult<()> {
        match mode {
            constants::FUNC_ADD | constants::FUNC_SUBTRACT | constants::FUNC_REVERSE_SUBTRACT => {
                Ok(())
            },
            EXTBlendMinmaxConstants::MIN_EXT | EXTBlendMinmaxConstants::MAX_EXT
                if self.extension_manager.is_blend_minmax_enabled() =>
            {
                Ok(())
            },
            _ => Err(InvalidEnum),
        }
    }

    pub(crate) fn initialize_framebuffer(&self, clear_bits: u32) {
        if clear_bits == 0 {
            return;
        }
        self.send_command(WebGLCommand::InitializeFramebuffer {
            color: clear_bits & constants::COLOR_BUFFER_BIT != 0,
            depth: clear_bits & constants::DEPTH_BUFFER_BIT != 0,
            stencil: clear_bits & constants::STENCIL_BUFFER_BIT != 0,
        });
    }

    pub(crate) fn extension_manager(&self) -> &WebGLExtensions {
        &self.extension_manager
    }

    #[allow(unsafe_code)]
    pub(crate) fn buffer_data(
        &self,
        target: u32,
        data: Option<ArrayBufferViewOrArrayBuffer>,
        usage: u32,
        bound_buffer: Option<DomRoot<WebGLBuffer>>,
    ) {
        let data = handle_potential_webgl_error!(self, data.ok_or(InvalidValue), return);
        let bound_buffer =
            handle_potential_webgl_error!(self, bound_buffer.ok_or(InvalidOperation), return);

        let data = unsafe {
            // Safe because we don't do anything with JS until the end of the method.
            match data {
                ArrayBufferViewOrArrayBuffer::ArrayBuffer(ref data) => data.as_slice(),
                ArrayBufferViewOrArrayBuffer::ArrayBufferView(ref data) => data.as_slice(),
            }
        };
        handle_potential_webgl_error!(self, bound_buffer.buffer_data(target, data, usage));
    }

    pub(crate) fn buffer_data_(
        &self,
        target: u32,
        size: i64,
        usage: u32,
        bound_buffer: Option<DomRoot<WebGLBuffer>>,
    ) {
        let bound_buffer =
            handle_potential_webgl_error!(self, bound_buffer.ok_or(InvalidOperation), return);

        if size < 0 {
            return self.webgl_error(InvalidValue);
        }

        // FIXME: Allocating a buffer based on user-requested size is
        // not great, but we don't have a fallible allocation to try.
        let data = vec![0u8; size as usize];
        handle_potential_webgl_error!(self, bound_buffer.buffer_data(target, &data, usage));
    }

    #[allow(unsafe_code)]
    pub(crate) fn buffer_sub_data(
        &self,
        target: u32,
        offset: i64,
        data: ArrayBufferViewOrArrayBuffer,
        bound_buffer: Option<DomRoot<WebGLBuffer>>,
    ) {
        let bound_buffer =
            handle_potential_webgl_error!(self, bound_buffer.ok_or(InvalidOperation), return);

        if offset < 0 {
            return self.webgl_error(InvalidValue);
        }

        let data = unsafe {
            // Safe because we don't do anything with JS until the end of the method.
            match data {
                ArrayBufferViewOrArrayBuffer::ArrayBuffer(ref data) => data.as_slice(),
                ArrayBufferViewOrArrayBuffer::ArrayBufferView(ref data) => data.as_slice(),
            }
        };
        if (offset as u64) + data.len() as u64 > bound_buffer.capacity() as u64 {
            return self.webgl_error(InvalidValue);
        }
        let (sender, receiver) = ipc::bytes_channel().unwrap();
        self.send_command(WebGLCommand::BufferSubData(
            target,
            offset as isize,
            receiver,
        ));
        sender.send(data).unwrap();
    }

    pub(crate) fn bind_buffer_maybe(
        &self,
        slot: &MutNullableDom<WebGLBuffer>,
        target: u32,
        buffer: Option<&WebGLBuffer>,
    ) {
        if let Some(buffer) = buffer {
            handle_potential_webgl_error!(self, self.validate_ownership(buffer), return);

            if buffer.is_marked_for_deletion() {
                return self.webgl_error(InvalidOperation);
            }
            handle_potential_webgl_error!(self, buffer.set_target_maybe(target), return);
            buffer.increment_attached_counter();
        }

        self.send_command(WebGLCommand::BindBuffer(target, buffer.map(|b| b.id())));
        if let Some(old) = slot.get() {
            old.decrement_attached_counter(Operation::Infallible);
        }

        slot.set(buffer);
    }

    pub(crate) fn current_program(&self) -> Option<DomRoot<WebGLProgram>> {
        self.current_program.get()
    }

    pub(crate) fn uniform_check_program(
        &self,
        program: &WebGLProgram,
        location: &WebGLUniformLocation,
    ) -> WebGLResult<()> {
        self.validate_ownership(program)?;

        if program.is_deleted() ||
            !program.is_linked() ||
            self.context_id() != location.context_id() ||
            program.id() != location.program_id() ||
            program.link_generation() != location.link_generation()
        {
            return Err(InvalidOperation);
        }

        Ok(())
    }

    fn uniform_vec_section_int(
        &self,
        vec: Int32ArrayOrLongSequence,
        offset: u32,
        length: u32,
        uniform_size: usize,
        uniform_location: &WebGLUniformLocation,
    ) -> WebGLResult<Vec<i32>> {
        let vec = match vec {
            Int32ArrayOrLongSequence::Int32Array(v) => v.to_vec(),
            Int32ArrayOrLongSequence::LongSequence(v) => v,
        };
        self.uniform_vec_section::<i32>(vec, offset, length, uniform_size, uniform_location)
    }

    fn uniform_vec_section_float(
        &self,
        vec: Float32ArrayOrUnrestrictedFloatSequence,
        offset: u32,
        length: u32,
        uniform_size: usize,
        uniform_location: &WebGLUniformLocation,
    ) -> WebGLResult<Vec<f32>> {
        let vec = match vec {
            Float32ArrayOrUnrestrictedFloatSequence::Float32Array(v) => v.to_vec(),
            Float32ArrayOrUnrestrictedFloatSequence::UnrestrictedFloatSequence(v) => v,
        };
        self.uniform_vec_section::<f32>(vec, offset, length, uniform_size, uniform_location)
    }

    pub(crate) fn uniform_vec_section<T: Clone>(
        &self,
        vec: Vec<T>,
        offset: u32,
        length: u32,
        uniform_size: usize,
        uniform_location: &WebGLUniformLocation,
    ) -> WebGLResult<Vec<T>> {
        let offset = offset as usize;
        if offset > vec.len() {
            return Err(InvalidValue);
        }

        let length = if length > 0 {
            length as usize
        } else {
            vec.len() - offset
        };
        if offset + length > vec.len() {
            return Err(InvalidValue);
        }

        let vec = if offset == 0 && length == vec.len() {
            vec
        } else {
            vec[offset..offset + length].to_vec()
        };

        if vec.len() < uniform_size || vec.len() % uniform_size != 0 {
            return Err(InvalidValue);
        }
        if uniform_location.size().is_none() && vec.len() != uniform_size {
            return Err(InvalidOperation);
        }

        Ok(vec)
    }

    pub(crate) fn uniform_matrix_section(
        &self,
        vec: Float32ArrayOrUnrestrictedFloatSequence,
        offset: u32,
        length: u32,
        transpose: bool,
        uniform_size: usize,
        uniform_location: &WebGLUniformLocation,
    ) -> WebGLResult<Vec<f32>> {
        let vec = match vec {
            Float32ArrayOrUnrestrictedFloatSequence::Float32Array(v) => v.to_vec(),
            Float32ArrayOrUnrestrictedFloatSequence::UnrestrictedFloatSequence(v) => v,
        };
        if transpose {
            return Err(InvalidValue);
        }
        self.uniform_vec_section::<f32>(vec, offset, length, uniform_size, uniform_location)
    }

    pub(crate) fn get_draw_framebuffer_slot(&self) -> &MutNullableDom<WebGLFramebuffer> {
        &self.bound_draw_framebuffer
    }

    pub(crate) fn get_read_framebuffer_slot(&self) -> &MutNullableDom<WebGLFramebuffer> {
        &self.bound_read_framebuffer
    }

    pub(crate) fn validate_new_framebuffer_binding(
        &self,
        framebuffer: Option<&WebGLFramebuffer>,
    ) -> WebGLResult<()> {
        if let Some(fb) = framebuffer {
            self.validate_ownership(fb)?;
            if fb.is_deleted() {
                // From the WebGL spec:
                //
                //     "An attempt to bind a deleted framebuffer will
                //      generate an INVALID_OPERATION error, and the
                //      current binding will remain untouched."
                return Err(InvalidOperation);
            }
        }
        Ok(())
    }

    pub(crate) fn bind_framebuffer_to(
        &self,
        target: u32,
        framebuffer: Option<&WebGLFramebuffer>,
        slot: &MutNullableDom<WebGLFramebuffer>,
    ) {
        match framebuffer {
            Some(framebuffer) => framebuffer.bind(target),
            None => {
                // Bind the default framebuffer
                let cmd =
                    WebGLCommand::BindFramebuffer(target, WebGLFramebufferBindingRequest::Default);
                self.send_command(cmd);
            },
        }
        slot.set(framebuffer);
    }

    pub(crate) fn renderbuffer_storage(
        &self,
        target: u32,
        samples: i32,
        internal_format: u32,
        width: i32,
        height: i32,
    ) {
        if target != constants::RENDERBUFFER {
            return self.webgl_error(InvalidEnum);
        }

        let max = self.limits.max_renderbuffer_size;

        if samples < 0 || width < 0 || width as u32 > max || height < 0 || height as u32 > max {
            return self.webgl_error(InvalidValue);
        }

        let rb = handle_potential_webgl_error!(
            self,
            self.bound_renderbuffer.get().ok_or(InvalidOperation),
            return
        );
        handle_potential_webgl_error!(
            self,
            rb.storage(self.api_type, samples, internal_format, width, height)
        );
        if let Some(fb) = self.bound_draw_framebuffer.get() {
            fb.invalidate_renderbuffer(&rb);
        }

        // FIXME: https://github.com/servo/servo/issues/13710
    }

    pub(crate) fn valid_color_attachment_enum(&self, attachment: u32) -> bool {
        let last_slot = constants::COLOR_ATTACHMENT0 + self.limits().max_color_attachments - 1;
        constants::COLOR_ATTACHMENT0 <= attachment && attachment <= last_slot
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn compressed_tex_image_2d(
        &self,
        target: u32,
        level: i32,
        internal_format: u32,
        width: i32,
        height: i32,
        border: i32,
        data: &[u8],
    ) {
        let validator = CompressedTexImage2DValidator::new(
            self,
            target,
            level,
            width,
            height,
            border,
            internal_format,
            data.len(),
        );
        let CommonCompressedTexImage2DValidatorResult {
            texture,
            target,
            level,
            width,
            height,
            compression,
        } = match validator.validate() {
            Ok(result) => result,
            Err(_) => return,
        };

        if texture.is_immutable() {
            return self.webgl_error(InvalidOperation);
        }

        let size = Size2D::new(width, height);
        let buff = IpcSharedMemory::from_bytes(data);
        let pixels = TexPixels::from_array(buff, size);
        let data = pixels.data;

        handle_potential_webgl_error!(
            self,
            texture.initialize(
                target,
                size.width,
                size.height,
                1,
                compression.format,
                level,
                Some(TexDataType::UnsignedByte)
            )
        );

        self.send_command(WebGLCommand::CompressedTexImage2D {
            target: target.as_gl_constant(),
            level,
            internal_format,
            size: Size2D::new(width, height),
            data: data.into(),
        });

        if let Some(fb) = self.bound_draw_framebuffer.get() {
            fb.invalidate_texture(&texture);
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn compressed_tex_sub_image_2d(
        &self,
        target: u32,
        level: i32,
        xoffset: i32,
        yoffset: i32,
        width: i32,
        height: i32,
        format: u32,
        data: &[u8],
    ) {
        let validator = CompressedTexSubImage2DValidator::new(
            self,
            target,
            level,
            xoffset,
            yoffset,
            width,
            height,
            format,
            data.len(),
        );
        let CommonCompressedTexImage2DValidatorResult {
            texture: _,
            target,
            level,
            width,
            height,
            ..
        } = match validator.validate() {
            Ok(result) => result,
            Err(_) => return,
        };

        let buff = IpcSharedMemory::from_bytes(data);
        let pixels = TexPixels::from_array(buff, Size2D::new(width, height));
        let data = pixels.data;

        self.send_command(WebGLCommand::CompressedTexSubImage2D {
            target: target.as_gl_constant(),
            level: level as i32,
            xoffset,
            yoffset,
            size: Size2D::new(width, height),
            format,
            data: data.into(),
        });
    }

    pub(crate) fn uniform1iv(
        &self,
        location: Option<&WebGLUniformLocation>,
        val: Int32ArrayOrLongSequence,
        src_offset: u32,
        src_length: u32,
    ) {
        self.with_location(location, |location| {
            match location.type_() {
                constants::BOOL |
                constants::INT |
                constants::SAMPLER_2D |
                constants::SAMPLER_CUBE => {},
                _ => return Err(InvalidOperation),
            }

            let val = self.uniform_vec_section_int(val, src_offset, src_length, 1, location)?;

            match location.type_() {
                constants::SAMPLER_2D | constants::SAMPLER_CUBE => {
                    for &v in val
                        .iter()
                        .take(cmp::min(location.size().unwrap_or(1) as usize, val.len()))
                    {
                        if v < 0 || v as u32 >= self.limits.max_combined_texture_image_units {
                            return Err(InvalidValue);
                        }
                    }
                },
                _ => {},
            }
            self.send_command(WebGLCommand::Uniform1iv(location.id(), val));
            Ok(())
        });
    }

    pub(crate) fn uniform1fv(
        &self,
        location: Option<&WebGLUniformLocation>,
        val: Float32ArrayOrUnrestrictedFloatSequence,
        src_offset: u32,
        src_length: u32,
    ) {
        self.with_location(location, |location| {
            match location.type_() {
                constants::BOOL | constants::FLOAT => {},
                _ => return Err(InvalidOperation),
            }
            let val = self.uniform_vec_section_float(val, src_offset, src_length, 1, location)?;
            self.send_command(WebGLCommand::Uniform1fv(location.id(), val));
            Ok(())
        });
    }

    pub(crate) fn uniform2fv(
        &self,
        location: Option<&WebGLUniformLocation>,
        val: Float32ArrayOrUnrestrictedFloatSequence,
        src_offset: u32,
        src_length: u32,
    ) {
        self.with_location(location, |location| {
            match location.type_() {
                constants::BOOL_VEC2 | constants::FLOAT_VEC2 => {},
                _ => return Err(InvalidOperation),
            }
            let val = self.uniform_vec_section_float(val, src_offset, src_length, 2, location)?;
            self.send_command(WebGLCommand::Uniform2fv(location.id(), val));
            Ok(())
        });
    }

    pub(crate) fn uniform2iv(
        &self,
        location: Option<&WebGLUniformLocation>,
        val: Int32ArrayOrLongSequence,
        src_offset: u32,
        src_length: u32,
    ) {
        self.with_location(location, |location| {
            match location.type_() {
                constants::BOOL_VEC2 | constants::INT_VEC2 => {},
                _ => return Err(InvalidOperation),
            }
            let val = self.uniform_vec_section_int(val, src_offset, src_length, 2, location)?;
            self.send_command(WebGLCommand::Uniform2iv(location.id(), val));
            Ok(())
        });
    }

    pub(crate) fn uniform3fv(
        &self,
        location: Option<&WebGLUniformLocation>,
        val: Float32ArrayOrUnrestrictedFloatSequence,
        src_offset: u32,
        src_length: u32,
    ) {
        self.with_location(location, |location| {
            match location.type_() {
                constants::BOOL_VEC3 | constants::FLOAT_VEC3 => {},
                _ => return Err(InvalidOperation),
            }
            let val = self.uniform_vec_section_float(val, src_offset, src_length, 3, location)?;
            self.send_command(WebGLCommand::Uniform3fv(location.id(), val));
            Ok(())
        });
    }

    pub(crate) fn uniform3iv(
        &self,
        location: Option<&WebGLUniformLocation>,
        val: Int32ArrayOrLongSequence,
        src_offset: u32,
        src_length: u32,
    ) {
        self.with_location(location, |location| {
            match location.type_() {
                constants::BOOL_VEC3 | constants::INT_VEC3 => {},
                _ => return Err(InvalidOperation),
            }
            let val = self.uniform_vec_section_int(val, src_offset, src_length, 3, location)?;
            self.send_command(WebGLCommand::Uniform3iv(location.id(), val));
            Ok(())
        });
    }

    pub(crate) fn uniform4iv(
        &self,
        location: Option<&WebGLUniformLocation>,
        val: Int32ArrayOrLongSequence,
        src_offset: u32,
        src_length: u32,
    ) {
        self.with_location(location, |location| {
            match location.type_() {
                constants::BOOL_VEC4 | constants::INT_VEC4 => {},
                _ => return Err(InvalidOperation),
            }
            let val = self.uniform_vec_section_int(val, src_offset, src_length, 4, location)?;
            self.send_command(WebGLCommand::Uniform4iv(location.id(), val));
            Ok(())
        });
    }

    pub(crate) fn uniform4fv(
        &self,
        location: Option<&WebGLUniformLocation>,
        val: Float32ArrayOrUnrestrictedFloatSequence,
        src_offset: u32,
        src_length: u32,
    ) {
        self.with_location(location, |location| {
            match location.type_() {
                constants::BOOL_VEC4 | constants::FLOAT_VEC4 => {},
                _ => return Err(InvalidOperation),
            }
            let val = self.uniform_vec_section_float(val, src_offset, src_length, 4, location)?;
            self.send_command(WebGLCommand::Uniform4fv(location.id(), val));
            Ok(())
        });
    }

    pub(crate) fn uniform_matrix_2fv(
        &self,
        location: Option<&WebGLUniformLocation>,
        transpose: bool,
        val: Float32ArrayOrUnrestrictedFloatSequence,
        src_offset: u32,
        src_length: u32,
    ) {
        self.with_location(location, |location| {
            match location.type_() {
                constants::FLOAT_MAT2 => {},
                _ => return Err(InvalidOperation),
            }
            let val =
                self.uniform_matrix_section(val, src_offset, src_length, transpose, 4, location)?;
            self.send_command(WebGLCommand::UniformMatrix2fv(location.id(), val));
            Ok(())
        });
    }

    pub(crate) fn uniform_matrix_3fv(
        &self,
        location: Option<&WebGLUniformLocation>,
        transpose: bool,
        val: Float32ArrayOrUnrestrictedFloatSequence,
        src_offset: u32,
        src_length: u32,
    ) {
        self.with_location(location, |location| {
            match location.type_() {
                constants::FLOAT_MAT3 => {},
                _ => return Err(InvalidOperation),
            }
            let val =
                self.uniform_matrix_section(val, src_offset, src_length, transpose, 9, location)?;
            self.send_command(WebGLCommand::UniformMatrix3fv(location.id(), val));
            Ok(())
        });
    }

    pub(crate) fn uniform_matrix_4fv(
        &self,
        location: Option<&WebGLUniformLocation>,
        transpose: bool,
        val: Float32ArrayOrUnrestrictedFloatSequence,
        src_offset: u32,
        src_length: u32,
    ) {
        self.with_location(location, |location| {
            match location.type_() {
                constants::FLOAT_MAT4 => {},
                _ => return Err(InvalidOperation),
            }
            let val =
                self.uniform_matrix_section(val, src_offset, src_length, transpose, 16, location)?;
            self.send_command(WebGLCommand::UniformMatrix4fv(location.id(), val));
            Ok(())
        });
    }

    pub(crate) fn get_buffer_param(
        &self,
        buffer: Option<DomRoot<WebGLBuffer>>,
        parameter: u32,
        mut retval: MutableHandleValue,
    ) {
        let buffer = handle_potential_webgl_error!(
            self,
            buffer.ok_or(InvalidOperation),
            return retval.set(NullValue())
        );

        retval.set(match parameter {
            constants::BUFFER_SIZE => Int32Value(buffer.capacity() as i32),
            constants::BUFFER_USAGE => Int32Value(buffer.usage() as i32),
            _ => {
                self.webgl_error(InvalidEnum);
                NullValue()
            },
        })
    }
}

#[cfg(not(feature = "webgl_backtrace"))]
#[inline]
pub(crate) fn capture_webgl_backtrace<T: DomObject>(_: &T) -> WebGLCommandBacktrace {
    WebGLCommandBacktrace {}
}

#[cfg(feature = "webgl_backtrace")]
#[cfg_attr(feature = "webgl_backtrace", allow(unsafe_code))]
pub(crate) fn capture_webgl_backtrace<T: DomObject>(obj: &T) -> WebGLCommandBacktrace {
    let bt = Backtrace::new();
    unsafe {
        capture_stack!(in(*obj.global().get_cx()) let stack);
        WebGLCommandBacktrace {
            backtrace: format!("{:?}", bt),
            js_backtrace: stack.and_then(|s| s.as_string(None, js::jsapi::StackFormat::Default)),
        }
    }
}

impl Drop for WebGLRenderingContext {
    fn drop(&mut self) {
        let _ = self.webgl_sender.send_remove();
    }
}

impl WebGLRenderingContextMethods<crate::DomTypeHolder> for WebGLRenderingContext {
    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.1
    fn Canvas(&self) -> HTMLCanvasElementOrOffscreenCanvas {
        self.canvas.clone()
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

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.5
    fn GetBufferParameter(
        &self,
        _cx: SafeJSContext,
        target: u32,
        parameter: u32,
        mut retval: MutableHandleValue,
    ) {
        let buffer = handle_potential_webgl_error!(
            self,
            self.bound_buffer(target),
            return retval.set(NullValue())
        );
        self.get_buffer_param(buffer, parameter, retval)
    }

    #[allow(unsafe_code)]
    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn GetParameter(&self, cx: SafeJSContext, parameter: u32, mut retval: MutableHandleValue) {
        if !self
            .extension_manager
            .is_get_parameter_name_enabled(parameter)
        {
            self.webgl_error(WebGLError::InvalidEnum);
            return retval.set(NullValue());
        }

        match parameter {
            constants::ARRAY_BUFFER_BINDING => unsafe {
                self.bound_buffer_array.get().to_jsval(*cx, retval);
                return;
            },
            constants::CURRENT_PROGRAM => unsafe {
                self.current_program.get().to_jsval(*cx, retval);
                return;
            },
            constants::ELEMENT_ARRAY_BUFFER_BINDING => unsafe {
                let buffer = self.current_vao().element_array_buffer().get();
                buffer.to_jsval(*cx, retval);
                return;
            },
            constants::FRAMEBUFFER_BINDING => unsafe {
                self.bound_draw_framebuffer.get().to_jsval(*cx, retval);
                return;
            },
            constants::RENDERBUFFER_BINDING => unsafe {
                self.bound_renderbuffer.get().to_jsval(*cx, retval);
                return;
            },
            constants::TEXTURE_BINDING_2D => unsafe {
                let texture = self
                    .textures
                    .active_texture_slot(constants::TEXTURE_2D, self.webgl_version())
                    .unwrap()
                    .get();
                texture.to_jsval(*cx, retval);
                return;
            },
            constants::TEXTURE_BINDING_CUBE_MAP => unsafe {
                let texture = self
                    .textures
                    .active_texture_slot(constants::TEXTURE_CUBE_MAP, self.webgl_version())
                    .unwrap()
                    .get();
                texture.to_jsval(*cx, retval);
                return;
            },
            OESVertexArrayObjectConstants::VERTEX_ARRAY_BINDING_OES => unsafe {
                let vao = self.current_vao.get().filter(|vao| vao.id().is_some());
                vao.to_jsval(*cx, retval);
                return;
            },
            // In readPixels we currently support RGBA/UBYTE only.  If
            // we wanted to support other formats, we could ask the
            // driver, but we would need to check for
            // GL_OES_read_format support (assuming an underlying GLES
            // driver. Desktop is happy to format convert for us).
            constants::IMPLEMENTATION_COLOR_READ_FORMAT => {
                if self.validate_framebuffer().is_err() {
                    self.webgl_error(InvalidOperation);
                    return retval.set(NullValue());
                }
                return retval.set(Int32Value(constants::RGBA as i32));
            },
            constants::IMPLEMENTATION_COLOR_READ_TYPE => {
                if self.validate_framebuffer().is_err() {
                    self.webgl_error(InvalidOperation);
                    return retval.set(NullValue());
                }
                return retval.set(Int32Value(constants::UNSIGNED_BYTE as i32));
            },
            constants::COMPRESSED_TEXTURE_FORMATS => unsafe {
                let format_ids = self.extension_manager.get_tex_compression_ids();

                rooted!(in(*cx) let mut rval = ptr::null_mut::<JSObject>());
                Uint32Array::create(*cx, CreateWith::Slice(&format_ids), rval.handle_mut())
                    .unwrap();
                return retval.set(ObjectValue(rval.get()));
            },
            constants::VERSION => unsafe {
                "WebGL 1.0".to_jsval(*cx, retval);
                return;
            },
            constants::RENDERER | constants::VENDOR => unsafe {
                "Mozilla/Servo".to_jsval(*cx, retval);
                return;
            },
            constants::SHADING_LANGUAGE_VERSION => unsafe {
                "WebGL GLSL ES 1.0".to_jsval(*cx, retval);
                return;
            },
            constants::UNPACK_FLIP_Y_WEBGL => {
                let unpack = self.texture_unpacking_settings.get();
                retval.set(BooleanValue(unpack.contains(TextureUnpacking::FLIP_Y_AXIS)));
                return;
            },
            constants::UNPACK_PREMULTIPLY_ALPHA_WEBGL => {
                let unpack = self.texture_unpacking_settings.get();
                retval.set(BooleanValue(
                    unpack.contains(TextureUnpacking::PREMULTIPLY_ALPHA),
                ));
                return;
            },
            constants::PACK_ALIGNMENT => {
                retval.set(UInt32Value(self.texture_packing_alignment.get() as u32));
                return;
            },
            constants::UNPACK_ALIGNMENT => {
                retval.set(UInt32Value(self.texture_unpacking_alignment.get()));
                return;
            },
            constants::UNPACK_COLORSPACE_CONVERSION_WEBGL => {
                let unpack = self.texture_unpacking_settings.get();
                retval.set(UInt32Value(
                    if unpack.contains(TextureUnpacking::CONVERT_COLORSPACE) {
                        constants::BROWSER_DEFAULT_WEBGL
                    } else {
                        constants::NONE
                    },
                ));
                return;
            },
            _ => {},
        }

        // Handle any MAX_ parameters by retrieving the limits that were stored
        // when this context was created.
        let limit = match parameter {
            constants::MAX_VERTEX_ATTRIBS => Some(self.limits.max_vertex_attribs),
            constants::MAX_TEXTURE_SIZE => Some(self.limits.max_tex_size),
            constants::MAX_CUBE_MAP_TEXTURE_SIZE => Some(self.limits.max_cube_map_tex_size),
            constants::MAX_COMBINED_TEXTURE_IMAGE_UNITS => {
                Some(self.limits.max_combined_texture_image_units)
            },
            constants::MAX_FRAGMENT_UNIFORM_VECTORS => {
                Some(self.limits.max_fragment_uniform_vectors)
            },
            constants::MAX_RENDERBUFFER_SIZE => Some(self.limits.max_renderbuffer_size),
            constants::MAX_TEXTURE_IMAGE_UNITS => Some(self.limits.max_texture_image_units),
            constants::MAX_VARYING_VECTORS => Some(self.limits.max_varying_vectors),
            constants::MAX_VERTEX_TEXTURE_IMAGE_UNITS => {
                Some(self.limits.max_vertex_texture_image_units)
            },
            constants::MAX_VERTEX_UNIFORM_VECTORS => Some(self.limits.max_vertex_uniform_vectors),
            _ => None,
        };
        if let Some(limit) = limit {
            retval.set(UInt32Value(limit));
            return;
        }

        if let Ok(value) = self.capabilities.is_enabled(parameter) {
            retval.set(BooleanValue(value));
            return;
        }

        match handle_potential_webgl_error!(
            self,
            Parameter::from_u32(parameter),
            return retval.set(NullValue())
        ) {
            Parameter::Bool(param) => {
                let (sender, receiver) = webgl_channel().unwrap();
                self.send_command(WebGLCommand::GetParameterBool(param, sender));
                retval.set(BooleanValue(receiver.recv().unwrap()))
            },
            Parameter::Bool4(param) => unsafe {
                let (sender, receiver) = webgl_channel().unwrap();
                self.send_command(WebGLCommand::GetParameterBool4(param, sender));
                receiver.recv().unwrap().to_jsval(*cx, retval);
            },
            Parameter::Int(param) => {
                let (sender, receiver) = webgl_channel().unwrap();
                self.send_command(WebGLCommand::GetParameterInt(param, sender));
                retval.set(Int32Value(receiver.recv().unwrap()))
            },
            Parameter::Int2(param) => unsafe {
                let (sender, receiver) = webgl_channel().unwrap();
                self.send_command(WebGLCommand::GetParameterInt2(param, sender));
                rooted!(in(*cx) let mut rval = ptr::null_mut::<JSObject>());
                Int32Array::create(
                    *cx,
                    CreateWith::Slice(&receiver.recv().unwrap()),
                    rval.handle_mut(),
                )
                .unwrap();
                retval.set(ObjectValue(rval.get()))
            },
            Parameter::Int4(param) => unsafe {
                let (sender, receiver) = webgl_channel().unwrap();
                self.send_command(WebGLCommand::GetParameterInt4(param, sender));
                rooted!(in(*cx) let mut rval = ptr::null_mut::<JSObject>());
                Int32Array::create(
                    *cx,
                    CreateWith::Slice(&receiver.recv().unwrap()),
                    rval.handle_mut(),
                )
                .unwrap();
                retval.set(ObjectValue(rval.get()))
            },
            Parameter::Float(param) => {
                let (sender, receiver) = webgl_channel().unwrap();
                self.send_command(WebGLCommand::GetParameterFloat(param, sender));
                retval.set(DoubleValue(receiver.recv().unwrap() as f64))
            },
            Parameter::Float2(param) => unsafe {
                let (sender, receiver) = webgl_channel().unwrap();
                self.send_command(WebGLCommand::GetParameterFloat2(param, sender));
                rooted!(in(*cx) let mut rval = ptr::null_mut::<JSObject>());
                Float32Array::create(
                    *cx,
                    CreateWith::Slice(&receiver.recv().unwrap()),
                    rval.handle_mut(),
                )
                .unwrap();
                retval.set(ObjectValue(rval.get()))
            },
            Parameter::Float4(param) => unsafe {
                let (sender, receiver) = webgl_channel().unwrap();
                self.send_command(WebGLCommand::GetParameterFloat4(param, sender));
                rooted!(in(*cx) let mut rval = ptr::null_mut::<JSObject>());
                Float32Array::create(
                    *cx,
                    CreateWith::Slice(&receiver.recv().unwrap()),
                    rval.handle_mut(),
                )
                .unwrap();
                retval.set(ObjectValue(rval.get()))
            },
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn GetTexParameter(
        &self,
        _cx: SafeJSContext,
        target: u32,
        pname: u32,
        mut retval: MutableHandleValue,
    ) {
        let texture_slot = handle_potential_webgl_error!(
            self,
            self.textures
                .active_texture_slot(target, self.webgl_version()),
            return retval.set(NullValue())
        );
        let texture = handle_potential_webgl_error!(
            self,
            texture_slot.get().ok_or(InvalidOperation),
            return retval.set(NullValue())
        );

        if !self
            .extension_manager
            .is_get_tex_parameter_name_enabled(pname)
        {
            self.webgl_error(InvalidEnum);
            return retval.set(NullValue());
        }

        match pname {
            constants::TEXTURE_MAG_FILTER => return retval.set(UInt32Value(texture.mag_filter())),
            constants::TEXTURE_MIN_FILTER => return retval.set(UInt32Value(texture.min_filter())),
            _ => {},
        }

        let texparam = handle_potential_webgl_error!(
            self,
            TexParameter::from_u32(pname),
            return retval.set(NullValue())
        );
        if self.webgl_version() < texparam.required_webgl_version() {
            self.webgl_error(InvalidEnum);
            return retval.set(NullValue());
        }

        if let Some(value) = texture.maybe_get_tex_parameter(texparam) {
            match value {
                TexParameterValue::Float(v) => retval.set(DoubleValue(v as f64)),
                TexParameterValue::Int(v) => retval.set(Int32Value(v)),
                TexParameterValue::Bool(v) => retval.set(BooleanValue(v)),
            }
            return;
        }

        match texparam {
            TexParameter::Float(param) => {
                let (sender, receiver) = webgl_channel().unwrap();
                self.send_command(WebGLCommand::GetTexParameterFloat(target, param, sender));
                retval.set(DoubleValue(receiver.recv().unwrap() as f64))
            },
            TexParameter::Int(param) => {
                let (sender, receiver) = webgl_channel().unwrap();
                self.send_command(WebGLCommand::GetTexParameterInt(target, param, sender));
                retval.set(Int32Value(receiver.recv().unwrap()))
            },
            TexParameter::Bool(param) => {
                let (sender, receiver) = webgl_channel().unwrap();
                self.send_command(WebGLCommand::GetTexParameterBool(target, param, sender));
                retval.set(BooleanValue(receiver.recv().unwrap()))
            },
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
        let backtrace = capture_webgl_backtrace(self);
        if self
            .webgl_sender
            .send(WebGLCommand::GetContextAttributes(sender), backtrace)
            .is_err()
        {
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
            stencil: attrs.stencil,
        })
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.13
    fn IsContextLost(&self) -> bool {
        false
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.14
    fn GetSupportedExtensions(&self) -> Option<Vec<DOMString>> {
        self.extension_manager
            .init_once(|| self.get_gl_extensions());
        let extensions = self.extension_manager.get_supported_extensions();
        Some(
            extensions
                .iter()
                .map(|name| DOMString::from(*name))
                .collect(),
        )
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.14
    fn GetExtension(&self, _cx: SafeJSContext, name: DOMString) -> Option<NonNull<JSObject>> {
        self.extension_manager
            .init_once(|| self.get_gl_extensions());
        self.extension_manager.get_or_init_extension(&name, self)
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn ActiveTexture(&self, texture: u32) {
        handle_potential_webgl_error!(self, self.textures.set_active_unit_enum(texture), return);
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

        self.send_command(WebGLCommand::BlendFuncSeparate(
            src_rgb, dest_rgb, src_alpha, dest_alpha,
        ));
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
        let current_vao;
        let slot = match target {
            constants::ARRAY_BUFFER => &self.bound_buffer_array,
            constants::ELEMENT_ARRAY_BUFFER => {
                current_vao = self.current_vao();
                current_vao.element_array_buffer()
            },
            _ => return self.webgl_error(InvalidEnum),
        };
        self.bind_buffer_maybe(slot, target, buffer);
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.6
    fn BindFramebuffer(&self, target: u32, framebuffer: Option<&WebGLFramebuffer>) {
        handle_potential_webgl_error!(
            self,
            self.validate_new_framebuffer_binding(framebuffer),
            return
        );

        if target != constants::FRAMEBUFFER {
            return self.webgl_error(InvalidEnum);
        }

        self.bind_framebuffer_to(target, framebuffer, &self.bound_draw_framebuffer)
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
            },
            _ => {
                if renderbuffer.is_some() {
                    self.webgl_error(InvalidOperation);
                }

                self.bound_renderbuffer.set(None);
                // Unbind the currently bound renderbuffer
                self.send_command(WebGLCommand::BindRenderbuffer(target, None));
            },
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn BindTexture(&self, target: u32, texture: Option<&WebGLTexture>) {
        if let Some(texture) = texture {
            handle_potential_webgl_error!(self, self.validate_ownership(texture), return);
        }

        let texture_slot = handle_potential_webgl_error!(
            self,
            self.textures
                .active_texture_slot(target, self.webgl_version()),
            return
        );

        if let Some(texture) = texture {
            handle_potential_webgl_error!(self, texture.bind(target), return);
        } else {
            self.send_command(WebGLCommand::BindTexture(target, None));
        }
        texture_slot.set(texture);
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn GenerateMipmap(&self, target: u32) {
        let texture_slot = handle_potential_webgl_error!(
            self,
            self.textures
                .active_texture_slot(target, self.webgl_version()),
            return
        );
        let texture =
            handle_potential_webgl_error!(self, texture_slot.get().ok_or(InvalidOperation), return);
        handle_potential_webgl_error!(self, texture.generate_mipmap());
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.5
    fn BufferData_(&self, target: u32, data: Option<ArrayBufferViewOrArrayBuffer>, usage: u32) {
        let usage = handle_potential_webgl_error!(self, self.buffer_usage(usage), return);
        let bound_buffer = handle_potential_webgl_error!(self, self.bound_buffer(target), return);
        self.buffer_data(target, data, usage, bound_buffer)
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.5
    fn BufferData(&self, target: u32, size: i64, usage: u32) {
        let usage = handle_potential_webgl_error!(self, self.buffer_usage(usage), return);
        let bound_buffer = handle_potential_webgl_error!(self, self.bound_buffer(target), return);
        self.buffer_data_(target, size, usage, bound_buffer)
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.5
    #[allow(unsafe_code)]
    fn BufferSubData(&self, target: u32, offset: i64, data: ArrayBufferViewOrArrayBuffer) {
        let bound_buffer = handle_potential_webgl_error!(self, self.bound_buffer(target), return);
        self.buffer_sub_data(target, offset, data, bound_buffer)
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    #[allow(unsafe_code)]
    fn CompressedTexImage2D(
        &self,
        target: u32,
        level: i32,
        internal_format: u32,
        width: i32,
        height: i32,
        border: i32,
        data: CustomAutoRooterGuard<ArrayBufferView>,
    ) {
        let data = unsafe { data.as_slice() };
        self.compressed_tex_image_2d(target, level, internal_format, width, height, border, data)
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    #[allow(unsafe_code)]
    fn CompressedTexSubImage2D(
        &self,
        target: u32,
        level: i32,
        xoffset: i32,
        yoffset: i32,
        width: i32,
        height: i32,
        format: u32,
        data: CustomAutoRooterGuard<ArrayBufferView>,
    ) {
        let data = unsafe { data.as_slice() };
        self.compressed_tex_sub_image_2d(
            target, level, xoffset, yoffset, width, height, format, data,
        )
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
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
        handle_potential_webgl_error!(self, self.validate_framebuffer(), return);

        let validator = CommonTexImage2DValidator::new(
            self,
            target,
            level,
            internal_format,
            width,
            height,
            border,
        );
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

        if texture.is_immutable() {
            return self.webgl_error(InvalidOperation);
        }

        let framebuffer_format = match self.bound_draw_framebuffer.get() {
            Some(fb) => match fb.attachment(constants::COLOR_ATTACHMENT0) {
                Some(WebGLFramebufferAttachmentRoot::Renderbuffer(rb)) => {
                    TexFormat::from_gl_constant(rb.internal_format())
                },
                Some(WebGLFramebufferAttachmentRoot::Texture(texture)) => texture
                    .image_info_for_target(&target, 0)
                    .map(|info| info.internal_format()),
                None => None,
            },
            None => {
                let attrs = self.GetContextAttributes().unwrap();
                Some(if attrs.alpha {
                    TexFormat::RGBA
                } else {
                    TexFormat::RGB
                })
            },
        };

        let framebuffer_format = match framebuffer_format {
            Some(f) => f,
            None => {
                self.webgl_error(InvalidOperation);
                return;
            },
        };

        match (framebuffer_format, internal_format) {
            (a, b) if a == b => (),
            (TexFormat::RGBA, TexFormat::RGB) => (),
            (TexFormat::RGBA, TexFormat::Alpha) => (),
            (TexFormat::RGBA, TexFormat::Luminance) => (),
            (TexFormat::RGBA, TexFormat::LuminanceAlpha) => (),
            (TexFormat::RGB, TexFormat::Luminance) => (),
            _ => {
                self.webgl_error(InvalidOperation);
                return;
            },
        }

        // NB: TexImage2D depth is always equal to 1
        handle_potential_webgl_error!(
            self,
            texture.initialize(target, width, height, 1, internal_format, level, None)
        );

        let msg = WebGLCommand::CopyTexImage2D(
            target.as_gl_constant(),
            level as i32,
            internal_format.as_gl_constant(),
            x,
            y,
            width as i32,
            height as i32,
            border as i32,
        );

        self.send_command(msg);
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
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
        handle_potential_webgl_error!(self, self.validate_framebuffer(), return);

        // NB: We use a dummy (valid) format and border in order to reuse the
        // common validations, but this should have its own validator.
        let validator = CommonTexImage2DValidator::new(
            self,
            target,
            level,
            TexFormat::RGBA.as_gl_constant(),
            width,
            height,
            0,
        );
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

        let image_info = match texture.image_info_for_target(&target, level) {
            Some(info) => info,
            None => return self.webgl_error(InvalidOperation),
        };

        // GL_INVALID_VALUE is generated if:
        //   - xoffset or yoffset is less than 0
        //   - x offset plus the width is greater than the texture width
        //   - y offset plus the height is greater than the texture height
        if xoffset < 0 ||
            (xoffset as u32 + width) > image_info.width() ||
            yoffset < 0 ||
            (yoffset as u32 + height) > image_info.height()
        {
            self.webgl_error(InvalidValue);
            return;
        }

        let msg = WebGLCommand::CopyTexSubImage2D(
            target.as_gl_constant(),
            level as i32,
            xoffset,
            yoffset,
            x,
            y,
            width as i32,
            height as i32,
        );

        self.send_command(msg);
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.11
    fn Clear(&self, mask: u32) {
        handle_potential_webgl_error!(self, self.validate_framebuffer(), return);
        if mask &
            !(constants::DEPTH_BUFFER_BIT |
                constants::STENCIL_BUFFER_BIT |
                constants::COLOR_BUFFER_BIT) !=
            0
        {
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
            constants::FRONT | constants::BACK | constants::FRONT_AND_BACK => {
                self.send_command(WebGLCommand::CullFace(mode))
            },
            _ => self.webgl_error(InvalidEnum),
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn FrontFace(&self, mode: u32) {
        match mode {
            constants::CW | constants::CCW => self.send_command(WebGLCommand::FrontFace(mode)),
            _ => self.webgl_error(InvalidEnum),
        }
    }
    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn DepthFunc(&self, func: u32) {
        match func {
            constants::NEVER |
            constants::LESS |
            constants::EQUAL |
            constants::LEQUAL |
            constants::GREATER |
            constants::NOTEQUAL |
            constants::GEQUAL |
            constants::ALWAYS => self.send_command(WebGLCommand::DepthFunc(func)),
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
                self.api_type,
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
            },
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
        if self.bound_buffer_array.get().is_some_and(|b| buffer == &*b) {
            self.bound_buffer_array.set(None);
            buffer.decrement_attached_counter(Operation::Infallible);
        }
        buffer.mark_for_deletion(Operation::Infallible);
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.6
    fn DeleteFramebuffer(&self, framebuffer: Option<&WebGLFramebuffer>) {
        if let Some(framebuffer) = framebuffer {
            // https://immersive-web.github.io/webxr/#opaque-framebuffer
            // Can opaque framebuffers be deleted?
            // https://github.com/immersive-web/webxr/issues/855
            handle_potential_webgl_error!(self, framebuffer.validate_transparent(), return);
            handle_potential_webgl_error!(self, self.validate_ownership(framebuffer), return);
            handle_object_deletion!(
                self,
                self.bound_draw_framebuffer,
                framebuffer,
                Some(WebGLCommand::BindFramebuffer(
                    framebuffer.target().unwrap(),
                    WebGLFramebufferBindingRequest::Default
                ))
            );
            framebuffer.delete(Operation::Infallible)
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.7
    fn DeleteRenderbuffer(&self, renderbuffer: Option<&WebGLRenderbuffer>) {
        if let Some(renderbuffer) = renderbuffer {
            handle_potential_webgl_error!(self, self.validate_ownership(renderbuffer), return);
            handle_object_deletion!(
                self,
                self.bound_renderbuffer,
                renderbuffer,
                Some(WebGLCommand::BindRenderbuffer(
                    constants::RENDERBUFFER,
                    None
                ))
            );
            renderbuffer.delete(Operation::Infallible)
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
            let mut active_unit_enum = self.textures.active_unit_enum();
            for (unit_enum, slot) in self.textures.iter() {
                if let Some(target) = slot.unbind(texture) {
                    if unit_enum != active_unit_enum {
                        self.send_command(WebGLCommand::ActiveTexture(unit_enum));
                        active_unit_enum = unit_enum;
                    }
                    self.send_command(WebGLCommand::BindTexture(target, None));
                }
            }

            // Restore bound texture unit if it has been changed.
            if active_unit_enum != self.textures.active_unit_enum() {
                self.send_command(WebGLCommand::ActiveTexture(
                    self.textures.active_unit_enum(),
                ));
            }

            texture.delete(Operation::Infallible)
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn DeleteProgram(&self, program: Option<&WebGLProgram>) {
        if let Some(program) = program {
            handle_potential_webgl_error!(self, self.validate_ownership(program), return);
            program.mark_for_deletion(Operation::Infallible)
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn DeleteShader(&self, shader: Option<&WebGLShader>) {
        if let Some(shader) = shader {
            handle_potential_webgl_error!(self, self.validate_ownership(shader), return);
            shader.mark_for_deletion(Operation::Infallible)
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.11
    fn DrawArrays(&self, mode: u32, first: i32, count: i32) {
        handle_potential_webgl_error!(self, self.draw_arrays_instanced(mode, first, count, 1));
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.11
    fn DrawElements(&self, mode: u32, count: i32, type_: u32, offset: i64) {
        handle_potential_webgl_error!(
            self,
            self.draw_elements_instanced(mode, count, type_, offset, 1)
        );
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn EnableVertexAttribArray(&self, attrib_id: u32) {
        if attrib_id >= self.limits.max_vertex_attribs {
            return self.webgl_error(InvalidValue);
        }
        match self.webgl_version() {
            WebGLVersion::WebGL1 => self
                .current_vao()
                .enabled_vertex_attrib_array(attrib_id, true),
            WebGLVersion::WebGL2 => self
                .current_vao_webgl2()
                .enabled_vertex_attrib_array(attrib_id, true),
        };
        self.send_command(WebGLCommand::EnableVertexAttribArray(attrib_id));
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn DisableVertexAttribArray(&self, attrib_id: u32) {
        if attrib_id >= self.limits.max_vertex_attribs {
            return self.webgl_error(InvalidValue);
        }
        match self.webgl_version() {
            WebGLVersion::WebGL1 => self
                .current_vao()
                .enabled_vertex_attrib_array(attrib_id, false),
            WebGLVersion::WebGL2 => self
                .current_vao_webgl2()
                .enabled_vertex_attrib_array(attrib_id, false),
        };
        self.send_command(WebGLCommand::DisableVertexAttribArray(attrib_id));
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn GetActiveUniform(
        &self,
        program: &WebGLProgram,
        index: u32,
    ) -> Option<DomRoot<WebGLActiveInfo>> {
        handle_potential_webgl_error!(self, self.validate_ownership(program), return None);
        match program.get_active_uniform(index) {
            Ok(ret) => Some(ret),
            Err(e) => {
                self.webgl_error(e);
                None
            },
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn GetActiveAttrib(
        &self,
        program: &WebGLProgram,
        index: u32,
    ) -> Option<DomRoot<WebGLActiveInfo>> {
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
    fn GetFramebufferAttachmentParameter(
        &self,
        cx: SafeJSContext,
        target: u32,
        attachment: u32,
        pname: u32,
        mut retval: MutableHandleValue,
    ) {
        // Check if currently bound framebuffer is non-zero as per spec.
        if let Some(fb) = self.bound_draw_framebuffer.get() {
            // Opaque framebuffers cannot have their attachments inspected
            // https://immersive-web.github.io/webxr/#opaque-framebuffer
            handle_potential_webgl_error!(
                self,
                fb.validate_transparent(),
                return retval.set(NullValue())
            );
        } else {
            self.webgl_error(InvalidOperation);
            return retval.set(NullValue());
        }

        // Note: commented out stuff is for the WebGL2 standard.
        let target_matches = match target {
            // constants::READ_FRAMEBUFFER |
            // constants::DRAW_FRAMEBUFFER => true,
            constants::FRAMEBUFFER => true,
            _ => false,
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
            _ => false,
        };

        let bound_attachment_matches = match self
            .bound_draw_framebuffer
            .get()
            .unwrap()
            .attachment(attachment)
        {
            Some(attachment_root) => match attachment_root {
                WebGLFramebufferAttachmentRoot::Renderbuffer(_) => matches!(
                    pname,
                    constants::FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE |
                        constants::FRAMEBUFFER_ATTACHMENT_OBJECT_NAME
                ),
                WebGLFramebufferAttachmentRoot::Texture(_) => matches!(
                    pname,
                    constants::FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE |
                        constants::FRAMEBUFFER_ATTACHMENT_OBJECT_NAME |
                        constants::FRAMEBUFFER_ATTACHMENT_TEXTURE_LEVEL |
                        constants::FRAMEBUFFER_ATTACHMENT_TEXTURE_CUBE_MAP_FACE
                ),
            },
            _ => matches!(pname, constants::FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE),
        };

        if !target_matches || !attachment_matches || !pname_matches || !bound_attachment_matches {
            self.webgl_error(InvalidEnum);
            return retval.set(NullValue());
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
            let fb = self.bound_draw_framebuffer.get().unwrap();
            if let Some(webgl_attachment) = fb.attachment(attachment) {
                match webgl_attachment {
                    WebGLFramebufferAttachmentRoot::Renderbuffer(rb) => unsafe {
                        rb.to_jsval(*cx, retval);
                        return;
                    },
                    WebGLFramebufferAttachmentRoot::Texture(texture) => unsafe {
                        texture.to_jsval(*cx, retval);
                        return;
                    },
                }
            }
            self.webgl_error(InvalidEnum);
            return retval.set(NullValue());
        }

        let (sender, receiver) = webgl_channel().unwrap();
        self.send_command(WebGLCommand::GetFramebufferAttachmentParameter(
            target, attachment, pname, sender,
        ));

        retval.set(Int32Value(receiver.recv().unwrap()))
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.7
    fn GetRenderbufferParameter(
        &self,
        _cx: SafeJSContext,
        target: u32,
        pname: u32,
        mut retval: MutableHandleValue,
    ) {
        // We do not check to see if the renderbuffer came from an opaque framebuffer
        // https://github.com/immersive-web/webxr/issues/862
        let target_matches = target == constants::RENDERBUFFER;

        let pname_matches = matches!(
            pname,
            constants::RENDERBUFFER_WIDTH |
                constants::RENDERBUFFER_HEIGHT |
                constants::RENDERBUFFER_INTERNAL_FORMAT |
                constants::RENDERBUFFER_RED_SIZE |
                constants::RENDERBUFFER_GREEN_SIZE |
                constants::RENDERBUFFER_BLUE_SIZE |
                constants::RENDERBUFFER_ALPHA_SIZE |
                constants::RENDERBUFFER_DEPTH_SIZE |
                constants::RENDERBUFFER_STENCIL_SIZE
        );

        if !target_matches || !pname_matches {
            self.webgl_error(InvalidEnum);
            return retval.set(NullValue());
        }

        if self.bound_renderbuffer.get().is_none() {
            self.webgl_error(InvalidOperation);
            return retval.set(NullValue());
        }

        let result = if pname == constants::RENDERBUFFER_INTERNAL_FORMAT {
            let rb = self.bound_renderbuffer.get().unwrap();
            rb.internal_format() as i32
        } else {
            let (sender, receiver) = webgl_channel().unwrap();
            self.send_command(WebGLCommand::GetRenderbufferParameter(
                target, pname, sender,
            ));
            receiver.recv().unwrap()
        };

        retval.set(Int32Value(result))
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn GetProgramInfoLog(&self, program: &WebGLProgram) -> Option<DOMString> {
        handle_potential_webgl_error!(self, self.validate_ownership(program), return None);
        match program.get_info_log() {
            Ok(value) => Some(DOMString::from(value)),
            Err(e) => {
                self.webgl_error(e);
                None
            },
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn GetProgramParameter(
        &self,
        _: SafeJSContext,
        program: &WebGLProgram,
        param: u32,
        mut retval: MutableHandleValue,
    ) {
        handle_potential_webgl_error!(
            self,
            self.validate_ownership(program),
            return retval.set(NullValue())
        );
        if program.is_deleted() {
            self.webgl_error(InvalidOperation);
            return retval.set(NullValue());
        }
        retval.set(match param {
            constants::DELETE_STATUS => BooleanValue(program.is_marked_for_deletion()),
            constants::LINK_STATUS => BooleanValue(program.is_linked()),
            constants::VALIDATE_STATUS => {
                // FIXME(nox): This could be cached on the DOM side when we call validateProgram
                // but I'm not sure when the value should be reset.
                let (sender, receiver) = webgl_channel().unwrap();
                self.send_command(WebGLCommand::GetProgramValidateStatus(program.id(), sender));
                BooleanValue(receiver.recv().unwrap())
            },
            constants::ATTACHED_SHADERS => {
                // FIXME(nox): This allocates a vector and roots a couple of shaders for nothing.
                Int32Value(
                    program
                        .attached_shaders()
                        .map(|shaders| shaders.len() as i32)
                        .unwrap_or(0),
                )
            },
            constants::ACTIVE_ATTRIBUTES => Int32Value(program.active_attribs().len() as i32),
            constants::ACTIVE_UNIFORMS => Int32Value(program.active_uniforms().len() as i32),
            _ => {
                self.webgl_error(InvalidEnum);
                NullValue()
            },
        })
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn GetShaderInfoLog(&self, shader: &WebGLShader) -> Option<DOMString> {
        handle_potential_webgl_error!(self, self.validate_ownership(shader), return None);
        Some(shader.info_log())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn GetShaderParameter(
        &self,
        _: SafeJSContext,
        shader: &WebGLShader,
        param: u32,
        mut retval: MutableHandleValue,
    ) {
        handle_potential_webgl_error!(
            self,
            self.validate_ownership(shader),
            return retval.set(NullValue())
        );
        if shader.is_deleted() {
            self.webgl_error(InvalidValue);
            return retval.set(NullValue());
        }
        retval.set(match param {
            constants::DELETE_STATUS => BooleanValue(shader.is_marked_for_deletion()),
            constants::COMPILE_STATUS => BooleanValue(shader.successfully_compiled()),
            constants::SHADER_TYPE => UInt32Value(shader.gl_type()),
            _ => {
                self.webgl_error(InvalidEnum);
                NullValue()
            },
        })
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn GetShaderPrecisionFormat(
        &self,
        shader_type: u32,
        precision_type: u32,
    ) -> Option<DomRoot<WebGLShaderPrecisionFormat>> {
        match shader_type {
            constants::FRAGMENT_SHADER | constants::VERTEX_SHADER => (),
            _ => {
                self.webgl_error(InvalidEnum);
                return None;
            },
        }

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
        self.send_command(WebGLCommand::GetShaderPrecisionFormat(
            shader_type,
            precision_type,
            sender,
        ));

        let (range_min, range_max, precision) = receiver.recv().unwrap();
        Some(WebGLShaderPrecisionFormat::new(
            self.global().as_window(),
            range_min,
            range_max,
            precision,
            CanGc::note(),
        ))
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
    fn GetVertexAttrib(
        &self,
        cx: SafeJSContext,
        index: u32,
        param: u32,
        mut retval: MutableHandleValue,
    ) {
        let mut get_attrib = |data: Ref<VertexAttribData>| {
            if param == constants::CURRENT_VERTEX_ATTRIB {
                let attrib = self.current_vertex_attribs.borrow()[index as usize];
                match attrib {
                    VertexAttrib::Float(x, y, z, w) => {
                        let value = [x, y, z, w];
                        unsafe {
                            rooted!(in(*cx) let mut result = ptr::null_mut::<JSObject>());
                            Float32Array::create(
                                *cx,
                                CreateWith::Slice(&value),
                                result.handle_mut(),
                            )
                            .unwrap();
                            return retval.set(ObjectValue(result.get()));
                        }
                    },
                    VertexAttrib::Int(x, y, z, w) => {
                        let value = [x, y, z, w];
                        unsafe {
                            rooted!(in(*cx) let mut result = ptr::null_mut::<JSObject>());
                            Int32Array::create(*cx, CreateWith::Slice(&value), result.handle_mut())
                                .unwrap();
                            return retval.set(ObjectValue(result.get()));
                        }
                    },
                    VertexAttrib::Uint(x, y, z, w) => {
                        let value = [x, y, z, w];
                        unsafe {
                            rooted!(in(*cx) let mut result = ptr::null_mut::<JSObject>());
                            Uint32Array::create(
                                *cx,
                                CreateWith::Slice(&value),
                                result.handle_mut(),
                            )
                            .unwrap();
                            return retval.set(ObjectValue(result.get()));
                        }
                    },
                };
            }
            if !self
                .extension_manager
                .is_get_vertex_attrib_name_enabled(param)
            {
                self.webgl_error(WebGLError::InvalidEnum);
                return retval.set(NullValue());
            }

            match param {
                constants::VERTEX_ATTRIB_ARRAY_ENABLED => {
                    retval.set(BooleanValue(data.enabled_as_array))
                },
                constants::VERTEX_ATTRIB_ARRAY_SIZE => retval.set(Int32Value(data.size as i32)),
                constants::VERTEX_ATTRIB_ARRAY_TYPE => retval.set(Int32Value(data.type_ as i32)),
                constants::VERTEX_ATTRIB_ARRAY_NORMALIZED => {
                    retval.set(BooleanValue(data.normalized))
                },
                constants::VERTEX_ATTRIB_ARRAY_STRIDE => retval.set(Int32Value(data.stride as i32)),
                constants::VERTEX_ATTRIB_ARRAY_BUFFER_BINDING => unsafe {
                    if let Some(buffer) = data.buffer() {
                        buffer.to_jsval(*cx, retval);
                    } else {
                        retval.set(NullValue());
                    }
                },
                ANGLEInstancedArraysConstants::VERTEX_ATTRIB_ARRAY_DIVISOR_ANGLE => {
                    retval.set(UInt32Value(data.divisor))
                },
                _ => {
                    self.webgl_error(InvalidEnum);
                    retval.set(NullValue())
                },
            }
        };

        match self.webgl_version() {
            WebGLVersion::WebGL1 => {
                let current_vao = self.current_vao();
                let data = handle_potential_webgl_error!(
                    self,
                    current_vao.get_vertex_attrib(index).ok_or(InvalidValue),
                    return retval.set(NullValue())
                );
                get_attrib(data)
            },
            WebGLVersion::WebGL2 => {
                let current_vao = self.current_vao_webgl2();
                let data = handle_potential_webgl_error!(
                    self,
                    current_vao.get_vertex_attrib(index).ok_or(InvalidValue),
                    return retval.set(NullValue())
                );
                get_attrib(data)
            },
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn GetVertexAttribOffset(&self, index: u32, pname: u32) -> i64 {
        if pname != constants::VERTEX_ATTRIB_ARRAY_POINTER {
            self.webgl_error(InvalidEnum);
            return 0;
        }
        match self.webgl_version() {
            WebGLVersion::WebGL1 => {
                let current_vao = self.current_vao();
                let data = handle_potential_webgl_error!(
                    self,
                    current_vao.get_vertex_attrib(index).ok_or(InvalidValue),
                    return 0
                );
                data.offset as i64
            },
            WebGLVersion::WebGL2 => {
                let current_vao = self.current_vao_webgl2();
                let data = handle_potential_webgl_error!(
                    self,
                    current_vao.get_vertex_attrib(index).ok_or(InvalidValue),
                    return 0
                );
                data.offset as i64
            },
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn Hint(&self, target: u32, mode: u32) {
        if target != constants::GENERATE_MIPMAP_HINT &&
            !self.extension_manager.is_hint_target_enabled(target)
        {
            return self.webgl_error(InvalidEnum);
        }

        match mode {
            constants::FASTEST | constants::NICEST | constants::DONT_CARE => (),

            _ => return self.webgl_error(InvalidEnum),
        }

        self.send_command(WebGLCommand::Hint(target, mode));
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.5
    fn IsBuffer(&self, buffer: Option<&WebGLBuffer>) -> bool {
        buffer.is_some_and(|buf| {
            self.validate_ownership(buf).is_ok() && buf.target().is_some() && !buf.is_deleted()
        })
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn IsEnabled(&self, cap: u32) -> bool {
        handle_potential_webgl_error!(self, self.capabilities.is_enabled(cap), false)
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.6
    fn IsFramebuffer(&self, frame_buffer: Option<&WebGLFramebuffer>) -> bool {
        frame_buffer.is_some_and(|buf| {
            self.validate_ownership(buf).is_ok() && buf.target().is_some() && !buf.is_deleted()
        })
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn IsProgram(&self, program: Option<&WebGLProgram>) -> bool {
        program.is_some_and(|p| self.validate_ownership(p).is_ok() && !p.is_deleted())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.7
    fn IsRenderbuffer(&self, render_buffer: Option<&WebGLRenderbuffer>) -> bool {
        render_buffer.is_some_and(|buf| {
            self.validate_ownership(buf).is_ok() && buf.ever_bound() && !buf.is_deleted()
        })
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9
    fn IsShader(&self, shader: Option<&WebGLShader>) -> bool {
        shader.is_some_and(|s| self.validate_ownership(s).is_ok() && !s.is_deleted())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn IsTexture(&self, texture: Option<&WebGLTexture>) -> bool {
        texture.is_some_and(|tex| {
            self.validate_ownership(tex).is_ok() && tex.target().is_some() && !tex.is_invalid()
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
                texture_settings.set(TextureUnpacking::FLIP_Y_AXIS, param_value != 0);
            },
            constants::UNPACK_PREMULTIPLY_ALPHA_WEBGL => {
                texture_settings.set(TextureUnpacking::PREMULTIPLY_ALPHA, param_value != 0);
            },
            constants::UNPACK_COLORSPACE_CONVERSION_WEBGL => {
                let convert = match param_value as u32 {
                    constants::BROWSER_DEFAULT_WEBGL => true,
                    constants::NONE => false,
                    _ => return self.webgl_error(InvalidEnum),
                };
                texture_settings.set(TextureUnpacking::CONVERT_COLORSPACE, convert);
            },
            constants::UNPACK_ALIGNMENT => {
                match param_value {
                    1 | 2 | 4 | 8 => (),
                    _ => return self.webgl_error(InvalidValue),
                }
                self.texture_unpacking_alignment.set(param_value as u32);
                return;
            },
            constants::PACK_ALIGNMENT => {
                match param_value {
                    1 | 2 | 4 | 8 => (),
                    _ => return self.webgl_error(InvalidValue),
                }
                // We never actually change the actual value on the GL side
                // because it's better to receive the pixels without the padding
                // and then write the result at the right place in ReadPixels.
                self.texture_packing_alignment.set(param_value as u8);
                return;
            },
            _ => return self.webgl_error(InvalidEnum),
        }
        self.texture_unpacking_settings.set(texture_settings);
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn PolygonOffset(&self, factor: f32, units: f32) {
        self.send_command(WebGLCommand::PolygonOffset(factor, units))
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.12
    #[allow(unsafe_code)]
    fn ReadPixels(
        &self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        format: u32,
        pixel_type: u32,
        mut pixels: CustomAutoRooterGuard<Option<ArrayBufferView>>,
    ) {
        handle_potential_webgl_error!(self, self.validate_framebuffer(), return);

        let pixels =
            handle_potential_webgl_error!(self, pixels.as_mut().ok_or(InvalidValue), return);

        if width < 0 || height < 0 {
            return self.webgl_error(InvalidValue);
        }

        if format != constants::RGBA || pixel_type != constants::UNSIGNED_BYTE {
            return self.webgl_error(InvalidOperation);
        }

        if pixels.get_array_type() != Type::Uint8 {
            return self.webgl_error(InvalidOperation);
        }

        let (fb_width, fb_height) = handle_potential_webgl_error!(
            self,
            self.get_current_framebuffer_size().ok_or(InvalidOperation),
            return
        );

        if width == 0 || height == 0 {
            return;
        }

        let bytes_per_pixel = 4;

        let row_len = handle_potential_webgl_error!(
            self,
            width.checked_mul(bytes_per_pixel).ok_or(InvalidOperation),
            return
        );

        let pack_alignment = self.texture_packing_alignment.get() as i32;
        let dest_padding = match row_len % pack_alignment {
            0 => 0,
            remainder => pack_alignment - remainder,
        };
        let dest_stride = row_len + dest_padding;

        let full_rows_len = handle_potential_webgl_error!(
            self,
            dest_stride.checked_mul(height - 1).ok_or(InvalidOperation),
            return
        );
        let required_dest_len = handle_potential_webgl_error!(
            self,
            full_rows_len.checked_add(row_len).ok_or(InvalidOperation),
            return
        );

        let dest = unsafe { pixels.as_mut_slice() };
        if dest.len() < required_dest_len as usize {
            return self.webgl_error(InvalidOperation);
        }

        let src_origin = Point2D::new(x, y);
        let src_size = Size2D::new(width as u32, height as u32);
        let fb_size = Size2D::new(fb_width as u32, fb_height as u32);
        let src_rect = match pixels::clip(src_origin, src_size.to_u64(), fb_size.to_u64()) {
            Some(rect) => rect,
            None => return,
        };

        // Note: we're casting a Rect<u64> back into a Rect<u32> here, but it's okay because
        //  it used u32 data types to begin with. It just got converted to Rect<u64> in
        //  pixels::clip
        let src_rect = src_rect.to_u32();

        let mut dest_offset = 0;
        if x < 0 {
            dest_offset += -x * bytes_per_pixel;
        }
        if y < 0 {
            dest_offset += -y * row_len;
        }

        let (sender, receiver) = ipc::bytes_channel().unwrap();
        self.send_command(WebGLCommand::ReadPixels(
            src_rect, format, pixel_type, sender,
        ));
        let src = receiver.recv().unwrap();

        let src_row_len = src_rect.size.width as usize * bytes_per_pixel as usize;
        for i in 0..src_rect.size.height {
            let dest_start = dest_offset as usize + i as usize * dest_stride as usize;
            let dest_end = dest_start + src_row_len;
            let src_start = i as usize * src_row_len;
            let src_end = src_start + src_row_len;
            dest[dest_start..dest_end].copy_from_slice(&src[src_start..src_end]);
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn SampleCoverage(&self, value: f32, invert: bool) {
        self.send_command(WebGLCommand::SampleCoverage(value, invert));
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.4
    fn Scissor(&self, x: i32, y: i32, width: i32, height: i32) {
        if width < 0 || height < 0 {
            return self.webgl_error(InvalidValue);
        }

        let width = width as u32;
        let height = height as u32;

        self.current_scissor.set((x, y, width, height));
        self.send_command(WebGLCommand::Scissor(x, y, width, height));
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn StencilFunc(&self, func: u32, ref_: i32, mask: u32) {
        match func {
            constants::NEVER |
            constants::LESS |
            constants::EQUAL |
            constants::LEQUAL |
            constants::GREATER |
            constants::NOTEQUAL |
            constants::GEQUAL |
            constants::ALWAYS => self.send_command(WebGLCommand::StencilFunc(func, ref_, mask)),
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
            constants::NEVER |
            constants::LESS |
            constants::EQUAL |
            constants::LEQUAL |
            constants::GREATER |
            constants::NOTEQUAL |
            constants::GEQUAL |
            constants::ALWAYS => {
                self.send_command(WebGLCommand::StencilFuncSeparate(face, func, ref_, mask))
            },
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
            constants::FRONT | constants::BACK | constants::FRONT_AND_BACK => {
                self.send_command(WebGLCommand::StencilMaskSeparate(face, mask))
            },
            _ => self.webgl_error(InvalidEnum),
        };
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3
    fn StencilOp(&self, fail: u32, zfail: u32, zpass: u32) {
        if self.validate_stencil_actions(fail) &&
            self.validate_stencil_actions(zfail) &&
            self.validate_stencil_actions(zpass)
        {
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

        if self.validate_stencil_actions(fail) &&
            self.validate_stencil_actions(zfail) &&
            self.validate_stencil_actions(zpass)
        {
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
    fn Uniform1f(&self, location: Option<&WebGLUniformLocation>, val: f32) {
        self.with_location(location, |location| {
            match location.type_() {
                constants::BOOL | constants::FLOAT => {},
                _ => return Err(InvalidOperation),
            }
            self.send_command(WebGLCommand::Uniform1f(location.id(), val));
            Ok(())
        });
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform1i(&self, location: Option<&WebGLUniformLocation>, val: i32) {
        self.with_location(location, |location| {
            match location.type_() {
                constants::BOOL | constants::INT => {},
                constants::SAMPLER_2D | constants::SAMPLER_CUBE => {
                    if val < 0 || val as u32 >= self.limits.max_combined_texture_image_units {
                        return Err(InvalidValue);
                    }
                },
                _ => return Err(InvalidOperation),
            }
            self.send_command(WebGLCommand::Uniform1i(location.id(), val));
            Ok(())
        });
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform1iv(&self, location: Option<&WebGLUniformLocation>, val: Int32ArrayOrLongSequence) {
        self.uniform1iv(location, val, 0, 0)
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform1fv(
        &self,
        location: Option<&WebGLUniformLocation>,
        val: Float32ArrayOrUnrestrictedFloatSequence,
    ) {
        self.uniform1fv(location, val, 0, 0)
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform2f(&self, location: Option<&WebGLUniformLocation>, x: f32, y: f32) {
        self.with_location(location, |location| {
            match location.type_() {
                constants::BOOL_VEC2 | constants::FLOAT_VEC2 => {},
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
        self.uniform2fv(location, val, 0, 0)
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform2i(&self, location: Option<&WebGLUniformLocation>, x: i32, y: i32) {
        self.with_location(location, |location| {
            match location.type_() {
                constants::BOOL_VEC2 | constants::INT_VEC2 => {},
                _ => return Err(InvalidOperation),
            }
            self.send_command(WebGLCommand::Uniform2i(location.id(), x, y));
            Ok(())
        });
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform2iv(&self, location: Option<&WebGLUniformLocation>, val: Int32ArrayOrLongSequence) {
        self.uniform2iv(location, val, 0, 0)
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform3f(&self, location: Option<&WebGLUniformLocation>, x: f32, y: f32, z: f32) {
        self.with_location(location, |location| {
            match location.type_() {
                constants::BOOL_VEC3 | constants::FLOAT_VEC3 => {},
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
        self.uniform3fv(location, val, 0, 0)
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform3i(&self, location: Option<&WebGLUniformLocation>, x: i32, y: i32, z: i32) {
        self.with_location(location, |location| {
            match location.type_() {
                constants::BOOL_VEC3 | constants::INT_VEC3 => {},
                _ => return Err(InvalidOperation),
            }
            self.send_command(WebGLCommand::Uniform3i(location.id(), x, y, z));
            Ok(())
        });
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform3iv(&self, location: Option<&WebGLUniformLocation>, val: Int32ArrayOrLongSequence) {
        self.uniform3iv(location, val, 0, 0)
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform4i(&self, location: Option<&WebGLUniformLocation>, x: i32, y: i32, z: i32, w: i32) {
        self.with_location(location, |location| {
            match location.type_() {
                constants::BOOL_VEC4 | constants::INT_VEC4 => {},
                _ => return Err(InvalidOperation),
            }
            self.send_command(WebGLCommand::Uniform4i(location.id(), x, y, z, w));
            Ok(())
        });
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform4iv(&self, location: Option<&WebGLUniformLocation>, val: Int32ArrayOrLongSequence) {
        self.uniform4iv(location, val, 0, 0)
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn Uniform4f(&self, location: Option<&WebGLUniformLocation>, x: f32, y: f32, z: f32, w: f32) {
        self.with_location(location, |location| {
            match location.type_() {
                constants::BOOL_VEC4 | constants::FLOAT_VEC4 => {},
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
        self.uniform4fv(location, val, 0, 0)
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn UniformMatrix2fv(
        &self,
        location: Option<&WebGLUniformLocation>,
        transpose: bool,
        val: Float32ArrayOrUnrestrictedFloatSequence,
    ) {
        self.uniform_matrix_2fv(location, transpose, val, 0, 0)
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn UniformMatrix3fv(
        &self,
        location: Option<&WebGLUniformLocation>,
        transpose: bool,
        val: Float32ArrayOrUnrestrictedFloatSequence,
    ) {
        self.uniform_matrix_3fv(location, transpose, val, 0, 0)
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    fn UniformMatrix4fv(
        &self,
        location: Option<&WebGLUniformLocation>,
        transpose: bool,
        val: Float32ArrayOrUnrestrictedFloatSequence,
    ) {
        self.uniform_matrix_4fv(location, transpose, val, 0, 0)
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10
    #[allow(unsafe_code)]
    fn GetUniform(
        &self,
        cx: SafeJSContext,
        program: &WebGLProgram,
        location: &WebGLUniformLocation,
        mut rval: MutableHandleValue,
    ) {
        handle_potential_webgl_error!(
            self,
            self.uniform_check_program(program, location),
            return rval.set(NullValue())
        );

        let triple = (self, program.id(), location.id());

        match location.type_() {
            constants::BOOL => rval.set(BooleanValue(uniform_get(
                triple,
                WebGLCommand::GetUniformBool,
            ))),
            constants::BOOL_VEC2 => unsafe {
                uniform_get(triple, WebGLCommand::GetUniformBool2).to_jsval(*cx, rval);
            },
            constants::BOOL_VEC3 => unsafe {
                uniform_get(triple, WebGLCommand::GetUniformBool3).to_jsval(*cx, rval);
            },
            constants::BOOL_VEC4 => unsafe {
                uniform_get(triple, WebGLCommand::GetUniformBool4).to_jsval(*cx, rval);
            },
            constants::INT | constants::SAMPLER_2D | constants::SAMPLER_CUBE => {
                rval.set(Int32Value(uniform_get(triple, WebGLCommand::GetUniformInt)))
            },
            constants::INT_VEC2 => unsafe {
                uniform_typed::<Int32>(
                    *cx,
                    &uniform_get(triple, WebGLCommand::GetUniformInt2),
                    rval,
                )
            },
            constants::INT_VEC3 => unsafe {
                uniform_typed::<Int32>(
                    *cx,
                    &uniform_get(triple, WebGLCommand::GetUniformInt3),
                    rval,
                )
            },
            constants::INT_VEC4 => unsafe {
                uniform_typed::<Int32>(
                    *cx,
                    &uniform_get(triple, WebGLCommand::GetUniformInt4),
                    rval,
                )
            },
            constants::FLOAT => rval
                .set(DoubleValue(
                    uniform_get(triple, WebGLCommand::GetUniformFloat) as f64,
                )),
            constants::FLOAT_VEC2 => unsafe {
                uniform_typed::<Float32>(
                    *cx,
                    &uniform_get(triple, WebGLCommand::GetUniformFloat2),
                    rval,
                )
            },
            constants::FLOAT_VEC3 => unsafe {
                uniform_typed::<Float32>(
                    *cx,
                    &uniform_get(triple, WebGLCommand::GetUniformFloat3),
                    rval,
                )
            },
            constants::FLOAT_VEC4 | constants::FLOAT_MAT2 => unsafe {
                uniform_typed::<Float32>(
                    *cx,
                    &uniform_get(triple, WebGLCommand::GetUniformFloat4),
                    rval,
                )
            },
            constants::FLOAT_MAT3 => unsafe {
                uniform_typed::<Float32>(
                    *cx,
                    &uniform_get(triple, WebGLCommand::GetUniformFloat9),
                    rval,
                )
            },
            constants::FLOAT_MAT4 => unsafe {
                uniform_typed::<Float32>(
                    *cx,
                    &uniform_get(triple, WebGLCommand::GetUniformFloat16),
                    rval,
                )
            },
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
            _ => {},
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
        if values.is_empty() {
            // https://github.com/KhronosGroup/WebGL/issues/2700
            return self.webgl_error(InvalidValue);
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
            // https://github.com/KhronosGroup/WebGL/issues/2700
            return self.webgl_error(InvalidValue);
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
            // https://github.com/KhronosGroup/WebGL/issues/2700
            return self.webgl_error(InvalidValue);
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
            // https://github.com/KhronosGroup/WebGL/issues/2700
            return self.webgl_error(InvalidValue);
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
        let res = match self.webgl_version() {
            WebGLVersion::WebGL1 => self
                .current_vao()
                .vertex_attrib_pointer(index, size, type_, normalized, stride, offset),
            WebGLVersion::WebGL2 => self
                .current_vao_webgl2()
                .vertex_attrib_pointer(index, size, type_, normalized, stride, offset),
        };
        handle_potential_webgl_error!(self, res);
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.4
    fn Viewport(&self, x: i32, y: i32, width: i32, height: i32) {
        if width < 0 || height < 0 {
            return self.webgl_error(InvalidValue);
        }

        self.send_command(WebGLCommand::SetViewport(x, y, width, height))
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    #[allow(unsafe_code)]
    fn TexImage2D(
        &self,
        target: u32,
        level: i32,
        internal_format: i32,
        width: i32,
        height: i32,
        border: i32,
        format: u32,
        data_type: u32,
        pixels: CustomAutoRooterGuard<Option<ArrayBufferView>>,
    ) -> ErrorResult {
        if !self.extension_manager.is_tex_type_enabled(data_type) {
            self.webgl_error(InvalidEnum);
            return Ok(());
        }

        let validator = TexImage2DValidator::new(
            self,
            target,
            level,
            internal_format as u32,
            width,
            height,
            border,
            format,
            data_type,
        );

        let TexImage2DValidatorResult {
            texture,
            target,
            width,
            height,
            level,
            border,
            internal_format,
            format,
            data_type,
        } = match validator.validate() {
            Ok(result) => result,
            Err(_) => return Ok(()), // NB: The validator sets the correct error for us.
        };

        if !internal_format.compatible_data_types().contains(&data_type) {
            return {
                self.webgl_error(InvalidOperation);
                Ok(())
            };
        }
        if texture.is_immutable() {
            return {
                self.webgl_error(InvalidOperation);
                Ok(())
            };
        }

        let unpacking_alignment = self.texture_unpacking_alignment.get();

        let expected_byte_length = match self.validate_tex_image_2d_data(
            width,
            height,
            format,
            data_type,
            unpacking_alignment,
            pixels.as_ref(),
        ) {
            Ok(byte_length) => byte_length,
            Err(()) => return Ok(()),
        };

        // If data is null, a buffer of sufficient size
        // initialized to 0 is passed.
        let buff = match *pixels {
            None => IpcSharedMemory::from_bytes(&vec![0u8; expected_byte_length as usize]),
            Some(ref data) => IpcSharedMemory::from_bytes(unsafe { data.as_slice() }),
        };

        // From the WebGL spec:
        //
        //     "If pixels is non-null but its size is less than what
        //      is required by the specified width, height, format,
        //      type, and pixel storage parameters, generates an
        //      INVALID_OPERATION error."
        if buff.len() < expected_byte_length as usize {
            return {
                self.webgl_error(InvalidOperation);
                Ok(())
            };
        }

        let size = Size2D::new(width, height);

        if !self.validate_filterable_texture(
            &texture,
            target,
            level,
            internal_format,
            size,
            data_type,
        ) {
            // FIXME(nox): What is the spec for this? No error is emitted ever
            // by validate_filterable_texture.
            return Ok(());
        }

        let size = Size2D::new(width, height);

        self.tex_image_2d(
            &texture,
            target,
            data_type,
            internal_format,
            format,
            level,
            border,
            unpacking_alignment,
            size,
            TexSource::Pixels(TexPixels::from_array(buff, size)),
        );

        Ok(())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    fn TexImage2D_(
        &self,
        target: u32,
        level: i32,
        internal_format: i32,
        format: u32,
        data_type: u32,
        source: TexImageSource,
    ) -> ErrorResult {
        if !self.extension_manager.is_tex_type_enabled(data_type) {
            self.webgl_error(InvalidEnum);
            return Ok(());
        }

        let pixels = match self.get_image_pixels(source)? {
            Some(pixels) => pixels,
            None => return Ok(()),
        };

        let validator = TexImage2DValidator::new(
            self,
            target,
            level,
            internal_format as u32,
            pixels.size().width as i32,
            pixels.size().height as i32,
            0,
            format,
            data_type,
        );

        let TexImage2DValidatorResult {
            texture,
            target,
            level,
            border,
            internal_format,
            format,
            data_type,
            ..
        } = match validator.validate() {
            Ok(result) => result,
            Err(_) => return Ok(()), // NB: The validator sets the correct error for us.
        };

        if !internal_format.compatible_data_types().contains(&data_type) {
            return {
                self.webgl_error(InvalidOperation);
                Ok(())
            };
        }
        if texture.is_immutable() {
            return {
                self.webgl_error(InvalidOperation);
                Ok(())
            };
        }

        if !self.validate_filterable_texture(
            &texture,
            target,
            level,
            internal_format,
            pixels.size(),
            data_type,
        ) {
            // FIXME(nox): What is the spec for this? No error is emitted ever
            // by validate_filterable_texture.
            return Ok(());
        }

        self.tex_image_2d(
            &texture,
            target,
            data_type,
            internal_format,
            format,
            level,
            border,
            1,
            pixels.size(),
            TexSource::Pixels(pixels),
        );
        Ok(())
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8
    #[allow(unsafe_code)]
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
    ) -> ErrorResult {
        let validator = TexImage2DValidator::new(
            self, target, level, format, width, height, 0, format, data_type,
        );
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

        let expected_byte_length = match self.validate_tex_image_2d_data(
            width,
            height,
            format,
            data_type,
            unpacking_alignment,
            pixels.as_ref(),
        ) {
            Ok(byte_length) => byte_length,
            Err(()) => return Ok(()),
        };

        let buff = handle_potential_webgl_error!(
            self,
            pixels
                .as_ref()
                .map(|p| IpcSharedMemory::from_bytes(unsafe { p.as_slice() }))
                .ok_or(InvalidValue),
            return Ok(())
        );

        // From the WebGL spec:
        //
        //     "If pixels is non-null but its size is less than what
        //      is required by the specified width, height, format,
        //      type, and pixel storage parameters, generates an
        //      INVALID_OPERATION error."
        if buff.len() < expected_byte_length as usize {
            return {
                self.webgl_error(InvalidOperation);
                Ok(())
            };
        }

        self.tex_sub_image_2d(
            texture,
            target,
            level,
            xoffset,
            yoffset,
            format,
            data_type,
            unpacking_alignment,
            TexPixels::from_array(buff, Size2D::new(width, height)),
        );
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
        source: TexImageSource,
    ) -> ErrorResult {
        let pixels = match self.get_image_pixels(source)? {
            Some(pixels) => pixels,
            None => return Ok(()),
        };

        let validator = TexImage2DValidator::new(
            self,
            target,
            level,
            format,
            pixels.size().width as i32,
            pixels.size().height as i32,
            0,
            format,
            data_type,
        );
        let TexImage2DValidatorResult {
            texture,
            target,
            level,
            format,
            data_type,
            ..
        } = match validator.validate() {
            Ok(result) => result,
            Err(_) => return Ok(()), // NB: The validator sets the correct error for us.
        };

        self.tex_sub_image_2d(
            texture, target, level, xoffset, yoffset, format, data_type, 1, pixels,
        );
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

        match self.bound_draw_framebuffer.get() {
            Some(fb) => fb.check_status(),
            None => constants::FRAMEBUFFER_COMPLETE,
        }
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.7
    fn RenderbufferStorage(&self, target: u32, internal_format: u32, width: i32, height: i32) {
        self.renderbuffer_storage(target, 0, internal_format, width, height)
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

        match self.bound_draw_framebuffer.get() {
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

        // From the GLES 2.0.25 spec, page 113:
        //
        //     "level specifies the mipmap level of the texture image
        //      to be attached to the framebuffer and must be
        //      0. Otherwise, INVALID_VALUE is generated."
        if level != 0 {
            return self.webgl_error(InvalidValue);
        }

        match self.bound_draw_framebuffer.get() {
            Some(fb) => handle_potential_webgl_error!(
                self,
                fb.texture2d(attachment, textarget, texture, level)
            ),
            None => self.webgl_error(InvalidOperation),
        };
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9>
    fn GetAttachedShaders(&self, program: &WebGLProgram) -> Option<Vec<DomRoot<WebGLShader>>> {
        handle_potential_webgl_error!(self, self.validate_ownership(program), return None);
        handle_potential_webgl_error!(self, program.attached_shaders().map(Some), None)
    }

    /// <https://immersive-web.github.io/webxr/#dom-webglrenderingcontextbase-makexrcompatible>
    #[cfg(feature = "webxr")]
    fn MakeXRCompatible(&self, can_gc: CanGc) -> Rc<Promise> {
        // XXXManishearth Fill in with compatibility checks when rust-webxr supports this
        let p = Promise::new(&self.global(), can_gc);
        p.resolve_native(&());
        p
    }
}

impl LayoutCanvasRenderingContextHelpers for LayoutDom<'_, WebGLRenderingContext> {
    fn canvas_data_source(self) -> HTMLCanvasDataSource {
        (*self.unsafe_get()).layout_handle()
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
        capabilities!($name, $($rest,)* [$($tt)* $name = Self::$prev.bits() << 1;]);
    };
    ($prev:ident, [$($name:ident = $value:expr;)*]) => {
        #[derive(Clone, Copy, JSTraceable, MallocSizeOf)]
        pub(crate) struct CapFlags(u16);

        bitflags! {
            impl CapFlags: u16 {
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

#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
#[derive(JSTraceable, MallocSizeOf)]
pub(crate) struct Textures {
    active_unit: Cell<u32>,
    units: Box<[TextureUnit]>,
}

impl Textures {
    fn new(max_combined_textures: u32) -> Self {
        Self {
            active_unit: Default::default(),
            units: (0..max_combined_textures)
                .map(|_| Default::default())
                .collect::<Vec<_>>()
                .into(),
        }
    }

    pub(crate) fn active_unit_enum(&self) -> u32 {
        self.active_unit.get() + constants::TEXTURE0
    }

    fn set_active_unit_enum(&self, index: u32) -> WebGLResult<()> {
        if index < constants::TEXTURE0 || (index - constants::TEXTURE0) as usize > self.units.len()
        {
            return Err(InvalidEnum);
        }
        self.active_unit.set(index - constants::TEXTURE0);
        Ok(())
    }

    pub(crate) fn active_texture_slot(
        &self,
        target: u32,
        webgl_version: WebGLVersion,
    ) -> WebGLResult<&MutNullableDom<WebGLTexture>> {
        let active_unit = self.active_unit();
        let is_webgl2 = webgl_version == WebGLVersion::WebGL2;
        match target {
            constants::TEXTURE_2D => Ok(&active_unit.tex_2d),
            constants::TEXTURE_CUBE_MAP => Ok(&active_unit.tex_cube_map),
            WebGL2RenderingContextConstants::TEXTURE_2D_ARRAY if is_webgl2 => {
                Ok(&active_unit.tex_2d_array)
            },
            WebGL2RenderingContextConstants::TEXTURE_3D if is_webgl2 => Ok(&active_unit.tex_3d),
            _ => Err(InvalidEnum),
        }
    }

    pub(crate) fn active_texture_for_image_target(
        &self,
        target: TexImageTarget,
    ) -> Option<DomRoot<WebGLTexture>> {
        let active_unit = self.active_unit();
        match target {
            TexImageTarget::Texture2D => active_unit.tex_2d.get(),
            TexImageTarget::Texture2DArray => active_unit.tex_2d_array.get(),
            TexImageTarget::Texture3D => active_unit.tex_3d.get(),
            TexImageTarget::CubeMap |
            TexImageTarget::CubeMapPositiveX |
            TexImageTarget::CubeMapNegativeX |
            TexImageTarget::CubeMapPositiveY |
            TexImageTarget::CubeMapNegativeY |
            TexImageTarget::CubeMapPositiveZ |
            TexImageTarget::CubeMapNegativeZ => active_unit.tex_cube_map.get(),
        }
    }

    fn active_unit(&self) -> &TextureUnit {
        &self.units[self.active_unit.get() as usize]
    }

    fn iter(&self) -> impl Iterator<Item = (u32, &TextureUnit)> {
        self.units
            .iter()
            .enumerate()
            .map(|(index, unit)| (index as u32 + constants::TEXTURE0, unit))
    }
}

#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
#[derive(Default, JSTraceable, MallocSizeOf)]
struct TextureUnit {
    tex_2d: MutNullableDom<WebGLTexture>,
    tex_cube_map: MutNullableDom<WebGLTexture>,
    tex_2d_array: MutNullableDom<WebGLTexture>,
    tex_3d: MutNullableDom<WebGLTexture>,
}

impl TextureUnit {
    fn unbind(&self, texture: &WebGLTexture) -> Option<u32> {
        let fields = [
            (&self.tex_2d, constants::TEXTURE_2D),
            (&self.tex_cube_map, constants::TEXTURE_CUBE_MAP),
            (
                &self.tex_2d_array,
                WebGL2RenderingContextConstants::TEXTURE_2D_ARRAY,
            ),
            (&self.tex_3d, WebGL2RenderingContextConstants::TEXTURE_3D),
        ];
        for &(slot, target) in &fields {
            if slot.get().is_some_and(|t| texture == &*t) {
                slot.set(None);
                return Some(target);
            }
        }
        None
    }
}

pub(crate) struct TexPixels {
    data: IpcSharedMemory,
    size: Size2D<u32>,
    pixel_format: Option<PixelFormat>,
    premultiplied: bool,
}

impl TexPixels {
    fn new(
        data: IpcSharedMemory,
        size: Size2D<u32>,
        pixel_format: PixelFormat,
        premultiplied: bool,
    ) -> Self {
        Self {
            data,
            size,
            pixel_format: Some(pixel_format),
            premultiplied,
        }
    }

    pub(crate) fn from_array(data: IpcSharedMemory, size: Size2D<u32>) -> Self {
        Self {
            data,
            size,
            pixel_format: None,
            premultiplied: false,
        }
    }

    pub(crate) fn size(&self) -> Size2D<u32> {
        self.size
    }
}

pub(crate) enum TexSource {
    Pixels(TexPixels),
    BufferOffset(i64),
}

#[derive(JSTraceable)]
pub(crate) struct WebGLCommandSender {
    #[no_trace]
    sender: WebGLChan,
}

impl WebGLCommandSender {
    pub(crate) fn new(sender: WebGLChan) -> WebGLCommandSender {
        WebGLCommandSender { sender }
    }

    pub(crate) fn send(&self, msg: WebGLMsg) -> WebGLSendResult {
        self.sender.send(msg)
    }
}

#[derive(JSTraceable, MallocSizeOf)]
pub(crate) struct WebGLMessageSender {
    #[no_trace]
    sender: WebGLMsgSender,
}

impl Clone for WebGLMessageSender {
    fn clone(&self) -> WebGLMessageSender {
        WebGLMessageSender {
            sender: self.sender.clone(),
        }
    }
}

impl WebGLMessageSender {
    pub(crate) fn new(sender: WebGLMsgSender) -> WebGLMessageSender {
        WebGLMessageSender { sender }
    }

    pub(crate) fn context_id(&self) -> WebGLContextId {
        self.sender.context_id()
    }

    pub(crate) fn send(
        &self,
        msg: WebGLCommand,
        backtrace: WebGLCommandBacktrace,
    ) -> WebGLSendResult {
        self.sender.send(msg, backtrace)
    }

    pub(crate) fn send_resize(
        &self,
        size: Size2D<u32>,
        sender: WebGLSender<Result<(), String>>,
    ) -> WebGLSendResult {
        self.sender.send_resize(size, sender)
    }

    pub(crate) fn send_remove(&self) -> WebGLSendResult {
        self.sender.send_remove()
    }
}

fn array_buffer_type_to_sized_type(type_: Type) -> Option<SizedDataType> {
    match type_ {
        Type::Uint8 | Type::Uint8Clamped => Some(SizedDataType::Uint8),
        Type::Uint16 => Some(SizedDataType::Uint16),
        Type::Uint32 => Some(SizedDataType::Uint32),
        Type::Int8 => Some(SizedDataType::Int8),
        Type::Int16 => Some(SizedDataType::Int16),
        Type::Int32 => Some(SizedDataType::Int32),
        Type::Float32 => Some(SizedDataType::Float32),
        Type::Float16 |
        Type::Float64 |
        Type::BigInt64 |
        Type::BigUint64 |
        Type::MaxTypedArrayViewType |
        Type::Int64 |
        Type::Simd128 => None,
    }
}
