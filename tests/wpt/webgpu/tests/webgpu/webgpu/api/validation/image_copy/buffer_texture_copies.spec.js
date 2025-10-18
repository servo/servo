/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
copyTextureToBuffer and copyBufferToTexture validation tests not covered by
the general image_copy tests, or by destroyed,*.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { assert, unreachable } from '../../../../common/util/util.js';
import { kBufferUsages, kTextureDimensions, kTextureUsages } from '../../../capability_info.js';
import { GPUConst } from '../../../constants.js';
import {
  kDepthStencilFormats,
  depthStencilBufferTextureCopySupported,
  depthStencilFormatAspectSize,
  kColorTextureFormats,
  canCopyFromAllAspectsOfTextureFormat,
  canCopyToAllAspectsOfTextureFormat,
  textureFormatAndDimensionPossiblyCompatible,
  getBlockInfoForColorTextureFormat } from
'../../../format_info.js';
import { AllFeaturesMaxLimitsGPUTest } from '../../../gpu_test.js';
import { align } from '../../../util/math.js';
import { kBufferCopyAlignment, kBytesPerRowAlignment } from '../../../util/texture/layout.js';

class ImageCopyTest extends AllFeaturesMaxLimitsGPUTest {
  testCopyBufferToTexture(
  source,
  destination,
  copySize,
  isSuccess)
  {
    const { encoder, validateFinishAndSubmit } = this.createEncoder('non-pass');
    encoder.copyBufferToTexture(source, destination, copySize);
    validateFinishAndSubmit(isSuccess, true);
  }

  testCopyTextureToBuffer(
  source,
  destination,
  copySize,
  isSuccess)
  {
    const { encoder, validateFinishAndSubmit } = this.createEncoder('non-pass');
    encoder.copyTextureToBuffer(source, destination, copySize);
    validateFinishAndSubmit(isSuccess, true);
  }

  testWriteTexture(
  destination,
  uploadData,
  dataLayout,
  copySize,
  isSuccess)
  {
    this.expectGPUError(
      'validation',
      () => this.queue.writeTexture(destination, uploadData, dataLayout, copySize),
      !isSuccess
    );
  }
}

export const g = makeTestGroup(ImageCopyTest);

g.test('depth_stencil_format,copy_usage_and_aspect').
desc(
  `
  Validate the combination of usage and aspect of each depth stencil format in copyBufferToTexture,
  copyTextureToBuffer and writeTexture. See https://gpuweb.github.io/gpuweb/#depth-formats for more
  details.
  `
).
params((u) =>
u //
.combine('format', kDepthStencilFormats).
beginSubcases().
combine('aspect', ['all', 'depth-only', 'stencil-only'])
).
fn((t) => {
  const { format, aspect } = t.params;
  t.skipIfTextureFormatNotSupported(format);

  const textureSize = { width: 1, height: 1, depthOrArrayLayers: 1 };
  const texture = t.createTextureTracked({
    size: textureSize,
    format,
    usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.COPY_DST
  });

  const uploadBufferSize = 32;
  const buffer = t.createBufferTracked({
    size: uploadBufferSize,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  });

  {
    const success = depthStencilBufferTextureCopySupported('CopyB2T', format, aspect);
    t.testCopyBufferToTexture({ buffer }, { texture, aspect }, textureSize, success);
  }

  {
    const success = depthStencilBufferTextureCopySupported('CopyT2B', format, aspect);
    t.testCopyTextureToBuffer({ texture, aspect }, { buffer }, textureSize, success);
  }

  {
    const success = depthStencilBufferTextureCopySupported('WriteTexture', format, aspect);
    const uploadData = new Uint8Array(uploadBufferSize);
    t.testWriteTexture({ texture, aspect }, uploadData, {}, textureSize, success);
  }
});

