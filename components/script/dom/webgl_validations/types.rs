/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::OESTextureHalfFloatBinding::OESTextureHalfFloatConstants;
use dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLRenderingContextConstants as constants;

/// This macro creates type-safe wrappers for WebGL types, associating variants
/// with gl constants.
macro_rules! type_safe_wrapper {
    ($name: ident, $($variant:ident => $mod:ident::$constant:ident, )+) => {
        #[derive(Clone, Copy, Debug, Eq, Hash, HeapSizeOf, JSTraceable, PartialEq)]
        #[repr(u32)]
        pub enum $name {
            $(
                $variant = $mod::$constant,
            )+
        }

        impl $name {
            pub fn from_gl_constant(constant: u32) -> Option<Self> {
                Some(match constant {
                    $($mod::$constant => $name::$variant, )+
                    _ => return None,
                })
            }

            #[inline]
            pub fn as_gl_constant(&self) -> u32 {
                *self as u32
            }
        }
    }
}

type_safe_wrapper! { TexImageTarget,
    Texture2D => constants::TEXTURE_2D,
    CubeMapPositiveX => constants::TEXTURE_CUBE_MAP_POSITIVE_X,
    CubeMapNegativeX => constants::TEXTURE_CUBE_MAP_NEGATIVE_X,
    CubeMapPositiveY => constants::TEXTURE_CUBE_MAP_POSITIVE_Y,
    CubeMapNegativeY => constants::TEXTURE_CUBE_MAP_NEGATIVE_Y,
    CubeMapPositiveZ => constants::TEXTURE_CUBE_MAP_POSITIVE_Z,
    CubeMapNegativeZ => constants::TEXTURE_CUBE_MAP_NEGATIVE_Z,
}

impl TexImageTarget {
    pub fn is_cubic(&self) -> bool {
        match *self {
            TexImageTarget::Texture2D => false,
            _ => true,
        }
    }
}

type_safe_wrapper! { TexDataType,
    UnsignedByte => constants::UNSIGNED_BYTE,
    UnsignedShort4444 => constants::UNSIGNED_SHORT_4_4_4_4,
    UnsignedShort5551 => constants::UNSIGNED_SHORT_5_5_5_1,
    UnsignedShort565 => constants::UNSIGNED_SHORT_5_6_5,
    Float => constants::FLOAT,
    HalfFloat => OESTextureHalfFloatConstants::HALF_FLOAT_OES,
}

impl TexDataType {
    /// Returns the size in bytes of each element of data.
    pub fn element_size(&self) -> u32 {
        use self::TexDataType::*;
        match *self {
            UnsignedByte => 1,
            UnsignedShort4444 |
            UnsignedShort5551 |
            UnsignedShort565 => 2,
            Float => 4,
            HalfFloat => 2,
        }
    }

    /// Returns how many components a single element may hold. For example, a
    /// UnsignedShort4444 holds four components, each with 4 bits of data.
    pub fn components_per_element(&self) -> u32 {
        use self::TexDataType::*;
        match *self {
            UnsignedByte => 1,
            UnsignedShort565 => 3,
            UnsignedShort5551 => 4,
            UnsignedShort4444 => 4,
            Float => 1,
            HalfFloat => 1,
        }
    }
}

type_safe_wrapper! { TexFormat,
    DepthComponent => constants::DEPTH_COMPONENT,
    Alpha => constants::ALPHA,
    RGB => constants::RGB,
    RGBA => constants::RGBA,
    Luminance => constants::LUMINANCE,
    LuminanceAlpha => constants::LUMINANCE_ALPHA,
}

impl TexFormat {
    /// Returns how many components does this format need. For example, RGBA
    /// needs 4 components, while RGB requires 3.
    pub fn components(&self) -> u32 {
        use self::TexFormat::*;
        match *self {
            DepthComponent => 1,
            Alpha => 1,
            Luminance => 1,
            LuminanceAlpha => 2,
            RGB => 3,
            RGBA => 4,
        }
    }
}
