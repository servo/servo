#line 1
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

void main(void) {
    CachePrimitiveInstance cpi = fetch_cache_instance(gl_InstanceID);
    RenderTaskData task = fetch_render_task(cpi.render_task_index);
    BoxShadow bs = fetch_boxshadow(cpi.specific_prim_index);

    vec2 p0 = task.data0.xy;
    vec2 p1 = p0 + task.data0.zw;

    vec2 pos = mix(p0, p1, aPosition.xy);

    vBorderRadii = bs.border_radius_edge_size_blur_radius_inverted.xx;
    vBlurRadius = bs.border_radius_edge_size_blur_radius_inverted.z;
    vInverted = bs.border_radius_edge_size_blur_radius_inverted.w;
    vBoxShadowRect = vec4(bs.bs_rect.xy, bs.bs_rect.xy + bs.bs_rect.zw);

    // The fragment shader expects logical units, beginning at where the
    // blur radius begins.
    // The first path of the equation gets the virtual position in
    // logical pixels within the patch rectangle (accounting for
    // bilinear offset). Then we add the start position of the
    // box shadow rect and subtract the blur radius to get the
    // virtual coordinates that the FS expects.
    vPos = (pos - 1.0 - p0) / uDevicePixelRatio + bs.bs_rect.xy - vec2(2.0 * vBlurRadius);

    gl_Position = uTransform * vec4(pos, 0.0, 1.0);
}