g.test('depth_stencil_format,copy_buffer_size').
desc(
  `
  Validate the minimum buffer size for each depth stencil format in copyBufferToTexture,
  copyTextureToBuffer and writeTexture.

  Given a depth stencil format, a copy aspect ('depth-only' or 'stencil-only'), the copy method
  (buffer-to-texture or texture-to-buffer) and the copy size, validate
  - if the copy can be successfully executed with the minimum required buffer size.
  - if the copy fails with a validation error when the buffer size is less than the minimum
  required buffer size.
  `
).
params((u) =>
u.
combine('format', kDepthStencilFormats).
combine('aspect', ['depth-only', 'stencil-only']).
combine('copyType', ['CopyB2T', 'CopyT2B', 'WriteTexture']).
filter((param) =>
depthStencilBufferTextureCopySupported(param.copyType, param.format, param.aspect)
).
beginSubcases().
combine('copySize', [
{ width: 8, height: 1, depthOrArrayLayers: 1 },
{ width: 4, height: 4, depthOrArrayLayers: 1 },
{ width: 4, height: 4, depthOrArrayLayers: 3 }]
)
).
fn((t) => {
  const { format, aspect, copyType, copySize } = t.params;
  t.skipIfTextureFormatNotSupported(format);

  const texture = t.createTextureTracked({
    size: copySize,
    format,
    usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.COPY_DST
  });

  const texelAspectSize = depthStencilFormatAspectSize(format, aspect);
  assert(texelAspectSize > 0);

  const bytesPerRowAlignment = copyType === 'WriteTexture' ? 1 : kBytesPerRowAlignment;
  const bytesPerRow = align(texelAspectSize * copySize.width, bytesPerRowAlignment);
  const rowsPerImage = copySize.height;
  const minimumBufferSize =
  bytesPerRow * (rowsPerImage * copySize.depthOrArrayLayers - 1) +
  align(texelAspectSize * copySize.width, kBufferCopyAlignment);
  assert(minimumBufferSize > kBufferCopyAlignment);

  const bigEnoughBuffer = t.createBufferTracked({
    size: minimumBufferSize,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  });
  const smallerBuffer = t.createBufferTracked({
    size: minimumBufferSize - kBufferCopyAlignment,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  });

  if (copyType === 'CopyB2T') {
    t.testCopyBufferToTexture(
      { buffer: bigEnoughBuffer, bytesPerRow, rowsPerImage },
      { texture, aspect },
      copySize,
      true
    );
    t.testCopyBufferToTexture(
      { buffer: smallerBuffer, bytesPerRow, rowsPerImage },
      { texture, aspect },
      copySize,
      false
    );
  } else if (copyType === 'CopyT2B') {
    t.testCopyTextureToBuffer(
      { texture, aspect },
      { buffer: bigEnoughBuffer, bytesPerRow, rowsPerImage },
      copySize,
      true
    );
    t.testCopyTextureToBuffer(
      { texture, aspect },
      { buffer: smallerBuffer, bytesPerRow, rowsPerImage },
      copySize,
      false
    );
  } else if (copyType === 'WriteTexture') {
    const enoughUploadData = new Uint8Array(minimumBufferSize);
    const smallerUploadData = new Uint8Array(minimumBufferSize - kBufferCopyAlignment);
    t.testWriteTexture(
      { texture, aspect },
      enoughUploadData,
      {
        bytesPerRow,
        rowsPerImage
      },
      copySize,
      true
    );

    t.testWriteTexture(
      { texture, aspect },
      smallerUploadData,
      {
        bytesPerRow,
        rowsPerImage
      },
      copySize,
      false
    );
  } else {
    unreachable();
  }
});

g.test('depth_stencil_format,copy_buffer_offset').
desc(
  `
    Validate for every depth stencil formats the buffer offset must be a multiple of 4 in
    copyBufferToTexture() and copyTextureToBuffer(), but the offset in writeTexture() doesn't always
    need to be a multiple of 4.
    `
).
params((u) =>
u.
combine('format', kDepthStencilFormats).
combine('aspect', ['depth-only', 'stencil-only']).
combine('copyType', ['CopyB2T', 'CopyT2B', 'WriteTexture']).
filter((param) =>
depthStencilBufferTextureCopySupported(param.copyType, param.format, param.aspect)
).
beginSubcases().
combine('offset', [1, 2, 4, 6, 8])
).
fn((t) => {
  const { format, aspect, copyType, offset } = t.params;
  t.skipIfTextureFormatNotSupported(format);

  const textureSize = { width: 4, height: 4, depthOrArrayLayers: 1 };

  const texture = t.createTextureTracked({
    size: textureSize,
    format,
    usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.COPY_DST
  });

  const texelAspectSize = depthStencilFormatAspectSize(format, aspect);
  assert(texelAspectSize > 0);

  const bytesPerRowAlignment = copyType === 'WriteTexture' ? 1 : kBytesPerRowAlignment;
  const bytesPerRow = align(texelAspectSize * textureSize.width, bytesPerRowAlignment);
  const rowsPerImage = textureSize.height;
  const minimumBufferSize =
  bytesPerRow * (rowsPerImage * textureSize.depthOrArrayLayers - 1) +
  align(texelAspectSize * textureSize.width, kBufferCopyAlignment);
  assert(minimumBufferSize > kBufferCopyAlignment);

  const buffer = t.createBufferTracked({
    size: align(minimumBufferSize + offset, kBufferCopyAlignment),
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  });

  const isSuccess = copyType === 'WriteTexture' ? true : offset % 4 === 0;

  if (copyType === 'CopyB2T') {
    t.testCopyBufferToTexture(
      { buffer, offset, bytesPerRow, rowsPerImage },
      { texture, aspect },
      textureSize,
      isSuccess
    );
  } else if (copyType === 'CopyT2B') {
    t.testCopyTextureToBuffer(
      { texture, aspect },
      { buffer, offset, bytesPerRow, rowsPerImage },
      textureSize,
      isSuccess
    );
  } else if (copyType === 'WriteTexture') {
    const uploadData = new Uint8Array(minimumBufferSize + offset);
    t.testWriteTexture(
      { texture, aspect },
      uploadData,
      {
        offset,
        bytesPerRow,
        rowsPerImage
      },
      textureSize,
      isSuccess
    );
  } else {
    unreachable();
  }
});

