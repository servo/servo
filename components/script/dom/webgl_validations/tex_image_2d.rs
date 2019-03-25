/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::types::TexImageTarget;
use super::WebGLValidator;
use crate::dom::bindings::root::DomRoot;
use crate::dom::webglrenderingcontext::WebGLRenderingContext;
use crate::dom::webgltexture::{ImageInfo, WebGLTexture};
use crate::dom::webgltexture::{TexCompression, TexCompressionValidation};
use canvas_traits::webgl::{TexDataType, TexFormat, WebGLError::*};
use std::{self, fmt};

/// The errors that the texImage* family of functions can generate.
#[derive(Debug)]
pub enum TexImageValidationError {
    /// An invalid texture target was passed, it contains the invalid target.
    InvalidTextureTarget(u32),
    /// The passed texture target was not bound.
    TextureTargetNotBound(u32),
    /// Invalid texture dimensions were given.
    InvalidCubicTextureDimensions,
    /// A negative level was passed.
    NegativeLevel,
    /// A level too high to be allowed by the implementation was passed.
    LevelTooHigh,
    /// A negative width and height was passed.
    NegativeDimension,
    /// A bigger with and height were passed than what the implementation
    /// allows.
    TextureTooBig,
    /// An invalid data type was passed.
    InvalidDataType,
    /// An invalid texture format was passed.
    InvalidTextureFormat,
    /// Format did not match internal_format.
    TextureFormatMismatch,
    /// Invalid data type for the given format.
    InvalidTypeForFormat,
    /// Invalid border
    InvalidBorder,
    /// Expected a power of two texture.
    NonPotTexture,
    /// Unrecognized texture compression format.
    InvalidCompressionFormat,
    /// Invalid X/Y texture offset parameters.
    InvalidOffsets,
}

impl std::error::Error for TexImageValidationError {
    fn description(&self) -> &str {
        use self::TexImageValidationError::*;
        match *self {
            InvalidTextureTarget(_) => "Invalid texture target",
            TextureTargetNotBound(_) => "Texture was not bound",
            InvalidCubicTextureDimensions => {
                "Invalid dimensions were given for a cubic texture target"
            },
            NegativeLevel => "A negative level was passed",
            LevelTooHigh => "Level too high",
            NegativeDimension => "Negative dimensions were passed",
            TextureTooBig => "Dimensions given are too big",
            InvalidDataType => "Invalid data type",
            InvalidTextureFormat => "Invalid texture format",
            TextureFormatMismatch => "Texture format mismatch",
            InvalidTypeForFormat => "Invalid type for the given format",
            InvalidBorder => "Invalid border",
            NonPotTexture => "Expected a power of two texture",
            InvalidCompressionFormat => "Unrecognized texture compression format",
            InvalidOffsets => "Invalid X/Y texture offset parameters",
        }
    }
}

impl fmt::Display for TexImageValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "TexImageValidationError({})",
            std::error::Error::description(self)
        )
    }
}

fn log2(n: u32) -> u32 {
    31 - n.leading_zeros()
}

pub struct CommonTexImage2DValidator<'a> {
    context: &'a WebGLRenderingContext,
    target: u32,
    level: i32,
    internal_format: u32,
    width: i32,
    height: i32,
    border: i32,
}

pub struct CommonTexImage2DValidatorResult {
    pub texture: DomRoot<WebGLTexture>,
    pub target: TexImageTarget,
    pub level: u32,
    pub internal_format: TexFormat,
    pub width: u32,
    pub height: u32,
    pub border: u32,
}

