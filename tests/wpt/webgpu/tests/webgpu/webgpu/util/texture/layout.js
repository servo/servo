/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { assert, memcpy } from '../../../common/util/util.js';import { kTextureFormatInfo,
resolvePerAspectFormat } from


'../../format_info.js';
import { align } from '../math.js';
import { reifyExtent3D } from '../unions.js';

import { physicalMipSize, virtualMipSize } from './base.js';

/** The minimum `bytesPerRow` alignment, per spec. */
export const kBytesPerRowAlignment = 256;
/** The minimum buffer copy alignment, per spec. */
export const kBufferCopyAlignment = 4;

/**
 * Overridable layout options for {@link getTextureCopyLayout}.
 */







const kDefaultLayoutOptions = {
  mipLevel: 0,
  bytesPerRow: undefined,
  rowsPerImage: undefined,
  aspect: 'all'
};

/** The info returned by {@link getTextureSubCopyLayout}. */














/** The info returned by {@link getTextureCopyLayout}. */




/**
 * Computes layout information for a copy of the whole subresource at `mipLevel` of a GPUTexture
 * of size `baseSize` with the provided `format` and `dimension`.
 *
 * Computes default values for `bytesPerRow` and `rowsPerImage` if not specified.
 *
 * MAINTENANCE_TODO: Change input/output to Required<GPUExtent3DDict> for consistency.
 */
export function getTextureCopyLayout(
format,
dimension,
baseSize,
{ mipLevel, bytesPerRow, rowsPerImage, aspect } = kDefaultLayoutOptions)
{
  const mipSize = physicalMipSize(
    { width: baseSize[0], height: baseSize[1], depthOrArrayLayers: baseSize[2] },
    format,
    dimension,
    mipLevel
  );

  const layout = getTextureSubCopyLayout(format, mipSize, { bytesPerRow, rowsPerImage, aspect });
  return { ...layout, mipSize: [mipSize.width, mipSize.height, mipSize.depthOrArrayLayers] };
}

/**
 * Computes layout information for a copy of size `copySize` to/from a GPUTexture with the provided
 * `format`.
 *
 * Computes default values for `bytesPerRow` and `rowsPerImage` if not specified.
 */
export function getTextureSubCopyLayout(
format,
copySize,
{
  bytesPerRow,
  rowsPerImage,
  aspect = 'all'




} = {})
{
  format = resolvePerAspectFormat(format, aspect);
  const { blockWidth, blockHeight, bytesPerBlock } = kTextureFormatInfo[format];
  assert(bytesPerBlock !== undefined);

  const copySize_ = reifyExtent3D(copySize);
  assert(
    copySize_.width > 0 && copySize_.height > 0 && copySize_.depthOrArrayLayers > 0,
    'not implemented for empty copySize'
  );
  assert(
    copySize_.width % blockWidth === 0 && copySize_.height % blockHeight === 0,
    () =>
    `copySize (${copySize_.width},${copySize_.height}) must be a multiple of the block size (${blockWidth},${blockHeight})`
  );
  const copySizeBlocks = {
    width: copySize_.width / blockWidth,
    height: copySize_.height / blockHeight,
    depthOrArrayLayers: copySize_.depthOrArrayLayers
  };

  const minBytesPerRow = copySizeBlocks.width * bytesPerBlock;
  const alignedMinBytesPerRow = align(minBytesPerRow, kBytesPerRowAlignment);
  if (bytesPerRow !== undefined) {
    assert(bytesPerRow >= alignedMinBytesPerRow);
    assert(bytesPerRow % kBytesPerRowAlignment === 0);
  } else {
    bytesPerRow = alignedMinBytesPerRow;
  }

  if (rowsPerImage !== undefined) {
    assert(rowsPerImage >= copySizeBlocks.height);
  } else {
    rowsPerImage = copySizeBlocks.height;
  }

  const bytesPerSlice = bytesPerRow * rowsPerImage;
  const sliceSize =
  bytesPerRow * (copySizeBlocks.height - 1) + bytesPerBlock * copySizeBlocks.width;
  const byteLength = bytesPerSlice * (copySizeBlocks.depthOrArrayLayers - 1) + sliceSize;

  return {
    bytesPerBlock,
    byteLength: align(byteLength, kBufferCopyAlignment),
    minBytesPerRow,
    bytesPerRow,
    rowsPerImage
  };
}

