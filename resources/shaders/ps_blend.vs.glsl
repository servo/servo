#line 1
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

struct Blend {
    uvec4 target_rect;
    uvec4 src_rect;
    vec4 opacity;
};

layout(std140) uniform Items {
    Blend blends[WR_MAX_PRIM_ITEMS];
};

void main(void) {
    Blend blend = blends[gl_InstanceID];

    vec2 local_pos = mix(vec2(blend.target_rect.xy),
                         vec2(blend.target_rect.xy + blend.target_rect.zw),
                         aPosition.xy);

    vec2 st0 = vec2(blend.src_rect.xy) / 2048.0;
    vec2 st1 = vec2(blend.src_rect.xy + blend.src_rect.zw) / 2048.0;
    vUv = mix(st0, st1, aPosition.xy);
    vOpacity = blend.opacity.x;

    gl_Position = uTransform * vec4(local_pos, 0, 1);
}