g.test('sample_count').
desc(
  `
  Test that the texture sample count. Check that a validation error is generated if sample count is
  not 1.
  `
).
params((u) =>
u //
// writeTexture is handled by writeTexture.spec.ts.
.combine('copyType', ['CopyB2T', 'CopyT2B']).
beginSubcases().
combine('sampleCount', [1, 4])
).
fn((t) => {
  const { sampleCount, copyType } = t.params;

  let usage = GPUTextureUsage.COPY_SRC | GPUTextureUsage.COPY_DST;
  // WebGPU SPEC requires multisampled textures must have RENDER_ATTACHMENT usage.
  if (sampleCount > 1) {
    usage |= GPUTextureUsage.RENDER_ATTACHMENT;
  }

  const textureSize = { width: 16, height: 1, depthOrArrayLayers: 1 };
  const texture = t.createTextureTracked({
    size: textureSize,
    sampleCount,
    format: 'bgra8unorm',
    usage
  });

  const uploadBufferSize = 64;
  const buffer = t.createBufferTracked({
    size: uploadBufferSize,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  });

  const isSuccess = sampleCount === 1;

  if (copyType === 'CopyB2T') {
    t.testCopyBufferToTexture({ buffer }, { texture }, textureSize, isSuccess);
  } else if (copyType === 'CopyT2B') {
    t.testCopyTextureToBuffer({ texture }, { buffer }, textureSize, isSuccess);
  }
});

const kRequiredTextureUsage = {
  CopyT2B: GPUConst.TextureUsage.COPY_SRC,
  CopyB2T: GPUConst.TextureUsage.COPY_DST
};
const kRequiredBufferUsage = {
  CopyB2T: GPUConst.BufferUsage.COPY_SRC,
  CopyT2B: GPUConst.BufferUsage.COPY_DST
};

g.test('texture_buffer_usages').
desc(
  `
  Tests calling copyTextureToBuffer or copyBufferToTexture with the texture and the buffer missed
  COPY_SRC, COPY_DST usage respectively.
    - texture and buffer {with, without} COPY_SRC and COPY_DST usage.
  `
).
params((u) =>
u //
.combine('copyType', ['CopyB2T', 'CopyT2B']).
beginSubcases().
combine('textureUsage', kTextureUsages).
expand('_textureUsageValid', (p) => [p.textureUsage === kRequiredTextureUsage[p.copyType]]).
combine('bufferUsage', kBufferUsages).
expand('_bufferUsageValid', (p) => [p.bufferUsage === kRequiredBufferUsage[p.copyType]]).
filter((p) => p._textureUsageValid || p._bufferUsageValid)
).
fn((t) => {
  const { copyType, textureUsage, _textureUsageValid, bufferUsage, _bufferUsageValid } = t.params;

  const texture = t.createTextureTracked({
    size: { width: 16, height: 16 },
    format: 'rgba8unorm',
    usage: textureUsage
  });

  const uploadBufferSize = 32;
  const buffer = t.createBufferTracked({
    size: uploadBufferSize,
    usage: bufferUsage
  });

  const textureSize = { width: 1, height: 1, depthOrArrayLayers: 1 };

  const isSuccess = _textureUsageValid && _bufferUsageValid;
  if (copyType === 'CopyB2T') {
    t.testCopyBufferToTexture({ buffer }, { texture }, textureSize, isSuccess);
  } else if (copyType === 'CopyT2B') {
    t.testCopyTextureToBuffer({ texture }, { buffer }, textureSize, isSuccess);
  }
});

