/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ import { assert, unreachable } from '../../common/util/util.js';
import { multiplyMatrices } from './math.js';

// These color space conversion function definitions are copied directly from
// CSS Color Module Level 4 Sample Code: https://drafts.csswg.org/css-color/#color-conversion-code
// *EXCEPT* the conversion matrices are replaced with exact rational forms computed here:
// https://github.com/kainino0x/exact_css_xyz_matrices
//   using this Rust crate: https://crates.io/crates/rgb_derivation
//   as described for sRGB on this page: https://mina86.com/2019/srgb-xyz-matrix/
//   but using the numbers from the CSS spec: https://www.w3.org/TR/css-color-4/#predefined

// Sample code for color conversions
// Conversion can also be done using ICC profiles and a Color Management System
// For clarity, a library is used for matrix multiplication (multiply-matrices.js)

// sRGB-related functions

/**
 * convert an array of sRGB values
 * where in-gamut values are in the range [0 - 1]
 * to linear light (un-companded) form.
 * https://en.wikipedia.org/wiki/SRGB
 * Extended transfer function:
 * for negative values,  linear portion is extended on reflection of axis,
 * then reflected power function is used.
 */
function lin_sRGB(RGB) {
  return RGB.map(val => {
    const sign = val < 0 ? -1 : 1;
    const abs = Math.abs(val);

    if (abs < 0.04045) {
      return val / 12.92;
    }

    return sign * Math.pow((abs + 0.055) / 1.055, 2.4);
  });
}

/**
 * convert an array of linear-light sRGB values in the range 0.0-1.0
 * to gamma corrected form
 * https://en.wikipedia.org/wiki/SRGB
 * Extended transfer function:
 * For negative values, linear portion extends on reflection
 * of axis, then uses reflected pow below that
 */
function gam_sRGB(RGB) {
  return RGB.map(val => {
    const sign = val < 0 ? -1 : 1;
    const abs = Math.abs(val);

    if (abs > 0.0031308) {
      return sign * (1.055 * Math.pow(abs, 1 / 2.4) - 0.055);
    }

    return 12.92 * val;
  });
}

/**
 * convert an array of linear-light sRGB values to CIE XYZ
 * using sRGB's own white, D65 (no chromatic adaptation)
 */
function lin_sRGB_to_XYZ(rgb) {
  const M = [
    [506752 / 1228815, 87881 / 245763, 12673 / 70218],
    [87098 / 409605, 175762 / 245763, 12673 / 175545],
    [7918 / 409605, 87881 / 737289, 1001167 / 1053270],
  ];

  return multiplyMatrices(M, rgb);
}

/**
 * convert XYZ to linear-light sRGB
 * using sRGB's own white, D65 (no chromatic adaptation)
 */
function XYZ_to_lin_sRGB(XYZ) {
  const M = [
    [12831 / 3959, -329 / 214, -1974 / 3959],
    [-851781 / 878810, 1648619 / 878810, 36519 / 878810],
    [705 / 12673, -2585 / 12673, 705 / 667],
  ];

  return multiplyMatrices(M, XYZ);
}

//  display-p3-related functions

/**
 * convert an array of display-p3 RGB values in the range 0.0 - 1.0
 * to linear light (un-companded) form.
 */
function lin_P3(RGB) {
  return lin_sRGB(RGB); // same as sRGB
}

/**
 * convert an array of linear-light display-p3 RGB  in the range 0.0-1.0
 * to gamma corrected form
 */
function gam_P3(RGB) {
  return gam_sRGB(RGB); // same as sRGB
}

/**
 * convert an array of linear-light display-p3 values to CIE XYZ
 * using display-p3's D65 (no chromatic adaptation)
 */
function lin_P3_to_XYZ(rgb) {
  const M = [
    [608311 / 1250200, 189793 / 714400, 198249 / 1000160],
    [35783 / 156275, 247089 / 357200, 198249 / 2500400],
    [0 / 1, 32229 / 714400, 5220557 / 5000800],
  ];

  return multiplyMatrices(M, rgb);
}

/**
 * convert XYZ to linear-light P3
 * using display-p3's own white, D65 (no chromatic adaptation)
 */
function XYZ_to_lin_P3(XYZ) {
  const M = [
    [446124 / 178915, -333277 / 357830, -72051 / 178915],
    [-14852 / 17905, 63121 / 35810, 423 / 17905],
    [11844 / 330415, -50337 / 660830, 316169 / 330415],
  ];

  return multiplyMatrices(M, XYZ);
}