impl<'a> WebGLValidator for CommonTexImage2DValidator<'a> {
    type Error = TexImageValidationError;
    type ValidatedOutput = CommonTexImage2DValidatorResult;
    fn validate(self) -> Result<Self::ValidatedOutput, TexImageValidationError> {
        // GL_INVALID_ENUM is generated if target is not GL_TEXTURE_2D,
        // GL_TEXTURE_CUBE_MAP_POSITIVE_X, GL_TEXTURE_CUBE_MAP_NEGATIVE_X,
        // GL_TEXTURE_CUBE_MAP_POSITIVE_Y, GL_TEXTURE_CUBE_MAP_NEGATIVE_Y,
        // GL_TEXTURE_CUBE_MAP_POSITIVE_Z, or GL_TEXTURE_CUBE_MAP_NEGATIVE_Z.
        let target = match TexImageTarget::from_gl_constant(self.target) {
            Some(target) => target,
            None => {
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
            Some(format) => format,
            None => {
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

        Ok(CommonTexImage2DValidatorResult {
            texture: texture,
            target: target,
            level: level,
            internal_format: internal_format,
            width: width,
            height: height,
            border: self.border as u32,
        })
    }
}

impl<'a> CommonTexImage2DValidator<'a> {
    pub fn new(
        context: &'a WebGLRenderingContext,
        target: u32,
        level: i32,
        internal_format: u32,
        width: i32,
        height: i32,
        border: i32,
    ) -> Self {
        CommonTexImage2DValidator {
            context: context,
            target: target,
            level: level,
            internal_format: internal_format,
            width: width,
            height: height,
            border: border,
        }
    }
}

pub struct TexImage2DValidator<'a> {
    common_validator: CommonTexImage2DValidator<'a>,
    format: u32,
    data_type: u32,
}

impl<'a> TexImage2DValidator<'a> {
    // TODO: Move data validation logic here.
    pub fn new(
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
        TexImage2DValidator {
            common_validator: CommonTexImage2DValidator::new(
                context,
                target,
                level,
                internal_format,
                width,
                height,
                border,
            ),
            format: format,
            data_type: data_type,
        }
    }
}

/// The validated result of a TexImage2DValidator-validated call.
pub struct TexImage2DValidatorResult {
    /// NB: width, height and level are already unsigned after validation.
    pub width: u32,
    pub height: u32,
    pub level: u32,
    pub border: u32,
    pub texture: DomRoot<WebGLTexture>,
    pub target: TexImageTarget,
    pub format: TexFormat,
    pub data_type: TexDataType,
}

/// TexImage2d validator as per
/// <https://www.khronos.org/opengles/sdk/docs/man/xhtml/glTexImage2D.xml>
impl<'a> WebGLValidator for TexImage2DValidator<'a> {
    type ValidatedOutput = TexImage2DValidatorResult;
    type Error = TexImageValidationError;

    fn validate(self) -> Result<Self::ValidatedOutput, TexImageValidationError> {
        let context = self.common_validator.context;
        let CommonTexImage2DValidatorResult {
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
            Some(data_type) => data_type,
            None => {
                context.webgl_error(InvalidEnum);
                return Err(TexImageValidationError::InvalidDataType);
            },
        };

        let format = match TexFormat::from_gl_constant(self.format) {
            Some(format) => format,
            None => {
                context.webgl_error(InvalidEnum);
                return Err(TexImageValidationError::InvalidTextureFormat);
            },
        };

        // GL_INVALID_OPERATION is generated if format does not match
        // internal_format.
        if format != internal_format {
            context.webgl_error(InvalidOperation);
            return Err(TexImageValidationError::TextureFormatMismatch);
        }

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
            }
            TexDataType::UnsignedShort565 if format != TexFormat::RGB => {
                context.webgl_error(InvalidOperation);
                return Err(TexImageValidationError::InvalidTypeForFormat);
            },
            _ => {},
        }

        Ok(TexImage2DValidatorResult {
            width: width,
            height: height,
            level: level,
            border: border,
            texture: texture,
            target: target,
            format: format,
            data_type: data_type,
        })
    }
}

pub struct CommonCompressedTexImage2DValidator<'a> {
    common_validator: CommonTexImage2DValidator<'a>,
    compression_format: u32,
    data_len: usize,
}

