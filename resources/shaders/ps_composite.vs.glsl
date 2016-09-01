#line 1
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

void main(void) {
    Composite composite = fetch_composite(gl_InstanceID);

    vec2 local_pos = mix(vec2(composite.target_rect.xy),
                         vec2(composite.target_rect.xy + composite.target_rect.zw),
                         aPosition.xy);

    vec2 st0 = vec2(composite.src0.xy) / 2048.0;
    vec2 st1 = vec2(composite.src0.xy + composite.src0.zw) / 2048.0;
    vUv0 = mix(st0, st1, aPosition.xy);

    st0 = vec2(composite.src1.xy) / 2048.0;
    st1 = vec2(composite.src1.xy + composite.src1.zw) / 2048.0;
    vUv1 = mix(st0, st1, aPosition.xy);

    vInfo = ivec2(composite.info_amount.xy);
    vAmount = composite.info_amount.z;

    gl_Position = uTransform * vec4(local_pos, 0, 1);
}
