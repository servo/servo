#line 1
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

void main(void) {
    Composite composite = fetch_composite(gl_InstanceID);
    Tile src0 = fetch_tile(int(composite.src0_src1_target_id.x));
    Tile src1 = fetch_tile(int(composite.src0_src1_target_id.y));
    Tile dest = fetch_tile(int(composite.src0_src1_target_id.z));

    vec2 local_pos = mix(vec2(dest.target_rect.xy),
                         vec2(dest.target_rect.xy + dest.target_rect.zw),
                         aPosition.xy);

    vec2 st0 = vec2(src0.target_rect.xy) / 2048.0;
    vec2 st1 = vec2(src0.target_rect.xy + src0.target_rect.zw) / 2048.0;
    vUv0 = mix(st0, st1, aPosition.xy);

    st0 = vec2(src1.target_rect.xy) / 2048.0;
    st1 = vec2(src1.target_rect.xy + src1.target_rect.zw) / 2048.0;
    vUv1 = mix(st0, st1, aPosition.xy);

    vInfo = ivec2(composite.info_amount.xy);
    vAmount = composite.info_amount.z;

    gl_Position = uTransform * vec4(local_pos, 0, 1);
}
