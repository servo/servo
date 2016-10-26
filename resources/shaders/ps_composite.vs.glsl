#line 1
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

void main(void) {
    Composite composite = fetch_composite(gl_InstanceID);
    Tile src0 = fetch_tile(composite.src0_src1_target_id_op.x);
    Tile src1 = fetch_tile(composite.src0_src1_target_id_op.y);
    Tile dest = fetch_tile(composite.src0_src1_target_id_op.z);

    vec2 local_pos = mix(dest.screen_origin_task_origin.zw,
                         dest.screen_origin_task_origin.zw + dest.size_target_index.xy,
                         aPosition.xy);

    vec2 texture_size = vec2(textureSize(sCache, 0));
    vec2 st0 = src0.screen_origin_task_origin.zw / texture_size;
    vec2 st1 = (src0.screen_origin_task_origin.zw + src0.size_target_index.xy) / texture_size;
    vUv0 = vec3(mix(st0, st1, aPosition.xy), src0.size_target_index.z);

    st0 = vec2(src1.screen_origin_task_origin.zw) / texture_size;
    st1 = vec2(src1.screen_origin_task_origin.zw + src1.size_target_index.xy) / texture_size;
    vec2 local_virtual_pos = mix(dest.screen_origin_task_origin.xy,
                                 dest.screen_origin_task_origin.xy + dest.size_target_index.xy,
                                 aPosition.xy);
    vec2 f = (local_virtual_pos - src1.screen_origin_task_origin.xy) / src1.size_target_index.xy;
    vUv1 = vec3(mix(st0, st1, f), src1.size_target_index.z);
    vUv1Rect = vec4(st0, st1);

    vOp = composite.src0_src1_target_id_op.w;

    gl_Position = uTransform * vec4(local_pos, 0, 1);
}