/**
 * @returns the converted pixels in `{R: number, G: number, B: number, A: number}`.
 *
 * Follow conversion steps in CSS Color Module Level 4
 * https://drafts.csswg.org/css-color/#predefined-to-predefined
 * display-p3 and sRGB share the same white points.
 */
export function displayP3ToSrgb(pixel) {
  assert(
    pixel.R !== undefined && pixel.G !== undefined && pixel.B !== undefined,
    'color space conversion requires all of R, G and B components'
  );

  let rgbVec = [pixel.R, pixel.G, pixel.B];
  rgbVec = lin_P3(rgbVec);
  let rgbMatrix = [[rgbVec[0]], [rgbVec[1]], [rgbVec[2]]];
  rgbMatrix = XYZ_to_lin_sRGB(lin_P3_to_XYZ(rgbMatrix));
  rgbVec = [rgbMatrix[0][0], rgbMatrix[1][0], rgbMatrix[2][0]];
  rgbVec = gam_sRGB(rgbVec);

  pixel.R = rgbVec[0];
  pixel.G = rgbVec[1];
  pixel.B = rgbVec[2];

  return pixel;
}
/**
 * @returns the converted pixels in `{R: number, G: number, B: number, A: number}`.
 *
 * Follow conversion steps in CSS Color Module Level 4
 * https://drafts.csswg.org/css-color/#predefined-to-predefined
 * display-p3 and sRGB share the same white points.
 */
export function srgbToDisplayP3(pixel) {
  assert(
    pixel.R !== undefined && pixel.G !== undefined && pixel.B !== undefined,
    'color space conversion requires all of R, G and B components'
  );

  let rgbVec = [pixel.R, pixel.G, pixel.B];
  rgbVec = lin_sRGB(rgbVec);
  let rgbMatrix = [[rgbVec[0]], [rgbVec[1]], [rgbVec[2]]];
  rgbMatrix = XYZ_to_lin_P3(lin_sRGB_to_XYZ(rgbMatrix));
  rgbVec = [rgbMatrix[0][0], rgbMatrix[1][0], rgbMatrix[2][0]];
  rgbVec = gam_P3(rgbVec);

  pixel.R = rgbVec[0];
  pixel.G = rgbVec[1];
  pixel.B = rgbVec[2];

  return pixel;
}

/**
 * Returns a function which applies the specified colorspace/premultiplication conversion.
 * Does not clamp, so may return values outside of the `dstColorSpace` gamut, due to either
 * color space conversion or alpha premultiplication.
 */
export function makeInPlaceColorConversion({
  srcPremultiplied,
  dstPremultiplied,
  srcColorSpace = 'srgb',
  dstColorSpace = 'srgb',
}) {
  const requireColorSpaceConversion = srcColorSpace !== dstColorSpace;
  const requireUnpremultiplyAlpha =
    srcPremultiplied && (requireColorSpaceConversion || srcPremultiplied !== dstPremultiplied);
  const requirePremultiplyAlpha =
    dstPremultiplied && (requireColorSpaceConversion || srcPremultiplied !== dstPremultiplied);

  return rgba => {
    assert(rgba.A >= 0.0 && rgba.A <= 1.0, 'rgba.A out of bounds');

    if (requireUnpremultiplyAlpha) {
      if (rgba.A !== 0.0) {
        rgba.R /= rgba.A;
        rgba.G /= rgba.A;
        rgba.B /= rgba.A;
      } else {
        assert(
          rgba.R === 0.0 && rgba.G === 0.0 && rgba.B === 0.0 && rgba.A === 0.0,
          'Unpremultiply ops with alpha value 0.0 requires all channels equals to 0.0'
        );
      }
    }
    // It's possible RGB are now > 1.
    // This technically represents colors outside the src gamut, so no clamping yet.

    if (requireColorSpaceConversion) {
      // WebGPU currently only supports dstColorSpace = 'srgb'.
      if (srcColorSpace === 'display-p3' && dstColorSpace === 'srgb') {
        rgba = displayP3ToSrgb(rgba);
      } else {
        unreachable();
      }
    }
    // Now RGB may also be negative if the src gamut is larger than the dst gamut.

    if (requirePremultiplyAlpha) {
      rgba.R *= rgba.A;
      rgba.G *= rgba.A;
      rgba.B *= rgba.A;
    }
  };
}
