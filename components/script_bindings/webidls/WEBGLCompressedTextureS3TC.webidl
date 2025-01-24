/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * WebGL IDL definitions from the Khronos specification:
 * https://www.khronos.org/registry/webgl/extensions/WEBGL_compressed_texture_s3tc/
 */

[LegacyNoInterfaceObject, Exposed=Window]
interface WEBGLCompressedTextureS3TC {
    /* Compressed Texture Formats */
    const GLenum COMPRESSED_RGB_S3TC_DXT1_EXT  = 0x83F0;
    const GLenum COMPRESSED_RGBA_S3TC_DXT1_EXT = 0x83F1;
    const GLenum COMPRESSED_RGBA_S3TC_DXT3_EXT = 0x83F2;
    const GLenum COMPRESSED_RGBA_S3TC_DXT5_EXT = 0x83F3;
}; // interface WEBGLCompressedTextureS3TC