impl<'a> CommonCompressedTexImage2DValidator<'a> {
    pub fn new(
        context: &'a WebGLRenderingContext,
        target: u32,
        level: i32,
        width: i32,
        height: i32,
        border: i32,
        compression_format: u32,
        data_len: usize,
    ) -> Self {
        CommonCompressedTexImage2DValidator {
            common_validator: CommonTexImage2DValidator::new(
                context,
                target,
                level,
                TexFormat::RGBA.as_gl_constant(),
                width,
                height,
                border,
            ),
            compression_format,
            data_len,
        }
    }
}

pub struct CommonCompressedTexImage2DValidatorResult {
    pub texture: DomRoot<WebGLTexture>,
    pub target: TexImageTarget,
    pub level: u32,
    pub width: u32,
    pub height: u32,
    pub compression: TexCompression,
}

fn valid_s3tc_dimension(level: u32, side_length: u32, block_size: u32) -> bool {
    (side_length % block_size == 0) || (level > 0 && [0, 1, 2].contains(&side_length))
}

fn valid_compressed_data_len(
    data_len: usize,
    width: u32,
    height: u32,
    compression: &TexCompression,
) -> bool {
    let block_width = compression.block_width as u32;
    let block_height = compression.block_height as u32;

    let required_blocks_hor = (width + block_width - 1) / block_width;
    let required_blocks_ver = (height + block_height - 1) / block_height;
    let required_blocks = required_blocks_hor * required_blocks_ver;

    let required_bytes = required_blocks * compression.bytes_per_block as u32;
    data_len == required_bytes as usize
}

fn is_subimage_blockaligned(
    xoffset: u32,
    yoffset: u32,
    width: u32,
    height: u32,
    compression: &TexCompression,
    tex_info: &ImageInfo,
) -> bool {
    let block_width = compression.block_width as u32;
    let block_height = compression.block_height as u32;

    (xoffset % block_width == 0 && yoffset % block_height == 0) &&
        (width % block_width == 0 || height % block_height == 0) &&
        (height % block_height == 0 || yoffset + height == tex_info.height())
}

impl<'a> WebGLValidator for CommonCompressedTexImage2DValidator<'a> {
    type Error = TexImageValidationError;
    type ValidatedOutput = CommonCompressedTexImage2DValidatorResult;

    fn validate(self) -> Result<Self::ValidatedOutput, TexImageValidationError> {
        let context = self.common_validator.context;
        let CommonTexImage2DValidatorResult {
            texture,
            target,
            level,
            internal_format: _,
            width,
            height,
            border: _,
        } = self.common_validator.validate()?;

        // GL_INVALID_ENUM is generated if internalformat is not a supported
        // format returned in GL_COMPRESSED_TEXTURE_FORMATS.
        let compression = context
            .extension_manager()
            .get_tex_compression_format(self.compression_format);
        let compression = match compression {
            Some(compression) => compression,
            None => {
                context.webgl_error(InvalidEnum);
                return Err(TexImageValidationError::InvalidCompressionFormat);
            },
        };

        // GL_INVALID_VALUE is generated if imageSize is not consistent with the
        // format, dimensions, and contents of the specified compressed image data.
        if !valid_compressed_data_len(self.data_len, width, height, &compression) {
            context.webgl_error(InvalidValue);
            return Err(TexImageValidationError::TextureFormatMismatch);
        }

        Ok(CommonCompressedTexImage2DValidatorResult {
            texture,
            target,
            level,
            width,
            height,
            compression,
        })
    }
}

pub struct CompressedTexImage2DValidator<'a> {
    compression_validator: CommonCompressedTexImage2DValidator<'a>,
}

impl<'a> CompressedTexImage2DValidator<'a> {
    pub fn new(
        context: &'a WebGLRenderingContext,
        target: u32,
        level: i32,
        width: i32,
        height: i32,
        border: i32,
        compression_format: u32,
        data_len: usize,
    ) -> Self {
        CompressedTexImage2DValidator {
            compression_validator: CommonCompressedTexImage2DValidator::new(
                context,
                target,
                level,
                width,
                height,
                border,
                compression_format,
                data_len,
            ),
        }
    }
}

