/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use canvas_traits::gl_enums;
use crate::dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLRenderingContextConstants;

gl_enums! {
    pub enum TexImageTarget {
        Texture2D = WebGLRenderingContextConstants::TEXTURE_2D,
        CubeMapPositiveX = WebGLRenderingContextConstants::TEXTURE_CUBE_MAP_POSITIVE_X,
        CubeMapNegativeX = WebGLRenderingContextConstants::TEXTURE_CUBE_MAP_NEGATIVE_X,
        CubeMapPositiveY = WebGLRenderingContextConstants::TEXTURE_CUBE_MAP_POSITIVE_Y,
        CubeMapNegativeY = WebGLRenderingContextConstants::TEXTURE_CUBE_MAP_NEGATIVE_Y,
        CubeMapPositiveZ = WebGLRenderingContextConstants::TEXTURE_CUBE_MAP_POSITIVE_Z,
        CubeMapNegativeZ = WebGLRenderingContextConstants::TEXTURE_CUBE_MAP_NEGATIVE_Z,
    }
}

impl TexImageTarget {
    pub fn is_cubic(&self) -> bool {
        match *self {
            TexImageTarget::Texture2D => false,
            _ => true,
        }
    }
}
