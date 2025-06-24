/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::{self, cmp, fmt};

use canvas_traits::webgl::WebGLError::*;
use canvas_traits::webgl::{TexDataType, TexFormat};

use super::WebGLValidator;
use super::tex_image_2d::TexImageValidationError;
use super::types::TexImageTarget;
use crate::dom::bindings::root::DomRoot;
use crate::dom::webglrenderingcontext::WebGLRenderingContext;
use crate::dom::webgltexture::{ImageInfo, TexCompression, TexCompressionValidation, WebGLTexture};

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
    border: i32,
}

pub(crate) struct CommonTexImage3DValidatorResult {
    pub(crate) texture: DomRoot<WebGLTexture>,
    pub(crate) target: TexImageTarget,
    pub(crate) level: u32,
    pub(crate) internal_format: TexFormat,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) border: u32,
}

impl WebGLValidator for CommonTexImage3DValidator<'_> {
    type Error = TexImageValidationError;
    type ValidatedOutput = CommonTexImage3DValidatorResult;
    fn validate(self) -> Result<Self::ValidatedOutput, TexImageValidationError> {
        // GL_INVALID_ENUM is generated if target is not GL_TEXTURE_2D,
        // GL_TEXTURE_CUBE_MAP_POSITIVE_X, GL_TEXTURE_CUBE_MAP_NEGATIVE_X,
        // GL_TEXTURE_CUBE_MAP_POSITIVE_Y, GL_TEXTURE_CUBE_MAP_NEGATIVE_Y,
        // GL_TEXTURE_CUBE_MAP_POSITIVE_Z, or GL_TEXTURE_CUBE_MAP_NEGATIVE_Z.
        let target = match TexImageTarget::from_gl_constant(self.target) {
            Some(target) if target.dimensions() == 2 => target,
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

        let max_size = if target.is_cubic() {
            limits.max_cube_map_tex_size
        } else {
            limits.max_tex_size
        };

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
                if format.required_webgl_version() <= self.context.webgl_version()
                    && format.usable_as_internal() =>
            {
                format
            },
            _ => {
                self.context.webgl_error(InvalidEnum);
                return Err(TexImageValidationError::InvalidTextureFormat);
            },
        };

        // GL_INVALID_VALUE is generated if target is one of the six cube map 2D
        // image targets and the width and height parameters are not equal.
        if target.is_cubic() && self.width != self.height {
            self.context.webgl_error(InvalidValue);
            return Err(TexImageValidationError::InvalidCubicTextureDimensions);
        }

        // GL_INVALID_VALUE is generated if level is less than 0.
        if self.level < 0 {
            self.context.webgl_error(InvalidValue);
            return Err(TexImageValidationError::NegativeLevel);
        }

        // GL_INVALID_VALUE is generated if width or height is less than 0
        if self.width < 0 || self.height < 0 {
            self.context.webgl_error(InvalidValue);
            return Err(TexImageValidationError::NegativeDimension);
        }

        let width = self.width as u32;
        let height = self.height as u32;
        let level = self.level as u32;

        // GL_INVALID_VALUE is generated if width or height is greater than
        // GL_MAX_TEXTURE_SIZE when target is GL_TEXTURE_2D or
        // GL_MAX_CUBE_MAP_TEXTURE_SIZE when target is not GL_TEXTURE_2D.
        if width > max_size >> level || height > max_size >> level {
            self.context.webgl_error(InvalidValue);
            return Err(TexImageValidationError::TextureTooBig);
        }

        // GL_INVALID_VALUE is generated if level is greater than zero and the
        // texture is not power of two.
        if level > 0 && (!width.is_power_of_two() || !height.is_power_of_two()) {
            self.context.webgl_error(InvalidValue);
            return Err(TexImageValidationError::NonPotTexture);
        }

        // GL_INVALID_VALUE may be generated if level is greater than
        // log_2(max), where max is the returned value of GL_MAX_TEXTURE_SIZE
        // when target is GL_TEXTURE_2D or GL_MAX_CUBE_MAP_TEXTURE_SIZE when
        // target is not GL_TEXTURE_2D.
        if level > log2(max_size) {
            self.context.webgl_error(InvalidValue);
            return Err(TexImageValidationError::LevelTooHigh);
        }

        // GL_INVALID_VALUE is generated if border is not 0.
        if self.border != 0 {
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
            border: self.border as u32,
        })
    }
}

impl<'a> CommonTexImage3DValidator<'a> {
    pub(crate) fn new(
        context: &'a WebGLRenderingContext,
        target: u32,
        level: i32,
        internal_format: u32,
        width: i32,
        height: i32,
        border: i32,
    ) -> Self {
        CommonTexImage3DValidator {
            context,
            target,
            level,
            internal_format,
            width,
            height,
            border,
        }
    }
}

pub(crate) struct TexImage3DValidator<'a> {
    common_validator: CommonTexImage3DValidator<'a>,
    format: u32,
    data_type: u32,
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
        border: i32,
        format: u32,
        data_type: u32,
    ) -> Self {
        TexImage3DValidator {
            common_validator: CommonTexImage3DValidator::new(
                context,
                target,
                level,
                internal_format,
                width,
                height,
                border,
            ),
            format,
            data_type,
        }
    }
}

/// The validated result of a TexImage2DValidator-validated call.
pub(crate) struct TexImage3DValidatorResult {
    /// NB: width, height and level are already unsigned after validation.
    pub(crate) width: u32,
    pub(crate) height: u32,
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
            border,
        } = self.common_validator.validate()?;

        // GL_INVALID_ENUM is generated if format or data_type is not an
        // accepted value.
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

        // NOTE: In WebGL2 data type check should be done based on the internal
        // format, but in some functions this validator is called with the
        // regular unsized format as parameter (eg. TexSubImage2D). For now
        // it's left here to avoid duplication.
        //
        // GL_INVALID_OPERATION is generated if type is
        // GL_UNSIGNED_SHORT_4_4_4_4 or GL_UNSIGNED_SHORT_5_5_5_1 and format is
        // not GL_RGBA.
        //
        // GL_INVALID_OPERATION is generated if type is GL_UNSIGNED_SHORT_5_6_5
        // and format is not GL_RGB.
        match data_type {
            TexDataType::UnsignedShort4444 | TexDataType::UnsignedShort5551
                if format != TexFormat::RGBA =>
            {
                context.webgl_error(InvalidOperation);
                return Err(TexImageValidationError::InvalidTypeForFormat);
            },
            TexDataType::UnsignedShort565 if format != TexFormat::RGB => {
                context.webgl_error(InvalidOperation);
                return Err(TexImageValidationError::InvalidTypeForFormat);
            },
            _ => {},
        }

        Ok(TexImage3DValidatorResult {
            width,
            height,
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
