/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `Validation tests for the linear data layout of linear data <-> texture copies

TODO check if the tests need to be updated to support aspects of depth-stencil textures`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { assert } from '../../../../common/util/util.js';
import { kTextureDimensions } from '../../../capability_info.js';
import {
  kTextureFormatInfo,
  kSizedTextureFormats,
  textureDimensionAndFormatCompatible } from
'../../../format_info.js';
import { align } from '../../../util/math.js';
import {
  bytesInACompleteRow,
  dataBytesForCopyOrOverestimate,
  dataBytesForCopyOrFail,
  kImageCopyTypes } from
'../../../util/texture/layout.js';

import {
  ImageCopyTest,
  texelBlockAlignmentTestExpanderForOffset,
  texelBlockAlignmentTestExpanderForRowsPerImage,
  formatCopyableWithMethod } from
'./image_copy.js';

export const g = makeTestGroup(ImageCopyTest);

g.test('bound_on_rows_per_image').
desc(
  `
Test that rowsPerImage must be at least the copy height (if defined).
- for various copy methods
- for all texture dimensions
- for various values of rowsPerImage including undefined
- for various copy heights
- for various copy depths
`
).
params((u) =>
u.
combine('method', kImageCopyTypes).
combineWithParams([
{ dimension: '1d', size: [4, 1, 1] },
{ dimension: '2d', size: [4, 4, 1] },
{ dimension: '2d', size: [4, 4, 3] },
{ dimension: '3d', size: [4, 4, 3] }]
).
beginSubcases().
combine('rowsPerImage', [undefined, 0, 1, 2, 1024]).
combine('copyHeightInBlocks', [0, 1, 2]).
combine('copyDepth', [1, 3]).
unless((p) => p.dimension === '1d' && p.copyHeightInBlocks !== 1).
unless((p) => p.copyDepth > p.size[2])
).
fn((t) => {
  const { rowsPerImage, copyHeightInBlocks, copyDepth, dimension, size, method } = t.params;

  const format = 'rgba8unorm';
  const copyHeight = copyHeightInBlocks * kTextureFormatInfo[format].blockHeight;

  const texture = t.createTextureTracked({
    size,
    dimension,
    format,
    usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.COPY_DST
  });

  const layout = { bytesPerRow: 1024, rowsPerImage };
  const copySize = { width: 0, height: copyHeight, depthOrArrayLayers: copyDepth };
  const { minDataSizeOrOverestimate, copyValid } = dataBytesForCopyOrOverestimate({
    layout,
    format,
    copySize,
    method
  });

  t.testRun({ texture }, layout, copySize, {
    dataSize: minDataSizeOrOverestimate,
    method,
    success: copyValid
  });
});

g.test('copy_end_overflows_u64').
desc(
  `
Test an error is produced when offset+requiredBytesInCopy overflows GPUSize64.
- for various copy methods
`
).
params((u) =>
u.
combine('method', kImageCopyTypes).
beginSubcases().
combineWithParams([
{ bytesPerRow: 2 ** 31, rowsPerImage: 2 ** 31, depthOrArrayLayers: 1, _success: true }, // success case
{ bytesPerRow: 2 ** 31, rowsPerImage: 2 ** 31, depthOrArrayLayers: 16, _success: false } // bytesPerRow * rowsPerImage * (depthOrArrayLayers - 1) overflows.
])
).
fn((t) => {
  const { method, bytesPerRow, rowsPerImage, depthOrArrayLayers, _success } = t.params;

  const texture = t.createTextureTracked({
    size: [1, 1, depthOrArrayLayers],
    format: 'rgba8unorm',
    usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.COPY_DST
  });

  t.testRun(
    { texture },
    { bytesPerRow, rowsPerImage },
    { width: 1, height: 1, depthOrArrayLayers },
    {
      dataSize: 10000,
      method,
      success: _success
    }
  );
});

