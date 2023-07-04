/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include shared,ellipse

// For edges, the colors are the same. For corners, these
// are the colors of each edge making up the corner.
flat varying vec4 vColor00;
flat varying vec4 vColor01;
flat varying vec4 vColor10;
flat varying vec4 vColor11;

// A point + tangent defining the line where the edge
// transition occurs. Used for corners only.
flat varying vec4 vColorLine;

// x = segment, y = styles, z = edge axes, w = clip mode
// Since by default in GLES the vertex shader uses highp 
// and the fragment shader uses mediump, we explicitely 
// use mediump precision so we align with the default 
// mediump precision in the fragment shader.
flat varying mediump ivec4 vConfig;

// xy = Local space position of the clip center.
// zw = Scale the rect origin by this to get the outer
// corner from the segment rectangle.
flat varying vec4 vClipCenter_Sign;

// An outer and inner elliptical radii for border
// corner clipping.
flat varying vec4 vClipRadii;

// Reference point for determine edge clip lines.
flat varying vec4 vEdgeReference;

// Stores widths/2 and widths/3 to save doing this in FS.
flat varying vec4 vPartialWidths;

// Clipping parameters for dot or dash.
flat varying vec4 vClipParams1;
flat varying vec4 vClipParams2;

// Local space position
varying vec2 vPos;

#define SEGMENT_TOP_LEFT        0
#define SEGMENT_TOP_RIGHT       1
#define SEGMENT_BOTTOM_RIGHT    2
#define SEGMENT_BOTTOM_LEFT     3
#define SEGMENT_LEFT            4
#define SEGMENT_TOP             5
#define SEGMENT_RIGHT           6
#define SEGMENT_BOTTOM          7

// Border styles as defined in webrender_api/types.rs
#define BORDER_STYLE_NONE         0
#define BORDER_STYLE_SOLID        1
#define BORDER_STYLE_DOUBLE       2
#define BORDER_STYLE_DOTTED       3
#define BORDER_STYLE_DASHED       4
#define BORDER_STYLE_HIDDEN       5
#define BORDER_STYLE_GROOVE       6
#define BORDER_STYLE_RIDGE        7
#define BORDER_STYLE_INSET        8
#define BORDER_STYLE_OUTSET       9

#define CLIP_NONE        0
#define CLIP_DASH_CORNER 1
#define CLIP_DASH_EDGE   2
#define CLIP_DOT         3

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

// NOTE(emilio): If you change this algorithm, do the same change
// in border.rs
vec4 mod_color(vec4 color, bool is_black, bool lighter) {
    const float light_black = 0.7;
    const float dark_black = 0.3;

    const float dark_scale = 0.66666666;
    const float light_scale = 1.0;

    if (is_black) {
        if (lighter) {
            return vec4(vec3(light_black), color.a);
        }
        return vec4(vec3(dark_black), color.a);
    }

    if (lighter) {
        return vec4(color.rgb * light_scale, color.a);
    }
    return vec4(color.rgb * dark_scale, color.a);
}

vec4[2] get_colors_for_side(vec4 color, int style) {
    vec4 result[2];

    bool is_black = color.rgb == vec3(0.0, 0.0, 0.0);

    switch (style) {
        case BORDER_STYLE_GROOVE:
            result[0] = mod_color(color, is_black, true);
            result[1] = mod_color(color, is_black, false);
            break;
        case BORDER_STYLE_RIDGE:
            result[0] = mod_color(color, is_black, false);
            result[1] = mod_color(color, is_black, true);
            break;
        default:
            result[0] = color;
            result[1] = color;
            break;
    }

    return result;
}

