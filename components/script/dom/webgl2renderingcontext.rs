/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::cmp;
use std::ptr::{self, NonNull};

use canvas_traits::webgl::WebGLError::*;
use canvas_traits::webgl::{
    webgl_channel, GLContextAttributes, InternalFormatParameter, WebGLCommand, WebGLResult,
    WebGLVersion,
};
use dom_struct::dom_struct;
use euclid::default::{Point2D, Rect, Size2D};
use ipc_channel::ipc::{self, IpcSharedMemory};
use js::jsapi::{JSObject, Type};
use js::jsval::{
    BooleanValue, DoubleValue, Int32Value, JSVal, NullValue, ObjectValue, UInt32Value,
    UndefinedValue,
};
use js::rust::{CustomAutoRooterGuard, HandleObject};
use js::typedarray::{ArrayBufferView, CreateWith, Float32, Int32Array, Uint32, Uint32Array};
use script_layout_interface::HTMLCanvasDataSource;
use servo_config::pref;
use url::Host;

use crate::dom::bindings::codegen::Bindings::WebGL2RenderingContextBinding::{
    WebGL2RenderingContextConstants as constants, WebGL2RenderingContextMethods,
};
use crate::dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::{
    WebGLContextAttributes, WebGLRenderingContextMethods,
};
use crate::dom::bindings::codegen::UnionTypes::{
    ArrayBufferViewOrArrayBuffer, Float32ArrayOrUnrestrictedFloatSequence,
    ImageDataOrHTMLImageElementOrHTMLCanvasElementOrHTMLVideoElement, Int32ArrayOrLongSequence,
    Uint32ArrayOrUnsignedLongSequence,
};
use crate::dom::bindings::error::{ErrorResult, Fallible};
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot, LayoutDom, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmlcanvaselement::{HTMLCanvasElement, LayoutCanvasRenderingContextHelpers};
use crate::dom::webgl_validations::tex_image_2d::{
    TexImage2DValidator, TexImage2DValidatorResult, TexStorageValidator, TexStorageValidatorResult,
};
use crate::dom::webgl_validations::WebGLValidator;
use crate::dom::webglactiveinfo::WebGLActiveInfo;
use crate::dom::webglbuffer::WebGLBuffer;
use crate::dom::webglframebuffer::{WebGLFramebuffer, WebGLFramebufferAttachmentRoot};
use crate::dom::webglprogram::WebGLProgram;
use crate::dom::webglquery::WebGLQuery;
use crate::dom::webglrenderbuffer::WebGLRenderbuffer;
use crate::dom::webglrenderingcontext::{
    uniform_get, uniform_typed, Operation, TexPixels, TexSource, VertexAttrib,
    WebGLRenderingContext,
};
use crate::dom::webglsampler::{WebGLSampler, WebGLSamplerValue};
use crate::dom::webglshader::WebGLShader;
use crate::dom::webglshaderprecisionformat::WebGLShaderPrecisionFormat;
use crate::dom::webglsync::WebGLSync;
use crate::dom::webgltexture::WebGLTexture;
use crate::dom::webgltransformfeedback::WebGLTransformFeedback;
use crate::dom::webgluniformlocation::WebGLUniformLocation;
use crate::dom::webglvertexarrayobject::WebGLVertexArrayObject;
use crate::dom::window::Window;
use crate::js::conversions::ToJSValConvertible;
use crate::script_runtime::JSContext;

#[crown::unrooted_must_root_lint::must_root]
#[derive(JSTraceable, MallocSizeOf)]
struct IndexedBinding {
    buffer: MutNullableDom<WebGLBuffer>,
    start: Cell<i64>,
    size: Cell<i64>,
}

impl IndexedBinding {
    fn new() -> IndexedBinding {
        IndexedBinding {
            buffer: MutNullableDom::new(None),
            start: Cell::new(0),
            size: Cell::new(0),
        }
    }
}

#[dom_struct]
pub struct WebGL2RenderingContext {
    reflector_: Reflector,
    base: Dom<WebGLRenderingContext>,
    occlusion_query: MutNullableDom<WebGLQuery>,
    primitives_query: MutNullableDom<WebGLQuery>,
    samplers: Box<[MutNullableDom<WebGLSampler>]>,
    bound_copy_read_buffer: MutNullableDom<WebGLBuffer>,
    bound_copy_write_buffer: MutNullableDom<WebGLBuffer>,
    bound_pixel_pack_buffer: MutNullableDom<WebGLBuffer>,
    bound_pixel_unpack_buffer: MutNullableDom<WebGLBuffer>,
    bound_transform_feedback_buffer: MutNullableDom<WebGLBuffer>,
    bound_uniform_buffer: MutNullableDom<WebGLBuffer>,
    indexed_uniform_buffer_bindings: Box<[IndexedBinding]>,
    indexed_transform_feedback_buffer_bindings: Box<[IndexedBinding]>,
    current_transform_feedback: MutNullableDom<WebGLTransformFeedback>,
    texture_pack_row_length: Cell<usize>,
    texture_pack_skip_pixels: Cell<usize>,
    texture_pack_skip_rows: Cell<usize>,
    enable_rasterizer_discard: Cell<bool>,
    default_fb_readbuffer: Cell<u32>,
    default_fb_drawbuffer: Cell<u32>,
}

fn typedarray_elem_size(typeid: Type) -> usize {
    match typeid {
        Type::Int8 | Type::Uint8 | Type::Uint8Clamped => 1,
        Type::Int16 | Type::Uint16 => 2,
        Type::Int32 | Type::Uint32 | Type::Float32 => 4,
        Type::Int64 | Type::Float64 => 8,
        Type::BigInt64 | Type::BigUint64 => 8,
        Type::Simd128 | Type::MaxTypedArrayViewType => unreachable!(),
    }
}

struct ReadPixelsAllowedFormats<'a> {
    array_types: &'a [Type],
    channels: usize,
}

struct ReadPixelsSizes {
    row_stride: usize,
    skipped_bytes: usize,
    size: usize,
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
        let indexed_uniform_buffer_bindings = (0..base.limits().max_uniform_buffer_bindings)
            .map(|_| IndexedBinding::new())
            .collect::<Vec<_>>()
            .into();
        let indexed_transform_feedback_buffer_bindings =
            (0..base.limits().max_transform_feedback_separate_attribs)
                .map(|_| IndexedBinding::new())
                .collect::<Vec<_>>()
                .into();

        Some(WebGL2RenderingContext {
            reflector_: Reflector::new(),
            base: Dom::from_ref(&*base),
            occlusion_query: MutNullableDom::new(None),
            primitives_query: MutNullableDom::new(None),
            samplers: samplers,
            bound_copy_read_buffer: MutNullableDom::new(None),
            bound_copy_write_buffer: MutNullableDom::new(None),
            bound_pixel_pack_buffer: MutNullableDom::new(None),
            bound_pixel_unpack_buffer: MutNullableDom::new(None),
            bound_transform_feedback_buffer: MutNullableDom::new(None),
            bound_uniform_buffer: MutNullableDom::new(None),
            indexed_uniform_buffer_bindings,
            indexed_transform_feedback_buffer_bindings,
            current_transform_feedback: MutNullableDom::new(None),
            texture_pack_row_length: Cell::new(0),
            texture_pack_skip_pixels: Cell::new(0),
            texture_pack_skip_rows: Cell::new(0),
            enable_rasterizer_discard: Cell::new(false),
            default_fb_readbuffer: Cell::new(constants::BACK),
            default_fb_drawbuffer: Cell::new(constants::BACK),
        })
    }

    #[allow(crown::unrooted_must_root)]
    pub fn new(
        window: &Window,
        canvas: &HTMLCanvasElement,
        size: Size2D<u32>,
        attrs: GLContextAttributes,
    ) -> Option<DomRoot<WebGL2RenderingContext>> {
        WebGL2RenderingContext::new_inherited(window, canvas, size, attrs)
            .map(|ctx| reflect_dom_object(Box::new(ctx), window))
    }

    #[allow(unsafe_code)]
    pub fn is_webgl2_enabled(_cx: JSContext, global: HandleObject) -> bool {
        if pref!(dom.webgl2.enabled) {
            return true;
        }

        let global = unsafe { GlobalScope::from_object(global.get()) };
        let origin = global.origin();
        let host = origin.host();
        WEBGL2_ORIGINS
            .iter()
            .any(|origin| host == Host::parse(origin).ok().as_ref())
    }
}

/// List of domains for which WebGL 2 is enabled automatically, regardless
/// of the status of the dom.webgl2.enabled preference.
static WEBGL2_ORIGINS: &[&str] = &["www.servoexperiments.com"];

impl WebGL2RenderingContext {
    pub fn recreate(&self, size: Size2D<u32>) {
        self.base.recreate(size)
    }

    pub fn current_vao(&self) -> DomRoot<WebGLVertexArrayObject> {
        self.base.current_vao_webgl2()
    }

    pub fn validate_uniform_block_for_draw(&self) {
        let program = match self.base.current_program() {
            Some(program) => program,
            None => return,
        };
        for uniform_block in program.active_uniform_blocks().iter() {
            let data_size = uniform_block.size as usize;
            for block in program.active_uniforms().iter() {
                let index = match block.bind_index {
                    Some(index) => index,
                    None => continue,
                };
                let indexed = &self.indexed_uniform_buffer_bindings[index as usize];
                let buffer = match indexed.buffer.get() {
                    Some(buffer) => buffer,
                    None => {
                        self.base.webgl_error(InvalidOperation);
                        return;
                    },
                };
                if indexed.size.get() == 0 {
                    if data_size > buffer.capacity() {
                        self.base.webgl_error(InvalidOperation);
                        return;
                    }
                } else {
                    let start = indexed.start.get() as usize;
                    let mut size = indexed.size.get() as usize;
                    if start >= size {
                        self.base.webgl_error(InvalidOperation);
                        return;
                    }
                    size -= start;
                    if data_size > size {
                        self.base.webgl_error(InvalidOperation);
                        return;
                    }
                }
            }
        }
    }

    fn validate_vertex_attribs_for_draw(&self) {
        let program = match self.base.current_program() {
            Some(program) => program,
            None => return,
        };
        let groups = [
            [
                constants::INT,
                constants::INT_VEC2,
                constants::INT_VEC3,
                constants::INT_VEC4,
            ],
            [
                constants::UNSIGNED_INT,
                constants::UNSIGNED_INT_VEC2,
                constants::UNSIGNED_INT_VEC3,
                constants::UNSIGNED_INT_VEC4,
            ],
            [
                constants::FLOAT,
                constants::FLOAT_VEC2,
                constants::FLOAT_VEC3,
                constants::FLOAT_VEC4,
            ],
        ];
        let vao = self.current_vao();
        for prog_attrib in program.active_attribs().iter() {
            let attrib = handle_potential_webgl_error!(
                self.base,
                vao.get_vertex_attrib(prog_attrib.location as u32)
                    .ok_or(InvalidOperation),
                return
            );

            let current_vertex_attrib =
                self.base.current_vertex_attribs()[prog_attrib.location as usize];
            let attrib_data_base_type = if !attrib.enabled_as_array {
                match current_vertex_attrib {
                    VertexAttrib::Int(_, _, _, _) => constants::INT,
                    VertexAttrib::Uint(_, _, _, _) => constants::UNSIGNED_INT,
                    VertexAttrib::Float(_, _, _, _) => constants::FLOAT,
                }
            } else {
                attrib.type_
            };

            let contains = groups
                .iter()
                .find(|g| g.contains(&attrib_data_base_type) && g.contains(&prog_attrib.type_));
            if contains.is_none() {
                self.base.webgl_error(InvalidOperation);
                return;
            }
        }
    }

    pub fn base_context(&self) -> DomRoot<WebGLRenderingContext> {
        DomRoot::from_ref(&*self.base)
    }

    fn bound_buffer(&self, target: u32) -> WebGLResult<Option<DomRoot<WebGLBuffer>>> {
        match target {
            constants::COPY_READ_BUFFER => Ok(self.bound_copy_read_buffer.get()),
            constants::COPY_WRITE_BUFFER => Ok(self.bound_copy_write_buffer.get()),
            constants::PIXEL_PACK_BUFFER => Ok(self.bound_pixel_pack_buffer.get()),
            constants::PIXEL_UNPACK_BUFFER => Ok(self.bound_pixel_unpack_buffer.get()),
            constants::TRANSFORM_FEEDBACK_BUFFER => Ok(self.bound_transform_feedback_buffer.get()),
            constants::UNIFORM_BUFFER => Ok(self.bound_uniform_buffer.get()),
            constants::ELEMENT_ARRAY_BUFFER => Ok(self.current_vao().element_array_buffer().get()),
            _ => self.base.bound_buffer(target),
        }
    }

    pub fn buffer_usage(&self, usage: u32) -> WebGLResult<u32> {
        match usage {
            constants::STATIC_READ |
            constants::DYNAMIC_READ |
            constants::STREAM_READ |
            constants::STATIC_COPY |
            constants::DYNAMIC_COPY |
            constants::STREAM_COPY => Ok(usage),
            _ => self.base.buffer_usage(usage),
        }
    }

    fn unbind_from(&self, slot: &MutNullableDom<WebGLBuffer>, buffer: &WebGLBuffer) {
        if slot.get().map_or(false, |b| buffer == &*b) {
            buffer.decrement_attached_counter(Operation::Infallible);
            slot.set(None);
        }
    }

    fn calc_read_pixel_formats(
        &self,
        pixel_type: u32,
        format: u32,
    ) -> WebGLResult<ReadPixelsAllowedFormats> {
        let array_types = match pixel_type {
            constants::BYTE => &[Type::Int8][..],
            constants::SHORT => &[Type::Int16][..],
            constants::INT => &[Type::Int32][..],
            constants::UNSIGNED_BYTE => &[Type::Uint8, Type::Uint8Clamped][..],
            constants::UNSIGNED_SHORT |
            constants::UNSIGNED_SHORT_4_4_4_4 |
            constants::UNSIGNED_SHORT_5_5_5_1 |
            constants::UNSIGNED_SHORT_5_6_5 => &[Type::Uint16][..],
            constants::UNSIGNED_INT |
            constants::UNSIGNED_INT_2_10_10_10_REV |
            constants::UNSIGNED_INT_10F_11F_11F_REV |
            constants::UNSIGNED_INT_5_9_9_9_REV => &[Type::Uint32][..],
            constants::FLOAT => &[Type::Float32][..],
            constants::HALF_FLOAT => &[Type::Uint16][..],
            _ => return Err(InvalidEnum),
        };
        let channels = match format {
            constants::ALPHA | constants::RED | constants::RED_INTEGER => 1,
            constants::RG | constants::RG_INTEGER => 2,
            constants::RGB | constants::RGB_INTEGER => 3,
            constants::RGBA | constants::RGBA_INTEGER => 4,
            _ => return Err(InvalidEnum),
        };
        Ok(ReadPixelsAllowedFormats {
            array_types,
            channels,
        })
    }

