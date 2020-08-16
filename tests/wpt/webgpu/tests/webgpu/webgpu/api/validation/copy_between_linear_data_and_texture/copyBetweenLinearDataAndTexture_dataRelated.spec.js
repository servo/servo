/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = '';
import { params, poptions } from '../../../../common/framework/params_builder.js';
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import {
  kUncompressedTextureFormatInfo,
  kSizedTextureFormats,
  kSizedTextureFormatInfo,
} from '../../../capability_info.js';
import { align } from '../../../util/math.js';

import {
  CopyBetweenLinearDataAndTextureTest,
  kAllTestMethods,
  texelBlockAlignmentTestExpanderForOffset,
  texelBlockAlignmentTestExpanderForRowsPerImage,
  formatCopyableWithMethod,
} from './copyBetweenLinearDataAndTexture.js';

export const g = makeTestGroup(CopyBetweenLinearDataAndTextureTest);

g.test('bound_on_rows_per_image')
  .params(
    params()
      .combine(poptions('method', kAllTestMethods))
      .combine(poptions('rowsPerImageInBlocks', [0, 1, 2]))
      .combine(poptions('copyHeightInBlocks', [0, 1, 2]))
      .combine(poptions('copyDepth', [1, 3]))
  )
  .fn(async t => {
    const { rowsPerImageInBlocks, copyHeightInBlocks, copyDepth, method } = t.params;

    const format = 'rgba8unorm';
    const rowsPerImage = rowsPerImageInBlocks * kUncompressedTextureFormatInfo[format].blockHeight;
    const copyHeight = copyHeightInBlocks * kUncompressedTextureFormatInfo[format].blockHeight;

    const texture = t.device.createTexture({
      size: { width: 4, height: 4, depth: 3 },
      format,
      usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.COPY_DST,
    });

    // The WebGPU spec:
    // If layout.rowsPerImage is not 0, it must be greater than or equal to copyExtent.height.
    // If copyExtent.depth is greater than 1: layout.rowsPerImage must be greater than or equal to copyExtent.height.
    // TODO: Update this if https://github.com/gpuweb/gpuweb/issues/984 changes the spec.

    let success = true;
    if (rowsPerImage !== 0 && rowsPerImage < copyHeight) {
      success = false;
    }
    if (copyDepth > 1 && rowsPerImage < copyHeight) {
      success = false;
    }

    t.testRun(
      { texture },
      { bytesPerRow: 1024, rowsPerImage },
      { width: 0, height: copyHeight, depth: copyDepth },
      { dataSize: 1, method, success }
    );
  });

// Test with offset + requiredBytesIsCopy overflowing GPUSize64.
g.test('offset_plus_required_bytes_in_copy_overflow')
  .params(
    params()
      .combine(poptions('method', kAllTestMethods))
      .combine([
        { bytesPerRow: 2 ** 31, rowsPerImage: 2 ** 31, depth: 1, _success: true }, // success case
        { bytesPerRow: 2 ** 31, rowsPerImage: 2 ** 31, depth: 16, _success: false }, // bytesPerRow * rowsPerImage * (depth - 1) overflows.
      ])
  )
  .fn(async t => {
    const { method, bytesPerRow, rowsPerImage, depth, _success } = t.params;

    const texture = t.device.createTexture({
      size: [1, 1, depth],
      format: 'rgba8unorm',
      usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.COPY_DST,
    });

    t.testRun(
      { texture },
      { bytesPerRow, rowsPerImage },
      { width: 1, height: 1, depth },
      {
        dataSize: 10000,
        method,
        success: _success,
      }
    );
  });