void main(void) {
    int segment = aFlags & 0xff;
    int style0 = (aFlags >> 8) & 0xff;
    int style1 = (aFlags >> 16) & 0xff;
    int clip_mode = (aFlags >> 24) & 0x0f;

    vec2 outer_scale = get_outer_corner_scale(segment);
    vec2 outer = outer_scale * aRect.zw;
    vec2 clip_sign = 1.0 - 2.0 * outer_scale;

    // Set some flags used by the FS to determine the
    // orientation of the two edges in this corner.
    ivec2 edge_axis = ivec2(0, 0);
    // Derive the positions for the edge clips, which must be handled
    // differently between corners and edges.
    vec2 edge_reference = vec2(0.0);
    switch (segment) {
        case SEGMENT_TOP_LEFT:
            edge_axis = ivec2(0, 1);
            edge_reference = outer;
            break;
        case SEGMENT_TOP_RIGHT:
            edge_axis = ivec2(1, 0);
            edge_reference = vec2(outer.x - aWidths.x, outer.y);
            break;
        case SEGMENT_BOTTOM_RIGHT:
            edge_axis = ivec2(0, 1);
            edge_reference = outer - aWidths;
            break;
        case SEGMENT_BOTTOM_LEFT:
            edge_axis = ivec2(1, 0);
            edge_reference = vec2(outer.x, outer.y - aWidths.y);
            break;
        case SEGMENT_TOP:
        case SEGMENT_BOTTOM:
            edge_axis = ivec2(1, 1);
            break;
        case SEGMENT_LEFT:
        case SEGMENT_RIGHT:
        default:
            break;
    }

    vConfig = ivec4(
        segment,
        style0 | (style1 << 8),
        edge_axis.x | (edge_axis.y << 8),
        clip_mode
    );
    vPartialWidths = vec4(aWidths / 3.0, aWidths / 2.0);
    vPos = aRect.zw * aPosition.xy;

    vec4[2] color0 = get_colors_for_side(aColor0, style0);
    vColor00 = color0[0];
    vColor01 = color0[1];
    vec4[2] color1 = get_colors_for_side(aColor1, style1);
    vColor10 = color1[0];
    vColor11 = color1[1];
    vClipCenter_Sign = vec4(outer + clip_sign * aRadii, clip_sign);
    vClipRadii = vec4(aRadii, max(aRadii - aWidths, 0.0));
    vColorLine = vec4(outer, aWidths.y * -clip_sign.y, aWidths.x * clip_sign.x);
    vEdgeReference = vec4(edge_reference, edge_reference + aWidths);
    vClipParams1 = aClipParams1;
    vClipParams2 = aClipParams2;

    // For the case of dot and dash clips, optimize the number of pixels that
    // are hit to just include the dot itself.
    if (clip_mode == CLIP_DOT) {
        float radius = aClipParams1.z;

        // Expand by a small amount to allow room for AA around
        // the dot if it's big enough.
        if (radius > 0.5)
            radius += 2.0;

        vPos = vClipParams1.xy + radius * (2.0 * aPosition.xy - 1.0);
        vPos = clamp(vPos, vec2(0.0), aRect.zw);
    } else if (clip_mode == CLIP_DASH_CORNER) {
        vec2 center = (aClipParams1.xy + aClipParams2.xy) * 0.5;
        // This is a gross approximation which works out because dashes don't have
        // a strong curvature and we will overshoot by inflating the geometry by
        // this amount on each side (sqrt(2) * length(dash) would be enough and we
        // compute 2 * approx_length(dash)).
        float dash_length = length(aClipParams1.xy - aClipParams2.xy);
        float width = max(aWidths.x, aWidths.y);
        // expand by a small amout for AA just like we do for dots.
        vec2 r = vec2(max(dash_length, width)) + 2.0;
        vPos = clamp(vPos, center - r, center + r);
    }

    gl_Position = uTransform * vec4(aTaskOrigin + aRect.xy + vPos, 0.0, 1.0);
}
#endif

#ifdef WR_FRAGMENT_SHADER
vec4 evaluate_color_for_style_in_corner(
    vec2 clip_relative_pos,
    int style,
    vec4 color0,
    vec4 color1,
    vec4 clip_radii,
    float mix_factor,
    int segment,
    float aa_range
) {
    switch (style) {
        case BORDER_STYLE_DOUBLE: {
            // Get the distances from 0.33 of the radii, and
            // also 0.67 of the radii. Use these to form a
            // SDF subtraction which will clip out the inside
            // third of the rounded edge.
            float d_radii_a = distance_to_ellipse(
                clip_relative_pos,
                clip_radii.xy - vPartialWidths.xy,
                aa_range
            );
            float d_radii_b = distance_to_ellipse(
                clip_relative_pos,
                clip_radii.xy - 2.0 * vPartialWidths.xy,
                aa_range
            );
            float d = min(-d_radii_a, d_radii_b);
            color0 *= distance_aa(aa_range, d);
            break;
        }
        case BORDER_STYLE_GROOVE:
        case BORDER_STYLE_RIDGE: {
            float d = distance_to_ellipse(
                clip_relative_pos,
                clip_radii.xy - vPartialWidths.zw,
                aa_range
            );
            float alpha = distance_aa(aa_range, d);
            float swizzled_factor;
            switch (segment) {
                case SEGMENT_TOP_LEFT: swizzled_factor = 0.0; break;
                case SEGMENT_TOP_RIGHT: swizzled_factor = mix_factor; break;
                case SEGMENT_BOTTOM_RIGHT: swizzled_factor = 1.0; break;
                case SEGMENT_BOTTOM_LEFT: swizzled_factor = 1.0 - mix_factor; break;
                default: swizzled_factor = 0.0; break;
            };
            vec4 c0 = mix(color1, color0, swizzled_factor);
            vec4 c1 = mix(color0, color1, swizzled_factor);
            color0 = mix(c0, c1, alpha);
            break;
        }
        default:
            break;
    }

    return color0;
}