    fn calc_read_pixel_sizes(
        &self,
        width: i32,
        height: i32,
        bytes_per_pixel: usize,
    ) -> WebGLResult<ReadPixelsSizes> {
        if width < 0 || height < 0 {
            return Err(InvalidValue);
        }

        // See also https://www.khronos.org/registry/webgl/specs/latest/2.0/#5.36
        let pixels_per_row = if self.texture_pack_row_length.get() > 0 {
            self.texture_pack_row_length.get()
        } else {
            width as usize
        };
        if self.texture_pack_skip_pixels.get() + width as usize > pixels_per_row {
            return Err(InvalidOperation);
        }

        let bytes_per_row = pixels_per_row
            .checked_mul(bytes_per_pixel)
            .ok_or(InvalidOperation)?;
        let row_padding_bytes = {
            let pack_alignment = self.base.get_texture_packing_alignment() as usize;
            match bytes_per_row % pack_alignment {
                0 => 0,
                remainder => pack_alignment - remainder,
            }
        };
        let row_stride = bytes_per_row + row_padding_bytes;
        let size = if width == 0 || height == 0 {
            0
        } else {
            let full_row_bytes = row_stride
                .checked_mul(height as usize - 1)
                .ok_or(InvalidOperation)?;
            let last_row_bytes = bytes_per_pixel
                .checked_mul(width as usize)
                .ok_or(InvalidOperation)?;
            let result = full_row_bytes
                .checked_add(last_row_bytes)
                .ok_or(InvalidOperation)?;
            result
        };
        let skipped_bytes = {
            let skipped_row_bytes = self
                .texture_pack_skip_rows
                .get()
                .checked_mul(row_stride)
                .ok_or(InvalidOperation)?;
            let skipped_pixel_bytes = self
                .texture_pack_skip_pixels
                .get()
                .checked_mul(bytes_per_pixel)
                .ok_or(InvalidOperation)?;
            let result = skipped_row_bytes
                .checked_add(skipped_pixel_bytes)
                .ok_or(InvalidOperation)?;
            result
        };
        Ok(ReadPixelsSizes {
            row_stride,
            skipped_bytes,
            size,
        })
    }

    #[allow(unsafe_code)]
    fn read_pixels_into(
        &self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        format: u32,
        pixel_type: u32,
        dst: &mut ArrayBufferView,
        dst_elem_offset: u32,
    ) {
        handle_potential_webgl_error!(self.base, self.base.validate_framebuffer(), return);

        if self.bound_pixel_pack_buffer.get().is_some() {
            return self.base.webgl_error(InvalidOperation);
        }

        let fb_slot = self.base.get_draw_framebuffer_slot();
        let fb_readbuffer_valid = match fb_slot.get() {
            Some(fb) => fb.attachment(fb.read_buffer()).is_some(),
            None => self.default_fb_readbuffer.get() != constants::NONE,
        };
        if !fb_readbuffer_valid {
            return self.base.webgl_error(InvalidOperation);
        }

        let dst_byte_offset = {
            let dst_elem_size = typedarray_elem_size(dst.get_array_type());
            dst_elem_offset as usize * dst_elem_size
        };
        if dst_byte_offset > dst.len() {
            return self.base.webgl_error(InvalidValue);
        }

        let dst_array_type = dst.get_array_type();
        let ReadPixelsAllowedFormats {
            array_types: allowed_array_types,
            channels,
        } = match self.calc_read_pixel_formats(pixel_type, format) {
            Ok(result) => result,
            Err(error) => return self.base.webgl_error(error),
        };
        if !allowed_array_types.contains(&dst_array_type) {
            return self.base.webgl_error(InvalidOperation);
        }
        if format != constants::RGBA || pixel_type != constants::UNSIGNED_BYTE {
            return self.base.webgl_error(InvalidOperation);
        }

        let bytes_per_pixel = typedarray_elem_size(dst_array_type) * channels;
        let ReadPixelsSizes {
            row_stride,
            skipped_bytes,
            size,
        } = match self.calc_read_pixel_sizes(width, height, bytes_per_pixel) {
            Ok(result) => result,
            Err(error) => return self.base.webgl_error(error),
        };
        let dst_end = dst_byte_offset + skipped_bytes + size;
        let dst_pixels = unsafe { dst.as_mut_slice() };
        if dst_pixels.len() < dst_end {
            return self.base.webgl_error(InvalidOperation);
        }

        let dst_byte_offset = {
            let margin_left = cmp::max(0, -x) as usize;
            let margin_top = cmp::max(0, -y) as usize;
            dst_byte_offset +
                skipped_bytes +
                margin_left * bytes_per_pixel +
                margin_top * row_stride
        };
        let src_rect = {
            let (fb_width, fb_height) = handle_potential_webgl_error!(
                self.base,
                self.base
                    .get_current_framebuffer_size()
                    .ok_or(InvalidOperation),
                return
            );
            let src_origin = Point2D::new(x, y);
            let src_size = Size2D::new(width as u32, height as u32);
            let fb_size = Size2D::new(fb_width as u32, fb_height as u32);
            match pixels::clip(src_origin, src_size.to_u64(), fb_size.to_u64()) {
                Some(rect) => rect.to_u32(),
                None => return,
            }
        };
        let src_row_bytes = handle_potential_webgl_error!(
            self.base,
            src_rect
                .size
                .width
                .checked_mul(bytes_per_pixel as u32)
                .ok_or(InvalidOperation),
            return
        );

        let (sender, receiver) = ipc::bytes_channel().unwrap();
        self.base.send_command(WebGLCommand::ReadPixels(
            src_rect, format, pixel_type, sender,
        ));
        let src = receiver.recv().unwrap();

        for i in 0..src_rect.size.height as usize {
            let src_start = i * src_row_bytes as usize;
            let dst_start = dst_byte_offset + i * row_stride;
            dst_pixels[dst_start..dst_start + src_row_bytes as usize]
                .copy_from_slice(&src[src_start..src_start + src_row_bytes as usize]);
        }
    }

    fn uniform_vec_section_uint(
        &self,
        vec: Uint32ArrayOrUnsignedLongSequence,
        offset: u32,
        length: u32,
        uniform_size: usize,
        uniform_location: &WebGLUniformLocation,
    ) -> WebGLResult<Vec<u32>> {
        let vec = match vec {
            Uint32ArrayOrUnsignedLongSequence::Uint32Array(v) => v.to_vec(),
            Uint32ArrayOrUnsignedLongSequence::UnsignedLongSequence(v) => v,
        };
        self.base
            .uniform_vec_section::<u32>(vec, offset, length, uniform_size, uniform_location)
    }

    #[allow(unsafe_code)]
    fn get_default_fb_attachment_param(&self, attachment: u32, pname: u32) -> WebGLResult<JSVal> {
        match attachment {
            constants::BACK | constants::DEPTH | constants::STENCIL => {},
            _ => return Err(InvalidEnum),
        }

        if pname == constants::FRAMEBUFFER_ATTACHMENT_OBJECT_NAME {
            return Ok(NullValue());
        }

        let attrs = self
            .GetContextAttributes()
            .unwrap_or_else(WebGLContextAttributes::empty);

        let intval = match pname {
            constants::FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE => match attachment {
                constants::DEPTH if !attrs.depth => constants::NONE as _,
                constants::STENCIL if !attrs.stencil => constants::NONE as _,
                _ => constants::FRAMEBUFFER_DEFAULT as _,
            },
            constants::FRAMEBUFFER_ATTACHMENT_RED_SIZE |
            constants::FRAMEBUFFER_ATTACHMENT_GREEN_SIZE |
            constants::FRAMEBUFFER_ATTACHMENT_BLUE_SIZE => match attachment {
                constants::BACK => 8,
                _ => 0,
            },
            constants::FRAMEBUFFER_ATTACHMENT_ALPHA_SIZE => match attachment {
                constants::BACK if attrs.alpha => 8,
                constants::BACK => return Err(InvalidOperation),
                _ => 0,
            },
            constants::FRAMEBUFFER_ATTACHMENT_DEPTH_SIZE => match attachment {
                constants::DEPTH if attrs.depth => 24,
                constants::DEPTH => return Err(InvalidOperation),
                _ => 0,
            },
            constants::FRAMEBUFFER_ATTACHMENT_STENCIL_SIZE => match attachment {
                constants::STENCIL if attrs.stencil => 8,
                constants::STENCIL => return Err(InvalidOperation),
                _ => 0,
            },
            constants::FRAMEBUFFER_ATTACHMENT_COMPONENT_TYPE => match attachment {
                constants::DEPTH if attrs.depth => constants::UNSIGNED_NORMALIZED as _,
                constants::STENCIL if attrs.stencil => constants::UNSIGNED_INT as _,
                constants::DEPTH => return Err(InvalidOperation),
                constants::STENCIL => return Err(InvalidOperation),
                _ => constants::UNSIGNED_NORMALIZED as _,
            },
            constants::FRAMEBUFFER_ATTACHMENT_COLOR_ENCODING => match attachment {
                constants::DEPTH if !attrs.depth => return Err(InvalidOperation),
                constants::STENCIL if !attrs.stencil => return Err(InvalidOperation),
                _ => constants::LINEAR as _,
            },
            _ => return Err(InvalidEnum),
        };
        Ok(Int32Value(intval))
    }

    #[allow(unsafe_code)]
    fn get_specific_fb_attachment_param(
        &self,
        cx: JSContext,
        fb: &WebGLFramebuffer,
        target: u32,
        attachment: u32,
        pname: u32,
    ) -> WebGLResult<JSVal> {
        use crate::dom::webglframebuffer::WebGLFramebufferAttachmentRoot::{Renderbuffer, Texture};

        match attachment {
            constants::DEPTH_ATTACHMENT | constants::STENCIL_ATTACHMENT => {},
            constants::DEPTH_STENCIL_ATTACHMENT => {
                if pname == constants::FRAMEBUFFER_ATTACHMENT_COMPONENT_TYPE {
                    return Err(InvalidOperation);
                }

                let a = fb.attachment(constants::DEPTH_ATTACHMENT);
                let b = fb.attachment(constants::STENCIL_ATTACHMENT);
                match (a, b) {
                    (Some(Renderbuffer(ref a)), Some(Renderbuffer(ref b))) if a.id() == b.id() => {
                    },
                    (Some(Texture(ref a)), Some(Texture(ref b))) if a.id() == b.id() => {},
                    _ => return Err(InvalidOperation),
                }
            },
            constants::COLOR_ATTACHMENT0..=constants::COLOR_ATTACHMENT15 => {
                let last_slot =
                    constants::COLOR_ATTACHMENT0 + self.base.limits().max_color_attachments - 1;
                if last_slot < attachment {
                    return Err(InvalidEnum);
                }
            },
            _ => return Err(InvalidEnum),
        }

        let attachment = match attachment {
            constants::DEPTH_STENCIL_ATTACHMENT => constants::DEPTH_ATTACHMENT,
            _ => attachment,
        };

        if pname == constants::FRAMEBUFFER_ATTACHMENT_OBJECT_NAME {
            rooted!(in(*cx) let mut rval = NullValue());
            match fb.attachment(attachment) {
                Some(Renderbuffer(rb)) => unsafe {
                    rb.to_jsval(*cx, rval.handle_mut());
                },
                Some(Texture(texture)) => unsafe {
                    texture.to_jsval(*cx, rval.handle_mut());
                },
                _ => {},
            }
            return Ok(rval.get());
        }

        match pname {
            constants::FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE => {},
            _ => match fb.attachment(attachment) {
                Some(webgl_attachment) => match pname {
                    constants::FRAMEBUFFER_ATTACHMENT_RED_SIZE |
                    constants::FRAMEBUFFER_ATTACHMENT_GREEN_SIZE |
                    constants::FRAMEBUFFER_ATTACHMENT_BLUE_SIZE |
                    constants::FRAMEBUFFER_ATTACHMENT_ALPHA_SIZE |
                    constants::FRAMEBUFFER_ATTACHMENT_DEPTH_SIZE |
                    constants::FRAMEBUFFER_ATTACHMENT_STENCIL_SIZE |
                    constants::FRAMEBUFFER_ATTACHMENT_COMPONENT_TYPE |
                    constants::FRAMEBUFFER_ATTACHMENT_COLOR_ENCODING => {},
                    _ => match webgl_attachment {
                        WebGLFramebufferAttachmentRoot::Renderbuffer(_) => return Err(InvalidEnum),
                        WebGLFramebufferAttachmentRoot::Texture(_) => match pname {
                            constants::FRAMEBUFFER_ATTACHMENT_TEXTURE_LEVEL |
                            constants::FRAMEBUFFER_ATTACHMENT_TEXTURE_CUBE_MAP_FACE |
                            constants::FRAMEBUFFER_ATTACHMENT_TEXTURE_LAYER => {},
                            _ => return Err(InvalidEnum),
                        },
                    },
                },
                None => return Err(InvalidOperation),
            },
        }

        let (sender, receiver) = webgl_channel().unwrap();
        self.base
            .send_command(WebGLCommand::GetFramebufferAttachmentParameter(
                target, attachment, pname, sender,
            ));

        let retval = receiver.recv().unwrap();
        Ok(Int32Value(retval))
    }

    fn clearbuffer_array_size(&self, buffer: u32, draw_buffer: i32) -> WebGLResult<usize> {
        match buffer {
            constants::COLOR => {
                if draw_buffer < 0 || draw_buffer as u32 >= self.base.limits().max_draw_buffers {
                    return Err(InvalidValue);
                }
                Ok(4)
            },
            constants::DEPTH | constants::STENCIL | constants::DEPTH_STENCIL => {
                if draw_buffer != 0 {
                    return Err(InvalidValue);
                }
                Ok(1)
            },
            _ => unreachable!(),
        }
    }

    fn clear_buffer<T: Clone>(
        &self,
        buffer: u32,
        draw_buffer: i32,
        valid_buffers: &[u32],
        src_offset: u32,
        array: Vec<T>,
        msg: fn(u32, i32, Vec<T>) -> WebGLCommand,
    ) {
        if !valid_buffers.contains(&buffer) {
            return self.base.webgl_error(InvalidEnum);
        }

        let array_size = handle_potential_webgl_error!(
            self.base,
            self.clearbuffer_array_size(buffer, draw_buffer),
            return
        );
        let src_offset = src_offset as usize;

        if array.len() < src_offset + array_size {
            return self.base.webgl_error(InvalidValue);
        }
        let array = array[src_offset..src_offset + array_size].to_vec();

        self.base.send_command(msg(buffer, draw_buffer, array));
    }

    fn valid_fb_attachment_values(&self, target: u32, attachments: &[u32]) -> bool {
        let fb_slot = match target {
            constants::FRAMEBUFFER | constants::DRAW_FRAMEBUFFER => {
                self.base.get_draw_framebuffer_slot()
            },
            constants::READ_FRAMEBUFFER => self.base.get_read_framebuffer_slot(),
            _ => {
                self.base.webgl_error(InvalidEnum);
                return false;
            },
        };

        if let Some(fb) = fb_slot.get() {
            if fb.check_status() != constants::FRAMEBUFFER_COMPLETE {
                return false;
            }

            for &attachment in attachments {
                match attachment {
                    constants::DEPTH_ATTACHMENT |
                    constants::STENCIL_ATTACHMENT |
                    constants::DEPTH_STENCIL_ATTACHMENT => {},
                    constants::COLOR_ATTACHMENT0..=constants::COLOR_ATTACHMENT15 => {
                        let last_slot = constants::COLOR_ATTACHMENT0 +
                            self.base.limits().max_color_attachments -
                            1;
                        if last_slot < attachment {
                            return false;
                        }
                    },
                    _ => return false,
                }
            }
        } else {
            for &attachment in attachments {
                match attachment {
                    constants::COLOR | constants::DEPTH | constants::STENCIL => {},
                    _ => return false,
                }
            }
        }

        true
    }

    fn vertex_attrib_i(&self, index: u32, x: i32, y: i32, z: i32, w: i32) {
        if index >= self.base.limits().max_vertex_attribs {
            return self.base.webgl_error(InvalidValue);
        }
        self.base.current_vertex_attribs()[index as usize] = VertexAttrib::Int(x, y, z, w);
        self.current_vao()
            .set_vertex_attrib_type(index, constants::INT);
        self.base
            .send_command(WebGLCommand::VertexAttribI(index, x, y, z, w));
    }

    fn vertex_attrib_u(&self, index: u32, x: u32, y: u32, z: u32, w: u32) {
        if index >= self.base.limits().max_vertex_attribs {
            return self.base.webgl_error(InvalidValue);
        }
        self.base.current_vertex_attribs()[index as usize] = VertexAttrib::Uint(x, y, z, w);
        self.current_vao()
            .set_vertex_attrib_type(index, constants::UNSIGNED_INT);
        self.base
            .send_command(WebGLCommand::VertexAttribU(index, x, y, z, w));
    }

