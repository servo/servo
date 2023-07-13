/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

flat varying vec4 vTransformBounds;

#ifdef WR_VERTEX_SHADER

#define VECS_PER_TRANSFORM   8U
uniform HIGHP_SAMPLER_FLOAT sampler2D sTransformPalette;

void init_transform_vs(vec4 local_bounds) {
    vTransformBounds = local_bounds;
}

struct Transform {
    mat4 m;
    mat4 inv_m;
    bool is_axis_aligned;
};

Transform fetch_transform(int id) {
    Transform transform;

    transform.is_axis_aligned = (id >> 24) == 0;
    int index = id & 0x00ffffff;

    // Create a UV base coord for each 8 texels.
    // This is required because trying to use an offset
    // of more than 8 texels doesn't work on some versions
    // of macOS.
    ivec2 uv = get_fetch_uv(index, VECS_PER_TRANSFORM);
    ivec2 uv0 = ivec2(uv.x + 0, uv.y);

    transform.m[0] = TEXEL_FETCH(sTransformPalette, uv0, 0, ivec2(0, 0));
    transform.m[1] = TEXEL_FETCH(sTransformPalette, uv0, 0, ivec2(1, 0));
    transform.m[2] = TEXEL_FETCH(sTransformPalette, uv0, 0, ivec2(2, 0));
    transform.m[3] = TEXEL_FETCH(sTransformPalette, uv0, 0, ivec2(3, 0));

    transform.inv_m[0] = TEXEL_FETCH(sTransformPalette, uv0, 0, ivec2(4, 0));
    transform.inv_m[1] = TEXEL_FETCH(sTransformPalette, uv0, 0, ivec2(5, 0));
    transform.inv_m[2] = TEXEL_FETCH(sTransformPalette, uv0, 0, ivec2(6, 0));
    transform.inv_m[3] = TEXEL_FETCH(sTransformPalette, uv0, 0, ivec2(7, 0));

    return transform;
}

// Return the intersection of the plane (set up by "normal" and "point")
// with the ray (set up by "ray_origin" and "ray_dir"),
// writing the resulting scaler into "t".
bool ray_plane(vec3 normal, vec3 pt, vec3 ray_origin, vec3 ray_dir, out float t)
{
    float denom = dot(normal, ray_dir);
    if (abs(denom) > 1e-6) {
        vec3 d = pt - ray_origin;
        t = dot(d, normal) / denom;
        return t >= 0.0;
    }

    return false;
}

// Apply the inverse transform "inv_transform"
// to the reference point "ref" in CSS space,
// producing a local point on a Transform plane,
// set by a base point "a" and a normal "n".
vec4 untransform(vec2 ref, vec3 n, vec3 a, mat4 inv_transform) {
    vec3 p = vec3(ref, -10000.0);
    vec3 d = vec3(0, 0, 1.0);

    float t = 0.0;
    // get an intersection of the Transform plane with Z axis vector,
    // originated from the "ref" point
    ray_plane(n, a, p, d, t);
    float z = p.z + d.z * t; // Z of the visible point on the Transform

    vec4 r = inv_transform * vec4(ref, z, 1.0);
    return r;
}

// Given a CSS space position, transform it back into the Transform space.
vec4 get_node_pos(vec2 pos, Transform transform) {
    // get a point on the scroll node plane
    vec4 ah = transform.m * vec4(0.0, 0.0, 0.0, 1.0);
    vec3 a = ah.xyz / ah.w;

    // get the normal to the scroll node plane
    vec3 n = transpose(mat3(transform.inv_m)) * vec3(0.0, 0.0, 1.0);
    return untransform(pos, n, a, transform.inv_m);
}

#endif //WR_VERTEX_SHADER

#ifdef WR_FRAGMENT_SHADER

// Assume transform bounds are set to a large scale to signal they are invalid.
bool has_valid_transform_bounds() {
    return vTransformBounds.w < 1.0e15;
}

float init_transform_fs(vec2 local_pos) {
    // Get signed distance from local rect bounds.
    float d = signed_distance_rect(
        local_pos,
        vTransformBounds.xy,
        vTransformBounds.zw
    );

    // Find the appropriate distance to apply the AA smoothstep over.
    float aa_range = compute_aa_range(local_pos);

    // Only apply AA to fragments outside the signed distance field.
    return distance_aa(aa_range, d);
}

float init_transform_rough_fs(vec2 local_pos) {
    return point_inside_rect(
        local_pos,
        vTransformBounds.xy,
        vTransformBounds.zw
    );
}

#endif //WR_FRAGMENT_SHADER