vec4 evaluate_color_for_style_in_edge(
    vec2 pos_vec,
    int style,
    vec4 color0,
    vec4 color1,
    float aa_range,
    int edge_axis_id
) {
    vec2 edge_axis = edge_axis_id != 0 ? vec2(0.0, 1.0) : vec2(1.0, 0.0);
    float pos = dot(pos_vec, edge_axis);
    switch (style) {
        case BORDER_STYLE_DOUBLE: {
            float d = -1.0;
            float partial_width = dot(vPartialWidths.xy, edge_axis);
            if (partial_width >= 1.0) {
                vec2 ref = vec2(
                    dot(vEdgeReference.xy, edge_axis) + partial_width,
                    dot(vEdgeReference.zw, edge_axis) - partial_width
                );
                d = min(pos - ref.x, ref.y - pos);
            }
            color0 *= distance_aa(aa_range, d);
            break;
        }
        case BORDER_STYLE_GROOVE:
        case BORDER_STYLE_RIDGE: {
            float ref = dot(vEdgeReference.xy + vPartialWidths.zw, edge_axis);
            float d = pos - ref;
            float alpha = distance_aa(aa_range, d);
            color0 = mix(color0, color1, alpha);
            break;
        }
        default:
            break;
    }

    return color0;
}

void main(void) {
    float aa_range = compute_aa_range(vPos);
    vec4 color0, color1;

    int segment = vConfig.x;
    ivec2 style = ivec2(vConfig.y & 0xff, vConfig.y >> 8);
    ivec2 edge_axis = ivec2(vConfig.z & 0xff, vConfig.z >> 8);
    int clip_mode = vConfig.w;

    float mix_factor = 0.0;
    if (edge_axis.x != edge_axis.y) {
        float d_line = distance_to_line(vColorLine.xy, vColorLine.zw, vPos);
        mix_factor = distance_aa(aa_range, -d_line);
    }

    // Check if inside corner clip-region
    vec2 clip_relative_pos = vPos - vClipCenter_Sign.xy;
    bool in_clip_region = all(lessThan(vClipCenter_Sign.zw * clip_relative_pos, vec2(0.0)));
    float d = -1.0;

    switch (clip_mode) {
        case CLIP_DOT: {
            // Set clip distance based or dot position and radius.
            d = distance(vClipParams1.xy, vPos) - vClipParams1.z;
            break;
        }
        case CLIP_DASH_EDGE: {
            bool is_vertical = vClipParams1.x == 0.;
            float half_dash = is_vertical ? vClipParams1.y : vClipParams1.x;
            // We want to draw something like:
            // +---+---+---+---+
            // |xxx|   |   |xxx|
            // +---+---+---+---+
            float pos = is_vertical ? vPos.y : vPos.x;
            bool in_dash = pos < half_dash || pos > 3.0 * half_dash;
            if (!in_dash) {
                d = 1.;
            }
            break;
        }
        case CLIP_DASH_CORNER: {
            // Get SDF for the two line/tangent clip lines,
            // do SDF subtract to get clip distance.
            float d0 = distance_to_line(vClipParams1.xy,
                                        vClipParams1.zw,
                                        vPos);
            float d1 = distance_to_line(vClipParams2.xy,
                                        vClipParams2.zw,
                                        vPos);
            d = max(d0, -d1);
            break;
        }
        case CLIP_NONE:
        default:
            break;
    }

    if (in_clip_region) {
        float d_radii_a = distance_to_ellipse(clip_relative_pos, vClipRadii.xy, aa_range);
        float d_radii_b = distance_to_ellipse(clip_relative_pos, vClipRadii.zw, aa_range);
        float d_radii = max(d_radii_a, -d_radii_b);
        d = max(d, d_radii);

        color0 = evaluate_color_for_style_in_corner(
            clip_relative_pos,
            style.x,
            vColor00,
            vColor01,
            vClipRadii,
            mix_factor,
            segment,
            aa_range
        );
        color1 = evaluate_color_for_style_in_corner(
            clip_relative_pos,
            style.y,
            vColor10,
            vColor11,
            vClipRadii,
            mix_factor,
            segment,
            aa_range
        );
    } else {
        color0 = evaluate_color_for_style_in_edge(
            vPos,
            style.x,
            vColor00,
            vColor01,
            aa_range,
            edge_axis.x
        );
        color1 = evaluate_color_for_style_in_edge(
            vPos,
            style.y,
            vColor10,
            vColor11,
            aa_range,
            edge_axis.y
        );
    }

    float alpha = distance_aa(aa_range, d);
    vec4 color = mix(color0, color1, mix_factor);
    oFragColor = color * alpha;
}
#endif