g.test('device_mismatch').
desc(
  `
    Tests copyBufferToTexture and copyTextureToBuffer cannot be called with a buffer or a texture
    created from another device.
  `
).
params((u) =>
u //
.combine('copyType', ['CopyB2T', 'CopyT2B']).
beginSubcases().
combineWithParams([
{ bufMismatched: false, texMismatched: false }, // control case
{ bufMismatched: true, texMismatched: false },
{ bufMismatched: false, texMismatched: true }]
)
).
beforeAllSubcases((t) => t.usesMismatchedDevice()).
fn((t) => {
  const { copyType, bufMismatched, texMismatched } = t.params;

  const uploadBufferSize = 32;
  const buffer = t.trackForCleanup(
    (bufMismatched ? t.mismatchedDevice : t.device).createBuffer({
      size: uploadBufferSize,
      usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
    })
  );

  const textureSize = { width: 1, height: 1, depthOrArrayLayers: 1 };
  const texture = t.trackForCleanup(
    (texMismatched ? t.mismatchedDevice : t.device).createTexture({
      size: textureSize,
      format: 'rgba8unorm',
      usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.COPY_DST
    })
  );

  const isValid = !bufMismatched && !texMismatched;

  if (copyType === 'CopyB2T') {
    t.testCopyBufferToTexture({ buffer }, { texture }, textureSize, isValid);
  } else if (copyType === 'CopyT2B') {
    t.testCopyTextureToBuffer({ texture }, { buffer }, textureSize, isValid);
  }
});

