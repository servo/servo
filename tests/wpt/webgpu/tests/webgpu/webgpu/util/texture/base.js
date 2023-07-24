/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ import { assert, unreachable } from '../../../common/util/util.js';
import { kTextureFormatInfo } from '../../format_info.js';
import { align } from '../../util/math.js';
import { reifyExtent3D } from '../../util/unions.js';

/**
 * Compute the maximum mip level count allowed for a given texture size and texture dimension.
 */
export function maxMipLevelCount({ size, dimension = '2d' }) {
  const sizeDict = reifyExtent3D(size);

  let maxMippedDimension = 0;
  switch (dimension) {
    case '1d':
      maxMippedDimension = 1; // No mipmaps allowed.
      break;
    case '2d':
      maxMippedDimension = Math.max(sizeDict.width, sizeDict.height);
      break;
    case '3d':
      maxMippedDimension = Math.max(sizeDict.width, sizeDict.height, sizeDict.depthOrArrayLayers);
      break;
  }

  return Math.floor(Math.log2(maxMippedDimension)) + 1;
}

/**
 * Compute the "physical size" of a mip level: the size of the level, rounded up to a
 * multiple of the texel block size.
 */
export function physicalMipSize(baseSize, format, dimension, level) {
  switch (dimension) {
    case '1d':
      assert(level === 0, '1d textures cannot be mipmapped');
      assert(baseSize.height === 1 && baseSize.depthOrArrayLayers === 1, '1d texture not Wx1x1');
      return { width: baseSize.width, height: 1, depthOrArrayLayers: 1 };

    case '2d': {
      assert(
        Math.max(baseSize.width, baseSize.height) >> level > 0,
        () => `level (${level}) too large for base size (${baseSize.width}x${baseSize.height})`
      );

      const virtualWidthAtLevel = Math.max(baseSize.width >> level, 1);
      const virtualHeightAtLevel = Math.max(baseSize.height >> level, 1);
      const physicalWidthAtLevel = align(
        virtualWidthAtLevel,
        kTextureFormatInfo[format].blockWidth
      );

      const physicalHeightAtLevel = align(
        virtualHeightAtLevel,
        kTextureFormatInfo[format].blockHeight
      );

      return {
        width: physicalWidthAtLevel,
        height: physicalHeightAtLevel,
        depthOrArrayLayers: baseSize.depthOrArrayLayers,
      };
    }

    case '3d': {
      assert(
        Math.max(baseSize.width, baseSize.height, baseSize.depthOrArrayLayers) >> level > 0,
        () =>
          `level (${level}) too large for base size (${baseSize.width}x${baseSize.height}x${baseSize.depthOrArrayLayers})`
      );

      assert(
        kTextureFormatInfo[format].blockWidth === 1 && kTextureFormatInfo[format].blockHeight === 1,
        'not implemented for 3d block formats'
      );

      return {
        width: Math.max(baseSize.width >> level, 1),
        height: Math.max(baseSize.height >> level, 1),
        depthOrArrayLayers: Math.max(baseSize.depthOrArrayLayers >> level, 1),
      };
    }
  }
}

/**
 * Compute the "virtual size" of a mip level of a texture (not accounting for texel block rounding).
 *
 * MAINTENANCE_TODO: Change input/output to Required<GPUExtent3DDict> for consistency.
 */
export function virtualMipSize(dimension, size, mipLevel) {
  const shiftMinOne = n => Math.max(1, n >> mipLevel);
  switch (dimension) {
    case '1d':
      assert(size[2] === 1);
      return [shiftMinOne(size[0]), size[1], size[2]];
    case '2d':
      return [shiftMinOne(size[0]), shiftMinOne(size[1]), size[2]];
    case '3d':
      return [shiftMinOne(size[0]), shiftMinOne(size[1]), shiftMinOne(size[2])];
    default:
      unreachable();
  }
}

/**
 * Get texture dimension from view dimension in order to create an compatible texture for a given
 * view dimension.
 */
export function getTextureDimensionFromView(viewDimension) {
  switch (viewDimension) {
    case '1d':
      return '1d';
    case '2d':
    case '2d-array':
    case 'cube':
    case 'cube-array':
      return '2d';
    case '3d':
      return '3d';
    default:
      unreachable();
  }
}

/** Returns the possible valid view dimensions for a given texture dimension. */
export function viewDimensionsForTextureDimension(textureDimension) {
  switch (textureDimension) {
    case '1d':
      return ['1d'];
    case '2d':
      return ['2d', '2d-array', 'cube', 'cube-array'];
    case '3d':
      return ['3d'];
  }
}

/** Returns the default view dimension for a given texture descriptor. */
export function defaultViewDimensionsForTexture(textureDescriptor) {
  switch (textureDescriptor.dimension) {
    case '1d':
      return '1d';
    case '2d': {
      const sizeDict = reifyExtent3D(textureDescriptor.size);
      return sizeDict.depthOrArrayLayers > 1 ? '2d-array' : '2d';
    }
    case '3d':
      return '3d';
    default:
      unreachable();
  }
}

/** Reifies the optional fields of `GPUTextureDescriptor`.
 * MAINTENANCE_TODO: viewFormats should not be omitted here, but it seems likely that the
 * @webgpu/types definition will have to change before we can include it again.
 */
export function reifyTextureDescriptor(desc) {
  return { dimension: '2d', mipLevelCount: 1, sampleCount: 1, ...desc };
}

/** Reifies the optional fields of `GPUTextureViewDescriptor` (given a `GPUTextureDescriptor`). */
export function reifyTextureViewDescriptor(textureDescriptor, view) {
  const texture = reifyTextureDescriptor(textureDescriptor);

  // IDL defaulting

  const baseMipLevel = view.baseMipLevel ?? 0;
  const baseArrayLayer = view.baseArrayLayer ?? 0;
  const aspect = view.aspect ?? 'all';

  // Spec defaulting

  const format = view.format ?? texture.format;
  const mipLevelCount = view.mipLevelCount ?? texture.mipLevelCount - baseMipLevel;
  const dimension = view.dimension ?? defaultViewDimensionsForTexture(texture);

  let arrayLayerCount = view.arrayLayerCount;
  if (arrayLayerCount === undefined) {
    if (dimension === '2d-array' || dimension === 'cube-array') {
      arrayLayerCount = reifyExtent3D(texture.size).depthOrArrayLayers - baseArrayLayer;
    } else if (dimension === 'cube') {
      arrayLayerCount = 6;
    } else {
      arrayLayerCount = 1;
    }
  }

  return {
    format,
    dimension,
    aspect,
    baseMipLevel,
    mipLevelCount,
    baseArrayLayer,
    arrayLayerCount,
  };
}