// Testing that the minimal data size condition is checked correctly.
// In the success case, we test the exact value.
// In the failing case, we test the exact value minus 1.
g.test('required_bytes_in_copy')
  .params(
    params()
      .combine(poptions('method', kAllTestMethods))
      .combine([
        { bytesPerRowPadding: 0, rowsPerImagePaddingInBlocks: 0 }, // no padding
        { bytesPerRowPadding: 0, rowsPerImagePaddingInBlocks: 6 }, // rowsPerImage padding
        { bytesPerRowPadding: 6, rowsPerImagePaddingInBlocks: 0 }, // bytesPerRow padding
        { bytesPerRowPadding: 15, rowsPerImagePaddingInBlocks: 17 }, // both paddings
      ])
      .combine([
        { copyWidthInBlocks: 3, copyHeightInBlocks: 4, copyDepth: 5, offsetInBlocks: 0 }, // standard copy
        { copyWidthInBlocks: 5, copyHeightInBlocks: 4, copyDepth: 3, offsetInBlocks: 11 }, // standard copy, offset > 0
        { copyWidthInBlocks: 256, copyHeightInBlocks: 3, copyDepth: 2, offsetInBlocks: 0 }, // copyWidth is 256-aligned
        { copyWidthInBlocks: 0, copyHeightInBlocks: 4, copyDepth: 5, offsetInBlocks: 0 }, // empty copy because of width
        { copyWidthInBlocks: 3, copyHeightInBlocks: 0, copyDepth: 5, offsetInBlocks: 0 }, // empty copy because of height
        { copyWidthInBlocks: 3, copyHeightInBlocks: 4, copyDepth: 0, offsetInBlocks: 13 }, // empty copy because of depth, offset > 0
        { copyWidthInBlocks: 1, copyHeightInBlocks: 4, copyDepth: 5, offsetInBlocks: 0 }, // copyWidth = 1
        { copyWidthInBlocks: 3, copyHeightInBlocks: 1, copyDepth: 5, offsetInBlocks: 15 }, // copyHeight = 1, offset > 0
        { copyWidthInBlocks: 5, copyHeightInBlocks: 4, copyDepth: 1, offsetInBlocks: 0 }, // copyDepth = 1
        { copyWidthInBlocks: 7, copyHeightInBlocks: 1, copyDepth: 1, offsetInBlocks: 0 }, // copyHeight = 1 and copyDepth = 1
      ])
      .combine(poptions('format', kSizedTextureFormats))
      .filter(formatCopyableWithMethod)
  )
  .fn(async t => {
    const {
      offsetInBlocks,
      bytesPerRowPadding,
      rowsPerImagePaddingInBlocks,
      copyWidthInBlocks,
      copyHeightInBlocks,
      copyDepth,
      format,
      method,
    } = t.params;

    // In the CopyB2T and CopyT2B cases we need to have bytesPerRow 256-aligned,
    // to make this happen we align the bytesInACompleteRow value and multiply
    // bytesPerRowPadding by 256.
    const bytesPerRowAlignment = method === 'WriteTexture' ? 1 : 256;

    const info = kSizedTextureFormatInfo[format];
    const copyWidth = copyWidthInBlocks * info.blockWidth;
    const copyHeight = copyHeightInBlocks * info.blockHeight;
    const offset = offsetInBlocks * info.bytesPerBlock;
    const rowsPerImage = copyHeight + rowsPerImagePaddingInBlocks * info.blockHeight;
    const bytesPerRow =
      align(t.bytesInACompleteRow(copyWidth, format), bytesPerRowAlignment) +
      bytesPerRowPadding * bytesPerRowAlignment;
    const size = { width: copyWidth, height: copyHeight, depth: copyDepth };

    const minDataSize =
      offset + t.requiredBytesInCopy({ offset, bytesPerRow, rowsPerImage }, format, size);

    const texture = t.createAlignedTexture(format, size);

    t.testRun({ texture }, { offset, bytesPerRow, rowsPerImage }, size, {
      dataSize: minDataSize,
      method,
      success: true,
    });

    if (minDataSize > 0) {
      t.testRun({ texture }, { offset, bytesPerRow, rowsPerImage }, size, {
        dataSize: minDataSize - 1,
        method,
        success: false,
      });
    }
  });

g.test('texel_block_alignment_on_rows_per_image')
  .params(
    params()
      .combine(poptions('method', kAllTestMethods))
      .combine(poptions('format', kSizedTextureFormats))
      .filter(formatCopyableWithMethod)
      .expand(texelBlockAlignmentTestExpanderForRowsPerImage)
  )
  .fn(async t => {
    const { rowsPerImage, format, method } = t.params;
    const size = { width: 0, height: 0, depth: 0 };

    const texture = t.createAlignedTexture(format, size);

    const success = rowsPerImage % kSizedTextureFormatInfo[format].blockHeight === 0;

    t.testRun({ texture }, { bytesPerRow: 0, rowsPerImage }, size, {
      dataSize: 1,
      method,
      success,
    });
  });