/**
 * Fill an ArrayBuffer with the linear-memory representation of a solid-color
 * texture where every texel has the byte value `texelValue`.
 * Preserves the contents of `outputBuffer` which are in "padding" space between image rows.
 *
 * Effectively emulates a copyTextureToBuffer from a solid-color texture to a buffer.
 */
export function fillTextureDataWithTexelValue(
texelValue,
format,
dimension,
outputBuffer,
size,
options = kDefaultLayoutOptions)
{
  const { blockWidth, blockHeight, bytesPerBlock } = kTextureFormatInfo[format];
  // Block formats are not handled correctly below.
  assert(blockWidth === 1);
  assert(blockHeight === 1);

  assert(bytesPerBlock === texelValue.byteLength, 'texelValue must be of size bytesPerBlock');

  const { byteLength, rowsPerImage, bytesPerRow } = getTextureCopyLayout(
    format,
    dimension,
    size,
    options
  );

  assert(byteLength <= outputBuffer.byteLength);

  const mipSize = virtualMipSize(dimension, size, options.mipLevel);

  const outputTexelValueBytes = new Uint8Array(outputBuffer);
  for (let slice = 0; slice < mipSize[2]; ++slice) {
    for (let row = 0; row < mipSize[1]; row += blockHeight) {
      for (let col = 0; col < mipSize[0]; col += blockWidth) {
        const byteOffset =
        slice * rowsPerImage * bytesPerRow + row * bytesPerRow + col * texelValue.byteLength;
        memcpy({ src: texelValue }, { dst: outputTexelValueBytes, start: byteOffset });
      }
    }
  }
}

/**
 * Create a `COPY_SRC` GPUBuffer containing the linear-memory representation of a solid-color
 * texture where every texel has the byte value `texelValue`.
 */
export function createTextureUploadBuffer(
texelValue,
device,
format,
dimension,
size,
options = kDefaultLayoutOptions)




{
  const { byteLength, bytesPerRow, rowsPerImage, bytesPerBlock } = getTextureCopyLayout(
    format,
    dimension,
    size,
    options
  );

  const buffer = device.createBuffer({
    mappedAtCreation: true,
    size: byteLength,
    usage: GPUBufferUsage.COPY_SRC
  });
  const mapping = buffer.getMappedRange();

  assert(texelValue.byteLength === bytesPerBlock);
  fillTextureDataWithTexelValue(texelValue, format, dimension, mapping, size, options);
  buffer.unmap();

  return {
    buffer,
    bytesPerRow,
    rowsPerImage
  };
}


export const kImageCopyTypes = [
'WriteTexture',
'CopyB2T',
'CopyT2B'];


/**
 * Computes `bytesInACompleteRow` (as defined by the WebGPU spec) for image copies (B2T/T2B/writeTexture).
 */
export function bytesInACompleteRow(copyWidth, format) {
  const info = kTextureFormatInfo[format];
  assert(copyWidth % info.blockWidth === 0);
  return info.bytesPerBlock * copyWidth / info.blockWidth;
}

function validateBytesPerRow({
  bytesPerRow,
  bytesInLastRow,
  sizeInBlocks




}) {
  // If specified, layout.bytesPerRow must be greater than or equal to bytesInLastRow.
  if (bytesPerRow !== undefined && bytesPerRow < bytesInLastRow) {
    return false;
  }
  // If heightInBlocks > 1, layout.bytesPerRow must be specified.
  // If copyExtent.depthOrArrayLayers > 1, layout.bytesPerRow and layout.rowsPerImage must be specified.
  if (
  bytesPerRow === undefined && (
  sizeInBlocks.height > 1 || sizeInBlocks.depthOrArrayLayers > 1))
  {
    return false;
  }
  return true;
}