impl<'a> WebGLValidator for CompressedTexImage2DValidator<'a> {
    type Error = TexImageValidationError;
    type ValidatedOutput = CommonCompressedTexImage2DValidatorResult;

    fn validate(self) -> Result<Self::ValidatedOutput, TexImageValidationError> {
        let context = self.compression_validator.common_validator.context;
        let CommonCompressedTexImage2DValidatorResult {
            texture,
            target,
            level,
            width,
            height,
            compression,
        } = self.compression_validator.validate()?;

        // GL_INVALID_OPERATION is generated if parameter combinations are not
        // supported by the specific compressed internal format as specified
        // in the specific texture compression extension.
        let compression_valid = match compression.validation {
            TexCompressionValidation::S3TC => {
                let valid_width =
                    valid_s3tc_dimension(level, width, compression.block_width as u32);
                let valid_height =
                    valid_s3tc_dimension(level, height, compression.block_height as u32);
                valid_width && valid_height
            },
            TexCompressionValidation::None => true,
        };
        if !compression_valid {
            context.webgl_error(InvalidOperation);
            return Err(TexImageValidationError::TextureFormatMismatch);
        }

        Ok(CommonCompressedTexImage2DValidatorResult {
            texture,
            target,
            level,
            width,
            height,
            compression,
        })
    }
}

pub struct CompressedTexSubImage2DValidator<'a> {
    compression_validator: CommonCompressedTexImage2DValidator<'a>,
    xoffset: i32,
    yoffset: i32,
}

impl<'a> CompressedTexSubImage2DValidator<'a> {
    pub fn new(
        context: &'a WebGLRenderingContext,
        target: u32,
        level: i32,
        xoffset: i32,
        yoffset: i32,
        width: i32,
        height: i32,
        compression_format: u32,
        data_len: usize,
    ) -> Self {
        CompressedTexSubImage2DValidator {
            compression_validator: CommonCompressedTexImage2DValidator::new(
                context,
                target,
                level,
                width,
                height,
                0,
                compression_format,
                data_len,
            ),
            xoffset,
            yoffset,
        }
    }
}

impl<'a> WebGLValidator for CompressedTexSubImage2DValidator<'a> {
    type Error = TexImageValidationError;
    type ValidatedOutput = CommonCompressedTexImage2DValidatorResult;

    fn validate(self) -> Result<Self::ValidatedOutput, TexImageValidationError> {
        let context = self.compression_validator.common_validator.context;
        let CommonCompressedTexImage2DValidatorResult {
            texture,
            target,
            level,
            width,
            height,
            compression,
        } = self.compression_validator.validate()?;

        // FIXME: tex_info.width() and .height() returns 0
        let tex_info = texture.image_info_for_target(&target, level);

        // GL_INVALID_VALUE is generated if:
        //   - xoffset or yoffset is less than 0
        //   - x offset plus the width is greater than the texture width
        //   - y offset plus the height is greater than the texture height
        if self.xoffset < 0 ||
            (self.xoffset as u32 + width) > tex_info.width() ||
            self.yoffset < 0 ||
            (self.yoffset as u32 + height) > tex_info.height()
        {
            context.webgl_error(InvalidValue);
            return Err(TexImageValidationError::InvalidOffsets);
        }

        // GL_INVALID_OPERATION is generated if parameter combinations are not
        // supported by the specific compressed internal format as specified
        // in the specific texture compression extension.
        let compression_valid = match compression.validation {
            TexCompressionValidation::S3TC => is_subimage_blockaligned(
                self.xoffset as u32,
                self.yoffset as u32,
                width,
                height,
                &compression,
                &tex_info,
            ),
            TexCompressionValidation::None => true,
        };
        if !compression_valid {
            context.webgl_error(InvalidOperation);
            return Err(TexImageValidationError::TextureFormatMismatch);
        }

        Ok(CommonCompressedTexImage2DValidatorResult {
            texture,
            target,
            level,
            width,
            height,
            compression,
        })
    }
}
