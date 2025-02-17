/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(unused_imports)]

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl

use std::cell::Cell;
use std::cmp;

use canvas_traits::webgl::{
    webgl_channel, TexDataType, TexFormat, TexParameter, TexParameterBool, TexParameterInt,
    WebGLCommand, WebGLError, WebGLResult, WebGLTextureId,
};
use dom_struct::dom_struct;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::EXTTextureFilterAnisotropicBinding::EXTTextureFilterAnisotropicConstants;
use crate::dom::bindings::codegen::Bindings::WebGL2RenderingContextBinding::WebGL2RenderingContextConstants as constants;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal};
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::webgl_validations::types::TexImageTarget;
use crate::dom::webglframebuffer::WebGLFramebuffer;
use crate::dom::webglobject::WebGLObject;
use crate::dom::webglrenderingcontext::{Operation, WebGLRenderingContext};
#[cfg(feature = "webxr")]
use crate::dom::xrsession::XRSession;
use crate::script_runtime::CanGc;

pub(crate) enum TexParameterValue {
    Float(f32),
    Int(i32),
    Bool(bool),
}

// Textures generated for WebXR are owned by the WebXR device, not by the WebGL thread
// so the GL texture should not be deleted when the texture is garbage collected.
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
#[derive(JSTraceable, MallocSizeOf)]
enum WebGLTextureOwner {
    WebGL,
    #[cfg(feature = "webxr")]
    WebXR(Dom<XRSession>),
}

const MAX_LEVEL_COUNT: usize = 31;
const MAX_FACE_COUNT: usize = 6;

#[dom_struct]
pub(crate) struct WebGLTexture {
    webgl_object: WebGLObject,
    #[no_trace]
    id: WebGLTextureId,
    /// The target to which this texture was bound the first time
    target: Cell<Option<u32>>,
    is_deleted: Cell<bool>,
    owner: WebGLTextureOwner,
    /// Stores information about mipmap levels and cubemap faces.
    #[ignore_malloc_size_of = "Arrays are cumbersome"]
    image_info_array: DomRefCell<[Option<ImageInfo>; MAX_LEVEL_COUNT * MAX_FACE_COUNT]>,
    /// Face count can only be 1 or 6
    face_count: Cell<u8>,
    base_mipmap_level: u32,
    // Store information for min and mag filters
    min_filter: Cell<u32>,
    mag_filter: Cell<u32>,
    /// Framebuffer that this texture is attached to.
    attached_framebuffer: MutNullableDom<WebGLFramebuffer>,
    /// Number of immutable levels.
    immutable_levels: Cell<Option<u32>>,
}

impl WebGLTexture {
    fn new_inherited(
        context: &WebGLRenderingContext,
        id: WebGLTextureId,
        #[cfg(feature = "webxr")] owner: Option<&XRSession>,
    ) -> Self {
        Self {
            webgl_object: WebGLObject::new_inherited(context),
            id,
            target: Cell::new(None),
            is_deleted: Cell::new(false),
            #[cfg(feature = "webxr")]
            owner: owner
                .map(|session| WebGLTextureOwner::WebXR(Dom::from_ref(session)))
                .unwrap_or(WebGLTextureOwner::WebGL),
            #[cfg(not(feature = "webxr"))]
            owner: WebGLTextureOwner::WebGL,
            immutable_levels: Cell::new(None),
            face_count: Cell::new(0),
            base_mipmap_level: 0,
            min_filter: Cell::new(constants::NEAREST_MIPMAP_LINEAR),
            mag_filter: Cell::new(constants::LINEAR),
            image_info_array: DomRefCell::new([None; MAX_LEVEL_COUNT * MAX_FACE_COUNT]),
            attached_framebuffer: Default::default(),
        }
    }

    pub(crate) fn maybe_new(context: &WebGLRenderingContext) -> Option<DomRoot<Self>> {
        let (sender, receiver) = webgl_channel().unwrap();
        context.send_command(WebGLCommand::CreateTexture(sender));
        receiver
            .recv()
            .unwrap()
            .map(|id| WebGLTexture::new(context, id, CanGc::note()))
    }

