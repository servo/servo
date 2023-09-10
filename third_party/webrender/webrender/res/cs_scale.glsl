/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// This shader must remain compatible with ESSL 1, at least for the
// WR_FEATURE_TEXTURE_EXTERNAL_ESSL1 feature, so that it can be used to render
// video on GLES devices without GL_OES_EGL_image_external_essl3 support.
// This means we cannot use textureSize(), int inputs/outputs, etc.

#include shared

varying vec2 vUv;
flat varying vec4 vUvRect;
#ifdef WR_FEATURE_TEXTURE_EXTERNAL_ESSL1
uniform vec2 uTextureSize;
#endif

#ifdef WR_VERTEX_SHADER

PER_INSTANCE attribute vec4 aScaleTargetRect;
PER_INSTANCE attribute vec4 aScaleSourceRect;

void main(void) {
    vec2 src_offset = aScaleSourceRect.xy;
    vec2 src_size = aScaleSourceRect.zw;

    // If this is in WR_FEATURE_TEXTURE_RECT mode, the rect and size use
    // non-normalized texture coordinates.
#ifdef WR_FEATURE_TEXTURE_RECT
    vec2 texture_size = vec2(1, 1);
#elif defined(WR_FEATURE_TEXTURE_EXTERNAL_ESSL1)
    vec2 texture_size = uTextureSize;
#else
    vec2 texture_size = vec2(TEX_SIZE(sColor0));
#endif

    vUvRect = vec4(src_offset + vec2(0.5),
                   src_offset + src_size - vec2(0.5)) / texture_size.xyxy;

    vec2 pos = aScaleTargetRect.xy + aScaleTargetRect.zw * aPosition.xy;
    vUv = (src_offset + src_size * aPosition.xy) / texture_size;

    gl_Position = uTransform * vec4(pos, 0.0, 1.0);
}

#endif

#ifdef WR_FRAGMENT_SHADER

void main(void) {
    vec2 st = clamp(vUv, vUvRect.xy, vUvRect.zw);
    oFragColor = TEX_SAMPLE(sColor0, st);
}

#ifdef SWGL_DRAW_SPAN
void swgl_drawSpanRGBA8() {
    swgl_commitTextureLinearRGBA8(sColor0, vUv, vUvRect);
}
#endif

#endif