g.test('required_bytes_in_copy').
desc(
  `
Test the computation of requiredBytesInCopy by computing the minimum data size for the copy and checking success/error at the boundary.
- for various copy methods
- for all formats
- for all dimensions
- for various extra bytesPerRow/rowsPerImage
- for various copy sizes
- for various offsets in the linear data
`
).
params((u) =>
u.
combine('method', kImageCopyTypes).
combine('format', kSizedTextureFormats).
filter(formatCopyableWithMethod).
combine('dimension', kTextureDimensions).
filter(({ dimension, format }) => textureDimensionAndFormatCompatible(dimension, format)).
beginSubcases().
combineWithParams([
{ bytesPerRowPadding: 0, rowsPerImagePaddingInBlocks: 0 }, // no padding
{ bytesPerRowPadding: 0, rowsPerImagePaddingInBlocks: 6 }, // rowsPerImage padding
{ bytesPerRowPadding: 6, rowsPerImagePaddingInBlocks: 0 }, // bytesPerRow padding
{ bytesPerRowPadding: 15, rowsPerImagePaddingInBlocks: 17 } // both paddings
]).
combineWithParams([
{ copyWidthInBlocks: 3, copyHeightInBlocks: 4, copyDepth: 5, _offsetMultiplier: 0 }, // standard copy
{ copyWidthInBlocks: 5, copyHeightInBlocks: 4, copyDepth: 3, _offsetMultiplier: 11 }, // standard copy, offset > 0
{ copyWidthInBlocks: 256, copyHeightInBlocks: 3, copyDepth: 2, _offsetMultiplier: 0 }, // copyWidth is 256-aligned
{ copyWidthInBlocks: 0, copyHeightInBlocks: 4, copyDepth: 5, _offsetMultiplier: 0 }, // empty copy because of width
{ copyWidthInBlocks: 3, copyHeightInBlocks: 0, copyDepth: 5, _offsetMultiplier: 0 }, // empty copy because of height
{ copyWidthInBlocks: 3, copyHeightInBlocks: 4, copyDepth: 0, _offsetMultiplier: 13 }, // empty copy because of depth, offset > 0
{ copyWidthInBlocks: 1, copyHeightInBlocks: 4, copyDepth: 5, _offsetMultiplier: 0 }, // copyWidth = 1
{ copyWidthInBlocks: 3, copyHeightInBlocks: 1, copyDepth: 5, _offsetMultiplier: 15 }, // copyHeight = 1, offset > 0
{ copyWidthInBlocks: 5, copyHeightInBlocks: 4, copyDepth: 1, _offsetMultiplier: 0 }, // copyDepth = 1
{ copyWidthInBlocks: 7, copyHeightInBlocks: 1, copyDepth: 1, _offsetMultiplier: 0 } // copyHeight = 1 and copyDepth = 1
])
// The test texture size will be rounded up from the copy size to the next valid texture size.
// If the format is a depth/stencil format, its copy size must equal to subresource's size.
// So filter out depth/stencil cases where the rounded-up texture size would be different from the copy size.
.filter(({ format, copyWidthInBlocks, copyHeightInBlocks, copyDepth }) => {
  const info = kTextureFormatInfo[format];
  return (
    !info.depth && !info.stencil ||
    copyWidthInBlocks > 0 && copyHeightInBlocks > 0 && copyDepth > 0);

}).
unless((p) => p.dimension === '1d' && (p.copyHeightInBlocks > 1 || p.copyDepth > 1)).
expand('offset', (p) => {
  const info = kTextureFormatInfo[p.format];
  if (info.depth || info.stencil) {
    return [p._offsetMultiplier * 4];
  }
  return [p._offsetMultiplier * info.color.bytes];
})
).
beforeAllSubcases((t) => {
  const info = kTextureFormatInfo[t.params.format];
  t.skipIfTextureFormatNotSupported(t.params.format);
  t.selectDeviceOrSkipTestCase(info.feature);
}).
fn((t) => {
  const {
    offset,
    bytesPerRowPadding,
    rowsPerImagePaddingInBlocks,
    copyWidthInBlocks,
    copyHeightInBlocks,
    copyDepth,
    format,
    dimension,
    method
  } = t.params;
  const info = kTextureFormatInfo[format];

  // In the CopyB2T and CopyT2B cases we need to have bytesPerRow 256-aligned,
  // to make this happen we align the bytesInACompleteRow value and multiply
  // bytesPerRowPadding by 256.
  const bytesPerRowAlignment = method === 'WriteTexture' ? 1 : 256;
  const copyWidth = copyWidthInBlocks * info.blockWidth;
  const copyHeight = copyHeightInBlocks * info.blockHeight;
  const rowsPerImage = copyHeight + rowsPerImagePaddingInBlocks * info.blockHeight;
  const bytesPerRow =
  align(bytesInACompleteRow(copyWidth, format), bytesPerRowAlignment) +
  bytesPerRowPadding * bytesPerRowAlignment;
  const copySize = { width: copyWidth, height: copyHeight, depthOrArrayLayers: copyDepth };

  const layout = { offset, bytesPerRow, rowsPerImage };
  const minDataSize = dataBytesForCopyOrFail({ layout, format, copySize, method });

  const texture = t.createAlignedTexture(format, copySize, undefined, dimension);

  t.testRun({ texture }, layout, copySize, {
    dataSize: minDataSize,
    method,
    success: true
  });

  if (minDataSize > 0) {
    t.testRun({ texture }, layout, copySize, {
      dataSize: minDataSize - 1,
      method,
      success: false
    });
  }
});

