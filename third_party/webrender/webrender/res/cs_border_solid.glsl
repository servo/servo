/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include shared,ellipse

#define DONT_MIX 0
#define MIX_AA 1
#define MIX_NO_AA 2

// For edges, the colors are the same. For corners, these
// are the colors of each edge making up the corner.
flat varying vec4 vColor0;
flat varying vec4 vColor1;

// A point + tangent defining the line where the edge
// transition occurs. Used for corners only.
flat varying vec4 vColorLine;

// A boolean indicating that we should be mixing between edge colors.
flat varying int vMixColors;

// xy = Local space position of the clip center.
// zw = Scale the rect origin by this to get the outer
// corner from the segment rectangle.
flat varying vec4 vClipCenter_Sign;

// An outer and inner elliptical radii for border
// corner clipping.
flat varying vec4 vClipRadii;

// Position, scale, and radii of horizontally and vertically adjacent corner clips.
flat varying vec4 vHorizontalClipCenter_Sign;
flat varying vec2 vHorizontalClipRadii;
flat varying vec4 vVerticalClipCenter_Sign;
flat varying vec2 vVerticalClipRadii;

// Local space position
varying vec2 vPos;

#define SEGMENT_TOP_LEFT        0
#define SEGMENT_TOP_RIGHT       1
#define SEGMENT_BOTTOM_RIGHT    2
#define SEGMENT_BOTTOM_LEFT     3

#ifdef WR_VERTEX_SHADER

PER_INSTANCE in vec2 aTaskOrigin;
PER_INSTANCE in vec4 aRect;
PER_INSTANCE in vec4 aColor0;
PER_INSTANCE in vec4 aColor1;
PER_INSTANCE in int aFlags;
PER_INSTANCE in vec2 aWidths;
PER_INSTANCE in vec2 aRadii;
PER_INSTANCE in vec4 aClipParams1;
PER_INSTANCE in vec4 aClipParams2;

vec2 get_outer_corner_scale(int segment) {
    vec2 p;

    switch (segment) {
        case SEGMENT_TOP_LEFT:
            p = vec2(0.0, 0.0);
            break;
        case SEGMENT_TOP_RIGHT:
            p = vec2(1.0, 0.0);
            break;
        case SEGMENT_BOTTOM_RIGHT:
            p = vec2(1.0, 1.0);
            break;
        case SEGMENT_BOTTOM_LEFT:
            p = vec2(0.0, 1.0);
            break;
        default:
            // The result is only used for non-default segment cases
            p = vec2(0.0);
            break;
    }

    return p;
}

void main(void) {
    int segment = aFlags & 0xff;
    bool do_aa = ((aFlags >> 24) & 0xf0) != 0;

    vec2 outer_scale = get_outer_corner_scale(segment);
    vec2 outer = outer_scale * aRect.zw;
    vec2 clip_sign = 1.0 - 2.0 * outer_scale;

    int mix_colors;
    switch (segment) {
        case SEGMENT_TOP_LEFT:
        case SEGMENT_TOP_RIGHT:
        case SEGMENT_BOTTOM_RIGHT:
        case SEGMENT_BOTTOM_LEFT: {
            mix_colors = do_aa ? MIX_AA : MIX_NO_AA;
            break;
        }
        default:
            mix_colors = DONT_MIX;
            break;
    }

    vMixColors = mix_colors;
    vPos = aRect.zw * aPosition.xy;

    vColor0 = aColor0;
    vColor1 = aColor1;
    vClipCenter_Sign = vec4(outer + clip_sign * aRadii, clip_sign);
    vClipRadii = vec4(aRadii, max(aRadii - aWidths, 0.0));
    vColorLine = vec4(outer, aWidths.y * -clip_sign.y, aWidths.x * clip_sign.x);

    vec2 horizontal_clip_sign = vec2(-clip_sign.x, clip_sign.y);
    vHorizontalClipCenter_Sign = vec4(aClipParams1.xy +
                                      horizontal_clip_sign * aClipParams1.zw,
                                      horizontal_clip_sign);
    vHorizontalClipRadii = aClipParams1.zw;

    vec2 vertical_clip_sign = vec2(clip_sign.x, -clip_sign.y);
    vVerticalClipCenter_Sign = vec4(aClipParams2.xy +
                                    vertical_clip_sign * aClipParams2.zw,
                                    vertical_clip_sign);
    vVerticalClipRadii = aClipParams2.zw;

    gl_Position = uTransform * vec4(aTaskOrigin + aRect.xy + vPos, 0.0, 1.0);
}
#endif

#ifdef WR_FRAGMENT_SHADER
void main(void) {
    float aa_range = compute_aa_range(vPos);
    bool do_aa = vMixColors != MIX_NO_AA;

    float mix_factor = 0.0;
    if (vMixColors != DONT_MIX) {
        float d_line = distance_to_line(vColorLine.xy, vColorLine.zw, vPos);
        if (do_aa) {
            mix_factor = distance_aa(aa_range, -d_line);
        } else {
            mix_factor = d_line + EPSILON >= 0. ? 1.0 : 0.0;
        }
    }

    // Check if inside main corner clip-region
    vec2 clip_relative_pos = vPos - vClipCenter_Sign.xy;
    bool in_clip_region = all(lessThan(vClipCenter_Sign.zw * clip_relative_pos, vec2(0.0)));

    float d = -1.0;
    if (in_clip_region) {
        float d_radii_a = distance_to_ellipse(clip_relative_pos, vClipRadii.xy, aa_range);
        float d_radii_b = distance_to_ellipse(clip_relative_pos, vClipRadii.zw, aa_range);
        d = max(d_radii_a, -d_radii_b);
    }

    // And again for horizontally-adjacent corner
    clip_relative_pos = vPos - vHorizontalClipCenter_Sign.xy;
    in_clip_region = all(lessThan(vHorizontalClipCenter_Sign.zw * clip_relative_pos, vec2(0.0)));
    if (in_clip_region) {
        float d_radii = distance_to_ellipse(clip_relative_pos, vHorizontalClipRadii.xy, aa_range);
        d = max(d_radii, d);
    }

    // And finally for vertically-adjacent corner
    clip_relative_pos = vPos - vVerticalClipCenter_Sign.xy;
    in_clip_region = all(lessThan(vVerticalClipCenter_Sign.zw * clip_relative_pos, vec2(0.0)));
    if (in_clip_region) {
        float d_radii = distance_to_ellipse(clip_relative_pos, vVerticalClipRadii.xy, aa_range);
        d = max(d_radii, d);
    }

    float alpha = do_aa ? distance_aa(aa_range, d) : 1.0;
    vec4 color = mix(vColor0, vColor1, mix_factor);
    oFragColor = color * alpha;
}
#endif