function validateRowsPerImage({
  rowsPerImage,
  sizeInBlocks



}) {
  // If specified, layout.rowsPerImage must be greater than or equal to heightInBlocks.
  if (rowsPerImage !== undefined && rowsPerImage < sizeInBlocks.height) {
    return false;
  }
  // If copyExtent.depthOrArrayLayers > 1, layout.bytesPerRow and layout.rowsPerImage must be specified.
  if (rowsPerImage === undefined && sizeInBlocks.depthOrArrayLayers > 1) {
    return false;
  }
  return true;
}








/**
 * Validate a copy and compute the number of bytes it needs. Throws if the copy is invalid.
 */
export function dataBytesForCopyOrFail(args) {
  const { minDataSizeOrOverestimate, copyValid } = dataBytesForCopyOrOverestimate(args);
  assert(copyValid, 'copy was invalid');
  return minDataSizeOrOverestimate;
}

/**
 * Validate a copy and compute the number of bytes it needs. If the copy is invalid, attempts to
 * "conservatively guess" (overestimate) the number of bytes that could be needed for a copy, even
 * if the copy parameters turn out to be invalid. This hopes to avoid "buffer too small" validation
 * errors when attempting to test other validation errors.
 */
export function dataBytesForCopyOrOverestimate({
  layout,
  format,
  copySize: copySize_,
  method
}) {
  const copyExtent = reifyExtent3D(copySize_);

  const info = kTextureFormatInfo[format];
  assert(copyExtent.width % info.blockWidth === 0);
  assert(copyExtent.height % info.blockHeight === 0);
  const sizeInBlocks = {
    width: copyExtent.width / info.blockWidth,
    height: copyExtent.height / info.blockHeight,
    depthOrArrayLayers: copyExtent.depthOrArrayLayers
  };
  const bytesInLastRow = sizeInBlocks.width * info.bytesPerBlock;

  let valid = true;
  const offset = layout.offset ?? 0;
  if (method !== 'WriteTexture') {
    if (offset % info.bytesPerBlock !== 0) valid = false;
    if (layout.bytesPerRow && layout.bytesPerRow % 256 !== 0) valid = false;
  }

  let requiredBytesInCopy = 0;
  {
    let { bytesPerRow, rowsPerImage } = layout;

    // If bytesPerRow or rowsPerImage is invalid, guess a value for the sake of various tests that
    // don't actually care about the exact value.
    // (In particular for validation tests that want to test invalid bytesPerRow or rowsPerImage but
    // need to make sure the total buffer size is still big enough.)
    if (!validateBytesPerRow({ bytesPerRow, bytesInLastRow, sizeInBlocks })) {
      bytesPerRow = undefined;
      valid = false;
    }
    if (!validateRowsPerImage({ rowsPerImage, sizeInBlocks })) {
      rowsPerImage = undefined;
      valid = false;
    }
    // Pick values for cases when (a) bpr/rpi was invalid or (b) they're validly undefined.
    bytesPerRow ??= align(info.bytesPerBlock * sizeInBlocks.width, 256);
    rowsPerImage ??= sizeInBlocks.height;

    if (copyExtent.depthOrArrayLayers > 1) {
      const bytesPerImage = bytesPerRow * rowsPerImage;
      const bytesBeforeLastImage = bytesPerImage * (copyExtent.depthOrArrayLayers - 1);
      requiredBytesInCopy += bytesBeforeLastImage;
    }
    if (copyExtent.depthOrArrayLayers > 0) {
      if (sizeInBlocks.height > 1) requiredBytesInCopy += bytesPerRow * (sizeInBlocks.height - 1);
      if (sizeInBlocks.height > 0) requiredBytesInCopy += bytesInLastRow;
    }
  }

  return { minDataSizeOrOverestimate: offset + requiredBytesInCopy, copyValid: valid };
}