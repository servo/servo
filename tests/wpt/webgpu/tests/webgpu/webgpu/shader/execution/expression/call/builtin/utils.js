/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { assert } from '../../../../../../common/util/util.js'; /* Valid types of Boundaries */















export function isBoundaryNegative(boundary) {
  return boundary.endsWith('min-wrap');
}

/**
 * Generates the boundary entries for the given number of dimensions
 *
 * @param numDimensions: The number of dimensions to generate for
 * @returns an array of generated coord boundaries
 */
export function generateCoordBoundaries(numDimensions) {
  const ret = ['in-bounds'];

  if (numDimensions < 1 || numDimensions > 3) {
    throw new Error(`invalid numDimensions: ${numDimensions}`);
  }

  const name = 'xyz';
  for (let i = 0; i < numDimensions; ++i) {
    for (const j of ['min', 'max']) {
      for (const k of ['wrap', 'boundary']) {
        ret.push(`${name[i]}-${j}-${k}`);
      }
    }
  }

  return ret;
}

/**
 * Generates a set of offset values to attempt in the range [-8, 7].
 *
 * @param numDimensions: The number of dimensions to generate for
 * @return an array of generated offset values
 */
export function generateOffsets(numDimensions) {
  assert(numDimensions >= 2 && numDimensions <= 3);
  const ret = [undefined];
  for (const val of [-8, 0, 1, 7]) {
    const v = [];
    for (let i = 0; i < numDimensions; ++i) {
      v.push(val);
    }
    ret.push(v);
  }
  return ret;
}