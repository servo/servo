/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// GLSL point in rect test.
// See: https://stackoverflow.com/questions/12751080/glsl-point-inside-box-test
bool PointInRect(vec2 p, vec2 p0, vec2 p1)
{
    vec2 s = step(p0, p) - step(p1, p);
    return s.x * s.y != 0.0;
}

void main(void)
{
    // Clip out.
    if (PointInRect(vPosition, vClipOutRect.xy, vClipOutRect.zw)) {
        discard;
    }

    // Clip in.
    if (!PointInRect(vPosition, vClipInRect.xy, vClipInRect.zw)) {
        discard;
    }

    // Apply image tiling parameters (offset and scale) to color UVs.
    vec2 colorTexCoord = vTileParams.xy + fract(vColorTexCoord.xy) * vTileParams.zw;
    vec2 maskTexCoord = vMaskTexCoord.xy;

    // Fetch the diffuse and mask texels.
    vec4 diffuse = Texture(sDiffuse, colorTexCoord);
    vec4 mask = Texture(sMask, maskTexCoord);

    // Extract alpha from the mask (component depends on platform)
    float alpha = GetAlphaFromMask(mask);

    // Write the final fragment color.
    SetFragColor(diffuse * vec4(vColor.rgb, vColor.a * alpha));
}
