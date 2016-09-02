#line 1
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

void main(void) {
    Rectangle rect = fetch_rectangle(gl_InstanceID);
    vColor = rect.color;
#ifdef WR_FEATURE_TRANSFORM
    TransformVertexInfo vi = write_transform_vertex(rect.info);
    vLocalRect = vi.clipped_local_rect;
    vLocalPos = vi.local_pos;
#else
    write_vertex(rect.info);
#endif
}