// TODO: Update this if https://github.com/gpuweb/gpuweb/issues/985 changes the spec.
g.test('texel_block_alignment_on_offset')
  .params(
    params()
      .combine(poptions('method', kAllTestMethods))
      .combine(poptions('format', kSizedTextureFormats))
      .filter(formatCopyableWithMethod)
      .expand(texelBlockAlignmentTestExpanderForOffset)
  )
  .fn(async t => {
    const { format, offset, method } = t.params;
    const size = { width: 0, height: 0, depth: 0 };

    const texture = t.createAlignedTexture(format, size);

    const success = offset % kSizedTextureFormatInfo[format].bytesPerBlock === 0;

    t.testRun({ texture }, { offset, bytesPerRow: 0 }, size, { dataSize: offset, method, success });
  });

g.test('bound_on_bytes_per_row')
  .params(
    params()
      .combine(poptions('method', kAllTestMethods))
      .combine([
        { blocksPerRow: 2, additionalPaddingPerRow: 0, copyWidthInBlocks: 2 }, // success
        { blocksPerRow: 2, additionalPaddingPerRow: 5, copyWidthInBlocks: 3 }, // success if bytesPerBlock <= 5
        { blocksPerRow: 1, additionalPaddingPerRow: 0, copyWidthInBlocks: 2 }, // failure, bytesPerRow > 0
        { blocksPerRow: 0, additionalPaddingPerRow: 0, copyWidthInBlocks: 1 }, // failure, bytesPerRow = 0
      ])
      .combine([
        { copyHeightInBlocks: 0, copyDepth: 1 }, // we don't have to check the bound
        { copyHeightInBlocks: 1, copyDepth: 0 }, // we don't have to check the bound
        { copyHeightInBlocks: 2, copyDepth: 1 }, // we have to check the bound
        { copyHeightInBlocks: 0, copyDepth: 2 }, // we have to check the bound
      ])
      .combine(poptions('format', kSizedTextureFormats))
      .filter(formatCopyableWithMethod)
  )
  .fn(async t => {
    const {
      blocksPerRow,
      additionalPaddingPerRow,
      copyWidthInBlocks,
      copyHeightInBlocks,
      copyDepth,
      format,
      method,
    } = t.params;

    // In the CopyB2T and CopyT2B cases we need to have bytesPerRow 256-aligned,
    // to make this happen we multiply copyWidth and bytesPerRow by 256, so that
    // the appropriate inequalities still hold.
    const bytesPerRowAlignment = method === 'WriteTexture' ? 1 : 256;

    const info = kSizedTextureFormatInfo[format];
    const copyWidth = copyWidthInBlocks * info.blockWidth * bytesPerRowAlignment;
    const copyHeight = copyHeightInBlocks * info.blockHeight;
    const bytesPerRow =
      (blocksPerRow * info.bytesPerBlock + additionalPaddingPerRow) * bytesPerRowAlignment;
    const size = { width: copyWidth, height: copyHeight, depth: copyDepth };

    const texture = t.createAlignedTexture(format, size);

    let success = true;
    if (copyHeight > 1 || copyDepth > 1) {
      success = bytesPerRow >= t.bytesInACompleteRow(copyWidth, format);
    }

    t.testRun({ texture }, { bytesPerRow, rowsPerImage: copyHeight }, size, {
      dataSize: 1024,
      method,
      success,
    });
  });

g.test('bound_on_offset')
  .params(
    params()
      .combine(poptions('method', kAllTestMethods))
      .combine(poptions('offsetInBlocks', [0, 1, 2]))
      .combine(poptions('dataSizeInBlocks', [0, 1, 2]))
  )
  .fn(async t => {
    const { offsetInBlocks, dataSizeInBlocks, method } = t.params;

    const format = 'rgba8unorm';
    const info = kSizedTextureFormatInfo[format];
    const offset = offsetInBlocks * info.bytesPerBlock;
    const dataSize = dataSizeInBlocks * info.bytesPerBlock;

    const texture = t.device.createTexture({
      size: { width: 4, height: 4, depth: 1 },
      format,
      usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.COPY_DST,
    });

    const success = offset <= dataSize;

    t.testRun(
      { texture },
      { offset, bytesPerRow: 0 },
      { width: 0, height: 0, depth: 0 },
      { dataSize, method, success }
    );
  });
