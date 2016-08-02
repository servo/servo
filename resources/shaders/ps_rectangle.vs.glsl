#line 1
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

struct Rectangle {
	PrimitiveInfo info;
	vec4 color;
};

layout(std140) uniform Items {
    Rectangle rects[WR_MAX_PRIM_ITEMS];
};

void main(void) {
    Rectangle rect = rects[gl_InstanceID];
    write_vertex(rect.info);
    vColor = rect.color;
}