    pub(crate) fn new(
        context: &WebGLRenderingContext,
        id: WebGLTextureId,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(WebGLTexture::new_inherited(
                context,
                id,
                #[cfg(feature = "webxr")]
                None,
            )),
            &*context.global(),
            can_gc,
        )
    }

    #[cfg(feature = "webxr")]
    pub(crate) fn new_webxr(
        context: &WebGLRenderingContext,
        id: WebGLTextureId,
        session: &XRSession,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(WebGLTexture::new_inherited(context, id, Some(session))),
            &*context.global(),
            can_gc,
        )
    }
}

impl WebGLTexture {
    pub(crate) fn id(&self) -> WebGLTextureId {
        self.id
    }

    // NB: Only valid texture targets come here
    pub(crate) fn bind(&self, target: u32) -> WebGLResult<()> {
        if self.is_invalid() {
            return Err(WebGLError::InvalidOperation);
        }

        if let Some(previous_target) = self.target.get() {
            if target != previous_target {
                return Err(WebGLError::InvalidOperation);
            }
        } else {
            // This is the first time binding
            let face_count = match target {
                constants::TEXTURE_2D => 1,
                constants::TEXTURE_CUBE_MAP => 6,
                _ => return Err(WebGLError::InvalidEnum),
            };
            self.face_count.set(face_count);
            self.target.set(Some(target));
        }

        self.upcast::<WebGLObject>()
            .context()
            .send_command(WebGLCommand::BindTexture(target, Some(self.id)));

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn initialize(
        &self,
        target: TexImageTarget,
        width: u32,
        height: u32,
        depth: u32,
        internal_format: TexFormat,
        level: u32,
        data_type: Option<TexDataType>,
    ) -> WebGLResult<()> {
        let image_info = ImageInfo {
            width,
            height,
            depth,
            internal_format,
            data_type,
        };

        let face_index = self.face_index_for_target(&target);
        self.set_image_infos_at_level_and_face(level, face_index, image_info);

        if let Some(fb) = self.attached_framebuffer.get() {
            fb.update_status();
        }

        Ok(())
    }

    pub(crate) fn generate_mipmap(&self) -> WebGLResult<()> {
        let target = match self.target.get() {
            Some(target) => target,
            None => {
                error!("Cannot generate mipmap on texture that has no target!");
                return Err(WebGLError::InvalidOperation);
            },
        };

        let base_image_info = self.base_image_info().ok_or(WebGLError::InvalidOperation)?;

        let is_cubic = target == constants::TEXTURE_CUBE_MAP;
        if is_cubic && !self.is_cube_complete() {
            return Err(WebGLError::InvalidOperation);
        }

        if !base_image_info.is_power_of_two() {
            return Err(WebGLError::InvalidOperation);
        }

        if base_image_info.is_compressed_format() {
            return Err(WebGLError::InvalidOperation);
        }

        self.upcast::<WebGLObject>()
            .context()
            .send_command(WebGLCommand::GenerateMipmap(target));

        if self.base_mipmap_level + base_image_info.get_max_mimap_levels() == 0 {
            return Err(WebGLError::InvalidOperation);
        }

        let last_level = self.base_mipmap_level + base_image_info.get_max_mimap_levels() - 1;
        self.populate_mip_chain(self.base_mipmap_level, last_level)
    }

    pub(crate) fn delete(&self, operation_fallibility: Operation) {
        if !self.is_deleted.get() {
            self.is_deleted.set(true);
            let context = self.upcast::<WebGLObject>().context();

            /*
            If a texture object is deleted while its image is attached to one or more attachment
            points in a currently bound framebuffer, then it is as if FramebufferTexture had been
            called, with a texture of zero, for each attachment point to which this im-age was
            attached in that framebuffer. In other words, this texture image is firstdetached from
            all attachment points in a currently bound framebuffer.
            - GLES 3.0, 4.4.2.3, "Attaching Texture Images to a Framebuffer"
            */
            if let Some(fb) = context.get_draw_framebuffer_slot().get() {
                let _ = fb.detach_texture(self);
            }
            if let Some(fb) = context.get_read_framebuffer_slot().get() {
                let _ = fb.detach_texture(self);
            }

            // We don't delete textures owned by WebXR
            #[cfg(feature = "webxr")]
            if let WebGLTextureOwner::WebXR(_) = self.owner {
                return;
            }

            let cmd = WebGLCommand::DeleteTexture(self.id);
            match operation_fallibility {
                Operation::Fallible => context.send_command_ignored(cmd),
                Operation::Infallible => context.send_command(cmd),
            }
        }
    }

    pub(crate) fn is_invalid(&self) -> bool {
        // https://immersive-web.github.io/layers/#xrwebglsubimagetype
        #[cfg(feature = "webxr")]
        if let WebGLTextureOwner::WebXR(ref session) = self.owner {
            if session.is_outside_raf() {
                return true;
            }
        }
        self.is_deleted.get()
    }

    pub(crate) fn is_immutable(&self) -> bool {
        self.immutable_levels.get().is_some()
    }

    pub(crate) fn target(&self) -> Option<u32> {
        self.target.get()
    }

    pub(crate) fn maybe_get_tex_parameter(&self, param: TexParameter) -> Option<TexParameterValue> {
        match param {
            TexParameter::Int(TexParameterInt::TextureImmutableLevels) => Some(
                TexParameterValue::Int(self.immutable_levels.get().unwrap_or(0) as i32),
            ),
            TexParameter::Bool(TexParameterBool::TextureImmutableFormat) => {
                Some(TexParameterValue::Bool(self.is_immutable()))
            },
            _ => None,
        }
    }

    /// We have to follow the conversion rules for GLES 2.0. See:
    /// <https://www.khronos.org/webgl/public-mailing-list/archives/1008/msg00014.html>
    pub(crate) fn tex_parameter(&self, param: u32, value: TexParameterValue) -> WebGLResult<()> {
        let target = self.target().unwrap();

        let (int_value, float_value) = match value {
            TexParameterValue::Int(int_value) => (int_value, int_value as f32),
            TexParameterValue::Float(float_value) => (float_value as i32, float_value),
            TexParameterValue::Bool(_) => unreachable!("no settable tex params should be booleans"),
        };

        let update_filter = |filter: &Cell<u32>| {
            if filter.get() == int_value as u32 {
                return Ok(());
            }
            filter.set(int_value as u32);
            self.upcast::<WebGLObject>()
                .context()
                .send_command(WebGLCommand::TexParameteri(target, param, int_value));
            Ok(())
        };
        match param {
            constants::TEXTURE_MIN_FILTER => match int_value as u32 {
                constants::NEAREST |
                constants::LINEAR |
                constants::NEAREST_MIPMAP_NEAREST |
                constants::LINEAR_MIPMAP_NEAREST |
                constants::NEAREST_MIPMAP_LINEAR |
                constants::LINEAR_MIPMAP_LINEAR => update_filter(&self.min_filter),
                _ => Err(WebGLError::InvalidEnum),
            },
            constants::TEXTURE_MAG_FILTER => match int_value as u32 {
                constants::NEAREST | constants::LINEAR => update_filter(&self.mag_filter),
                _ => Err(WebGLError::InvalidEnum),
            },
            constants::TEXTURE_WRAP_S | constants::TEXTURE_WRAP_T => match int_value as u32 {
                constants::CLAMP_TO_EDGE | constants::MIRRORED_REPEAT | constants::REPEAT => {
                    self.upcast::<WebGLObject>()
                        .context()
                        .send_command(WebGLCommand::TexParameteri(target, param, int_value));
                    Ok(())
                },
                _ => Err(WebGLError::InvalidEnum),
            },
            EXTTextureFilterAnisotropicConstants::TEXTURE_MAX_ANISOTROPY_EXT => {
                // NaN is not less than 1., what a time to be alive.
                if float_value < 1. || !float_value.is_normal() {
                    return Err(WebGLError::InvalidValue);
                }
                self.upcast::<WebGLObject>()
                    .context()
                    .send_command(WebGLCommand::TexParameterf(target, param, float_value));
                Ok(())
            },
            _ => Err(WebGLError::InvalidEnum),
        }
    }

    pub(crate) fn min_filter(&self) -> u32 {
        self.min_filter.get()
    }

    pub(crate) fn mag_filter(&self) -> u32 {
        self.mag_filter.get()
    }

    pub(crate) fn is_using_linear_filtering(&self) -> bool {
        let filters = [self.min_filter.get(), self.mag_filter.get()];
        filters.iter().any(|filter| {
            matches!(
                *filter,
                constants::LINEAR |
                    constants::NEAREST_MIPMAP_LINEAR |
                    constants::LINEAR_MIPMAP_NEAREST |
                    constants::LINEAR_MIPMAP_LINEAR
            )
        })
    }

    pub(crate) fn populate_mip_chain(&self, first_level: u32, last_level: u32) -> WebGLResult<()> {
        let base_image_info = self
            .image_info_at_face(0, first_level)
            .ok_or(WebGLError::InvalidOperation)?;

        let mut ref_width = base_image_info.width;
        let mut ref_height = base_image_info.height;

        if ref_width == 0 || ref_height == 0 {
            return Err(WebGLError::InvalidOperation);
        }

        for level in (first_level + 1)..last_level {
            if ref_width == 1 && ref_height == 1 {
                break;
            }

            ref_width = cmp::max(1, ref_width / 2);
            ref_height = cmp::max(1, ref_height / 2);

            let image_info = ImageInfo {
                width: ref_width,
                height: ref_height,
                depth: 0,
                internal_format: base_image_info.internal_format,
                data_type: base_image_info.data_type,
            };

            self.set_image_infos_at_level(level, image_info);
        }
        Ok(())
    }

    fn is_cube_complete(&self) -> bool {
        debug_assert_eq!(self.face_count.get(), 6);

        let image_info = match self.base_image_info() {
            Some(info) => info,
            None => return false,
        };

        let ref_width = image_info.width;
        let ref_format = image_info.internal_format;

        for face in 0..self.face_count.get() {
            let current_image_info = match self.image_info_at_face(face, self.base_mipmap_level) {
                Some(info) => info,
                None => return false,
            };

            // Compares height with width to enforce square dimensions
            if current_image_info.internal_format != ref_format ||
                current_image_info.width != ref_width ||
                current_image_info.height != ref_width
            {
                return false;
            }
        }

        true
    }

    fn face_index_for_target(&self, target: &TexImageTarget) -> u8 {
        match *target {
            TexImageTarget::CubeMapPositiveX => 0,
            TexImageTarget::CubeMapNegativeX => 1,
            TexImageTarget::CubeMapPositiveY => 2,
            TexImageTarget::CubeMapNegativeY => 3,
            TexImageTarget::CubeMapPositiveZ => 4,
            TexImageTarget::CubeMapNegativeZ => 5,
            _ => 0,
        }
    }

    pub(crate) fn image_info_for_target(
        &self,
        target: &TexImageTarget,
        level: u32,
    ) -> Option<ImageInfo> {
        let face_index = self.face_index_for_target(target);
        self.image_info_at_face(face_index, level)
    }

    pub(crate) fn image_info_at_face(&self, face: u8, level: u32) -> Option<ImageInfo> {
        let pos = (level * self.face_count.get() as u32) + face as u32;
        self.image_info_array.borrow()[pos as usize]
    }

    fn set_image_infos_at_level(&self, level: u32, image_info: ImageInfo) {
        for face in 0..self.face_count.get() {
            self.set_image_infos_at_level_and_face(level, face, image_info);
        }
    }

    fn set_image_infos_at_level_and_face(&self, level: u32, face: u8, image_info: ImageInfo) {
        debug_assert!(face < self.face_count.get());
        let pos = (level * self.face_count.get() as u32) + face as u32;
        self.image_info_array.borrow_mut()[pos as usize] = Some(image_info);
    }

    fn base_image_info(&self) -> Option<ImageInfo> {
        assert!((self.base_mipmap_level as usize) < MAX_LEVEL_COUNT);

        self.image_info_at_face(0, self.base_mipmap_level)
    }

    pub(crate) fn attach_to_framebuffer(&self, fb: &WebGLFramebuffer) {
        self.attached_framebuffer.set(Some(fb));
    }

    pub(crate) fn detach_from_framebuffer(&self) {
        self.attached_framebuffer.set(None);
    }

    pub(crate) fn storage(
        &self,
        target: TexImageTarget,
        levels: u32,
        internal_format: TexFormat,
        width: u32,
        height: u32,
        depth: u32,
    ) -> WebGLResult<()> {
        // Handled by the caller
        assert!(!self.is_immutable());
        assert!(self.target().is_some());

        let target_id = target.as_gl_constant();
        let command = match target {
            TexImageTarget::Texture2D | TexImageTarget::CubeMap => {
                WebGLCommand::TexStorage2D(target_id, levels, internal_format, width, height)
            },
            TexImageTarget::Texture3D | TexImageTarget::Texture2DArray => {
                WebGLCommand::TexStorage3D(target_id, levels, internal_format, width, height, depth)
            },
            _ => unreachable!(), // handled by the caller
        };
        self.upcast::<WebGLObject>().context().send_command(command);

        let mut width = width;
        let mut height = height;
        let mut depth = depth;
        for level in 0..levels {
            let image_info = ImageInfo {
                width,
                height,
                depth,
                internal_format,
                data_type: None,
            };
            self.set_image_infos_at_level(level, image_info);

            width = cmp::max(1, width / 2);
            height = cmp::max(1, height / 2);
            depth = cmp::max(1, depth / 2);
        }

        self.immutable_levels.set(Some(levels));

        if let Some(fb) = self.attached_framebuffer.get() {
            fb.update_status();
        }

        Ok(())
    }
}

impl Drop for WebGLTexture {
    fn drop(&mut self) {
        self.delete(Operation::Fallible);
    }
}

#[derive(Clone, Copy, Debug, JSTraceable, MallocSizeOf, PartialEq)]
pub(crate) struct ImageInfo {
    width: u32,
    height: u32,
    depth: u32,
    #[no_trace]
    internal_format: TexFormat,
    #[no_trace]
    data_type: Option<TexDataType>,
}

impl ImageInfo {
    pub(crate) fn width(&self) -> u32 {
        self.width
    }

