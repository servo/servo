#line 1
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// TODO(gw): Write a fast path blur that handles smaller blur radii
//           with a offset / weight uniform table and a constant
//           loop iteration count!

// TODO(gw): Make use of the bilinear sampling trick to reduce
//           the number of texture fetches needed for a gaussian blur.

float gauss(float x, float sigma) {
    return (1.0 / sqrt(6.283185307179586 * sigma * sigma)) * exp(-(x * x) / (2.0 * sigma * sigma));
}

void main(void) {
    vec4 sample = texture(sCache, vUv);
    vec4 color = vec4(sample.rgb * sample.a, sample.a) * gauss(0.0, vSigma);

    for (int i=1 ; i < vBlurRadius ; ++i) {
        vec2 offset = vec2(float(i)) * vOffsetScale;

        vec2 st0 = clamp(vUv.xy + offset, vUvRect.xy, vUvRect.zw);
        vec4 color0 = texture(sCache, vec3(st0, vUv.z));

        vec2 st1 = clamp(vUv.xy - offset, vUvRect.xy, vUvRect.zw);
        vec4 color1 = texture(sCache, vec3(st1, vUv.z));

        // Alpha must be premultiplied in order to properly blur the alpha channel.
        float weight = gauss(float(i), vSigma);
        color += vec4(color0.rgb * color0.a, color0.a) * weight;
        color += vec4(color1.rgb * color1.a, color1.a) * weight;
    }

    // Unpremultiply the alpha.
    color.rgb /= color.a;

    oFragColor = color;
}
