/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLRenderingContextConstants as constants;

/// This macro creates type-safe wrappers for WebGL types, associating variants
/// with gl constants.
macro_rules! type_safe_wrapper {
    ($name: ident, $($variant:ident => $constant:ident, )+) => {
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, JSTraceable, HeapSizeOf)]
        #[repr(u32)]
        pub enum $name {
            $(
                $variant = constants::$constant,
            )+
        }

        impl $name {
            pub fn from_gl_constant(constant: u32) -> Option<Self> {
                Some(match constant {
                    $(constants::$constant => $name::$variant, )+
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
    Texture2D => TEXTURE_2D,
    CubeMapPositiveX => TEXTURE_CUBE_MAP_POSITIVE_X,
    CubeMapNegativeX => TEXTURE_CUBE_MAP_NEGATIVE_X,
    CubeMapPositiveY => TEXTURE_CUBE_MAP_POSITIVE_Y,
    CubeMapNegativeY => TEXTURE_CUBE_MAP_NEGATIVE_Y,
    CubeMapPositiveZ => TEXTURE_CUBE_MAP_POSITIVE_Z,
    CubeMapNegativeZ => TEXTURE_CUBE_MAP_NEGATIVE_Z,
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
    UnsignedByte => UNSIGNED_BYTE,
    UnsignedShort4444 => UNSIGNED_SHORT_4_4_4_4,
    UnsignedShort5551 => UNSIGNED_SHORT_5_5_5_1,
    UnsignedShort565 => UNSIGNED_SHORT_5_6_5,
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
        }
    }
}

type_safe_wrapper! { TexFormat,
    DepthComponent => DEPTH_COMPONENT,
    Alpha => ALPHA,
    RGB => RGB,
    RGBA => RGBA,
    Luminance => LUMINANCE,
    LuminanceAlpha => LUMINANCE_ALPHA,
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
