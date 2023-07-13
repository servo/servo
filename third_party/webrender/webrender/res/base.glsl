/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#if defined(GL_ES)
    #if GL_ES == 1
        // Sampler default precision is lowp on mobile GPUs.
        // This causes RGBA32F texture data to be clamped to 16 bit floats on some GPUs (e.g. Mali-T880).
        // Define highp precision macro to allow lossless FLOAT texture sampling.
        #define HIGHP_SAMPLER_FLOAT highp

        // Default int precision in GLES 3 is highp (32 bits) in vertex shaders
        // and mediump (16 bits) in fragment shaders. If an int is being used as
        // a texel address in a fragment shader it, and therefore requires > 16
        // bits, it must be qualified with this.
        #define HIGHP_FS_ADDRESS highp

        // texelFetchOffset is buggy on some Android GPUs (see issue #1694).
        // Fallback to texelFetch on mobile GPUs.
        #define TEXEL_FETCH(sampler, position, lod, offset) texelFetch(sampler, position + offset, lod)
    #else
        #define HIGHP_SAMPLER_FLOAT
        #define HIGHP_FS_ADDRESS
        #define TEXEL_FETCH(sampler, position, lod, offset) texelFetchOffset(sampler, position, lod, offset)
    #endif
#else
    #define HIGHP_SAMPLER_FLOAT
    #define HIGHP_FS_ADDRESS
    #if defined(PLATFORM_MACOS) && !defined(SWGL)
        // texelFetchOffset introduces a variety of shader compilation bugs on macOS Intel so avoid it.
        #define TEXEL_FETCH(sampler, position, lod, offset) texelFetch(sampler, position + offset, lod)
    #else
        #define TEXEL_FETCH(sampler, position, lod, offset) texelFetchOffset(sampler, position, lod, offset)
    #endif
#endif

#ifdef SWGL
    #define SWGL_DRAW_SPAN
    #define SWGL_CLIP_MASK
    #define SWGL_ANTIALIAS
    #define SWGL_BLEND
    #define SWGL_CLIP_DIST
#endif

#ifdef WR_VERTEX_SHADER
    #ifdef SWGL
        // Annotate a vertex attribute as being flat per each drawn primitive instance.
        // SWGL can use this information to avoid redundantly loading the attribute in all SIMD lanes.
        #define PER_INSTANCE flat
    #else
        #define PER_INSTANCE
    #endif

    #if __VERSION__ != 100
        #define varying out
        #define attribute in
    #endif
#endif

#ifdef WR_FRAGMENT_SHADER
    precision highp float;
    #if __VERSION__ != 100
        #define varying in
    #endif
#endif

// Flat interpolation is not supported on ESSL 1
#if __VERSION__ == 100
    #define flat
#endif
