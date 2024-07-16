/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { assert, unreachable } from '../../../../../../common/util/util.js';import { virtualMipSize } from '../../../../../util/texture/base.js';
/* Valid types of Boundaries */















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



export function getMipLevelFromLevelSpec(mipLevelCount, levelSpec) {
  switch (levelSpec) {
    case -1:
      return -1;
    case 0:
      return 0;
    case 'numLevels':
      return mipLevelCount;
    case 'numLevels-1':
      return mipLevelCount - 1;
    default:
      unreachable();
  }
}

export function isLevelSpecNegative(levelSpec) {
  return levelSpec === -1;
}

function getCoordForSize(size, boundary) {
  const coord = size.map((v) => Math.floor(v / 2));
  switch (boundary) {
    case 'in-bounds':
      break;
    default:{
        const axis = boundary[0];
        const axisIndex = axis.charCodeAt(0) - 'x'.charCodeAt(0);
        const axisSize = size[axisIndex];
        const location = boundary.substring(2);
        let v = 0;
        switch (location) {
          case 'min-wrap':
            v = -1;
            break;
          case 'min-boundary':
            v = 0;
            break;
          case 'max-wrap':
            v = axisSize;
            break;
          case 'max-boundary':
            v = axisSize - 1;
            break;
          default:
            unreachable();
        }
        coord[axisIndex] = v;
      }
  }
  return coord;
}

function getNumDimensions(dimension) {
  switch (dimension) {
    case '1d':
      return 1;
    case '2d':
      return 2;
    case '3d':
      return 3;
  }
}

export function getCoordinateForBoundaries(
texture,
mipLevel,
boundary)
{
  const size = virtualMipSize(texture.dimension, texture, mipLevel);
  const coord = getCoordForSize(size, boundary);
  return coord.slice(0, getNumDimensions(texture.dimension));
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