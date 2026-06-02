/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::{WebGLExtension, WebGLExtensionSpec, WebGLExtensions};
use crate::dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLRenderingContextConstants as constants;

pub(crate) mod angleinstancedarrays;
pub(crate) mod extblendminmax;
pub(crate) mod extcolorbufferhalffloat;
pub(crate) mod extfragdepth;
pub(crate) mod extshadertexturelod;
pub(crate) mod exttexturefilteranisotropic;
pub(crate) mod oeselementindexuint;
pub(crate) mod oesstandardderivatives;
pub(crate) mod oestexturefloat;
pub(crate) mod oestexturefloatlinear;
pub(crate) mod oestexturehalffloat;
pub(crate) mod oestexturehalffloatlinear;
pub(crate) mod oesvertexarrayobject;
pub(crate) mod webglcolorbufferfloat;
pub(crate) mod webglcompressedtextureetc1;
pub(crate) mod webglcompressedtextures3tc;
