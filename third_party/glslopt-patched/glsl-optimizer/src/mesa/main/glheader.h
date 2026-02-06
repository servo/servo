/*
 * Mesa 3-D graphics library
 *
 * Copyright (C) 1999-2008  Brian Paul   All Rights Reserved.
 *
 * Permission is hereby granted, free of charge, to any person obtaining a
 * copy of this software and associated documentation files (the "Software"),
 * to deal in the Software without restriction, including without limitation
 * the rights to use, copy, modify, merge, publish, distribute, sublicense,
 * and/or sell copies of the Software, and to permit persons to whom the
 * Software is furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included
 * in all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
 * OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.  IN NO EVENT SHALL
 * THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR
 * OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE,
 * ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
 * OTHER DEALINGS IN THE SOFTWARE.
 */


/**
 * \file glheader.h
 * Wrapper for GL/gl.h and GL/glext.h
 */


#ifndef GLHEADER_H
#define GLHEADER_H


#define GL_GLEXT_PROTOTYPES
#include "GL/gl.h"
#include "GL/glext.h"


#ifdef __cplusplus
extern "C" {
#endif


/* Custom Mesa types to save space. */
typedef unsigned short GLenum16;
typedef unsigned char GLbitfield8;
typedef unsigned short GLbitfield16;
typedef GLuint64 GLbitfield64;

/* Common GLES 1.0 and 2.0 tokens */

#ifndef GL_OES_EGL_image_external
#define GL_TEXTURE_EXTERNAL_OES                                 0x8D65
#define GL_SAMPLER_EXTERNAL_OES                                 0x8D66
#define GL_TEXTURE_BINDING_EXTERNAL_OES                         0x8D67
#define GL_REQUIRED_TEXTURE_IMAGE_UNITS_OES                     0x8D68
#endif

#ifndef GL_OES_compressed_ETC1_RGB8_texture
#define GL_ETC1_RGB8_OES                                        0x8D64
#endif


/* GLES 1.0 only tokens */

typedef int GLclampx;

#ifndef GL_OES_point_size_array
#define GL_POINT_SIZE_ARRAY_OES                                 0x8B9C
#define GL_POINT_SIZE_ARRAY_TYPE_OES                            0x898A
#define GL_POINT_SIZE_ARRAY_STRIDE_OES                          0x898B
#define GL_POINT_SIZE_ARRAY_POINTER_OES                         0x898C
#define GL_POINT_SIZE_ARRAY_BUFFER_BINDING_OES                  0x8B9F
#endif


#ifndef GL_OES_draw_texture
#define GL_TEXTURE_CROP_RECT_OES                                0x8B9D
#endif

#ifndef GL_TEXTURE_GEN_STR_OES
#define GL_TEXTURE_GEN_STR_OES                                  0x8D60
#endif


/* GLES 2.0 only tokens */

#ifndef GL_PROGRAM_BINARY_LENGTH_OES
#define GL_PROGRAM_BINARY_LENGTH_OES                            0x8741
#endif

#ifndef GL_OES_texture_compression_astc
#define GL_COMPRESSED_RGBA_ASTC_3x3x3_OES                       0x93C0
#define GL_COMPRESSED_RGBA_ASTC_4x3x3_OES                       0x93C1
#define GL_COMPRESSED_RGBA_ASTC_4x4x3_OES                       0x93C2
#define GL_COMPRESSED_RGBA_ASTC_4x4x4_OES                       0x93C3
#define GL_COMPRESSED_RGBA_ASTC_5x4x4_OES                       0x93C4
#define GL_COMPRESSED_RGBA_ASTC_5x5x4_OES                       0x93C5
#define GL_COMPRESSED_RGBA_ASTC_5x5x5_OES                       0x93C6
#define GL_COMPRESSED_RGBA_ASTC_6x5x5_OES                       0x93C7
#define GL_COMPRESSED_RGBA_ASTC_6x6x5_OES                       0x93C8
#define GL_COMPRESSED_RGBA_ASTC_6x6x6_OES                       0x93C9
#define GL_COMPRESSED_SRGB8_ALPHA8_ASTC_3x3x3_OES               0x93E0
#define GL_COMPRESSED_SRGB8_ALPHA8_ASTC_4x3x3_OES               0x93E1
#define GL_COMPRESSED_SRGB8_ALPHA8_ASTC_4x4x3_OES               0x93E2
#define GL_COMPRESSED_SRGB8_ALPHA8_ASTC_4x4x4_OES               0x93E3
#define GL_COMPRESSED_SRGB8_ALPHA8_ASTC_5x4x4_OES               0x93E4
#define GL_COMPRESSED_SRGB8_ALPHA8_ASTC_5x5x4_OES               0x93E5
#define GL_COMPRESSED_SRGB8_ALPHA8_ASTC_5x5x5_OES               0x93E6
#define GL_COMPRESSED_SRGB8_ALPHA8_ASTC_6x5x5_OES               0x93E7
#define GL_COMPRESSED_SRGB8_ALPHA8_ASTC_6x6x5_OES               0x93E8
#define GL_COMPRESSED_SRGB8_ALPHA8_ASTC_6x6x6_OES               0x93E9
#endif

#ifndef GL_EXT_shader_framebuffer_fetch
#define GL_FRAGMENT_SHADER_DISCARDS_SAMPLES_EXT                 0x8A52
#endif

#ifndef GL_EXT_disjoint_timer_query
#define GL_GPU_DISJOINT_EXT                                     0x8FBB
#endif

/* Inexplicably, GL_HALF_FLOAT_OES has a different value than GL_HALF_FLOAT.
 */
#ifndef GL_HALF_FLOAT_OES
#define GL_HALF_FLOAT_OES                                       0x8D61
#endif

/* There is no formal spec for the following extension. */
#ifndef GL_ATI_texture_compression_3dc
#define GL_ATI_texture_compression_3dc                          1
#define GL_COMPRESSED_LUMINANCE_ALPHA_3DC_ATI                   0x8837
#endif

#ifndef GL_EXT_texture_sRGB_R8
#define GL_SR8_EXT                                              0x8FBD
#endif

#ifndef GL_AMD_compressed_ATC_texture
#define GL_ATC_RGB_AMD                                          0x8C92
#define GL_ATC_RGBA_EXPLICIT_ALPHA_AMD                          0x8C93
#define GL_ATC_RGBA_INTERPOLATED_ALPHA_AMD                      0x87EE
#endif

/**
 * Internal token to represent a GLSL shader program (a collection of
 * one or more shaders that get linked together).  Note that GLSL
 * shaders and shader programs share one name space (one hash table)
 * so we need a value that's different from any of the
 * GL_VERTEX/FRAGMENT/GEOMETRY_PROGRAM tokens.
 */
#define GL_SHADER_PROGRAM_MESA                                  0x9999

#ifndef GL_EXT_multisampled_render_to_texture
#define GL_FRAMEBUFFER_ATTACHMENT_TEXTURE_SAMPLES_EXT 0x8D6C
#endif

#ifdef __cplusplus
}
#endif

#endif /* GLHEADER_H */
