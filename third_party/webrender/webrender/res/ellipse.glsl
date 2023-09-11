/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// Preprocess the radii for computing the distance approximation. This should
// be used in the vertex shader if possible to avoid doing expensive division
// in the fragment shader. When dealing with a point (zero radii), approximate
// it as an ellipse with very small radii so that we don't need to branch.
vec2 inverse_radii_squared(vec2 radii) {
    return 1.0 / max(radii * radii, 1.0e-6);
}

#ifdef WR_FRAGMENT_SHADER

// One iteration of Newton's method on the 2D equation of an ellipse:
//
//     E(x, y) = x^2/a^2 + y^2/b^2 - 1
//
// The Jacobian of this equation is:
//
//     J(E(x, y)) = [ 2*x/a^2 2*y/b^2 ]
//
// We approximate the distance with:
//
//     E(x, y) / ||J(E(x, y))||
//
// See G. Taubin, "Distance Approximations for Rasterizing Implicit
// Curves", section 3.
//
// A scale relative to the unit scale of the ellipse may be passed in to cause
// the math to degenerate to length(p) when scale is 0, or otherwise give the
// normal distance approximation if scale is 1.
float distance_to_ellipse_approx(vec2 p, vec2 inv_radii_sq, float scale) {
    vec2 p_r = p * inv_radii_sq;
    float g = dot(p, p_r) - scale;
    vec2 dG = (1.0 + scale) * p_r;
    return g * inversesqrt(dot(dG, dG));
}

// Slower but more accurate version that uses the exact distance when dealing
// with a 0-radius point distance and otherwise uses the faster approximation
// when dealing with non-zero radii.
float distance_to_ellipse(vec2 p, vec2 radii) {
    return distance_to_ellipse_approx(p, inverse_radii_squared(radii),
                                      float(all(greaterThan(radii, vec2(0.0)))));
}

float distance_to_rounded_rect(
    vec2 pos,
    vec4 center_radius_tl,
    vec4 center_radius_tr,
    vec4 center_radius_br,
    vec4 center_radius_bl,
    vec4 rect_bounds
) {
    // Clip against each ellipse. If the fragment is in a corner, one of the
    // branches below will select it as the corner to calculate the distance
    // to. We want to choose the smallest distance inside either of the axis
    // bounds as the overall distance we use to compare which corner is closer
    // than another. If outside any ellipse, default to a small offset so a
    // negative distance is returned for it.
    vec4 corner = vec4(vec2(1.0e-6), vec2(1.0));
    center_radius_tl.xy = center_radius_tl.xy - pos;
    center_radius_tr.xy = (center_radius_tr.xy - pos) * vec2(-1.0, 1.0);
    center_radius_br.xy = pos - center_radius_br.xy;
    center_radius_bl.xy = (center_radius_bl.xy - pos) * vec2(1.0, -1.0);
    if (min(center_radius_tl.x, center_radius_tl.y) > min(corner.x, corner.y)) {
        corner = center_radius_tl;
    }
    if (min(center_radius_tr.x, center_radius_tr.y) > min(corner.x, corner.y)) {
        corner = center_radius_tr;
    }
    if (min(center_radius_br.x, center_radius_br.y) > min(corner.x, corner.y)) {
        corner = center_radius_br;
    }
    if (min(center_radius_bl.x, center_radius_bl.y) > min(corner.x, corner.y)) {
        corner = center_radius_bl;
    }

    // Calculate the distance of the selected corner and the rectangle bounds,
    // whichever is greater.
    return max(distance_to_ellipse_approx(corner.xy, corner.zw, 1.0),
               signed_distance_rect(pos, rect_bounds.xy, rect_bounds.zw));
}
#endif
