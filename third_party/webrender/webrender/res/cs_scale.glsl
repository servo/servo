/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include shared,prim_shared

varying vec3 vUv;
flat varying vec4 vUvRect;

#ifdef WR_VERTEX_SHADER

PER_INSTANCE in vec4 aScaleTargetRect;
PER_INSTANCE in ivec4 aScaleSourceRect;
PER_INSTANCE in int aScaleSourceLayer;

void main(void) {
    RectWithSize src_rect = RectWithSize(vec2(aScaleSourceRect.xy), vec2(aScaleSourceRect.zw));

    vec2 texture_size = vec2(textureSize(sColor0, 0).xy);
    vUv.z = float(aScaleSourceLayer);

    vUvRect = vec4(src_rect.p0 + vec2(0.5),
                   src_rect.p0 + src_rect.size - vec2(0.5)) / texture_size.xyxy;

    vec2 pos = aScaleTargetRect.xy + aScaleTargetRect.zw * aPosition.xy;
    vUv.xy = (src_rect.p0 + src_rect.size * aPosition.xy) / texture_size;

    gl_Position = uTransform * vec4(pos, 0.0, 1.0);
}

#endif

#ifdef WR_FRAGMENT_SHADER

void main(void) {
    vec2 st = clamp(vUv.xy, vUvRect.xy, vUvRect.zw);
    oFragColor = texture(sColor0, vec3(st, vUv.z));
}

#endif
