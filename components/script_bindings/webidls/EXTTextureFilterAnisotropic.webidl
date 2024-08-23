/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * WebGL IDL definitions from the Khronos specification:
 * https://www.khronos.org/registry/webgl/extensions/EXT_texture_filter_anisotropic/
 */

[LegacyNoInterfaceObject, Exposed=Window]
interface EXTTextureFilterAnisotropic {
  const GLenum TEXTURE_MAX_ANISOTROPY_EXT       = 0x84FE;
  const GLenum MAX_TEXTURE_MAX_ANISOTROPY_EXT   = 0x84FF;
};
