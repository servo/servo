/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use canvas_traits::webgl::WebGLError::*;
use canvas_traits::webgl::{TexDataType, TexFormat};
use js::jsapi::Type;
use js::typedarray::ArrayBufferView;

use super::WebGLValidator;
use super::tex_image_2d::TexImageValidationError;
use super::types::TexImageTarget;
use crate::dom::bindings::root::DomRoot;
use crate::dom::webglrenderingcontext::WebGLRenderingContext;
use crate::dom::webgltexture::WebGLTexture;

fn log2(n: u32) -> u32 {
    31 - n.leading_zeros()
}

pub(crate) struct CommonTexImage3DValidator<'a> {
    context: &'a WebGLRenderingContext,
    target: u32,
    level: i32,
    internal_format: u32,
    width: i32,
    height: i32,
    depth: i32,
    border: i32,
}

pub(crate) struct CommonTexImage3DValidatorResult {
    pub(crate) texture: DomRoot<WebGLTexture>,
    pub(crate) target: TexImageTarget,
    pub(crate) level: u32,
    pub(crate) internal_format: TexFormat,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) depth: u32,
    pub(crate) border: u32,
}

impl WebGLValidator for CommonTexImage3DValidator<'_> {
    type Error = TexImageValidationError;
    type ValidatedOutput = CommonTexImage3DValidatorResult;
    fn validate(self) -> Result<Self::ValidatedOutput, TexImageValidationError> {
        // GL_INVALID_ENUM is generated if target is not GL_TEXTURE_3D or GL_TEXTURE_2D_ARRAY.
        let target = match TexImageTarget::from_gl_constant(self.target) {
            Some(target) if target.dimensions() == 3 => target,
            _ => {
                self.context.webgl_error(InvalidEnum);
                return Err(TexImageValidationError::InvalidTextureTarget(self.target));
            },
        };

        let texture = self
            .context
            .textures()
            .active_texture_for_image_target(target);
        let limits = self.context.limits();

        let max_size = limits.max_3d_tex_size;

        //  If an attempt is made to call this function with no WebGLTexture
        //  bound, an INVALID_OPERATION error is generated.
        let texture = match texture {
            Some(texture) => texture,
            None => {
                self.context.webgl_error(InvalidOperation);
                return Err(TexImageValidationError::TextureTargetNotBound(self.target));
            },
        };

        // GL_INVALID_ENUM is generated if internal_format is not an accepted
        // format.
        let internal_format = match TexFormat::from_gl_constant(self.internal_format) {
            Some(format)
                if format.required_webgl_version() <= self.context.webgl_version() &&
                    format.usable_as_internal() =>
            {
                format
            },
            _ => {
                self.context.webgl_error(InvalidEnum);
                return Err(TexImageValidationError::InvalidTextureFormat);
            },
        };

        // GL_INVALID_VALUE is generated if width, height, or depth is less than 0 or greater than
        // GL_MAX_3D_TEXTURE_SIZE.
        if self.width < 0 || self.height < 0 || self.depth < 0 {
            self.context.webgl_error(InvalidValue);
            return Err(TexImageValidationError::NegativeDimension);
        }
        let width = self.width as u32;
        let height = self.height as u32;
        let depth = self.depth as u32;
        let level = self.level as u32;
        if width > max_size || height > max_size || level > max_size {
            self.context.webgl_error(InvalidValue);
            return Err(TexImageValidationError::TextureTooBig);
        }

        // GL_INVALID_VALUE may be generated if level is greater than log2(max),
        // where max is the returned value of GL_MAX_3D_TEXTURE_SIZE.
        if self.level < 0 {
            self.context.webgl_error(InvalidValue);
            return Err(TexImageValidationError::NegativeLevel);
        }
        if level > log2(max_size) {
            self.context.webgl_error(InvalidValue);
            return Err(TexImageValidationError::LevelTooHigh);
        }

        // GL_INVALID_VALUE is generated if border is not 0 or 1.
        if !(self.border == 0 || self.border == 1) {
            self.context.webgl_error(InvalidValue);
            return Err(TexImageValidationError::InvalidBorder);
        }

        Ok(CommonTexImage3DValidatorResult {
            texture,
            target,
            level,
            internal_format,
            width,
            height,
            depth,
            border: self.border as u32,
        })
    }
}

impl<'a> CommonTexImage3DValidator<'a> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        context: &'a WebGLRenderingContext,
        target: u32,
        level: i32,
        internal_format: u32,
        width: i32,
        height: i32,
        depth: i32,
        border: i32,
    ) -> Self {
        CommonTexImage3DValidator {
            context,
            target,
            level,
            internal_format,
            width,
            height,
            depth,
            border,
        }
    }
}