g.test('rows_per_image_alignment').
desc(
  `
Test that rowsPerImage has no alignment constraints.
- for various copy methods
- for all sized format
- for all dimensions
- for various rowsPerImage
`
).
params((u) =>
u.
combine('method', kImageCopyTypes).
combine('format', kSizedTextureFormats).
filter(formatCopyableWithMethod).
combine('dimension', kTextureDimensions).
filter(({ dimension, format }) => textureDimensionAndFormatCompatible(dimension, format)).
beginSubcases().
expand('rowsPerImage', texelBlockAlignmentTestExpanderForRowsPerImage)
// Copy height is info.blockHeight, so rowsPerImage must be equal or greater than it.
.filter(({ rowsPerImage, format }) => rowsPerImage >= kTextureFormatInfo[format].blockHeight)
).
beforeAllSubcases((t) => {
  const info = kTextureFormatInfo[t.params.format];
  t.skipIfTextureFormatNotSupported(t.params.format);
  t.selectDeviceOrSkipTestCase(info.feature);
}).
fn((t) => {
  const { rowsPerImage, format, method } = t.params;
  const info = kTextureFormatInfo[format];

  const size = { width: info.blockWidth, height: info.blockHeight, depthOrArrayLayers: 1 };
  const texture = t.createTextureTracked({
    size,
    format,
    usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.COPY_DST
  });

  t.testRun({ texture }, { bytesPerRow: 256, rowsPerImage }, size, {
    dataSize: info.bytesPerBlock,
    method,
    success: true
  });
});

g.test('offset_alignment').
desc(
  `
Test the alignment requirement on the linear data offset (block size, or 4 for depth-stencil).
- for various copy methods
- for all sized formats
- for all dimensions
- for various linear data offsets
`
).
params((u) =>
u.
combine('method', kImageCopyTypes).
combine('format', kSizedTextureFormats).
filter(formatCopyableWithMethod).
combine('dimension', kTextureDimensions).
filter(({ dimension, format }) => textureDimensionAndFormatCompatible(dimension, format)).
beginSubcases().
expand('offset', texelBlockAlignmentTestExpanderForOffset)
).
beforeAllSubcases((t) => {
  const info = kTextureFormatInfo[t.params.format];
  t.skipIfTextureFormatNotSupported(t.params.format);
  t.selectDeviceOrSkipTestCase(info.feature);
}).
fn((t) => {
  const { format, offset, method } = t.params;
  const info = kTextureFormatInfo[format];

  const size = { width: info.blockWidth, height: info.blockHeight, depthOrArrayLayers: 1 };
  const texture = t.createTextureTracked({
    size,
    format,
    usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.COPY_DST
  });

  let success = false;
  if (method === 'WriteTexture') success = true;
  if (info.depth || info.stencil) {
    if (offset % 4 === 0) success = true;
  } else {
    if (offset % info.color.bytes === 0) success = true;
  }

  t.testRun({ texture }, { offset, bytesPerRow: 256 }, size, {
    dataSize: offset + info.bytesPerBlock,
    method,
    success
  });
});

