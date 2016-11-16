#line 1
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

void main(void) {
    CacheClipInstance cci = fetch_clip_item(gl_InstanceID);
    ClipArea area = fetch_clip_area(cci.render_task_index);

    vec2 final_pos = mix(area.task_bounds.xy, area.task_bounds.zw, aPosition.xy);

    gl_Position = uTransform * vec4(final_pos, 0.0, 1.0);
}
