/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

struct RectWithSize {
    vec2 p0;
    vec2 size;
};

struct RectWithEndpoint {
    vec2 p0;
    vec2 p1;
};

RectWithEndpoint to_rect_with_endpoint(RectWithSize rect) {
    RectWithEndpoint result;
    result.p0 = rect.p0;
    result.p1 = rect.p0 + rect.size;

    return result;
}

RectWithSize to_rect_with_size(RectWithEndpoint rect) {
    RectWithSize result;
    result.p0 = rect.p0;
    result.size = rect.p1 - rect.p0;

    return result;
}

RectWithSize intersect_rects(RectWithSize a, RectWithSize b) {
    RectWithSize result;
    result.p0 = max(a.p0, b.p0);
    result.size = min(a.p0 + a.size, b.p0 + b.size) - result.p0;

    return result;
}

float point_inside_rect(vec2 p, vec2 p0, vec2 p1) {
    vec2 s = step(p0, p) - step(p1, p);
    return s.x * s.y;
}

float signed_distance_rect(vec2 pos, vec2 p0, vec2 p1) {
    vec2 d = max(p0 - pos, pos - p1);
    // Instead of using a true signed distance to rect here, we just use the
    // simpler approximation of the maximum distance on either axis from the
    // outside of the rectangle. This avoids expensive use of length() and only
    // causes mostly imperceptible differences at corner pixels.
    return max(d.x, d.y);
}

vec2 clamp_rect(vec2 pt, RectWithSize rect) {
    return clamp(pt, rect.p0, rect.p0 + rect.size);
}

