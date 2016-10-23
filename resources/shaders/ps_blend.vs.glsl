#line 1
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

void main(void) {
    Blend blend = fetch_blend(gl_InstanceID);
    Tile src = fetch_tile(blend.src_id_target_id_opacity.x);
    Tile dest = fetch_tile(blend.src_id_target_id_opacity.y);

    vec2 local_pos = mix(vec2(dest.target_rect.xy),
                         vec2(dest.target_rect.xy + dest.target_rect.zw),
                         aPosition.xy);

    vec2 st0 = vec2(src.target_rect.xy) / 2048.0;
    vec2 st1 = vec2(src.target_rect.xy + src.target_rect.zw) / 2048.0;
    vUv = mix(st0, st1, aPosition.xy);
    vBrightnessOpacity = blend.src_id_target_id_opacity.zw / 65535.0;

    gl_Position = uTransform * vec4(local_pos, 0, 1);
}