g.test('offset_and_bytesPerRow').
desc(
  `Test that for copyBufferToTexture, and copyTextureToBuffer
     * bytesPerRow must be a multiple of 256
     * offset must be a multiple of bytesPerBlock
     * the last row does not need to be a multiple of 256
       In other words, If the copy size is 4x2 of a r8unorm texture that's 4 bytes per row.
       To get from row 0 to row 1 in the buffer, bytesPerRow must be a multiple of 256.
       But, the size requirement for the buffer is only 256 + 4, not 256 * 2
     * origin.x must be a multiple of blockWidth
     * origin.y must be a multiple of blockHeight
     * copySize.width must be a multiple of blockWidth
     * copySize.height must be a multiple of blockHeight
`
).
params((u) =>
u.
combine('format', kColorTextureFormats).
combine('copyType', ['CopyB2T', 'CopyT2B']).
filter(
  ({ format }) =>
  canCopyToAllAspectsOfTextureFormat(format) && canCopyFromAllAspectsOfTextureFormat(format)
).
combine('dimension', kTextureDimensions).
filter(({ dimension, format }) =>
textureFormatAndDimensionPossiblyCompatible(dimension, format)
).
beginSubcases().
combineWithParams(
  [
  { xInBlocks: 1, yInBlocks: 1, copyWidthInBlocks: 64, copyHeightInBlocks: 2, offsetInBlocks: 1, bytesPerRowAlign: 256 }, // good
  { xInBlocks: 0, yInBlocks: 0, copyWidthInBlocks: 64, copyHeightInBlocks: 2, offsetInBlocks: 1.5, bytesPerRowAlign: 256 }, // bad as offset is not blockSize
  { xInBlocks: 0, yInBlocks: 0, copyWidthInBlocks: 64, copyHeightInBlocks: 2, offsetInBlocks: 0, bytesPerRowAlign: 128 }, // bad as bytesPerBlock is not multiple of 256
  { xInBlocks: 0, yInBlocks: 0, copyWidthInBlocks: 64, copyHeightInBlocks: 2, offsetInBlocks: 0, bytesPerRowAlign: 384 }, // bad as bytesPerBlock is not multiple of 256
  { xInBlocks: 1.5, yInBlocks: 0, copyWidthInBlocks: 64, copyHeightInBlocks: 2, offsetInBlocks: 0, bytesPerRowAlign: 256 }, // bad as origin.x is not multiple of blockSize
  { xInBlocks: 0, yInBlocks: 1.5, copyWidthInBlocks: 64, copyHeightInBlocks: 2, offsetInBlocks: 0, bytesPerRowAlign: 256 }, // bad as origin.y is not multiple of blockSize
  { xInBlocks: 0, yInBlocks: 0, copyWidthInBlocks: 64.5, copyHeightInBlocks: 2, offsetInBlocks: 0, bytesPerRowAlign: 256 }, // bad as copySize.width is not multiple of blockSize
  { xInBlocks: 0, yInBlocks: 0, copyWidthInBlocks: 64, copyHeightInBlocks: 2.5, offsetInBlocks: 0, bytesPerRowAlign: 256 } // bad as copySize.height is not multiple of blockSize
  ]
)
// Remove non-integer offsetInBlocks, copyWidthInBlocks, copyHeightInBlocks if bytesPerBlock === 1
.unless(
  (t) =>
  (t.offsetInBlocks % 1 !== 0 ||
  t.copyWidthInBlocks % 1 !== 0 ||
  t.copyHeightInBlocks % 1 !== 0) &&
  getBlockInfoForColorTextureFormat(t.format).bytesPerBlock > 1
)
// Remove yInBlocks > 0 if dimension is 1d
.unless((t) => t.dimension === '1d' && t.yInBlocks > 0)
).
fn((t) => {
  const {
    copyType,
    format,
    dimension,
    xInBlocks,
    yInBlocks,
    offsetInBlocks,
    copyWidthInBlocks,
    copyHeightInBlocks,
    bytesPerRowAlign
  } = t.params;
  t.skipIfTextureFormatNotSupported(format);
  t.skipIfTextureFormatAndDimensionNotCompatible(format, dimension);
  if (copyType === 'CopyT2B') {
    t.skipIfTextureFormatDoesNotSupportCopyTextureToBuffer(format);
  }

  const info = getBlockInfoForColorTextureFormat(format);

  // make a texture big enough that we have room for our copySize and our origin.
  // Note that xxxInBlocks may be factional so that we test origins and sizes not aligned to blocks.
  const widthBlocks = Math.ceil(xInBlocks) + Math.ceil(copyWidthInBlocks);
  const heightBlocks = Math.ceil(yInBlocks) + Math.ceil(copyHeightInBlocks);
  let copySizeBlocks = [copyWidthInBlocks, copyHeightInBlocks, 1];
  let texSizeBlocks = [widthBlocks, heightBlocks, 1];
  if (dimension === '1d') {
    copySizeBlocks = [copyWidthInBlocks, 1, 1];
    texSizeBlocks = [widthBlocks, 1, 1];
  }

  const origin = [
  Math.ceil(xInBlocks * info.blockWidth),
  Math.ceil(yInBlocks * info.blockHeight),
  0];

  const copySize = [
  Math.ceil(copySizeBlocks[0] * info.blockWidth),
  Math.ceil(copySizeBlocks[1] * info.blockHeight),
  copySizeBlocks[2]];

  const textureSize = [
  texSizeBlocks[0] * info.blockWidth,
  texSizeBlocks[1] * info.blockHeight,
  texSizeBlocks[2]];

  const textureBytePerRow = info.bytesPerBlock * texSizeBlocks[0];
  const rowsPerImage = Math.ceil(copySizeBlocks[1]);
  const offset = Math.ceil(offsetInBlocks * info.bytesPerBlock);
  const bytesPerRow = align(textureBytePerRow, bytesPerRowAlign);

  // Make sure our buffer is big enough for the required alignment
  // and offset but no bigger.
  const totalRows = rowsPerImage * copySizeBlocks[2];
  const bufferSize = offset + (totalRows - 1) * bytesPerRow + textureBytePerRow;

  const buffer = t.createBufferTracked({
    label: `buffer(${bufferSize})`,
    size: bufferSize,
    usage: copyType === 'CopyB2T' ? GPUBufferUsage.COPY_SRC : GPUBufferUsage.COPY_DST
  });

  const texture = t.createTextureTracked({
    size: textureSize,
    format,
    dimension,
    usage: copyType === 'CopyB2T' ? GPUTextureUsage.COPY_DST : GPUTextureUsage.COPY_SRC
  });

  const shouldSucceed =
  offset % info.bytesPerBlock === 0 &&
  bytesPerRow % 256 === 0 &&
  origin[0] % info.blockWidth === 0 &&
  origin[1] % info.blockHeight === 0 &&
  copySize[0] % info.blockWidth === 0 &&
  copySize[1] % info.blockHeight === 0;

  t.debug(
    () =>
    `offset: ${offset}, bytesPerRow: ${bytesPerRow}, copySize: ${copySize}, origin: ${origin}`
  );

  switch (copyType) {
    case 'CopyB2T':{
        t.testCopyBufferToTexture(
          { buffer, offset, bytesPerRow },
          { texture, origin },
          copySize,
          shouldSucceed
        );
        break;
      }
    case 'CopyT2B':{
        t.testCopyTextureToBuffer(
          { texture, origin },
          { buffer, offset, bytesPerRow },
          copySize,
          shouldSucceed
        );
        break;
      }
  }
});