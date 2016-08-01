#line 1

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

struct ClearTile {
    uvec4 rect;
};

layout(std140) uniform Tiles {
    ClearTile tiles[WR_MAX_CLEAR_TILES];
};


void main() {
    ClearTile tile = tiles[gl_InstanceID];

    vec4 rect = vec4(tile.rect);

    vec4 pos = vec4(mix(rect.xy, rect.xy + rect.zw, aPosition.xy), 0, 1);
    gl_Position = uTransform * pos;
}
