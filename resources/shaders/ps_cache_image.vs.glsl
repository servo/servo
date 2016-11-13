#line 1
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// Draw a cached primitive (e.g. a blurred text run) from the
// target cache to the framebuffer, applying tile clip boundaries.

void main(void) {
    Primitive prim = load_primitive(gl_InstanceID);

    VertexInfo vi = write_vertex(prim.local_rect,
                                 prim.local_clip_rect,
                                 prim.layer,
                                 prim.tile);

    RenderTaskData child_task = fetch_render_task(prim.user_data.x);
    vUv.z = child_task.data1.x;

    vec2 texture_size = vec2(textureSize(sCache, 0));
    vec2 uv0 = child_task.data0.xy / texture_size;
    vec2 uv1 = (child_task.data0.xy + child_task.data0.zw) / texture_size;

    vec2 f = (vi.local_clamped_pos - prim.local_rect.xy) / prim.local_rect.zw;

    vUv.xy = mix(uv0, uv1, f);
}