    pub(crate) fn height(&self) -> u32 {
        self.height
    }

    pub(crate) fn internal_format(&self) -> TexFormat {
        self.internal_format
    }

    pub(crate) fn data_type(&self) -> Option<TexDataType> {
        self.data_type
    }

    fn is_power_of_two(&self) -> bool {
        self.width.is_power_of_two() &&
            self.height.is_power_of_two() &&
            self.depth.is_power_of_two()
    }

    fn get_max_mimap_levels(&self) -> u32 {
        let largest = cmp::max(cmp::max(self.width, self.height), self.depth);
        if largest == 0 {
            return 0;
        }
        // FloorLog2(largest) + 1
        (largest as f64).log2() as u32 + 1
    }

    fn is_compressed_format(&self) -> bool {
        self.internal_format.is_compressed()
    }
}

#[derive(Clone, Copy, Debug, JSTraceable, MallocSizeOf)]
pub(crate) enum TexCompressionValidation {
    None,
    S3TC,
}

#[derive(Clone, Copy, Debug, JSTraceable, MallocSizeOf)]
pub(crate) struct TexCompression {
    #[no_trace]
    pub(crate) format: TexFormat,
    pub(crate) bytes_per_block: u8,
    pub(crate) block_width: u8,
    pub(crate) block_height: u8,
    pub(crate) validation: TexCompressionValidation,
}