    fn tex_storage(
        &self,
        dimensions: u8,
        target: u32,
        levels: i32,
        internal_format: u32,
        width: i32,
        height: i32,
        depth: i32,
    ) {
        let expected_dimensions = match target {
            constants::TEXTURE_2D | constants::TEXTURE_CUBE_MAP => 2,
            constants::TEXTURE_3D | constants::TEXTURE_2D_ARRAY => 3,
            _ => return self.base.webgl_error(InvalidEnum),
        };
        if dimensions != expected_dimensions {
            return self.base.webgl_error(InvalidEnum);
        }

        let validator = TexStorageValidator::new(
            &self.base,
            dimensions,
            target,
            levels,
            internal_format,
            width,
            height,
            depth,
        );
        let TexStorageValidatorResult {
            texture,
            target,
            levels,
            internal_format,
            width,
            height,
            depth,
        } = match validator.validate() {
            Ok(result) => result,
            Err(_) => return, // NB: The validator sets the correct error for us.
        };

        handle_potential_webgl_error!(
            self.base,
            texture.storage(target, levels, internal_format, width, height, depth),
            return
        );
    }
}

impl WebGL2RenderingContextMethods for WebGL2RenderingContext {
    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.1>
    fn Canvas(&self) -> DomRoot<HTMLCanvasElement> {
        self.base.Canvas()
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.11>
    fn Flush(&self) {
        self.base.Flush()
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.11>
    fn Finish(&self) {
        self.base.Finish()
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.1>
    fn DrawingBufferWidth(&self) -> i32 {
        self.base.DrawingBufferWidth()
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.1>
    fn DrawingBufferHeight(&self) -> i32 {
        self.base.DrawingBufferHeight()
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.5>
    fn GetBufferParameter(&self, _cx: JSContext, target: u32, parameter: u32) -> JSVal {
        let buffer =
            handle_potential_webgl_error!(self.base, self.bound_buffer(target), return NullValue());
        self.base.get_buffer_param(buffer, parameter)
    }

    #[allow(unsafe_code)]
    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3>
    fn GetParameter(&self, cx: JSContext, parameter: u32) -> JSVal {
        match parameter {
            constants::VERSION => unsafe {
                rooted!(in(*cx) let mut rval = UndefinedValue());
                "WebGL 2.0".to_jsval(*cx, rval.handle_mut());
                return rval.get();
            },
            constants::SHADING_LANGUAGE_VERSION => unsafe {
                rooted!(in(*cx) let mut rval = UndefinedValue());
                "WebGL GLSL ES 3.00".to_jsval(*cx, rval.handle_mut());
                return rval.get();
            },
            constants::MAX_CLIENT_WAIT_TIMEOUT_WEBGL => {
                return DoubleValue(
                    self.base.limits().max_client_wait_timeout_webgl.as_nanos() as f64
                );
            },
            constants::MAX_SERVER_WAIT_TIMEOUT => {
                return DoubleValue(self.base.limits().max_server_wait_timeout.as_nanos() as f64);
            },
            constants::SAMPLER_BINDING => unsafe {
                let idx = (self.base.textures().active_unit_enum() - constants::TEXTURE0) as usize;
                assert!(idx < self.samplers.len());
                let sampler = self.samplers[idx].get();
                return optional_root_object_to_js_or_null!(*cx, sampler);
            },
            constants::COPY_READ_BUFFER_BINDING => unsafe {
                return optional_root_object_to_js_or_null!(
                    *cx,
                    &self.bound_copy_read_buffer.get()
                );
            },
            constants::COPY_WRITE_BUFFER_BINDING => unsafe {
                return optional_root_object_to_js_or_null!(
                    *cx,
                    &self.bound_copy_write_buffer.get()
                );
            },
            constants::PIXEL_PACK_BUFFER_BINDING => unsafe {
                return optional_root_object_to_js_or_null!(
                    *cx,
                    &self.bound_pixel_pack_buffer.get()
                );
            },
            constants::PIXEL_UNPACK_BUFFER_BINDING => unsafe {
                return optional_root_object_to_js_or_null!(
                    *cx,
                    &self.bound_pixel_unpack_buffer.get()
                );
            },
            constants::TRANSFORM_FEEDBACK_BUFFER_BINDING => unsafe {
                return optional_root_object_to_js_or_null!(
                    *cx,
                    &self.bound_transform_feedback_buffer.get()
                );
            },
            constants::UNIFORM_BUFFER_BINDING => unsafe {
                return optional_root_object_to_js_or_null!(*cx, &self.bound_uniform_buffer.get());
            },
            constants::TRANSFORM_FEEDBACK_BINDING => unsafe {
                return optional_root_object_to_js_or_null!(
                    *cx,
                    self.current_transform_feedback.get()
                );
            },
            constants::ELEMENT_ARRAY_BUFFER_BINDING => unsafe {
                let buffer = self.current_vao().element_array_buffer().get();
                return optional_root_object_to_js_or_null!(*cx, buffer);
            },
            constants::VERTEX_ARRAY_BINDING => unsafe {
                let vao = self.current_vao();
                let vao = vao.id().map(|_| &*vao);
                return optional_root_object_to_js_or_null!(*cx, vao);
            },
            // NOTE: DRAW_FRAMEBUFFER_BINDING is the same as FRAMEBUFFER_BINDING, handled on the WebGL1 side
            constants::READ_FRAMEBUFFER_BINDING => unsafe {
                return optional_root_object_to_js_or_null!(
                    *cx,
                    &self.base.get_read_framebuffer_slot().get()
                );
            },
            constants::READ_BUFFER => {
                let buffer = match self.base.get_read_framebuffer_slot().get() {
                    Some(fb) => fb.read_buffer(),
                    None => self.default_fb_readbuffer.get(),
                };
                return UInt32Value(buffer);
            },
            constants::DRAW_BUFFER0..=constants::DRAW_BUFFER15 => {
                let buffer = match self.base.get_read_framebuffer_slot().get() {
                    Some(fb) => {
                        let idx = parameter - constants::DRAW_BUFFER0;
                        fb.draw_buffer_i(idx as usize)
                    },
                    None if parameter == constants::DRAW_BUFFER0 => {
                        self.default_fb_readbuffer.get()
                    },
                    None => constants::NONE,
                };
                return UInt32Value(buffer);
            },
            constants::MAX_TEXTURE_LOD_BIAS => {
                return DoubleValue(self.base.limits().max_texture_lod_bias as f64)
            },
            constants::MAX_COMBINED_FRAGMENT_UNIFORM_COMPONENTS => {
                return DoubleValue(
                    self.base.limits().max_combined_fragment_uniform_components as f64,
                )
            },
            constants::MAX_COMBINED_VERTEX_UNIFORM_COMPONENTS => {
                return DoubleValue(
                    self.base.limits().max_combined_vertex_uniform_components as f64,
                )
            },
            constants::MAX_ELEMENT_INDEX => {
                return DoubleValue(self.base.limits().max_element_index as f64)
            },
            constants::MAX_UNIFORM_BLOCK_SIZE => {
                return DoubleValue(self.base.limits().max_uniform_block_size as f64)
            },
            constants::MIN_PROGRAM_TEXEL_OFFSET => {
                return Int32Value(self.base.limits().min_program_texel_offset)
            },
            _ => {},
        }

        let limit = match parameter {
            constants::MAX_3D_TEXTURE_SIZE => Some(self.base.limits().max_3d_texture_size),
            constants::MAX_ARRAY_TEXTURE_LAYERS => {
                Some(self.base.limits().max_array_texture_layers)
            },
            constants::MAX_COLOR_ATTACHMENTS => Some(self.base.limits().max_color_attachments),
            constants::MAX_COMBINED_UNIFORM_BLOCKS => {
                Some(self.base.limits().max_combined_uniform_blocks)
            },
            constants::MAX_DRAW_BUFFERS => Some(self.base.limits().max_draw_buffers),
            constants::MAX_ELEMENTS_INDICES => Some(self.base.limits().max_elements_indices),
            constants::MAX_ELEMENTS_VERTICES => Some(self.base.limits().max_elements_vertices),
            constants::MAX_FRAGMENT_INPUT_COMPONENTS => {
                Some(self.base.limits().max_fragment_input_components)
            },
            constants::MAX_FRAGMENT_UNIFORM_BLOCKS => {
                Some(self.base.limits().max_fragment_uniform_blocks)
            },
            constants::MAX_FRAGMENT_UNIFORM_COMPONENTS => {
                Some(self.base.limits().max_fragment_uniform_components)
            },
            constants::MAX_PROGRAM_TEXEL_OFFSET => {
                Some(self.base.limits().max_program_texel_offset)
            },
            constants::MAX_SAMPLES => Some(self.base.limits().max_samples),
            constants::MAX_UNIFORM_BUFFER_BINDINGS => {
                Some(self.base.limits().max_uniform_buffer_bindings)
            },
            constants::MAX_VARYING_COMPONENTS => Some(self.base.limits().max_varying_components),
            constants::MAX_VERTEX_OUTPUT_COMPONENTS => {
                Some(self.base.limits().max_vertex_output_components)
            },
            constants::MAX_VERTEX_UNIFORM_BLOCKS => {
                Some(self.base.limits().max_vertex_uniform_blocks)
            },
            constants::MAX_VERTEX_UNIFORM_COMPONENTS => {
                Some(self.base.limits().max_vertex_uniform_components)
            },
            constants::UNIFORM_BUFFER_OFFSET_ALIGNMENT => {
                Some(self.base.limits().uniform_buffer_offset_alignment)
            },
            _ => None,
        };
        if let Some(limit) = limit {
            return UInt32Value(limit);
        }

        self.base.GetParameter(cx, parameter)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8>
    fn GetTexParameter(&self, cx: JSContext, target: u32, pname: u32) -> JSVal {
        self.base.GetTexParameter(cx, target, pname)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3>
    fn GetError(&self) -> u32 {
        self.base.GetError()
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.2>
    fn GetContextAttributes(&self) -> Option<WebGLContextAttributes> {
        self.base.GetContextAttributes()
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.13
    fn IsContextLost(&self) -> bool {
        self.base.IsContextLost()
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.14>
    fn GetSupportedExtensions(&self) -> Option<Vec<DOMString>> {
        self.base.GetSupportedExtensions()
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.14>
    fn GetExtension(&self, cx: JSContext, name: DOMString) -> Option<NonNull<JSObject>> {
        self.base.GetExtension(cx, name)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.4>
    fn GetFramebufferAttachmentParameter(
        &self,
        cx: JSContext,
        target: u32,
        attachment: u32,
        pname: u32,
    ) -> JSVal {
        let fb_slot = match target {
            constants::FRAMEBUFFER | constants::DRAW_FRAMEBUFFER => {
                self.base.get_draw_framebuffer_slot()
            },
            constants::READ_FRAMEBUFFER => &self.base.get_read_framebuffer_slot(),
            _ => {
                self.base.webgl_error(InvalidEnum);
                return NullValue();
            },
        };

        if let Some(fb) = fb_slot.get() {
            // A selected framebuffer is bound to the target
            handle_potential_webgl_error!(self.base, fb.validate_transparent(), return NullValue());
            handle_potential_webgl_error!(
                self.base,
                self.get_specific_fb_attachment_param(cx, &fb, target, attachment, pname),
                return NullValue()
            )
        } else {
            // The default framebuffer is bound to the target
            handle_potential_webgl_error!(
                self.base,
                self.get_default_fb_attachment_param(attachment, pname),
                return NullValue()
            )
        }
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.7>
    fn GetRenderbufferParameter(&self, cx: JSContext, target: u32, pname: u32) -> JSVal {
        self.base.GetRenderbufferParameter(cx, target, pname)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3>
    fn ActiveTexture(&self, texture: u32) {
        self.base.ActiveTexture(texture)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3>
    fn BlendColor(&self, r: f32, g: f32, b: f32, a: f32) {
        self.base.BlendColor(r, g, b, a)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3>
    fn BlendEquation(&self, mode: u32) {
        self.base.BlendEquation(mode)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3>
    fn BlendEquationSeparate(&self, mode_rgb: u32, mode_alpha: u32) {
        self.base.BlendEquationSeparate(mode_rgb, mode_alpha)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3>
    fn BlendFunc(&self, src_factor: u32, dest_factor: u32) {
        self.base.BlendFunc(src_factor, dest_factor)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3>
    fn BlendFuncSeparate(&self, src_rgb: u32, dest_rgb: u32, src_alpha: u32, dest_alpha: u32) {
        self.base
            .BlendFuncSeparate(src_rgb, dest_rgb, src_alpha, dest_alpha)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9>
    fn AttachShader(&self, program: &WebGLProgram, shader: &WebGLShader) {
        self.base.AttachShader(program, shader)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9>
    fn DetachShader(&self, program: &WebGLProgram, shader: &WebGLShader) {
        self.base.DetachShader(program, shader)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9>
    fn BindAttribLocation(&self, program: &WebGLProgram, index: u32, name: DOMString) {
        self.base.BindAttribLocation(program, index, name)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.2>
    fn BindBuffer(&self, target: u32, buffer: Option<&WebGLBuffer>) {
        let current_vao;
        let slot = match target {
            constants::COPY_READ_BUFFER => &self.bound_copy_read_buffer,
            constants::COPY_WRITE_BUFFER => &self.bound_copy_write_buffer,
            constants::PIXEL_PACK_BUFFER => &self.bound_pixel_pack_buffer,
            constants::PIXEL_UNPACK_BUFFER => &self.bound_pixel_unpack_buffer,
            constants::TRANSFORM_FEEDBACK_BUFFER => &self.bound_transform_feedback_buffer,
            constants::UNIFORM_BUFFER => &self.bound_uniform_buffer,
            constants::ELEMENT_ARRAY_BUFFER => {
                current_vao = self.current_vao();
                current_vao.element_array_buffer()
            },
            _ => return self.base.BindBuffer(target, buffer),
        };
        self.base.bind_buffer_maybe(&slot, target, buffer);
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.6>
    fn BindFramebuffer(&self, target: u32, framebuffer: Option<&WebGLFramebuffer>) {
        handle_potential_webgl_error!(
            self.base,
            self.base.validate_new_framebuffer_binding(framebuffer),
            return
        );

        let (bind_read, bind_draw) = match target {
            constants::FRAMEBUFFER => (true, true),
            constants::READ_FRAMEBUFFER => (true, false),
            constants::DRAW_FRAMEBUFFER => (false, true),
            _ => return self.base.webgl_error(InvalidEnum),
        };
        if bind_read {
            self.base.bind_framebuffer_to(
                target,
                framebuffer,
                &self.base.get_read_framebuffer_slot(),
            );
        }
        if bind_draw {
            self.base.bind_framebuffer_to(
                target,
                framebuffer,
                &self.base.get_draw_framebuffer_slot(),
            );
        }
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.7>
    fn BindRenderbuffer(&self, target: u32, renderbuffer: Option<&WebGLRenderbuffer>) {
        self.base.BindRenderbuffer(target, renderbuffer)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8>
    fn BindTexture(&self, target: u32, texture: Option<&WebGLTexture>) {
        self.base.BindTexture(target, texture)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8>
    fn GenerateMipmap(&self, target: u32) {
        self.base.GenerateMipmap(target)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.5>
    fn BufferData_(&self, target: u32, data: Option<ArrayBufferViewOrArrayBuffer>, usage: u32) {
        let usage = handle_potential_webgl_error!(self.base, self.buffer_usage(usage), return);
        let bound_buffer =
            handle_potential_webgl_error!(self.base, self.bound_buffer(target), return);
        self.base.buffer_data(target, data, usage, bound_buffer)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.5>
    fn BufferData(&self, target: u32, size: i64, usage: u32) {
        let usage = handle_potential_webgl_error!(self.base, self.buffer_usage(usage), return);
        let bound_buffer =
            handle_potential_webgl_error!(self.base, self.bound_buffer(target), return);
        self.base.buffer_data_(target, size, usage, bound_buffer)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.3>
    #[allow(unsafe_code)]
    fn BufferData__(
        &self,
        target: u32,
        data: CustomAutoRooterGuard<ArrayBufferView>,
        usage: u32,
        elem_offset: u32,
        length: u32,
    ) {
        let usage = handle_potential_webgl_error!(self.base, self.buffer_usage(usage), return);
        let bound_buffer =
            handle_potential_webgl_error!(self.base, self.bound_buffer(target), return);
        let bound_buffer =
            handle_potential_webgl_error!(self.base, bound_buffer.ok_or(InvalidOperation), return);

        let elem_size = typedarray_elem_size(data.get_array_type());
        let elem_count = data.len() / elem_size;
        let elem_offset = elem_offset as usize;
        let byte_offset = elem_offset * elem_size;

        if byte_offset > data.len() {
            return self.base.webgl_error(InvalidValue);
        }

        let copy_count = if length == 0 {
            elem_count - elem_offset
        } else {
            length as usize
        };
        if copy_count == 0 {
            return;
        }
        let copy_bytes = copy_count * elem_size;

        if byte_offset + copy_bytes > data.len() {
            return self.base.webgl_error(InvalidValue);
        }

        let data_end = byte_offset + copy_bytes;
        let data: &[u8] = unsafe { &data.as_slice()[byte_offset..data_end] };
        handle_potential_webgl_error!(self.base, bound_buffer.buffer_data(target, &data, usage));
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.5>
    fn BufferSubData(&self, target: u32, offset: i64, data: ArrayBufferViewOrArrayBuffer) {
        let bound_buffer =
            handle_potential_webgl_error!(self.base, self.bound_buffer(target), return);
        self.base
            .buffer_sub_data(target, offset, data, bound_buffer)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.3>
    #[allow(unsafe_code)]
    fn BufferSubData_(
        &self,
        target: u32,
        dst_byte_offset: i64,
        src_data: CustomAutoRooterGuard<ArrayBufferView>,
        src_elem_offset: u32,
        length: u32,
    ) {
        let bound_buffer =
            handle_potential_webgl_error!(self.base, self.bound_buffer(target), return);
        let bound_buffer =
            handle_potential_webgl_error!(self.base, bound_buffer.ok_or(InvalidOperation), return);

        let src_elem_size = typedarray_elem_size(src_data.get_array_type());
        let src_elem_count = src_data.len() / src_elem_size;
        let src_elem_offset = src_elem_offset as usize;
        let src_byte_offset = src_elem_offset * src_elem_size;

        if dst_byte_offset < 0 || src_byte_offset > src_data.len() {
            return self.base.webgl_error(InvalidValue);
        }

        let copy_count = if length == 0 {
            src_elem_count - src_elem_offset
        } else {
            length as usize
        };
        if copy_count == 0 {
            return;
        }
        let copy_bytes = copy_count * src_elem_size;

        let dst_byte_offset = dst_byte_offset as usize;
        if dst_byte_offset + copy_bytes > bound_buffer.capacity() ||
            src_byte_offset + copy_bytes > src_data.len()
        {
            return self.base.webgl_error(InvalidValue);
        }

        let (sender, receiver) = ipc::bytes_channel().unwrap();
        self.base.send_command(WebGLCommand::BufferSubData(
            target,
            dst_byte_offset as isize,
            receiver,
        ));
        let src_end = src_byte_offset + copy_bytes;
        let data: &[u8] = unsafe { &src_data.as_slice()[src_byte_offset..src_end] };
        sender.send(data).unwrap();
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.3>
    fn CopyBufferSubData(
        &self,
        read_target: u32,
        write_target: u32,
        read_offset: i64,
        write_offset: i64,
        size: i64,
    ) {
        if read_offset < 0 || write_offset < 0 || size < 0 {
            return self.base.webgl_error(InvalidValue);
        }

        let read_buffer =
            handle_potential_webgl_error!(self.base, self.bound_buffer(read_target), return);
        let read_buffer =
            handle_potential_webgl_error!(self.base, read_buffer.ok_or(InvalidOperation), return);

        let write_buffer =
            handle_potential_webgl_error!(self.base, self.bound_buffer(write_target), return);
        let write_buffer =
            handle_potential_webgl_error!(self.base, write_buffer.ok_or(InvalidOperation), return);

        let read_until = read_offset + size;
        let write_until = write_offset + size;
        if read_until as usize > read_buffer.capacity() ||
            write_until as usize > write_buffer.capacity()
        {
            return self.base.webgl_error(InvalidValue);
        }

        if read_target == write_target {
            let is_separate = read_until <= write_offset || write_until <= read_offset;
            if !is_separate {
                return self.base.webgl_error(InvalidValue);
            }
        }
        let src_is_elemarray = read_buffer
            .target()
            .map_or(false, |t| t == constants::ELEMENT_ARRAY_BUFFER);
        let dst_is_elemarray = write_buffer
            .target()
            .map_or(false, |t| t == constants::ELEMENT_ARRAY_BUFFER);
        if src_is_elemarray != dst_is_elemarray {
            return self.base.webgl_error(InvalidOperation);
        }

        self.base.send_command(WebGLCommand::CopyBufferSubData(
            read_target,
            write_target,
            read_offset,
            write_offset,
            size,
        ));
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.3>
    #[allow(unsafe_code)]
    fn GetBufferSubData(
        &self,
        target: u32,
        src_byte_offset: i64,
        mut dst_buffer: CustomAutoRooterGuard<ArrayBufferView>,
        dst_elem_offset: u32,
        length: u32,
    ) {
        let bound_buffer =
            handle_potential_webgl_error!(self.base, self.bound_buffer(target), return);
        let bound_buffer =
            handle_potential_webgl_error!(self.base, bound_buffer.ok_or(InvalidOperation), return);

        let dst_elem_size = typedarray_elem_size(dst_buffer.get_array_type());
        let dst_elem_count = dst_buffer.len() / dst_elem_size;
        let dst_elem_offset = dst_elem_offset as usize;
        let dst_byte_offset = dst_elem_offset * dst_elem_size;

        if src_byte_offset < 0 || dst_byte_offset > dst_buffer.len() {
            return self.base.webgl_error(InvalidValue);
        }

        let copy_count = if length == 0 {
            dst_elem_count - dst_elem_offset
        } else {
            length as usize
        };
        if copy_count == 0 {
            return;
        }
        let copy_bytes = copy_count * dst_elem_size;

        // TODO(mmatyas): Transform Feedback

        let src_byte_offset = src_byte_offset as usize;
        if src_byte_offset + copy_bytes > bound_buffer.capacity() ||
            dst_byte_offset + copy_bytes > dst_buffer.len()
        {
            return self.base.webgl_error(InvalidValue);
        }

        let (sender, receiver) = ipc::bytes_channel().unwrap();
        self.base.send_command(WebGLCommand::GetBufferSubData(
            target,
            src_byte_offset,
            copy_bytes,
            sender,
        ));
        let data = receiver.recv().unwrap();
        let dst_end = dst_byte_offset + copy_bytes;
        unsafe {
            dst_buffer.as_mut_slice()[dst_byte_offset..dst_end].copy_from_slice(&data);
        }
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.6>
    #[allow(unsafe_code)]
    fn CompressedTexImage2D(
        &self,
        target: u32,
        level: i32,
        internal_format: u32,
        width: i32,
        height: i32,
        border: i32,
        pixels: CustomAutoRooterGuard<ArrayBufferView>,
        src_offset: u32,
        src_length_override: u32,
    ) {
        let mut data = unsafe { pixels.as_slice() };
        let start = src_offset as usize;
        let end = (src_offset + src_length_override) as usize;
        if start > data.len() || end > data.len() {
            self.base.webgl_error(InvalidValue);
            return;
        }
        if src_length_override != 0 {
            data = &data[start..end];
        }
        self.base.compressed_tex_image_2d(
            target,
            level,
            internal_format,
            width,
            height,
            border,
            data,
        )
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8>
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
        pixels: CustomAutoRooterGuard<ArrayBufferView>,
        src_offset: u32,
        src_length_override: u32,
    ) {
        let mut data = unsafe { pixels.as_slice() };
        let start = src_offset as usize;
        let end = (src_offset + src_length_override) as usize;
        if start > data.len() || end > data.len() {
            self.base.webgl_error(InvalidValue);
            return;
        }
        if src_length_override != 0 {
            data = &data[start..end];
        }
        self.base.compressed_tex_sub_image_2d(
            target, level, xoffset, yoffset, width, height, format, data,
        )
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8>
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

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8>
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

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.11>
    fn Clear(&self, mask: u32) {
        self.base.Clear(mask)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3>
    fn ClearColor(&self, red: f32, green: f32, blue: f32, alpha: f32) {
        self.base.ClearColor(red, green, blue, alpha)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3>
    fn ClearDepth(&self, depth: f32) {
        self.base.ClearDepth(depth)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3>
    fn ClearStencil(&self, stencil: i32) {
        self.base.ClearStencil(stencil)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3>
    fn ColorMask(&self, r: bool, g: bool, b: bool, a: bool) {
        self.base.ColorMask(r, g, b, a)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3>
    fn CullFace(&self, mode: u32) {
        self.base.CullFace(mode)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3>
    fn FrontFace(&self, mode: u32) {
        self.base.FrontFace(mode)
    }
    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3>
    fn DepthFunc(&self, func: u32) {
        self.base.DepthFunc(func)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3>
    fn DepthMask(&self, flag: bool) {
        self.base.DepthMask(flag)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3>
    fn DepthRange(&self, near: f32, far: f32) {
        self.base.DepthRange(near, far)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3>
    fn Enable(&self, cap: u32) {
        match cap {
            constants::RASTERIZER_DISCARD => {
                self.enable_rasterizer_discard.set(true);
                self.base.send_command(WebGLCommand::Enable(cap));
            },
            _ => self.base.Enable(cap),
        }
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3>
    fn Disable(&self, cap: u32) {
        match cap {
            constants::RASTERIZER_DISCARD => {
                self.enable_rasterizer_discard.set(false);
                self.base.send_command(WebGLCommand::Disable(cap));
            },
            _ => self.base.Disable(cap),
        }
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9>
    fn CompileShader(&self, shader: &WebGLShader) {
        self.base.CompileShader(shader)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.5>
    fn CreateBuffer(&self) -> Option<DomRoot<WebGLBuffer>> {
        self.base.CreateBuffer()
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.6>
    fn CreateFramebuffer(&self) -> Option<DomRoot<WebGLFramebuffer>> {
        self.base.CreateFramebuffer()
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.7>
    fn CreateRenderbuffer(&self) -> Option<DomRoot<WebGLRenderbuffer>> {
        self.base.CreateRenderbuffer()
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8>
    fn CreateTexture(&self) -> Option<DomRoot<WebGLTexture>> {
        self.base.CreateTexture()
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9>
    fn CreateProgram(&self) -> Option<DomRoot<WebGLProgram>> {
        self.base.CreateProgram()
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9>
    fn CreateShader(&self, shader_type: u32) -> Option<DomRoot<WebGLShader>> {
        self.base.CreateShader(shader_type)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.17>
    fn CreateVertexArray(&self) -> Option<DomRoot<WebGLVertexArrayObject>> {
        self.base.create_vertex_array_webgl2()
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.5>
    fn DeleteBuffer(&self, buffer: Option<&WebGLBuffer>) {
        let buffer = match buffer {
            Some(buffer) => buffer,
            None => return,
        };
        handle_potential_webgl_error!(self.base, self.base.validate_ownership(buffer), return);
        if buffer.is_marked_for_deletion() {
            return;
        }
        self.current_vao().unbind_buffer(buffer);
        self.unbind_from(&self.base.array_buffer_slot(), &buffer);
        self.unbind_from(&self.bound_copy_read_buffer, &buffer);
        self.unbind_from(&self.bound_copy_write_buffer, &buffer);
        self.unbind_from(&self.bound_pixel_pack_buffer, &buffer);
        self.unbind_from(&self.bound_pixel_unpack_buffer, &buffer);
        self.unbind_from(&self.bound_transform_feedback_buffer, &buffer);
        self.unbind_from(&self.bound_uniform_buffer, &buffer);

        for binding in self.indexed_uniform_buffer_bindings.iter() {
            self.unbind_from(&binding.buffer, &buffer);
        }
        for binding in self.indexed_transform_feedback_buffer_bindings.iter() {
            self.unbind_from(&binding.buffer, &buffer);
        }

        buffer.mark_for_deletion(Operation::Infallible);
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.6>
    fn DeleteFramebuffer(&self, framebuffer: Option<&WebGLFramebuffer>) {
        self.base.DeleteFramebuffer(framebuffer)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.7>
    fn DeleteRenderbuffer(&self, renderbuffer: Option<&WebGLRenderbuffer>) {
        self.base.DeleteRenderbuffer(renderbuffer)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8>
    fn DeleteTexture(&self, texture: Option<&WebGLTexture>) {
        self.base.DeleteTexture(texture)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9>
    fn DeleteProgram(&self, program: Option<&WebGLProgram>) {
        self.base.DeleteProgram(program)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9>
    fn DeleteShader(&self, shader: Option<&WebGLShader>) {
        self.base.DeleteShader(shader)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.17>
    fn DeleteVertexArray(&self, vertex_array: Option<&WebGLVertexArrayObject>) {
        self.base.delete_vertex_array_webgl2(vertex_array);
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.11>
    fn DrawArrays(&self, mode: u32, first: i32, count: i32) {
        self.validate_uniform_block_for_draw();
        self.validate_vertex_attribs_for_draw();
        self.base.DrawArrays(mode, first, count)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.11>
    fn DrawElements(&self, mode: u32, count: i32, type_: u32, offset: i64) {
        self.validate_uniform_block_for_draw();
        self.validate_vertex_attribs_for_draw();
        self.base.DrawElements(mode, count, type_, offset)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10>
    fn EnableVertexAttribArray(&self, attrib_id: u32) {
        self.base.EnableVertexAttribArray(attrib_id)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10>
    fn DisableVertexAttribArray(&self, attrib_id: u32) {
        self.base.DisableVertexAttribArray(attrib_id)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10>
    fn GetActiveUniform(
        &self,
        program: &WebGLProgram,
        index: u32,
    ) -> Option<DomRoot<WebGLActiveInfo>> {
        self.base.GetActiveUniform(program, index)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10>
    fn GetActiveAttrib(
        &self,
        program: &WebGLProgram,
        index: u32,
    ) -> Option<DomRoot<WebGLActiveInfo>> {
        self.base.GetActiveAttrib(program, index)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10>
    fn GetAttribLocation(&self, program: &WebGLProgram, name: DOMString) -> i32 {
        self.base.GetAttribLocation(program, name)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.7>
    fn GetFragDataLocation(&self, program: &WebGLProgram, name: DOMString) -> i32 {
        handle_potential_webgl_error!(self.base, self.base.validate_ownership(program), return -1);
        handle_potential_webgl_error!(self.base, program.get_frag_data_location(name), -1)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9>
    fn GetProgramInfoLog(&self, program: &WebGLProgram) -> Option<DOMString> {
        self.base.GetProgramInfoLog(program)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9>
    fn GetProgramParameter(&self, cx: JSContext, program: &WebGLProgram, param_id: u32) -> JSVal {
        handle_potential_webgl_error!(
            self.base,
            self.base.validate_ownership(program),
            return NullValue()
        );
        if program.is_deleted() {
            self.base.webgl_error(InvalidOperation);
            return NullValue();
        }
        match param_id {
            constants::TRANSFORM_FEEDBACK_VARYINGS => {
                Int32Value(program.transform_feedback_varyings_length())
            },
            constants::TRANSFORM_FEEDBACK_BUFFER_MODE => {
                Int32Value(program.transform_feedback_buffer_mode())
            },
            _ => self.base.GetProgramParameter(cx, program, param_id),
        }
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9>
    fn GetShaderInfoLog(&self, shader: &WebGLShader) -> Option<DOMString> {
        self.base.GetShaderInfoLog(shader)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9>
    fn GetShaderParameter(&self, cx: JSContext, shader: &WebGLShader, param_id: u32) -> JSVal {
        self.base.GetShaderParameter(cx, shader, param_id)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9>
    fn GetShaderPrecisionFormat(
        &self,
        shader_type: u32,
        precision_type: u32,
    ) -> Option<DomRoot<WebGLShaderPrecisionFormat>> {
        self.base
            .GetShaderPrecisionFormat(shader_type, precision_type)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.2>
    #[allow(unsafe_code)]
    fn GetIndexedParameter(&self, cx: JSContext, target: u32, index: u32) -> JSVal {
        let bindings = match target {
            constants::TRANSFORM_FEEDBACK_BUFFER_BINDING |
            constants::TRANSFORM_FEEDBACK_BUFFER_SIZE |
            constants::TRANSFORM_FEEDBACK_BUFFER_START => {
                &self.indexed_transform_feedback_buffer_bindings
            },
            constants::UNIFORM_BUFFER_BINDING |
            constants::UNIFORM_BUFFER_SIZE |
            constants::UNIFORM_BUFFER_START => &self.indexed_uniform_buffer_bindings,
            _ => {
                self.base.webgl_error(InvalidEnum);
                return NullValue();
            },
        };

        let binding = match bindings.get(index as usize) {
            Some(binding) => binding,
            None => {
                self.base.webgl_error(InvalidValue);
                return NullValue();
            },
        };

        match target {
            constants::TRANSFORM_FEEDBACK_BUFFER_BINDING | constants::UNIFORM_BUFFER_BINDING => unsafe {
                optional_root_object_to_js_or_null!(*cx, binding.buffer.get())
            },
            constants::TRANSFORM_FEEDBACK_BUFFER_START | constants::UNIFORM_BUFFER_START => {
                Int32Value(binding.start.get() as _)
            },
            constants::TRANSFORM_FEEDBACK_BUFFER_SIZE | constants::UNIFORM_BUFFER_SIZE => {
                Int32Value(binding.size.get() as _)
            },
            _ => unreachable!(),
        }
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10>
    fn GetUniformLocation(
        &self,
        program: &WebGLProgram,
        name: DOMString,
    ) -> Option<DomRoot<WebGLUniformLocation>> {
        self.base.GetUniformLocation(program, name)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9>
    fn GetVertexAttrib(&self, cx: JSContext, index: u32, pname: u32) -> JSVal {
        self.base.GetVertexAttrib(cx, index, pname)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10>
    fn GetVertexAttribOffset(&self, index: u32, pname: u32) -> i64 {
        self.base.GetVertexAttribOffset(index, pname)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3>
    fn Hint(&self, target: u32, mode: u32) {
        self.base.Hint(target, mode)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.5>
    fn IsBuffer(&self, buffer: Option<&WebGLBuffer>) -> bool {
        self.base.IsBuffer(buffer)
    }

    // TODO: We could write this without IPC, recording the calls to `enable` and `disable`.
    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.2>
    fn IsEnabled(&self, cap: u32) -> bool {
        match cap {
            constants::RASTERIZER_DISCARD => self.enable_rasterizer_discard.get(),
            _ => self.base.IsEnabled(cap),
        }
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.6>
    fn IsFramebuffer(&self, frame_buffer: Option<&WebGLFramebuffer>) -> bool {
        self.base.IsFramebuffer(frame_buffer)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9>
    fn IsProgram(&self, program: Option<&WebGLProgram>) -> bool {
        self.base.IsProgram(program)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.7>
    fn IsRenderbuffer(&self, render_buffer: Option<&WebGLRenderbuffer>) -> bool {
        self.base.IsRenderbuffer(render_buffer)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9>
    fn IsShader(&self, shader: Option<&WebGLShader>) -> bool {
        self.base.IsShader(shader)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8>
    fn IsTexture(&self, texture: Option<&WebGLTexture>) -> bool {
        self.base.IsTexture(texture)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.17>
    fn IsVertexArray(&self, vertex_array: Option<&WebGLVertexArrayObject>) -> bool {
        self.base.is_vertex_array_webgl2(vertex_array)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3>
    fn LineWidth(&self, width: f32) {
        self.base.LineWidth(width)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.2>
    fn PixelStorei(&self, param_name: u32, param_value: i32) {
        if param_value < 0 {
            return self.base.webgl_error(InvalidValue);
        }

        match param_name {
            constants::PACK_ROW_LENGTH => self.texture_pack_row_length.set(param_value as _),
            constants::PACK_SKIP_PIXELS => self.texture_pack_skip_pixels.set(param_value as _),
            constants::PACK_SKIP_ROWS => self.texture_pack_skip_rows.set(param_value as _),
            _ => self.base.PixelStorei(param_name, param_value),
        }
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3>
    fn PolygonOffset(&self, factor: f32, units: f32) {
        self.base.PolygonOffset(factor, units)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.12>
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
        let pixels =
            handle_potential_webgl_error!(self.base, pixels.as_mut().ok_or(InvalidValue), return);

        self.read_pixels_into(x, y, width, height, format, pixel_type, pixels, 0)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.10>
    fn ReadPixels_(
        &self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        format: u32,
        pixel_type: u32,
        dst_byte_offset: i64,
    ) {
        handle_potential_webgl_error!(self.base, self.base.validate_framebuffer(), return);

        let dst = match self.bound_pixel_pack_buffer.get() {
            Some(buffer) => buffer,
            None => return self.base.webgl_error(InvalidOperation),
        };

        if dst_byte_offset < 0 {
            return self.base.webgl_error(InvalidValue);
        }
        let dst_byte_offset = dst_byte_offset as usize;
        if dst_byte_offset > dst.capacity() {
            return self.base.webgl_error(InvalidOperation);
        }

        let ReadPixelsAllowedFormats {
            array_types: _,
            channels: bytes_per_pixel,
        } = match self.calc_read_pixel_formats(pixel_type, format) {
            Ok(result) => result,
            Err(error) => return self.base.webgl_error(error),
        };
        if format != constants::RGBA || pixel_type != constants::UNSIGNED_BYTE {
            return self.base.webgl_error(InvalidOperation);
        }

        let ReadPixelsSizes {
            row_stride: _,
            skipped_bytes,
            size,
        } = match self.calc_read_pixel_sizes(width, height, bytes_per_pixel) {
            Ok(result) => result,
            Err(error) => return self.base.webgl_error(error),
        };
        let dst_end = dst_byte_offset + skipped_bytes + size;
        if dst.capacity() < dst_end {
            return self.base.webgl_error(InvalidOperation);
        }

        {
            let (fb_width, fb_height) = handle_potential_webgl_error!(
                self.base,
                self.base
                    .get_current_framebuffer_size()
                    .ok_or(InvalidOperation),
                return
            );
            let src_origin = Point2D::new(x, y);
            let src_size = Size2D::new(width as u32, height as u32);
            let fb_size = Size2D::new(fb_width as u32, fb_height as u32);
            if pixels::clip(src_origin, src_size.to_u64(), fb_size.to_u64()).is_none() {
                return;
            }
        }
        let src_rect = Rect::new(Point2D::new(x, y), Size2D::new(width, height));

        self.base.send_command(WebGLCommand::PixelStorei(
            constants::PACK_ALIGNMENT,
            self.base.get_texture_packing_alignment() as _,
        ));
        self.base.send_command(WebGLCommand::PixelStorei(
            constants::PACK_ROW_LENGTH,
            self.texture_pack_row_length.get() as _,
        ));
        self.base.send_command(WebGLCommand::PixelStorei(
            constants::PACK_SKIP_ROWS,
            self.texture_pack_skip_rows.get() as _,
        ));
        self.base.send_command(WebGLCommand::PixelStorei(
            constants::PACK_SKIP_PIXELS,
            self.texture_pack_skip_pixels.get() as _,
        ));
        self.base.send_command(WebGLCommand::ReadPixelsPP(
            src_rect,
            format,
            pixel_type,
            dst_byte_offset,
        ));
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.10>
    #[allow(unsafe_code)]
    fn ReadPixels__(
        &self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        format: u32,
        pixel_type: u32,
        mut dst: CustomAutoRooterGuard<ArrayBufferView>,
        dst_elem_offset: u32,
    ) {
        self.read_pixels_into(
            x,
            y,
            width,
            height,
            format,
            pixel_type,
            &mut dst,
            dst_elem_offset,
        )
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3>
    fn SampleCoverage(&self, value: f32, invert: bool) {
        self.base.SampleCoverage(value, invert)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.4>
    fn Scissor(&self, x: i32, y: i32, width: i32, height: i32) {
        self.base.Scissor(x, y, width, height)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3>
    fn StencilFunc(&self, func: u32, ref_: i32, mask: u32) {
        self.base.StencilFunc(func, ref_, mask)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3>
    fn StencilFuncSeparate(&self, face: u32, func: u32, ref_: i32, mask: u32) {
        self.base.StencilFuncSeparate(face, func, ref_, mask)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3>
    fn StencilMask(&self, mask: u32) {
        self.base.StencilMask(mask)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3>
    fn StencilMaskSeparate(&self, face: u32, mask: u32) {
        self.base.StencilMaskSeparate(face, mask)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3>
    fn StencilOp(&self, fail: u32, zfail: u32, zpass: u32) {
        self.base.StencilOp(fail, zfail, zpass)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.3>
    fn StencilOpSeparate(&self, face: u32, fail: u32, zfail: u32, zpass: u32) {
        self.base.StencilOpSeparate(face, fail, zfail, zpass)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9>
    fn LinkProgram(&self, program: &WebGLProgram) {
        self.base.LinkProgram(program)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9>
    fn ShaderSource(&self, shader: &WebGLShader, source: DOMString) {
        self.base.ShaderSource(shader, source)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9>
    fn GetShaderSource(&self, shader: &WebGLShader) -> Option<DOMString> {
        self.base.GetShaderSource(shader)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10>
    fn Uniform1f(&self, location: Option<&WebGLUniformLocation>, val: f32) {
        self.base.Uniform1f(location, val)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10>
    fn Uniform1i(&self, location: Option<&WebGLUniformLocation>, val: i32) {
        self.base.Uniform1i(location, val)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10>
    fn Uniform1iv(
        &self,
        location: Option<&WebGLUniformLocation>,
        v: Int32ArrayOrLongSequence,
        src_offset: u32,
        src_length: u32,
    ) {
        self.base.uniform1iv(location, v, src_offset, src_length)
    }

    // https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.8
    fn Uniform1ui(&self, location: Option<&WebGLUniformLocation>, val: u32) {
        self.base.with_location(location, |location| {
            match location.type_() {
                constants::BOOL | constants::UNSIGNED_INT => (),
                _ => return Err(InvalidOperation),
            }
            self.base
                .send_command(WebGLCommand::Uniform1ui(location.id(), val));
            Ok(())
        });
    }

    // https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.8
    fn Uniform1uiv(
        &self,
        location: Option<&WebGLUniformLocation>,
        val: Uint32ArrayOrUnsignedLongSequence,
        src_offset: u32,
        src_length: u32,
    ) {
        self.base.with_location(location, |location| {
            match location.type_() {
                constants::BOOL |
                constants::UNSIGNED_INT |
                constants::SAMPLER_2D |
                constants::SAMPLER_CUBE => {},
                _ => return Err(InvalidOperation),
            }

            let val = self.uniform_vec_section_uint(val, src_offset, src_length, 1, location)?;

            match location.type_() {
                constants::SAMPLER_2D | constants::SAMPLER_CUBE => {
                    for &v in val
                        .iter()
                        .take(cmp::min(location.size().unwrap_or(1) as usize, val.len()))
                    {
                        if v >= self.base.limits().max_combined_texture_image_units {
                            return Err(InvalidValue);
                        }
                    }
                },
                _ => {},
            }
            self.base
                .send_command(WebGLCommand::Uniform1uiv(location.id(), val));
            Ok(())
        });
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10>
    fn Uniform1fv(
        &self,
        location: Option<&WebGLUniformLocation>,
        v: Float32ArrayOrUnrestrictedFloatSequence,
        src_offset: u32,
        src_length: u32,
    ) {
        self.base.uniform1fv(location, v, src_offset, src_length);
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10>
    fn Uniform2f(&self, location: Option<&WebGLUniformLocation>, x: f32, y: f32) {
        self.base.Uniform2f(location, x, y)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10>
    fn Uniform2fv(
        &self,
        location: Option<&WebGLUniformLocation>,
        v: Float32ArrayOrUnrestrictedFloatSequence,
        src_offset: u32,
        src_length: u32,
    ) {
        self.base.uniform2fv(location, v, src_offset, src_length);
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10>
    fn Uniform2i(&self, location: Option<&WebGLUniformLocation>, x: i32, y: i32) {
        self.base.Uniform2i(location, x, y)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10>
    fn Uniform2iv(
        &self,
        location: Option<&WebGLUniformLocation>,
        v: Int32ArrayOrLongSequence,
        src_offset: u32,
        src_length: u32,
    ) {
        self.base.uniform2iv(location, v, src_offset, src_length)
    }

    // https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.8
    fn Uniform2ui(&self, location: Option<&WebGLUniformLocation>, x: u32, y: u32) {
        self.base.with_location(location, |location| {
            match location.type_() {
                constants::BOOL_VEC2 | constants::UNSIGNED_INT_VEC2 => {},
                _ => return Err(InvalidOperation),
            }
            self.base
                .send_command(WebGLCommand::Uniform2ui(location.id(), x, y));
            Ok(())
        });
    }

    // https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.8
    fn Uniform2uiv(
        &self,
        location: Option<&WebGLUniformLocation>,
        val: Uint32ArrayOrUnsignedLongSequence,
        src_offset: u32,
        src_length: u32,
    ) {
        self.base.with_location(location, |location| {
            match location.type_() {
                constants::BOOL_VEC2 | constants::UNSIGNED_INT_VEC2 => {},
                _ => return Err(InvalidOperation),
            }
            let val = self.uniform_vec_section_uint(val, src_offset, src_length, 2, location)?;
            self.base
                .send_command(WebGLCommand::Uniform2uiv(location.id(), val));
            Ok(())
        });
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10>
    fn Uniform3f(&self, location: Option<&WebGLUniformLocation>, x: f32, y: f32, z: f32) {
        self.base.Uniform3f(location, x, y, z)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10>
    fn Uniform3fv(
        &self,
        location: Option<&WebGLUniformLocation>,
        v: Float32ArrayOrUnrestrictedFloatSequence,
        src_offset: u32,
        src_length: u32,
    ) {
        self.base.uniform3fv(location, v, src_offset, src_length);
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10>
    fn Uniform3i(&self, location: Option<&WebGLUniformLocation>, x: i32, y: i32, z: i32) {
        self.base.Uniform3i(location, x, y, z)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10>
    fn Uniform3iv(
        &self,
        location: Option<&WebGLUniformLocation>,
        v: Int32ArrayOrLongSequence,
        src_offset: u32,
        src_length: u32,
    ) {
        self.base.uniform3iv(location, v, src_offset, src_length)
    }

    // https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.8
    fn Uniform3ui(&self, location: Option<&WebGLUniformLocation>, x: u32, y: u32, z: u32) {
        self.base.with_location(location, |location| {
            match location.type_() {
                constants::BOOL_VEC3 | constants::UNSIGNED_INT_VEC3 => {},
                _ => return Err(InvalidOperation),
            }
            self.base
                .send_command(WebGLCommand::Uniform3ui(location.id(), x, y, z));
            Ok(())
        });
    }

    // https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.8
    fn Uniform3uiv(
        &self,
        location: Option<&WebGLUniformLocation>,
        val: Uint32ArrayOrUnsignedLongSequence,
        src_offset: u32,
        src_length: u32,
    ) {
        self.base.with_location(location, |location| {
            match location.type_() {
                constants::BOOL_VEC3 | constants::UNSIGNED_INT_VEC3 => {},
                _ => return Err(InvalidOperation),
            }
            let val = self.uniform_vec_section_uint(val, src_offset, src_length, 3, location)?;
            self.base
                .send_command(WebGLCommand::Uniform3uiv(location.id(), val));
            Ok(())
        });
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10>
    fn Uniform4i(&self, location: Option<&WebGLUniformLocation>, x: i32, y: i32, z: i32, w: i32) {
        self.base.Uniform4i(location, x, y, z, w)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10>
    fn Uniform4iv(
        &self,
        location: Option<&WebGLUniformLocation>,
        v: Int32ArrayOrLongSequence,
        src_offset: u32,
        src_length: u32,
    ) {
        self.base.uniform4iv(location, v, src_offset, src_length)
    }

    // https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.8
    fn Uniform4ui(&self, location: Option<&WebGLUniformLocation>, x: u32, y: u32, z: u32, w: u32) {
        self.base.with_location(location, |location| {
            match location.type_() {
                constants::BOOL_VEC4 | constants::UNSIGNED_INT_VEC4 => {},
                _ => return Err(InvalidOperation),
            }
            self.base
                .send_command(WebGLCommand::Uniform4ui(location.id(), x, y, z, w));
            Ok(())
        });
    }

    // https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.8
    fn Uniform4uiv(
        &self,
        location: Option<&WebGLUniformLocation>,
        val: Uint32ArrayOrUnsignedLongSequence,
        src_offset: u32,
        src_length: u32,
    ) {
        self.base.with_location(location, |location| {
            match location.type_() {
                constants::BOOL_VEC4 | constants::UNSIGNED_INT_VEC4 => {},
                _ => return Err(InvalidOperation),
            }
            let val = self.uniform_vec_section_uint(val, src_offset, src_length, 4, location)?;
            self.base
                .send_command(WebGLCommand::Uniform4uiv(location.id(), val));
            Ok(())
        });
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10>
    fn Uniform4f(&self, location: Option<&WebGLUniformLocation>, x: f32, y: f32, z: f32, w: f32) {
        self.base.Uniform4f(location, x, y, z, w)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10>
    fn Uniform4fv(
        &self,
        location: Option<&WebGLUniformLocation>,
        v: Float32ArrayOrUnrestrictedFloatSequence,
        src_offset: u32,
        src_length: u32,
    ) {
        self.base.uniform4fv(location, v, src_offset, src_length);
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10>
    fn UniformMatrix2fv(
        &self,
        location: Option<&WebGLUniformLocation>,
        transpose: bool,
        v: Float32ArrayOrUnrestrictedFloatSequence,
        src_offset: u32,
        src_length: u32,
    ) {
        self.base
            .uniform_matrix_2fv(location, transpose, v, src_offset, src_length)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10>
    fn UniformMatrix3fv(
        &self,
        location: Option<&WebGLUniformLocation>,
        transpose: bool,
        v: Float32ArrayOrUnrestrictedFloatSequence,
        src_offset: u32,
        src_length: u32,
    ) {
        self.base
            .uniform_matrix_3fv(location, transpose, v, src_offset, src_length)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10>
    fn UniformMatrix4fv(
        &self,
        location: Option<&WebGLUniformLocation>,
        transpose: bool,
        v: Float32ArrayOrUnrestrictedFloatSequence,
        src_offset: u32,
        src_length: u32,
    ) {
        self.base
            .uniform_matrix_4fv(location, transpose, v, src_offset, src_length)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.8>
    fn UniformMatrix3x2fv(
        &self,
        location: Option<&WebGLUniformLocation>,
        transpose: bool,
        val: Float32ArrayOrUnrestrictedFloatSequence,
        src_offset: u32,
        src_length: u32,
    ) {
        self.base.with_location(location, |location| {
            match location.type_() {
                constants::FLOAT_MAT3x2 => {},
                _ => return Err(InvalidOperation),
            }
            let val = self.base.uniform_matrix_section(
                val,
                src_offset,
                src_length,
                transpose,
                3 * 2,
                location,
            )?;
            self.base
                .send_command(WebGLCommand::UniformMatrix3x2fv(location.id(), val));
            Ok(())
        });
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.8>
    fn UniformMatrix4x2fv(
        &self,
        location: Option<&WebGLUniformLocation>,
        transpose: bool,
        val: Float32ArrayOrUnrestrictedFloatSequence,
        src_offset: u32,
        src_length: u32,
    ) {
        self.base.with_location(location, |location| {
            match location.type_() {
                constants::FLOAT_MAT4x2 => {},
                _ => return Err(InvalidOperation),
            }
            let val = self.base.uniform_matrix_section(
                val,
                src_offset,
                src_length,
                transpose,
                4 * 2,
                location,
            )?;
            self.base
                .send_command(WebGLCommand::UniformMatrix4x2fv(location.id(), val));
            Ok(())
        });
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.8>
    fn UniformMatrix2x3fv(
        &self,
        location: Option<&WebGLUniformLocation>,
        transpose: bool,
        val: Float32ArrayOrUnrestrictedFloatSequence,
        src_offset: u32,
        src_length: u32,
    ) {
        self.base.with_location(location, |location| {
            match location.type_() {
                constants::FLOAT_MAT2x3 => {},
                _ => return Err(InvalidOperation),
            }
            let val = self.base.uniform_matrix_section(
                val,
                src_offset,
                src_length,
                transpose,
                2 * 3,
                location,
            )?;
            self.base
                .send_command(WebGLCommand::UniformMatrix2x3fv(location.id(), val));
            Ok(())
        });
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.8>
    fn UniformMatrix4x3fv(
        &self,
        location: Option<&WebGLUniformLocation>,
        transpose: bool,
        val: Float32ArrayOrUnrestrictedFloatSequence,
        src_offset: u32,
        src_length: u32,
    ) {
        self.base.with_location(location, |location| {
            match location.type_() {
                constants::FLOAT_MAT4x3 => {},
                _ => return Err(InvalidOperation),
            }
            let val = self.base.uniform_matrix_section(
                val,
                src_offset,
                src_length,
                transpose,
                4 * 3,
                location,
            )?;
            self.base
                .send_command(WebGLCommand::UniformMatrix4x3fv(location.id(), val));
            Ok(())
        });
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.8>
    fn UniformMatrix2x4fv(
        &self,
        location: Option<&WebGLUniformLocation>,
        transpose: bool,
        val: Float32ArrayOrUnrestrictedFloatSequence,
        src_offset: u32,
        src_length: u32,
    ) {
        self.base.with_location(location, |location| {
            match location.type_() {
                constants::FLOAT_MAT2x4 => {},
                _ => return Err(InvalidOperation),
            }
            let val = self.base.uniform_matrix_section(
                val,
                src_offset,
                src_length,
                transpose,
                2 * 4,
                location,
            )?;
            self.base
                .send_command(WebGLCommand::UniformMatrix2x4fv(location.id(), val));
            Ok(())
        });
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.8>
    fn UniformMatrix3x4fv(
        &self,
        location: Option<&WebGLUniformLocation>,
        transpose: bool,
        val: Float32ArrayOrUnrestrictedFloatSequence,
        src_offset: u32,
        src_length: u32,
    ) {
        self.base.with_location(location, |location| {
            match location.type_() {
                constants::FLOAT_MAT3x4 => {},
                _ => return Err(InvalidOperation),
            }
            let val = self.base.uniform_matrix_section(
                val,
                src_offset,
                src_length,
                transpose,
                3 * 4,
                location,
            )?;
            self.base
                .send_command(WebGLCommand::UniformMatrix3x4fv(location.id(), val));
            Ok(())
        });
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.8>
    #[allow(unsafe_code)]
    fn GetUniform(
        &self,
        cx: JSContext,
        program: &WebGLProgram,
        location: &WebGLUniformLocation,
    ) -> JSVal {
        handle_potential_webgl_error!(
            self.base,
            self.base.uniform_check_program(program, location),
            return NullValue()
        );

        let triple = (&*self.base, program.id(), location.id());

        match location.type_() {
            constants::UNSIGNED_INT => {
                UInt32Value(uniform_get(triple, WebGLCommand::GetUniformUint))
            },
            constants::UNSIGNED_INT_VEC2 => unsafe {
                uniform_typed::<Uint32>(*cx, &uniform_get(triple, WebGLCommand::GetUniformUint2))
            },
            constants::UNSIGNED_INT_VEC3 => unsafe {
                uniform_typed::<Uint32>(*cx, &uniform_get(triple, WebGLCommand::GetUniformUint3))
            },
            constants::UNSIGNED_INT_VEC4 => unsafe {
                uniform_typed::<Uint32>(*cx, &uniform_get(triple, WebGLCommand::GetUniformUint4))
            },
            constants::FLOAT_MAT2x3 => unsafe {
                uniform_typed::<Float32>(
                    *cx,
                    &uniform_get(triple, WebGLCommand::GetUniformFloat2x3),
                )
            },
            constants::FLOAT_MAT2x4 => unsafe {
                uniform_typed::<Float32>(
                    *cx,
                    &uniform_get(triple, WebGLCommand::GetUniformFloat2x4),
                )
            },
            constants::FLOAT_MAT3x2 => unsafe {
                uniform_typed::<Float32>(
                    *cx,
                    &uniform_get(triple, WebGLCommand::GetUniformFloat3x2),
                )
            },
            constants::FLOAT_MAT3x4 => unsafe {
                uniform_typed::<Float32>(
                    *cx,
                    &uniform_get(triple, WebGLCommand::GetUniformFloat3x4),
                )
            },
            constants::FLOAT_MAT4x2 => unsafe {
                uniform_typed::<Float32>(
                    *cx,
                    &uniform_get(triple, WebGLCommand::GetUniformFloat4x2),
                )
            },
            constants::FLOAT_MAT4x3 => unsafe {
                uniform_typed::<Float32>(
                    *cx,
                    &uniform_get(triple, WebGLCommand::GetUniformFloat4x3),
                )
            },
            constants::SAMPLER_3D | constants::SAMPLER_2D_ARRAY => {
                Int32Value(uniform_get(triple, WebGLCommand::GetUniformInt))
            },
            _ => self.base.GetUniform(cx, program, location),
        }
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9>
    fn UseProgram(&self, program: Option<&WebGLProgram>) {
        self.base.UseProgram(program)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9>
    fn ValidateProgram(&self, program: &WebGLProgram) {
        self.base.ValidateProgram(program)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10>
    fn VertexAttrib1f(&self, indx: u32, x: f32) {
        self.base.VertexAttrib1f(indx, x)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10>
    fn VertexAttrib1fv(&self, indx: u32, v: Float32ArrayOrUnrestrictedFloatSequence) {
        self.base.VertexAttrib1fv(indx, v)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10>
    fn VertexAttrib2f(&self, indx: u32, x: f32, y: f32) {
        self.base.VertexAttrib2f(indx, x, y)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10>
    fn VertexAttrib2fv(&self, indx: u32, v: Float32ArrayOrUnrestrictedFloatSequence) {
        self.base.VertexAttrib2fv(indx, v)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10>
    fn VertexAttrib3f(&self, indx: u32, x: f32, y: f32, z: f32) {
        self.base.VertexAttrib3f(indx, x, y, z)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10>
    fn VertexAttrib3fv(&self, indx: u32, v: Float32ArrayOrUnrestrictedFloatSequence) {
        self.base.VertexAttrib3fv(indx, v)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10>
    fn VertexAttrib4f(&self, indx: u32, x: f32, y: f32, z: f32, w: f32) {
        self.base.VertexAttrib4f(indx, x, y, z, w)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10>
    fn VertexAttrib4fv(&self, indx: u32, v: Float32ArrayOrUnrestrictedFloatSequence) {
        self.base.VertexAttrib4fv(indx, v)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.8>
    fn VertexAttribI4i(&self, index: u32, x: i32, y: i32, z: i32, w: i32) {
        self.vertex_attrib_i(index, x, y, z, w)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.8>
    fn VertexAttribI4iv(&self, index: u32, v: Int32ArrayOrLongSequence) {
        let values = match v {
            Int32ArrayOrLongSequence::Int32Array(v) => v.to_vec(),
            Int32ArrayOrLongSequence::LongSequence(v) => v,
        };
        if values.len() < 4 {
            return self.base.webgl_error(InvalidValue);
        }
        self.vertex_attrib_i(index, values[0], values[1], values[2], values[3]);
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.8>
    fn VertexAttribI4ui(&self, index: u32, x: u32, y: u32, z: u32, w: u32) {
        self.vertex_attrib_u(index, x, y, z, w)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.8>
    fn VertexAttribI4uiv(&self, index: u32, v: Uint32ArrayOrUnsignedLongSequence) {
        let values = match v {
            Uint32ArrayOrUnsignedLongSequence::Uint32Array(v) => v.to_vec(),
            Uint32ArrayOrUnsignedLongSequence::UnsignedLongSequence(v) => v,
        };
        if values.len() < 4 {
            return self.base.webgl_error(InvalidValue);
        }
        self.vertex_attrib_u(index, values[0], values[1], values[2], values[3]);
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.10>
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

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.8>
    fn VertexAttribIPointer(&self, index: u32, size: i32, type_: u32, stride: i32, offset: i64) {
        match type_ {
            constants::BYTE |
            constants::UNSIGNED_BYTE |
            constants::SHORT |
            constants::UNSIGNED_SHORT |
            constants::INT |
            constants::UNSIGNED_INT => {},
            _ => return self.base.webgl_error(InvalidEnum),
        };
        self.base
            .VertexAttribPointer(index, size, type_, false, stride, offset)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.4>
    fn Viewport(&self, x: i32, y: i32, width: i32, height: i32) {
        self.base.Viewport(x, y, width, height)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8>
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

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8>
    fn TexImage2D_(
        &self,
        target: u32,
        level: i32,
        internal_format: i32,
        format: u32,
        data_type: u32,
        source: ImageDataOrHTMLImageElementOrHTMLCanvasElementOrHTMLVideoElement,
    ) -> ErrorResult {
        self.base
            .TexImage2D_(target, level, internal_format, format, data_type, source)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.6>
    fn TexImage2D__(
        &self,
        target: u32,
        level: i32,
        internalformat: i32,
        width: i32,
        height: i32,
        border: i32,
        format: u32,
        type_: u32,
        pbo_offset: i64,
    ) -> Fallible<()> {
        let pixel_unpack_buffer = match self.bound_pixel_unpack_buffer.get() {
            Some(pixel_unpack_buffer) => pixel_unpack_buffer,
            None => return Ok(self.base.webgl_error(InvalidOperation)),
        };

        if let Some(tf_buffer) = self.bound_transform_feedback_buffer.get() {
            if pixel_unpack_buffer == tf_buffer {
                return Ok(self.base.webgl_error(InvalidOperation));
            }
        }

        if pbo_offset < 0 || pbo_offset as usize > pixel_unpack_buffer.capacity() {
            return Ok(self.base.webgl_error(InvalidValue));
        }

        let unpacking_alignment = self.base.texture_unpacking_alignment();

        let validator = TexImage2DValidator::new(
            &self.base,
            target,
            level,
            internalformat as u32,
            width,
            height,
            border,
            format,
            type_,
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
            Err(_) => return Ok(()),
        };

        self.base.tex_image_2d(
            &texture,
            target,
            data_type,
            internal_format,
            format,
            level,
            border,
            unpacking_alignment,
            Size2D::new(width, height),
            TexSource::BufferOffset(pbo_offset),
        );

        Ok(())
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.6>
    fn TexImage2D___(
        &self,
        target: u32,
        level: i32,
        internalformat: i32,
        width: i32,
        height: i32,
        border: i32,
        format: u32,
        type_: u32,
        source: ImageDataOrHTMLImageElementOrHTMLCanvasElementOrHTMLVideoElement,
    ) -> Fallible<()> {
        if self.bound_pixel_unpack_buffer.get().is_some() {
            return Ok(self.base.webgl_error(InvalidOperation));
        }

        let validator = TexImage2DValidator::new(
            &self.base,
            target,
            level,
            internalformat as u32,
            width,
            height,
            border,
            format,
            type_,
        );

        let TexImage2DValidatorResult {
            texture,
            target,
            width: _,
            height: _,
            level,
            border,
            internal_format,
            format,
            data_type,
        } = match validator.validate() {
            Ok(result) => result,
            Err(_) => return Ok(()),
        };

        let unpacking_alignment = self.base.texture_unpacking_alignment();

        let pixels = match self.base.get_image_pixels(source)? {
            Some(pixels) => pixels,
            None => return Ok(()),
        };

        self.base.tex_image_2d(
            &texture,
            target,
            data_type,
            internal_format,
            format,
            level,
            border,
            unpacking_alignment,
            pixels.size(),
            TexSource::Pixels(pixels),
        );

        Ok(())
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.6>
    #[allow(unsafe_code)]
    fn TexImage2D____(
        &self,
        target: u32,
        level: i32,
        internalformat: i32,
        width: i32,
        height: i32,
        border: i32,
        format: u32,
        type_: u32,
        src_data: CustomAutoRooterGuard<ArrayBufferView>,
        src_offset: u32,
    ) -> Fallible<()> {
        if self.bound_pixel_unpack_buffer.get().is_some() {
            return Ok(self.base.webgl_error(InvalidOperation));
        }

        if type_ == constants::FLOAT_32_UNSIGNED_INT_24_8_REV {
            return Ok(self.base.webgl_error(InvalidOperation));
        }

        let validator = TexImage2DValidator::new(
            &self.base,
            target,
            level,
            internalformat as u32,
            width,
            height,
            border,
            format,
            type_,
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
            Err(_) => return Ok(()),
        };

        let unpacking_alignment = self.base.texture_unpacking_alignment();

        let src_elem_size = typedarray_elem_size(src_data.get_array_type());
        let src_byte_offset = src_offset as usize * src_elem_size;

        if src_data.len() <= src_byte_offset {
            return Ok(self.base.webgl_error(InvalidOperation));
        }

        let buff = IpcSharedMemory::from_bytes(unsafe { &src_data.as_slice()[src_byte_offset..] });

        let expected_byte_length = match {
            self.base.validate_tex_image_2d_data(
                width,
                height,
                format,
                data_type,
                unpacking_alignment,
                Some(&*src_data),
            )
        } {
            Ok(byte_length) => byte_length,
            Err(()) => return Ok(()),
        };

        if expected_byte_length as usize > buff.len() {
            return Ok(self.base.webgl_error(InvalidOperation));
        }

        let size = Size2D::new(width, height);

        self.base.tex_image_2d(
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

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8>
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

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8>
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

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8>
    fn TexParameterf(&self, target: u32, name: u32, value: f32) {
        self.base.TexParameterf(target, name, value)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.8>
    fn TexParameteri(&self, target: u32, name: u32, value: i32) {
        self.base.TexParameteri(target, name, value)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.6>
    fn CheckFramebufferStatus(&self, target: u32) -> u32 {
        let fb_slot = match target {
            constants::FRAMEBUFFER | constants::DRAW_FRAMEBUFFER => {
                self.base.get_draw_framebuffer_slot()
            },
            constants::READ_FRAMEBUFFER => &self.base.get_read_framebuffer_slot(),
            _ => {
                self.base.webgl_error(InvalidEnum);
                return 0;
            },
        };
        match fb_slot.get() {
            Some(fb) => fb.check_status(),
            None => constants::FRAMEBUFFER_COMPLETE,
        }
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.7>
    fn RenderbufferStorage(&self, target: u32, internal_format: u32, width: i32, height: i32) {
        self.base
            .RenderbufferStorage(target, internal_format, width, height)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.6>
    fn FramebufferRenderbuffer(
        &self,
        target: u32,
        attachment: u32,
        renderbuffertarget: u32,
        rb: Option<&WebGLRenderbuffer>,
    ) {
        if let Some(rb) = rb {
            handle_potential_webgl_error!(self.base, self.base.validate_ownership(rb), return);
        }

        let fb_slot = match target {
            constants::FRAMEBUFFER | constants::DRAW_FRAMEBUFFER => {
                self.base.get_draw_framebuffer_slot()
            },
            constants::READ_FRAMEBUFFER => &self.base.get_read_framebuffer_slot(),
            _ => return self.base.webgl_error(InvalidEnum),
        };

        if renderbuffertarget != constants::RENDERBUFFER {
            return self.base.webgl_error(InvalidEnum);
        }

        match fb_slot.get() {
            Some(fb) => match attachment {
                constants::DEPTH_STENCIL_ATTACHMENT => {
                    handle_potential_webgl_error!(
                        self.base,
                        fb.renderbuffer(constants::DEPTH_ATTACHMENT, rb)
                    );
                    handle_potential_webgl_error!(
                        self.base,
                        fb.renderbuffer(constants::STENCIL_ATTACHMENT, rb)
                    );
                },
                _ => handle_potential_webgl_error!(self.base, fb.renderbuffer(attachment, rb)),
            },
            None => self.base.webgl_error(InvalidOperation),
        };
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.6>
    fn FramebufferTexture2D(
        &self,
        target: u32,
        attachment: u32,
        textarget: u32,
        texture: Option<&WebGLTexture>,
        level: i32,
    ) {
        if let Some(texture) = texture {
            handle_potential_webgl_error!(self.base, self.base.validate_ownership(texture), return);
        }

        let fb_slot = match target {
            constants::FRAMEBUFFER | constants::DRAW_FRAMEBUFFER => {
                self.base.get_draw_framebuffer_slot()
            },
            constants::READ_FRAMEBUFFER => self.base.get_read_framebuffer_slot(),
            _ => return self.base.webgl_error(InvalidEnum),
        };
        match fb_slot.get() {
            Some(fb) => handle_potential_webgl_error!(
                self.base,
                fb.texture2d(attachment, textarget, texture, level)
            ),
            None => self.base.webgl_error(InvalidOperation),
        }
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.9>
    fn GetAttachedShaders(&self, program: &WebGLProgram) -> Option<Vec<DomRoot<WebGLShader>>> {
        self.base.GetAttachedShaders(program)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.9>
    fn DrawArraysInstanced(&self, mode: u32, first: i32, count: i32, primcount: i32) {
        self.validate_uniform_block_for_draw();
        self.validate_vertex_attribs_for_draw();
        handle_potential_webgl_error!(
            self.base,
            self.base
                .draw_arrays_instanced(mode, first, count, primcount)
        )
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.9>
    fn DrawElementsInstanced(
        &self,
        mode: u32,
        count: i32,
        type_: u32,
        offset: i64,
        primcount: i32,
    ) {
        self.validate_uniform_block_for_draw();
        self.validate_vertex_attribs_for_draw();
        handle_potential_webgl_error!(
            self.base,
            self.base
                .draw_elements_instanced(mode, count, type_, offset, primcount)
        )
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.9>
    fn DrawRangeElements(
        &self,
        mode: u32,
        start: u32,
        end: u32,
        count: i32,
        type_: u32,
        offset: i64,
    ) {
        if end < start {
            self.base.webgl_error(InvalidValue);
            return;
        }
        self.validate_uniform_block_for_draw();
        self.validate_vertex_attribs_for_draw();
        handle_potential_webgl_error!(
            self.base,
            self.base
                .draw_elements_instanced(mode, count, type_, offset, 1)
        )
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.9>
    fn VertexAttribDivisor(&self, index: u32, divisor: u32) {
        self.base.vertex_attrib_divisor(index, divisor);
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.12>
    fn CreateQuery(&self) -> Option<DomRoot<WebGLQuery>> {
        Some(WebGLQuery::new(&self.base))
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.12>
    #[rustfmt::skip]
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

            query.delete(Operation::Infallible);
        }
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.12>
    fn IsQuery(&self, query: Option<&WebGLQuery>) -> bool {
        match query {
            Some(query) => self.base.validate_ownership(query).is_ok() && query.is_valid(),
            None => false,
        }
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.13>
    fn CreateSampler(&self) -> Option<DomRoot<WebGLSampler>> {
        Some(WebGLSampler::new(&self.base))
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.13>
    fn DeleteSampler(&self, sampler: Option<&WebGLSampler>) {
        if let Some(sampler) = sampler {
            handle_potential_webgl_error!(self.base, self.base.validate_ownership(sampler), return);
            for slot in self.samplers.iter() {
                if slot.get().map_or(false, |s| sampler == &*s) {
                    slot.set(None);
                }
            }
            sampler.delete(Operation::Infallible);
        }
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.13>
    fn IsSampler(&self, sampler: Option<&WebGLSampler>) -> bool {
        match sampler {
            Some(sampler) => self.base.validate_ownership(sampler).is_ok() && sampler.is_valid(),
            None => false,
        }
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.12>
    #[rustfmt::skip]
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

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.12>
    #[rustfmt::skip]
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

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.12>
    #[rustfmt::skip]
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

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.12>
    #[rustfmt::skip]
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

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.14>
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

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.14>
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

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.14>
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

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.14>
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

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.14>
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

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.14>
    fn DeleteSync(&self, sync: Option<&WebGLSync>) {
        if let Some(sync) = sync {
            handle_potential_webgl_error!(self.base, self.base.validate_ownership(sync), return);
            sync.delete(Operation::Infallible);
        }
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.13>
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

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.17>
    fn BindVertexArray(&self, array: Option<&WebGLVertexArrayObject>) {
        self.base.bind_vertex_array_webgl2(array);
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.13>
    fn SamplerParameteri(&self, sampler: &WebGLSampler, pname: u32, param: i32) {
        handle_potential_webgl_error!(self.base, self.base.validate_ownership(sampler), return);
        let param = WebGLSamplerValue::GLenum(param as u32);
        let result = sampler.set_parameter(&self.base, pname, param);
        if let Err(error) = result {
            self.base.webgl_error(error);
        }
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.13>
    fn SamplerParameterf(&self, sampler: &WebGLSampler, pname: u32, param: f32) {
        handle_potential_webgl_error!(self.base, self.base.validate_ownership(sampler), return);
        let param = WebGLSamplerValue::Float(param);
        let result = sampler.set_parameter(&self.base, pname, param);
        if let Err(error) = result {
            self.base.webgl_error(error);
        }
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.13>
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

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.15>
    fn CreateTransformFeedback(&self) -> Option<DomRoot<WebGLTransformFeedback>> {
        Some(WebGLTransformFeedback::new(&self.base))
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.15>
    fn DeleteTransformFeedback(&self, tf: Option<&WebGLTransformFeedback>) {
        if let Some(tf) = tf {
            handle_potential_webgl_error!(self.base, self.base.validate_ownership(tf), return);
            if tf.is_active() {
                self.base.webgl_error(InvalidOperation);
                return;
            }
            tf.delete(Operation::Infallible);
            self.current_transform_feedback.set(None);
        }
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.15>
    fn IsTransformFeedback(&self, tf: Option<&WebGLTransformFeedback>) -> bool {
        match tf {
            Some(tf) => {
                if !tf.is_valid() {
                    return false;
                }
                handle_potential_webgl_error!(
                    self.base,
                    self.base.validate_ownership(tf),
                    return false
                );
                let (sender, receiver) = webgl_channel().unwrap();
                self.base
                    .send_command(WebGLCommand::IsTransformFeedback(tf.id(), sender));
                receiver.recv().unwrap()
            },
            None => false,
        }
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.15>
    fn BindTransformFeedback(&self, target: u32, tf: Option<&WebGLTransformFeedback>) {
        if target != constants::TRANSFORM_FEEDBACK {
            self.base.webgl_error(InvalidEnum);
            return;
        }
        match tf {
            Some(transform_feedback) => {
                handle_potential_webgl_error!(
                    self.base,
                    self.base.validate_ownership(transform_feedback),
                    return
                );
                if !transform_feedback.is_valid() {
                    self.base.webgl_error(InvalidOperation);
                    return;
                }
                if let Some(current_tf) = self.current_transform_feedback.get() {
                    if current_tf.is_active() && !current_tf.is_paused() {
                        self.base.webgl_error(InvalidOperation);
                        return;
                    }
                }
                transform_feedback.bind(&self.base, target);
                self.current_transform_feedback
                    .set(Some(transform_feedback));
            },
            None => self
                .base
                .send_command(WebGLCommand::BindTransformFeedback(target, 0)),
        }
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.15>
    #[allow(non_snake_case)]
    fn BeginTransformFeedback(&self, primitiveMode: u32) {
        match primitiveMode {
            constants::POINTS | constants::LINES | constants::TRIANGLES => {},
            _ => {
                self.base.webgl_error(InvalidEnum);
                return;
            },
        };
        let current_tf = match self.current_transform_feedback.get() {
            Some(current_tf) => current_tf,
            None => {
                self.base.webgl_error(InvalidOperation);
                return;
            },
        };
        if current_tf.is_active() {
            self.base.webgl_error(InvalidOperation);
            return;
        };
        let program = match self.base.current_program() {
            Some(program) => program,
            None => {
                self.base.webgl_error(InvalidOperation);
                return;
            },
        };
        if !program.is_linked() || program.transform_feedback_varyings_length() == 0 {
            self.base.webgl_error(InvalidOperation);
            return;
        };
        current_tf.begin(&self.base, primitiveMode);
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.15>
    fn EndTransformFeedback(&self) {
        if let Some(current_tf) = self.current_transform_feedback.get() {
            if !current_tf.is_active() {
                self.base.webgl_error(InvalidOperation);
                return;
            }
            current_tf.end(&self.base);
        }
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.15>
    fn ResumeTransformFeedback(&self) {
        if let Some(current_tf) = self.current_transform_feedback.get() {
            if !current_tf.is_active() || !current_tf.is_paused() {
                self.base.webgl_error(InvalidOperation);
                return;
            }
            current_tf.resume(&self.base);
        }
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.15>
    fn PauseTransformFeedback(&self) {
        if let Some(current_tf) = self.current_transform_feedback.get() {
            if !current_tf.is_active() || current_tf.is_paused() {
                self.base.webgl_error(InvalidOperation);
                return;
            }
            current_tf.pause(&self.base);
        }
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.15>
    #[allow(non_snake_case)]
    fn TransformFeedbackVaryings(
        &self,
        program: &WebGLProgram,
        varyings: Vec<DOMString>,
        bufferMode: u32,
    ) {
        handle_potential_webgl_error!(self.base, program.validate(), return);
        let strs = varyings
            .iter()
            .map(|name| String::from(name.to_owned()))
            .collect::<Vec<String>>();
        match bufferMode {
            constants::INTERLEAVED_ATTRIBS => {
                self.base
                    .send_command(WebGLCommand::TransformFeedbackVaryings(
                        program.id(),
                        strs,
                        bufferMode,
                    ));
            },
            constants::SEPARATE_ATTRIBS => {
                let max_tf_sp_att =
                    self.base.limits().max_transform_feedback_separate_attribs as usize;
                if strs.len() >= max_tf_sp_att {
                    self.base.webgl_error(InvalidValue);
                    return;
                }
                self.base
                    .send_command(WebGLCommand::TransformFeedbackVaryings(
                        program.id(),
                        strs,
                        bufferMode,
                    ));
            },
            _ => self.base.webgl_error(InvalidEnum),
        }
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.15>
    fn GetTransformFeedbackVarying(
        &self,
        program: &WebGLProgram,
        index: u32,
    ) -> Option<DomRoot<WebGLActiveInfo>> {
        handle_potential_webgl_error!(self.base, program.validate(), return None);
        if index >= program.transform_feedback_varyings_length() as u32 {
            self.base.webgl_error(InvalidValue);
            return None;
        }

        let (sender, receiver) = webgl_channel().unwrap();
        self.base
            .send_command(WebGLCommand::GetTransformFeedbackVarying(
                program.id(),
                index,
                sender,
            ));
        let (size, ty, name) = receiver.recv().unwrap();
        Some(WebGLActiveInfo::new(
            self.base.global().as_window(),
            size,
            ty,
            DOMString::from(name),
        ))
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.16>
    fn BindBufferBase(&self, target: u32, index: u32, buffer: Option<&WebGLBuffer>) {
        let (generic_slot, indexed_bindings) = match target {
            constants::TRANSFORM_FEEDBACK_BUFFER => (
                &self.bound_transform_feedback_buffer,
                &self.indexed_transform_feedback_buffer_bindings,
            ),
            constants::UNIFORM_BUFFER => (
                &self.bound_uniform_buffer,
                &self.indexed_uniform_buffer_bindings,
            ),
            _ => return self.base.webgl_error(InvalidEnum),
        };
        let indexed_binding = match indexed_bindings.get(index as usize) {
            Some(slot) => slot,
            None => return self.base.webgl_error(InvalidValue),
        };

        if let Some(buffer) = buffer {
            handle_potential_webgl_error!(self.base, self.base.validate_ownership(buffer), return);

            if buffer.is_marked_for_deletion() {
                return self.base.webgl_error(InvalidOperation);
            }
            handle_potential_webgl_error!(self.base, buffer.set_target_maybe(target), return);

            // for both the generic and the indexed bindings
            buffer.increment_attached_counter();
            buffer.increment_attached_counter();
        }

        self.base.send_command(WebGLCommand::BindBufferBase(
            target,
            index,
            buffer.map(|b| b.id()),
        ));

        for slot in &[&generic_slot, &indexed_binding.buffer] {
            if let Some(old) = slot.get() {
                old.decrement_attached_counter(Operation::Infallible);
            }
            slot.set(buffer);
        }
        indexed_binding.start.set(0);
        indexed_binding.size.set(0);
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.16>
    fn BindBufferRange(
        &self,
        target: u32,
        index: u32,
        buffer: Option<&WebGLBuffer>,
        offset: i64,
        size: i64,
    ) {
        let (generic_slot, indexed_bindings) = match target {
            constants::TRANSFORM_FEEDBACK_BUFFER => (
                &self.bound_transform_feedback_buffer,
                &self.indexed_transform_feedback_buffer_bindings,
            ),
            constants::UNIFORM_BUFFER => (
                &self.bound_uniform_buffer,
                &self.indexed_uniform_buffer_bindings,
            ),
            _ => return self.base.webgl_error(InvalidEnum),
        };
        let indexed_binding = match indexed_bindings.get(index as usize) {
            Some(slot) => slot,
            None => return self.base.webgl_error(InvalidValue),
        };

        if offset < 0 || size < 0 {
            return self.base.webgl_error(InvalidValue);
        }
        if buffer.is_some() && size == 0 {
            return self.base.webgl_error(InvalidValue);
        }

        match target {
            constants::TRANSFORM_FEEDBACK_BUFFER => {
                if size % 4 != 0 && offset % 4 != 0 {
                    return self.base.webgl_error(InvalidValue);
                }
            },
            constants::UNIFORM_BUFFER => {
                let offset_alignment = self.base.limits().uniform_buffer_offset_alignment;
                if offset % offset_alignment as i64 != 0 {
                    return self.base.webgl_error(InvalidValue);
                }
            },
            _ => unreachable!(),
        }

        if let Some(buffer) = buffer {
            handle_potential_webgl_error!(self.base, self.base.validate_ownership(buffer), return);

            if buffer.is_marked_for_deletion() {
                return self.base.webgl_error(InvalidOperation);
            }
            handle_potential_webgl_error!(self.base, buffer.set_target_maybe(target), return);

            // for both the generic and the indexed bindings
            buffer.increment_attached_counter();
            buffer.increment_attached_counter();
        }

        self.base.send_command(WebGLCommand::BindBufferRange(
            target,
            index,
            buffer.map(|b| b.id()),
            offset,
            size,
        ));

        for slot in &[&generic_slot, &indexed_binding.buffer] {
            if let Some(old) = slot.get() {
                old.decrement_attached_counter(Operation::Infallible);
            }
            slot.set(buffer);
        }
        indexed_binding.start.set(offset);
        indexed_binding.size.set(size);
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.16>
    fn GetUniformIndices(&self, program: &WebGLProgram, names: Vec<DOMString>) -> Option<Vec<u32>> {
        handle_potential_webgl_error!(
            self.base,
            self.base.validate_ownership(program),
            return None
        );
        let indices = handle_potential_webgl_error!(
            self.base,
            program.get_uniform_indices(names),
            return None
        );
        Some(indices)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.16>
    #[allow(unsafe_code)]
    fn GetActiveUniforms(
        &self,
        cx: JSContext,
        program: &WebGLProgram,
        indices: Vec<u32>,
        pname: u32,
    ) -> JSVal {
        handle_potential_webgl_error!(
            self.base,
            self.base.validate_ownership(program),
            return NullValue()
        );
        let values = handle_potential_webgl_error!(
            self.base,
            program.get_active_uniforms(indices, pname),
            return NullValue()
        );

        rooted!(in(*cx) let mut rval = UndefinedValue());
        match pname {
            constants::UNIFORM_SIZE |
            constants::UNIFORM_TYPE |
            constants::UNIFORM_BLOCK_INDEX |
            constants::UNIFORM_OFFSET |
            constants::UNIFORM_ARRAY_STRIDE |
            constants::UNIFORM_MATRIX_STRIDE => unsafe {
                values.to_jsval(*cx, rval.handle_mut());
            },
            constants::UNIFORM_IS_ROW_MAJOR => unsafe {
                let values = values.iter().map(|&v| v != 0).collect::<Vec<_>>();
                values.to_jsval(*cx, rval.handle_mut());
            },
            _ => unreachable!(),
        }
        rval.get()
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.16>
    fn GetUniformBlockIndex(&self, program: &WebGLProgram, block_name: DOMString) -> u32 {
        handle_potential_webgl_error!(
            self.base,
            self.base.validate_ownership(program),
            return constants::INVALID_INDEX
        );
        let index = handle_potential_webgl_error!(
            self.base,
            program.get_uniform_block_index(block_name),
            return constants::INVALID_INDEX
        );
        index
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.16>
    #[allow(unsafe_code)]
    fn GetActiveUniformBlockParameter(
        &self,
        cx: JSContext,
        program: &WebGLProgram,
        block_index: u32,
        pname: u32,
    ) -> JSVal {
        handle_potential_webgl_error!(
            self.base,
            self.base.validate_ownership(program),
            return NullValue()
        );
        let values = handle_potential_webgl_error!(
            self.base,
            program.get_active_uniform_block_parameter(block_index, pname),
            return NullValue()
        );
        match pname {
            constants::UNIFORM_BLOCK_BINDING |
            constants::UNIFORM_BLOCK_DATA_SIZE |
            constants::UNIFORM_BLOCK_ACTIVE_UNIFORMS => {
                assert!(values.len() == 1);
                UInt32Value(values[0] as u32)
            },
            constants::UNIFORM_BLOCK_ACTIVE_UNIFORM_INDICES => unsafe {
                let values = values.iter().map(|&v| v as u32).collect::<Vec<_>>();
                rooted!(in(*cx) let mut result = ptr::null_mut::<JSObject>());
                let _ = Uint32Array::create(*cx, CreateWith::Slice(&values), result.handle_mut())
                    .unwrap();
                ObjectValue(result.get())
            },
            constants::UNIFORM_BLOCK_REFERENCED_BY_VERTEX_SHADER |
            constants::UNIFORM_BLOCK_REFERENCED_BY_FRAGMENT_SHADER => {
                assert!(values.len() == 1);
                BooleanValue(values[0] != 0)
            },
            _ => unreachable!(),
        }
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.16>
    fn GetActiveUniformBlockName(
        &self,
        program: &WebGLProgram,
        block_index: u32,
    ) -> Option<DOMString> {
        handle_potential_webgl_error!(
            self.base,
            self.base.validate_ownership(program),
            return None
        );
        let name = handle_potential_webgl_error!(
            self.base,
            program.get_active_uniform_block_name(block_index),
            return None
        );
        Some(DOMString::from(name))
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.16>
    fn UniformBlockBinding(&self, program: &WebGLProgram, block_index: u32, block_binding: u32) {
        handle_potential_webgl_error!(self.base, self.base.validate_ownership(program), return);

        if block_binding >= self.base.limits().max_uniform_buffer_bindings {
            return self.base.webgl_error(InvalidValue);
        }

        handle_potential_webgl_error!(
            self.base,
            program.bind_uniform_block(block_index, block_binding),
            return
        )
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.11>
    fn ClearBufferfv(
        &self,
        buffer: u32,
        draw_buffer: i32,
        values: Float32ArrayOrUnrestrictedFloatSequence,
        src_offset: u32,
    ) {
        let array = match values {
            Float32ArrayOrUnrestrictedFloatSequence::Float32Array(v) => v.to_vec(),
            Float32ArrayOrUnrestrictedFloatSequence::UnrestrictedFloatSequence(v) => v,
        };
        self.clear_buffer::<f32>(
            buffer,
            draw_buffer,
            &[constants::COLOR, constants::DEPTH],
            src_offset,
            array,
            WebGLCommand::ClearBufferfv,
        )
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.11>
    fn ClearBufferiv(
        &self,
        buffer: u32,
        draw_buffer: i32,
        values: Int32ArrayOrLongSequence,
        src_offset: u32,
    ) {
        let array = match values {
            Int32ArrayOrLongSequence::Int32Array(v) => v.to_vec(),
            Int32ArrayOrLongSequence::LongSequence(v) => v,
        };
        self.clear_buffer::<i32>(
            buffer,
            draw_buffer,
            &[constants::COLOR, constants::STENCIL],
            src_offset,
            array,
            WebGLCommand::ClearBufferiv,
        )
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.11>
    fn ClearBufferuiv(
        &self,
        buffer: u32,
        draw_buffer: i32,
        values: Uint32ArrayOrUnsignedLongSequence,
        src_offset: u32,
    ) {
        let array = match values {
            Uint32ArrayOrUnsignedLongSequence::Uint32Array(v) => v.to_vec(),
            Uint32ArrayOrUnsignedLongSequence::UnsignedLongSequence(v) => v,
        };
        self.clear_buffer::<u32>(
            buffer,
            draw_buffer,
            &[constants::COLOR],
            src_offset,
            array,
            WebGLCommand::ClearBufferuiv,
        )
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.11>
    fn ClearBufferfi(&self, buffer: u32, draw_buffer: i32, depth: f32, stencil: i32) {
        if buffer != constants::DEPTH_STENCIL {
            return self.base.webgl_error(InvalidEnum);
        }

        handle_potential_webgl_error!(
            self.base,
            self.clearbuffer_array_size(buffer, draw_buffer),
            return
        );

        self.base.send_command(WebGLCommand::ClearBufferfi(
            buffer,
            draw_buffer,
            depth,
            stencil,
        ));
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.4>
    fn InvalidateFramebuffer(&self, target: u32, attachments: Vec<u32>) {
        if !self.valid_fb_attachment_values(target, &attachments) {
            return;
        }

        self.base
            .send_command(WebGLCommand::InvalidateFramebuffer(target, attachments))
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.4>
    fn InvalidateSubFramebuffer(
        &self,
        target: u32,
        attachments: Vec<u32>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) {
        if !self.valid_fb_attachment_values(target, &attachments) {
            return;
        }

        if width < 0 || height < 0 {
            return self.base.webgl_error(InvalidValue);
        }

        self.base
            .send_command(WebGLCommand::InvalidateSubFramebuffer(
                target,
                attachments,
                x,
                y,
                width,
                height,
            ))
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.4>
    fn FramebufferTextureLayer(
        &self,
        target: u32,
        attachment: u32,
        texture: Option<&WebGLTexture>,
        level: i32,
        layer: i32,
    ) {
        if let Some(tex) = texture {
            handle_potential_webgl_error!(self.base, self.base.validate_ownership(tex), return);
        }

        let fb_slot = match target {
            constants::FRAMEBUFFER | constants::DRAW_FRAMEBUFFER => {
                self.base.get_draw_framebuffer_slot()
            },
            constants::READ_FRAMEBUFFER => self.base.get_read_framebuffer_slot(),
            _ => return self.base.webgl_error(InvalidEnum),
        };

        match fb_slot.get() {
            Some(fb) => handle_potential_webgl_error!(
                self.base,
                fb.texture_layer(attachment, texture, level, layer)
            ),
            None => self.base.webgl_error(InvalidOperation),
        }
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.5>
    #[allow(unsafe_code)]
    fn GetInternalformatParameter(
        &self,
        cx: JSContext,
        target: u32,
        internal_format: u32,
        pname: u32,
    ) -> JSVal {
        if target != constants::RENDERBUFFER {
            self.base.webgl_error(InvalidEnum);
            return NullValue();
        }

        match handle_potential_webgl_error!(
            self.base,
            InternalFormatParameter::from_u32(pname),
            return NullValue()
        ) {
            InternalFormatParameter::IntVec(param) => unsafe {
                let (sender, receiver) = webgl_channel().unwrap();
                self.base
                    .send_command(WebGLCommand::GetInternalFormatIntVec(
                        target,
                        internal_format,
                        param,
                        sender,
                    ));

                rooted!(in(*cx) let mut rval = ptr::null_mut::<JSObject>());
                let _ = Int32Array::create(
                    *cx,
                    CreateWith::Slice(&receiver.recv().unwrap()),
                    rval.handle_mut(),
                )
                .unwrap();
                ObjectValue(rval.get())
            },
        }
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.5>
    fn RenderbufferStorageMultisample(
        &self,
        target: u32,
        samples: i32,
        internal_format: u32,
        width: i32,
        height: i32,
    ) {
        self.base
            .renderbuffer_storage(target, samples, internal_format, width, height)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.4>
    fn ReadBuffer(&self, src: u32) {
        match src {
            constants::BACK | constants::NONE => {},
            _ if self.base.valid_color_attachment_enum(src) => {},
            _ => return self.base.webgl_error(InvalidEnum),
        }

        if let Some(fb) = self.base.get_read_framebuffer_slot().get() {
            handle_potential_webgl_error!(self.base, fb.set_read_buffer(src), return)
        } else {
            match src {
                constants::NONE | constants::BACK => {},
                _ => return self.base.webgl_error(InvalidOperation),
            }

            self.default_fb_readbuffer.set(src);
            self.base.send_command(WebGLCommand::ReadBuffer(src));
        }
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.11>
    fn DrawBuffers(&self, buffers: Vec<u32>) {
        if let Some(fb) = self.base.get_draw_framebuffer_slot().get() {
            handle_potential_webgl_error!(self.base, fb.set_draw_buffers(buffers), return)
        } else {
            if buffers.len() != 1 {
                return self.base.webgl_error(InvalidOperation);
            }

            match buffers[0] {
                constants::NONE | constants::BACK => {},
                _ => return self.base.webgl_error(InvalidOperation),
            }

            self.default_fb_drawbuffer.set(buffers[0]);
            self.base.send_command(WebGLCommand::DrawBuffers(buffers));
        }
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.6>
    fn TexStorage2D(
        &self,
        target: u32,
        levels: i32,
        internal_format: u32,
        width: i32,
        height: i32,
    ) {
        self.tex_storage(2, target, levels, internal_format, width, height, 1)
    }

    /// <https://www.khronos.org/registry/webgl/specs/latest/2.0/#3.7.6>
    fn TexStorage3D(
        &self,
        target: u32,
        levels: i32,
        internal_format: u32,
        width: i32,
        height: i32,
        depth: i32,
    ) {
        self.tex_storage(3, target, levels, internal_format, width, height, depth)
    }
}

impl LayoutCanvasRenderingContextHelpers for LayoutDom<'_, WebGL2RenderingContext> {
    #[allow(unsafe_code)]
    unsafe fn canvas_data_source(self) -> HTMLCanvasDataSource {
        let this = &*self.unsafe_get();
        (*this.base.to_layout().unsafe_get()).layout_handle()
    }
}