g.test('bound_on_bytes_per_row').
desc(
  `
Test that bytesPerRow, if specified must be big enough for a full copy row.
- for various copy methods
- for all sized formats
- for all dimension
- for various copy heights
- for various copy depths
- for various combinations of bytesPerRow and copy width.
`
).
params((u) =>
u.
combine('method', kImageCopyTypes).
combine('format', kSizedTextureFormats).
filter(formatCopyableWithMethod).
combine('dimension', kTextureDimensions).
filter(({ dimension, format }) => textureDimensionAndFormatCompatible(dimension, format)).
beginSubcases().
combine('copyHeightInBlocks', [1, 2]).
combine('copyDepth', [1, 2]).
unless((p) => p.dimension === '1d' && (p.copyHeightInBlocks > 1 || p.copyDepth > 1)).
expandWithParams((p) => {
  const info = kTextureFormatInfo[p.format];
  // We currently have a built-in assumption that for all formats, 128 % bytesPerBlock === 0.
  // This assumption ensures that all division below results in integers.
  assert(128 % info.bytesPerBlock === 0);
  return [
  // Copying exact fit with aligned bytesPerRow should work.
  {
    bytesPerRow: 256,
    widthInBlocks: 256 / info.bytesPerBlock,
    copyWidthInBlocks: 256 / info.bytesPerBlock,
    _success: true
  },
  // Copying into smaller texture when padding in bytesPerRow is enough should work unless
  // it is a depth/stencil typed format.
  {
    bytesPerRow: 256,
    widthInBlocks: 256 / info.bytesPerBlock,
    copyWidthInBlocks: 256 / info.bytesPerBlock - 1,
    _success: !(info.stencil || info.depth)
  },
  // Unaligned bytesPerRow should not work unless the method is 'WriteTexture'.
  {
    bytesPerRow: 128,
    widthInBlocks: 128 / info.bytesPerBlock,
    copyWidthInBlocks: 128 / info.bytesPerBlock,
    _success: p.method === 'WriteTexture'
  },
  {
    bytesPerRow: 384,
    widthInBlocks: 384 / info.bytesPerBlock,
    copyWidthInBlocks: 384 / info.bytesPerBlock,
    _success: p.method === 'WriteTexture'
  },
  // When bytesPerRow is smaller than bytesInLastRow copying should fail.
  {
    bytesPerRow: 256,
    widthInBlocks: 2 * 256 / info.bytesPerBlock,
    copyWidthInBlocks: 2 * 256 / info.bytesPerBlock,
    _success: false
  },
  // When copyHeightInBlocks > 1, bytesPerRow must be specified.
  {
    bytesPerRow: undefined,
    widthInBlocks: 256 / info.bytesPerBlock,
    copyWidthInBlocks: 256 / info.bytesPerBlock,
    _success: !(p.copyHeightInBlocks > 1 || p.copyDepth > 1)
  }];

})
).
beforeAllSubcases((t) => {
  const info = kTextureFormatInfo[t.params.format];
  t.skipIfTextureFormatNotSupported(t.params.format);
  t.selectDeviceOrSkipTestCase(info.feature);
}).
fn((t) => {
  const {
    method,
    format,
    bytesPerRow,
    widthInBlocks,
    copyWidthInBlocks,
    copyHeightInBlocks,
    copyDepth,
    _success
  } = t.params;
  const info = kTextureFormatInfo[format];

  // We create an aligned texture using the widthInBlocks which may be different from the
  // copyWidthInBlocks. This allows us to test scenarios where the two may be different.
  const texture = t.createAlignedTexture(format, {
    width: widthInBlocks * info.blockWidth,
    height: copyHeightInBlocks * info.blockHeight,
    depthOrArrayLayers: copyDepth
  });

  const layout = { bytesPerRow, rowsPerImage: copyHeightInBlocks };
  const copySize = {
    width: copyWidthInBlocks * info.blockWidth,
    height: copyHeightInBlocks * info.blockHeight,
    depthOrArrayLayers: copyDepth
  };
  const { minDataSizeOrOverestimate } = dataBytesForCopyOrOverestimate({
    layout,
    format,
    copySize,
    method
  });

  t.testRun({ texture }, layout, copySize, {
    dataSize: minDataSizeOrOverestimate,
    method,
    success: _success
  });
});

g.test('bound_on_offset').
desc(
  `
Test that the offset cannot be larger than the linear data size (even for an empty copy).
- for various offsets and data sizes
`
).
params((u) =>
u.
combine('method', kImageCopyTypes).
beginSubcases().
combine('offsetInBlocks', [0, 1, 2]).
combine('dataSizeInBlocks', [0, 1, 2])
).
fn((t) => {
  const { offsetInBlocks, dataSizeInBlocks, method } = t.params;

  const format = 'rgba8unorm';
  const info = kTextureFormatInfo[format];
  const offset = offsetInBlocks * info.color.bytes;
  const dataSize = dataSizeInBlocks * info.color.bytes;

  const texture = t.createTextureTracked({
    size: { width: 4, height: 4, depthOrArrayLayers: 1 },
    format,
    usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.COPY_DST
  });

  const success = offset <= dataSize;

  t.testRun(
    { texture },
    { offset, bytesPerRow: 0 },
    { width: 0, height: 0, depthOrArrayLayers: 0 },
    { dataSize, method, success }
  );
});