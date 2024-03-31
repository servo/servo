/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use canvas_traits::gl_enums;
use serde::{Deserialize, Serialize};

use crate::dom::bindings::codegen::Bindings::WebGL2RenderingContextBinding::WebGL2RenderingContextConstants as constants;

gl_enums! {
    pub enum TexImageTarget {
        Texture2D = constants::TEXTURE_2D,
        Texture2DArray = constants::TEXTURE_2D_ARRAY,
        Texture3D = constants::TEXTURE_3D,
        CubeMap = constants::TEXTURE_CUBE_MAP,
        CubeMapPositiveX = constants::TEXTURE_CUBE_MAP_POSITIVE_X,
        CubeMapNegativeX = constants::TEXTURE_CUBE_MAP_NEGATIVE_X,
        CubeMapPositiveY = constants::TEXTURE_CUBE_MAP_POSITIVE_Y,
        CubeMapNegativeY = constants::TEXTURE_CUBE_MAP_NEGATIVE_Y,
        CubeMapPositiveZ = constants::TEXTURE_CUBE_MAP_POSITIVE_Z,
        CubeMapNegativeZ = constants::TEXTURE_CUBE_MAP_NEGATIVE_Z,
    }
}

impl TexImageTarget {
    pub fn is_cubic(&self) -> bool {
        !matches!(*self, TexImageTarget::Texture2D)
    }

    pub fn dimensions(self) -> u8 {
        match self {
            TexImageTarget::Texture3D | TexImageTarget::Texture2DArray => 3,
            _ => 2,
        }
    }
}
