/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include shared

#define LINE_STYLE_SOLID        0
#define LINE_STYLE_DOTTED       1
#define LINE_STYLE_DASHED       2
#define LINE_STYLE_WAVY         3

// Fragment position in the coordinate system used for positioning decorations.
// To keep the code independent of whether the line is horizontal or vertical,
// vLocalPos.x is always parallel, and .y always perpendicular, to the line
// being decorated.
varying vec2 vLocalPos;

flat varying int vStyle;
flat varying vec4 vParams;

#ifdef WR_VERTEX_SHADER

// The size of the mask tile we're rendering, in pixels.
PER_INSTANCE in vec4 aTaskRect;

// The size of the mask tile. aLocalSize.x is always horizontal and .y vertical,
// regardless of the line's orientation. The size is chosen by
// prim_store::line_dec::get_line_decoration_sizes.
PER_INSTANCE in vec2 aLocalSize;

// A LINE_STYLE_* value, indicating what sort of line to draw.
PER_INSTANCE in int aStyle;

// 0.0 for a horizontal line, 1.0 for a vertical line.
PER_INSTANCE in float aAxisSelect;

// The thickness of the wavy line itself, not the amplitude of the waves (i.e.,
// the thickness of the final decorated line).
PER_INSTANCE in float aWavyLineThickness;

void main(void) {
    vec2 size = mix(aLocalSize, aLocalSize.yx, aAxisSelect);
    vStyle = aStyle;

    switch (vStyle) {
        case LINE_STYLE_SOLID: {
            break;
        }
        case LINE_STYLE_DASHED: {
            vParams = vec4(size.x,          // period
                           0.5 * size.x,    // dash length
                           0.0,
                           0.0);
            break;
        }
        case LINE_STYLE_DOTTED: {
            float diameter = size.y;
            float period = diameter * 2.0;
            float center_line = 0.5 * size.y;
            vParams = vec4(period,
                           diameter / 2.0, // radius
                           center_line,
                           0.0);
            break;
        }
        case LINE_STYLE_WAVY: {
            // This logic copied from gecko to get the same results
            float line_thickness = max(aWavyLineThickness, 1.0);
            // Difference in height between peaks and troughs
            // (and since slopes are 45 degrees, the length of each slope)
            float slope_length = size.y - line_thickness;
            // Length of flat runs
            float flat_length = max((line_thickness - 1.0) * 2.0, 1.0);

            vParams = vec4(line_thickness / 2.0,
                           slope_length,
                           flat_length,
                           size.y);
            break;
        }
        default:
            vParams = vec4(0.0);
    }

    vLocalPos = mix(aPosition.xy, aPosition.yx, aAxisSelect) * size;

    gl_Position = uTransform * vec4(aTaskRect.xy + aTaskRect.zw * aPosition.xy, 0.0, 1.0);
}
#endif

#ifdef WR_FRAGMENT_SHADER

#define MAGIC_WAVY_LINE_AA_SNAP         0.5

void main(void) {
    // Find the appropriate distance to apply the step over.
    vec2 pos = vLocalPos;
    float aa_range = compute_aa_range(pos);
    float alpha = 1.0;

    switch (vStyle) {
        case LINE_STYLE_SOLID: {
            break;
        }
        case LINE_STYLE_DASHED: {
            // Calculate dash alpha (on/off) based on dash length
            alpha = step(floor(pos.x + 0.5), vParams.y);
            break;
        }
        case LINE_STYLE_DOTTED: {
            // Get the dot alpha
            vec2 dot_relative_pos = pos - vParams.yz;
            float dot_distance = length(dot_relative_pos) - vParams.y;
            alpha = distance_aa(aa_range, dot_distance);
            break;
        }
        case LINE_STYLE_WAVY: {
            float half_line_thickness = vParams.x;
            float slope_length = vParams.y;
            float flat_length = vParams.z;
            float vertical_bounds = vParams.w;
            // Our pattern is just two slopes and two flats
            float half_period = slope_length + flat_length;

            float mid_height = vertical_bounds / 2.0;
            float peak_offset = mid_height - half_line_thickness;
            // Flip the wave every half period
            float flip = -2.0 * (step(mod(pos.x, 2.0 * half_period), half_period) - 0.5);
            // float flip = -1.0;
            peak_offset *= flip;
            float peak_height = mid_height + peak_offset;

            // Convert pos to a local position within one half period
            pos.x = mod(pos.x, half_period);

            // Compute signed distance to the 3 lines that make up an arc
            float dist1 = distance_to_line(vec2(0.0, peak_height),
                                           vec2(1.0, -flip),
                                           pos);
            float dist2 = distance_to_line(vec2(0.0, peak_height),
                                           vec2(0, -flip),
                                           pos);
            float dist3 = distance_to_line(vec2(flat_length, peak_height),
                                           vec2(-1.0, -flip),
                                           pos);
            float dist = abs(max(max(dist1, dist2), dist3));

            // Apply AA based on the thickness of the wave
            alpha = distance_aa(aa_range, dist - half_line_thickness);

            // Disable AA for thin lines
            if (half_line_thickness <= 1.0) {
                alpha = 1.0 - step(alpha, MAGIC_WAVY_LINE_AA_SNAP);
            }

            break;
        }
        default: break;
    }

    oFragColor = vec4(alpha);
}
#endif
