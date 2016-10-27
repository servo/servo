#line 1
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// See http://asciimath.org to render the equations here.

// The Gaussian function used for blurring:
//
//     G_sigma(x) = 1/sqrt(2 pi sigma^2) e^(-x^2/(2 sigma^2))
float gauss(float x, float sigma) {
    float sigmaPow2 = sigma * sigma;
    return 1.0 / sqrt(6.283185307179586 * sigmaPow2) * exp(-(x * x) / (2.0 * sigmaPow2));
}

// An approximation of the error function, which is related to the integral of the Gaussian
// function:
//
//     "erf"(x) = 2/sqrt(pi) int_0^x e^(-t^2) dt
//              ~~ 1 - 1 / (1 + a_1 x + a_2 x^2 + a_3 x^3 + a_4 x^4)^4
//
// where:
//
//     a_1 = 0.278393, a_2 = 0.230389, a_3 = 0.000972, a_4 = 0.078108
//
// This approximation is accurate to `5 xx 10^-4`, more than accurate enough for our purposes.
//
// See: https://en.wikipedia.org/wiki/Error_function#Approximation_with_elementary_functions
float erf(float x) {
    bool negative = x < 0.0;
    if (negative)
        x = -x;
    float x2 = x * x;
    float x3 = x2 * x;
    float x4 = x2 * x2;
    float denom = 1.0 + 0.278393 * x + 0.230389 * x2 + 0.000972 * x3 + 0.078108 * x4;
    float result = 1.0 - 1.0 / (denom * denom * denom * denom);
    return negative ? -result : result;
}

// A useful helper for calculating integrals of the Gaussian function via the error function:
//
//      "erf"_sigma(x) = 2 int 1/sqrt(2 pi sigma^2) e^(-x^2/(2 sigma^2)) dx
//                     = "erf"(x/(sigma sqrt(2)))
float erfSigma(float x, float sigma) {
    return erf(x / (sigma * 1.4142135623730951));
}

// Returns the blurred color value from the box itself (not counting any rounded corners). `p_0` is
// the vector distance to the top left corner of the box; `p_1` is the vector distance to its
// bottom right corner.
//
//      "colorFromRect"_sigma(p_0, p_1)
//          = int_{p_{0_y}}^{p_{1_y}} int_{p_{1_x}}^{p_{0_x}} G_sigma(y) G_sigma(x) dx dy
//          = 1/4 ("erf"_sigma(p_{1_x}) - "erf"_sigma(p_{0_x}))
//              ("erf"_sigma(p_{1_y}) - "erf"_sigma(p_{0_y}))
float colorFromRect(vec2 p0, vec2 p1, float sigma) {
    return (erfSigma(p1.x, sigma) - erfSigma(p0.x, sigma)) *
        (erfSigma(p1.y, sigma) - erfSigma(p0.y, sigma)) / 4.0;
}

// Returns the `x` coordinate on the ellipse with the given radii for the given `y` coordinate:
//
//      "ellipsePoint"(y, y_0, a, b) = a sqrt(1 - ((y - y_0) / b)^2)
float ellipsePoint(float y, float y0, vec2 radii) {
    float bStep = (y - y0) / radii.y;
    return radii.x * sqrt(1.0 - bStep * bStep);
}

// A helper function to compute the value that needs to be subtracted to accommodate the border
// corners.
//
//     "colorCutout"_sigma(x_{0_l}, x_{0_r}, y_0, y_{min}, y_{max}, a, b)
//          = int_{y_{min}}^{y_{max}}
//              int_{x_{0_r} + "ellipsePoint"(y, y_0, a, b)}^{x_{0_r} + a} G_sigma(y) G_sigma(x) dx
//              + int_{x_{0_l} - a}^{x_{0_l} - "ellipsePoint"(y, y_0, a, b)} G_sigma(y) G_sigma(x)
//                  dx dy
//          = int_{y_{min}}^{y_{max}} 1/2 G_sigma(y)
//              ("erf"_sigma(x_{0_r} + a) - "erf"_sigma(x_{0_r} + "ellipsePoint"(y, y_0, a, b)) +
//               "erf"_sigma(x_{0_l} - "ellipsePoint"(y, y_0, a, b)) - "erf"_sigma(x_{0_l} - a))
//
// with the outer integral evaluated numerically.
float colorCutoutGeneral(float x0l,
                         float x0r,
                         float y0,
                         float yMin,
                         float yMax,
                         vec2 radii,
                         float sigma) {
    float sum = 0.0;
    for (float y = yMin; y <= yMax; y += 1.0) {
        float xEllipsePoint = ellipsePoint(y, y0, radii);
        sum += gauss(y, sigma) *
            (erfSigma(x0r + radii.x, sigma) - erfSigma(x0r + xEllipsePoint, sigma) +
             erfSigma(x0l - xEllipsePoint, sigma) - erfSigma(x0l - radii.x, sigma));
    }
    return sum / 2.0;
}

// The value that needs to be subtracted to accommodate the top border corners.
float colorCutoutTop(float x0l, float x0r, float y0, vec2 radii, float sigma) {
    return colorCutoutGeneral(x0l, x0r, y0, y0, y0 + radii.y, radii, sigma);
}

// The value that needs to be subtracted to accommodate the bottom border corners.
float colorCutoutBottom(float x0l, float x0r, float y0, vec2 radii, float sigma) {
    return colorCutoutGeneral(x0l, x0r, y0, y0 - radii.y, y0, radii, sigma);
}

// The blurred color value for the point at `pos` with the top left corner of the box at
// `p_{0_"rect"}` and the bottom right corner of the box at `p_{1_"rect"}`.
float color(vec2 pos, vec2 p0Rect, vec2 p1Rect, vec2 radii, float sigma) {
    // Compute the vector distances `p_0` and `p_1`.
    vec2 p0 = p0Rect - pos, p1 = p1Rect - pos;

    // Compute the basic color `"colorFromRect"_sigma(p_0, p_1)`. This is all we have to do if
    // the box is unrounded.
    float cRect = colorFromRect(p0, p1, sigma);
    if (radii.x == 0.0 || radii.y == 0.0)
        return cRect;

    // Compute the inner corners of the box, taking border radii into account: `x_{0_l}`,
    // `y_{0_t}`, `x_{0_r}`, and `y_{0_b}`.
    float x0l = p0.x + radii.x;
    float y0t = p1.y - radii.y;
    float x0r = p1.x - radii.x;
    float y0b = p0.y + radii.y;

    // Compute the final color:
    //
    //     "colorFromRect"_sigma(p_0, p_1) -
    //          ("colorCutoutTop"_sigma(x_{0_l}, x_{0_r}, y_{0_t}, a, b) +
    //           "colorCutoutBottom"_sigma(x_{0_l}, x_{0_r}, y_{0_b}, a, b))
    float cCutoutTop = colorCutoutTop(x0l, x0r, y0t, radii, sigma);
    float cCutoutBottom = colorCutoutBottom(x0l, x0r, y0b, radii, sigma);
    return cRect - (cCutoutTop + cCutoutBottom);
}

void main(void) {
    vec2 pos = vPos.xy;
    vec2 p0Rect = vBoxShadowRect.xy, p1Rect = vBoxShadowRect.zw;
    vec2 radii = vBorderRadii.xy;
    float sigma = vBlurRadius / 2.0;
    float value = color(pos, p0Rect, p1Rect, radii, sigma);

    value = max(value, 0.0);
    oFragColor = vec4(1.0, 1.0, 1.0, vInverted == 1.0 ? 1.0 - value : value);
}