pub(crate) struct TexImage3DValidator<'a> {
    common_validator: CommonTexImage3DValidator<'a>,
    format: u32,
    data_type: u32,
    data: &'a Option<ArrayBufferView>,
}

impl<'a> TexImage3DValidator<'a> {
    /// TODO: Move data validation logic here.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        context: &'a WebGLRenderingContext,
        target: u32,
        level: i32,
        internal_format: u32,
        width: i32,
        height: i32,
        depth: i32,
        border: i32,
        format: u32,
        data_type: u32,
        data: &'a Option<ArrayBufferView>,
    ) -> Self {
        TexImage3DValidator {
            common_validator: CommonTexImage3DValidator::new(
                context,
                target,
                level,
                internal_format,
                width,
                height,
                depth,
                border,
            ),
            format,
            data_type,
            data,
        }
    }
}

/// The validated result of a TexImage2DValidator-validated call.
pub(crate) struct TexImage3DValidatorResult {
    /// NB: width, height and level are already unsigned after validation.
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) depth: u32,
    pub(crate) level: u32,
    pub(crate) border: u32,
    pub(crate) texture: DomRoot<WebGLTexture>,
    pub(crate) target: TexImageTarget,
    pub(crate) internal_format: TexFormat,
    pub(crate) format: TexFormat,
    pub(crate) data_type: TexDataType,
}

/// TexImage3d validator as per
/// <https://www.khronos.org/opengles/sdk/docs/man/xhtml/glTexImage3D.xml>
impl WebGLValidator for TexImage3DValidator<'_> {
    type ValidatedOutput = TexImage3DValidatorResult;
    type Error = TexImageValidationError;

    fn validate(self) -> Result<Self::ValidatedOutput, TexImageValidationError> {
        let context = self.common_validator.context;
        let CommonTexImage3DValidatorResult {
            texture,
            target,
            level,
            internal_format,
            width,
            height,
            depth,
            border,
        } = self.common_validator.validate()?;

        // GL_INVALID_ENUM is generated if format is not an accepted format constant.
        // Format constants other than GL_STENCIL_INDEX and GL_DEPTH_COMPONENT are accepted.
        let data_type = match TexDataType::from_gl_constant(self.data_type) {
            Some(data_type) if data_type.required_webgl_version() <= context.webgl_version() => {
                data_type
            },
            _ => {
                context.webgl_error(InvalidEnum);
                return Err(TexImageValidationError::InvalidDataType);
            },
        };
        let format = match TexFormat::from_gl_constant(self.format) {
            Some(format) if format.required_webgl_version() <= context.webgl_version() => format,
            _ => {
                context.webgl_error(InvalidEnum);
                return Err(TexImageValidationError::InvalidTextureFormat);
            },
        };

        // GL_INVALID_OPERATION is generated if format does not match
        // internal_format.
        if format != internal_format.to_unsized() {
            context.webgl_error(InvalidOperation);
            return Err(TexImageValidationError::TextureFormatMismatch);
        }

        if !internal_format.compatible_data_types().contains(&data_type) {
            context.webgl_error(InvalidOperation);
            return Err(TexImageValidationError::TextureFormatMismatch);
        }

        // GL_INVALID_OPERATION is generated if target is GL_TEXTURE_3D and
        // format is GL_DEPTH_COMPONENT, or GL_DEPTH_STENCIL.
        if target == TexImageTarget::Texture3D &&
            (format == TexFormat::DepthComponent || format == TexFormat::DepthStencil)
        {
            context.webgl_error(InvalidOperation);
            return Err(TexImageValidationError::InvalidTypeForFormat);
        }

        // If srcData is non-null, the type of srcData must match the type according to
        // the above table; otherwise, generate an INVALID_OPERATION error.
        let element_size = data_type.element_size();
        let received_size = match self.data {
            Some(buf) => match buf.get_array_type() {
                Type::Int8 => 1,
                Type::Uint8 => 1,
                Type::Int16 => 2,
                Type::Uint16 => 2,
                Type::Int32 => 4,
                Type::Uint32 => 4,
                Type::Float32 => 4,
                _ => {
                    context.webgl_error(InvalidOperation);
                    return Err(TexImageValidationError::InvalidTypeForFormat);
                },
            },
            None => element_size,
        };
        if received_size != element_size {
            context.webgl_error(InvalidOperation);
            return Err(TexImageValidationError::InvalidTypeForFormat);
        }

        Ok(TexImage3DValidatorResult {
            width,
            height,
            depth,
            level,
            border,
            texture,
            target,
            internal_format,
            format,
            data_type,
        })
    }
}
