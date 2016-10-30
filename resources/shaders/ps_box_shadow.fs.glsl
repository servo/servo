/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

void main(void) {
    vec2 uv = min(vec2(1.0), vMirrorPoint - abs(vUv.xy - vMirrorPoint));
    uv = mix(vCacheUvRectCoords.xy, vCacheUvRectCoords.zw, uv);
    oFragColor = vColor * texture(sCache, vec3(uv, vUv.z));
}
