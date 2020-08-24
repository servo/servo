/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `writeTexture + copyBufferToTexture + copyTextureToBuffer operation tests.

* copy_with_various_rows_per_image_and_bytes_per_row: test that copying data with various bytesPerRow (including { ==, > } bytesInACompleteRow) and\
 rowsPerImage (including { ==, > } copyExtent.height) values and minimum required bytes in copy works for every format. Also covers special code paths:
  - bufferSize - offset < bytesPerImage * copyExtent.depth
  - when bytesPerRow is not a multiple of 512 and copyExtent.depth > 1: copyExtent.depth % 2 == { 0, 1 }
  - bytesPerRow == bytesInACompleteCopyImage

* copy_with_various_offsets_and_data_sizes: test that copying data with various offset (including { ==, > } 0 and is/isn't power of 2) values and additional\
 data paddings works for every format with 2d and 2d-array textures. Also covers special code paths:
  - offset + bytesInCopyExtentPerRow { ==, > } bytesPerRow
  - offset > bytesInACompleteCopyImage

* copy_with_various_origins_and_copy_extents: test that copying slices of a texture works with various origin (including { origin.x, origin.y, origin.z }\
 { ==, > } 0 and is/isn't power of 2) and copyExtent (including { copyExtent.x, copyExtent.y, copyExtent.z } { ==, > } 0 and is/isn't power of 2) values\
 (also including {origin._ + copyExtent._ { ==, < } the subresource size of textureCopyView) works for all formats. origin and copyExtent values are passed\
 as [number, number, number] instead of GPUExtent3DDict.

* copy_various_mip_levels: test that copying various mip levels works for all formats. Also covers special code paths:
  - the physical size of the subresouce is not equal to the logical size
  - bufferSize - offset < bytesPerImage * copyExtent.depth and copyExtent needs to be clamped

* copy_with_no_image_or_slice_padding_and_undefined_values: test that when copying a single row we can set any bytesPerRow value and when copying a single\
 slice we can set rowsPerImage to 0. Also test setting offset, rowsPerImage, mipLevel, origin, origin.{x,y,z} to undefined.

* TODO:
  - add another initMethod which renders the texture
  - because of expectContests 4-bytes alignment we don't test CopyT2B with buffer size not divisible by 4
  - add tests for 1d / 3d textures
`;
import { params, poptions } from '../../../common/framework/params_builder.js';
import { makeTestGroup } from '../../../common/framework/test_group.js';
import { assert, unreachable } from '../../../common/framework/util/util.js';
import { kSizedTextureFormatInfo, kSizedTextureFormats } from '../../capability_info.js';
import { GPUTest } from '../../gpu_test.js';
import { align } from '../../util/math.js';
import { getTextureCopyLayout } from '../../util/texture/layout.js';

/** Each combination of methods assume that the ones before it were tested and work correctly. */
const kMethodsToTest = [
  // We make sure that CopyT2B works when copying the whole texture for renderable formats:
  // TODO
  // Then we make sure that WriteTexture works for all formats:
  { initMethod: 'WriteTexture', checkMethod: 'FullCopyT2B' },
  // Then we make sure that CopyB2T works for all formats:
  { initMethod: 'CopyB2T', checkMethod: 'FullCopyT2B' },
  // Then we make sure that CopyT2B works for all formats:
  { initMethod: 'WriteTexture', checkMethod: 'PartialCopyT2B' },
];

class CopyBetweenLinearDataAndTextureTest extends GPUTest {
  bytesInACompleteRow(copyWidth, format) {
    const blockWidth = kSizedTextureFormatInfo[format].blockWidth;
    assert(copyWidth % blockWidth === 0);
    const copyWidthInBlocks = copyWidth / blockWidth;
    return kSizedTextureFormatInfo[format].bytesPerBlock * copyWidthInBlocks;
  }

  requiredBytesInCopy(layout, format, copyExtent) {
    assert(layout.rowsPerImage % kSizedTextureFormatInfo[format].blockHeight === 0);
    assert(copyExtent.height % kSizedTextureFormatInfo[format].blockHeight === 0);
    assert(copyExtent.width % kSizedTextureFormatInfo[format].blockWidth === 0);
    if (copyExtent.width === 0 || copyExtent.height === 0 || copyExtent.depth === 0) {
      return 0;
    } else {
      const texelBlockRowsPerImage =
        layout.rowsPerImage / kSizedTextureFormatInfo[format].blockHeight;
      const bytesPerImage = layout.bytesPerRow * texelBlockRowsPerImage;
      const bytesInLastSlice =
        layout.bytesPerRow * (copyExtent.height / kSizedTextureFormatInfo[format].blockHeight - 1) +
        (copyExtent.width / kSizedTextureFormatInfo[format].blockWidth) *
          kSizedTextureFormatInfo[format].bytesPerBlock;
      return bytesPerImage * (copyExtent.depth - 1) + bytesInLastSlice;
    }
  }

  /** Offset for a particular texel in the linear texture data */
  getTexelOffsetInBytes(textureDataLayout, format, texel, origin = { x: 0, y: 0, z: 0 }) {
    const { offset, bytesPerRow, rowsPerImage } = textureDataLayout;
    const info = kSizedTextureFormatInfo[format];

    assert(texel.x >= origin.x && texel.y >= origin.y && texel.z >= origin.z);
    assert(rowsPerImage % info.blockHeight === 0);
    assert(texel.x % info.blockWidth === 0);
    assert(texel.y % info.blockHeight === 0);
    assert(origin.x % info.blockWidth === 0);
    assert(origin.y % info.blockHeight === 0);

    const bytesPerImage = (rowsPerImage / info.blockHeight) * bytesPerRow;

    return (
      offset +
      (texel.z - origin.z) * bytesPerImage +
      ((texel.y - origin.y) / info.blockHeight) * bytesPerRow +
      ((texel.x - origin.x) / info.blockWidth) * info.bytesPerBlock
    );
  }

  *iterateBlockRows(size, origin, format) {
    if (size.width === 0 || size.height === 0 || size.depth === 0) {
      // do not iterate anything for an empty region
      return;
    }
    const info = kSizedTextureFormatInfo[format];
    assert(size.height % info.blockHeight === 0);
    for (let y = 0; y < size.height / info.blockHeight; ++y) {
      for (let z = 0; z < size.depth; ++z) {
        yield {
          x: origin.x,
          y: origin.y + y * info.blockHeight,
          z: origin.z + z,
        };
      }
    }
  }

  generateData(byteSize, start = 0) {
    const arr = new Uint8Array(byteSize);
    for (let i = 0; i < byteSize; ++i) {
      arr[i] = (i ** 3 + i + start) % 251;
    }
    return arr;
  }

  /**
   * This is used for testing passing undefined members of `GPUTextureDataLayout` instead of actual
   * values where possible. Passing arguments as values and not as objects so that they are passed
   * by copy and not by reference.
   */
  undefDataLayoutIfNeeded(offset, rowsPerImage, bytesPerRow, changeBeforePass) {
    if (changeBeforePass === 'undefined') {
      if (offset === 0) {
        offset = undefined;
      }
      if (rowsPerImage === 0) {
        rowsPerImage = undefined;
      }
    }
    return { offset, bytesPerRow, rowsPerImage };
  }

  /**
   * This is used for testing passing undefined members of `GPUTextureCopyView` instead of actual
   * values where possible and also for testing passing the origin as `[number, number, number]`.
   * Passing arguments as values and not as objects so that they are passed by copy and not by
   * reference.
   */
  undefOrArrayCopyViewIfNeeded(texture, origin_x, origin_y, origin_z, mipLevel, changeBeforePass) {
    let origin = { x: origin_x, y: origin_y, z: origin_z };

    if (changeBeforePass === 'undefined') {
      if (origin_x === 0 && origin_y === 0 && origin_z === 0) {
        origin = undefined;
      } else {
        if (origin_x === 0) {
          origin_x = undefined;
        }
        if (origin_y === 0) {
          origin_y = undefined;
        }
        if (origin_z === 0) {
          origin_z = undefined;
        }
        origin = { x: origin_x, y: origin_y, z: origin_z };
      }

      if (mipLevel === 0) {
        mipLevel = undefined;
      }
    }

    if (changeBeforePass === 'arrays') {
      origin = [origin_x, origin_y, origin_z];
    }

    return { texture, origin, mipLevel };
  }

  /**
   * This is used for testing passing `GPUExtent3D` as `[number, number, number]` instead of
   * `GPUExtent3DDict`. Passing arguments as values and not as objects so that they are passed by
   * copy and not by reference.
   */
  arrayCopySizeIfNeeded(width, height, depth, changeBeforePass) {
    if (changeBeforePass === 'arrays') {
      return [width, height, depth];
    } else {
      return { width, height, depth };
    }
  }

  /** Run a CopyT2B command with appropriate arguments corresponding to `ChangeBeforePass` */
  copyTextureToBufferWithAppliedArguments(
    buffer,
    { offset, rowsPerImage, bytesPerRow },
    { width, height, depth },
    { texture, mipLevel, origin },
    changeBeforePass
  ) {
    const { x, y, z } = origin;

    const appliedCopyView = this.undefOrArrayCopyViewIfNeeded(
      texture,
      x,
      y,
      z,
      mipLevel,
      changeBeforePass
    );

    const appliedDataLayout = this.undefDataLayoutIfNeeded(
      offset,
      rowsPerImage,
      bytesPerRow,
      changeBeforePass
    );

    const appliedCheckSize = this.arrayCopySizeIfNeeded(width, height, depth, changeBeforePass);

    const encoder = this.device.createCommandEncoder();
    encoder.copyTextureToBuffer(
      appliedCopyView,
      { buffer, ...appliedDataLayout },
      appliedCheckSize
    );

    this.device.defaultQueue.submit([encoder.finish()]);
  }

  /** Put data into a part of the texture with an appropriate method. */
  uploadLinearTextureDataToTextureSubBox(
    textureCopyView,
    textureDataLayout,
    copySize,
    partialData,
    method,
    changeBeforePass
  ) {
    const { texture, mipLevel, origin } = textureCopyView;
    const { offset, rowsPerImage, bytesPerRow } = textureDataLayout;
    const { x, y, z } = origin;
    const { width, height, depth } = copySize;

    const appliedCopyView = this.undefOrArrayCopyViewIfNeeded(
      texture,
      x,
      y,
      z,
      mipLevel,
      changeBeforePass
    );

    const appliedDataLayout = this.undefDataLayoutIfNeeded(
      offset,
      rowsPerImage,
      bytesPerRow,
      changeBeforePass
    );

    const appliedCopySize = this.arrayCopySizeIfNeeded(width, height, depth, changeBeforePass);

    switch (method) {
      case 'WriteTexture': {
        this.device.defaultQueue.writeTexture(
          appliedCopyView,
          partialData,
          appliedDataLayout,
          appliedCopySize
        );

        break;
      }
      case 'CopyB2T': {
        const buffer = this.device.createBuffer({
          mappedAtCreation: true,
          size: align(partialData.byteLength, 4),
          usage: GPUBufferUsage.COPY_SRC,
        });

        new Uint8Array(buffer.getMappedRange()).set(partialData);
        buffer.unmap();

        const encoder = this.device.createCommandEncoder();
        encoder.copyBufferToTexture(
          { buffer, ...appliedDataLayout },
          appliedCopyView,
          appliedCopySize
        );

        this.device.defaultQueue.submit([encoder.finish()]);

        break;
      }
      default:
        unreachable();
    }
  }

  /**
   * We check an appropriate part of the texture against the given data.
   * Used directly with PartialCopyT2B check method (for a subpart of the texture)
   * and by `copyWholeTextureToBufferAndCheckContentsWithUpdatedData` with FullCopyT2B check method
   * (for the whole texture). We also ensure that CopyT2B doesn't overwrite bytes it's not supposed
   * to if validateOtherBytesInBuffer is set to true.
   */
  copyPartialTextureToBufferAndCheckContents(
    { texture, mipLevel, origin },
    checkSize,
    format,
    expected,
    expectedDataLayout,
    changeBeforePass = 'none'
  ) {
    // The alignment is necessary because we need to copy and map data from this buffer.
    const bufferSize = align(expected.byteLength, 4);
    // The start value ensures generated data here doesn't match the expected data.
    const bufferData = this.generateData(bufferSize, 17);

    const buffer = this.device.createBuffer({
      mappedAtCreation: true,
      size: bufferSize,
      usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST,
    });

    new Uint8Array(buffer.getMappedRange()).set(bufferData);
    buffer.unmap();

    this.copyTextureToBufferWithAppliedArguments(
      buffer,
      expectedDataLayout,
      checkSize,
      { texture, mipLevel, origin },
      changeBeforePass
    );

    this.updateLinearTextureDataSubBox(
      expectedDataLayout,
      expectedDataLayout,
      checkSize,
      origin,
      format,
      bufferData,
      expected
    );

    this.expectContents(buffer, bufferData);
  }

  /**
   * Copies the whole texture into linear data stored in a buffer for further checks.
   *
   * Used for `copyWholeTextureToBufferAndCheckContentsWithUpdatedData`.
   */
  copyWholeTextureToNewBuffer({ texture, mipLevel }, resultDataLayout) {
    const { mipSize, byteLength, bytesPerRow, rowsPerImage } = resultDataLayout;
    const buffer = this.device.createBuffer({
      size: align(byteLength, 4), // this is necessary because we need to copy and map data from this buffer
      usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST,
    });

    const encoder = this.device.createCommandEncoder();
    encoder.copyTextureToBuffer(
      { texture, mipLevel },
      { buffer, bytesPerRow, rowsPerImage },
      mipSize
    );

    this.device.defaultQueue.submit([encoder.finish()]);

    return buffer;
  }

  copyFromArrayToArray(src, srcOffset, dst, dstOffset, size) {
    dst.set(src.subarray(srcOffset, srcOffset + size), dstOffset);
  }

  /**
   * Takes the data returned by `copyWholeTextureToNewBuffer` and updates it after a copy operation
   * on the texture by emulating the copy behaviour here directly.
   */
  updateLinearTextureDataSubBox(
    { bytesPerRow, rowsPerImage },
    sourceDataLayout,
    copySize,
    origin,
    format,
    destination,
    source
  ) {
    for (const texel of this.iterateBlockRows(copySize, origin, format)) {
      const sourceOffset = this.getTexelOffsetInBytes(sourceDataLayout, format, texel, origin);
      const destinationOffset = this.getTexelOffsetInBytes(
        { bytesPerRow, rowsPerImage, offset: 0 },
        format,
        texel
      );

      const rowLength = this.bytesInACompleteRow(copySize.width, format);
      this.copyFromArrayToArray(source, sourceOffset, destination, destinationOffset, rowLength);
    }
  }

  /**
   * Used for checking whether the whole texture was updated correctly by
   * `uploadLinearTextureDataToTextureSubpart`. Takes fullData returned by
   * `copyWholeTextureToNewBuffer` before the copy operation which is the original texture data,
   * then updates it with `updateLinearTextureDataSubpart` and checks the texture against the
   * updated data after the copy operation.
   */
  copyWholeTextureToBufferAndCheckContentsWithUpdatedData(
    { texture, mipLevel, origin },
    fullTextureCopyLayout,
    texturePartialDataLayout,
    copySize,
    format,
    fullData,
    partialData
  ) {
    const { mipSize, bytesPerRow, rowsPerImage, byteLength } = fullTextureCopyLayout;
    const { dst, begin, end } = this.createAlignedCopyForMapRead(fullData, byteLength, 0);

    // We add an eventual async expectation which will update the full data and then add
    // other eventual async expectations to ensure it will be correct.
    this.eventualAsyncExpectation(async () => {
      await dst.mapAsync(GPUMapMode.READ);
      const actual = new Uint8Array(dst.getMappedRange()).subarray(begin, end);
      this.updateLinearTextureDataSubBox(
        fullTextureCopyLayout,
        texturePartialDataLayout,
        copySize,
        origin,
        format,
        actual,
        partialData
      );

      this.copyPartialTextureToBufferAndCheckContents(
        { texture, mipLevel, origin: { x: 0, y: 0, z: 0 } },
        { width: mipSize[0], height: mipSize[1], depth: mipSize[2] },
        format,
        actual,
        { bytesPerRow, rowsPerImage, offset: 0 }
      );

      dst.destroy();
    });
  }

  /**
   * Tests copy between linear data and texture by creating a texture, putting some data into it
   * with WriteTexture/CopyB2T, then getting data for the whole texture/for a part of it back and
   * comparing it with the expectation.
   */
  uploadTextureAndVerifyCopy({
    textureDataLayout,
    copySize,
    dataSize,
    mipLevel = 0,
    origin = { x: 0, y: 0, z: 0 },
    textureSize,
    format,
    dimension = '2d',
    initMethod,
    checkMethod,
    changeBeforePass = 'none',
  }) {
    const texture = this.device.createTexture({
      size: textureSize,
      format,
      dimension,
      mipLevelCount: mipLevel + 1,
      usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.COPY_DST,
    });

    const data = this.generateData(dataSize);

    switch (checkMethod) {
      case 'PartialCopyT2B': {
        this.uploadLinearTextureDataToTextureSubBox(
          { texture, mipLevel, origin },
          textureDataLayout,
          copySize,
          data,
          initMethod,
          changeBeforePass
        );

        this.copyPartialTextureToBufferAndCheckContents(
          { texture, mipLevel, origin },
          copySize,
          format,
          data,
          textureDataLayout,
          changeBeforePass
        );

        break;
      }
      case 'FullCopyT2B': {
        const fullTextureCopyLayout = getTextureCopyLayout(format, dimension, textureSize, {
          mipLevel,
        });

        const fullData = this.copyWholeTextureToNewBuffer(
          { texture, mipLevel },
          fullTextureCopyLayout
        );

        this.uploadLinearTextureDataToTextureSubBox(
          { texture, mipLevel, origin },
          textureDataLayout,
          copySize,
          data,
          initMethod,
          changeBeforePass
        );

        this.copyWholeTextureToBufferAndCheckContentsWithUpdatedData(
          { texture, mipLevel, origin },
          fullTextureCopyLayout,
          textureDataLayout,
          copySize,
          format,
          fullData,
          data
        );

        break;
      }
      default:
        unreachable();
    }
  }
}

/**
 * This is a helper function used for filtering test parameters
 *
 * TODO: Modify this after introducing tests with rendering.
 */
function formatCanBeTested({ format }) {
  return kSizedTextureFormatInfo[format].copyDst && kSizedTextureFormatInfo[format].copySrc;
}

export const g = makeTestGroup(CopyBetweenLinearDataAndTextureTest);

// Test that copying data with various bytesPerRow and rowsPerImage values and minimum required
// bytes in copy works for every format.
// Covers a special code path for Metal:
//    bufferSize - offset < bytesPerImage * copyExtent.depth
// Covers a special code path for D3D12:
//    when bytesPerRow is not a multiple of 512 and copyExtent.depth > 1: copyExtent.depth % 2 == { 0, 1 }
//    bytesPerRow == bytesInACompleteCopyImage */
g.test('copy_with_various_rows_per_image_and_bytes_per_row')
  .params(
    params()
      .combine(kMethodsToTest)
      .combine([
        { bytesPerRowPadding: 0, rowsPerImagePaddingInBlocks: 0 }, // no padding
        { bytesPerRowPadding: 0, rowsPerImagePaddingInBlocks: 6 }, // rowsPerImage padding
        { bytesPerRowPadding: 6, rowsPerImagePaddingInBlocks: 0 }, // bytesPerRow padding
        { bytesPerRowPadding: 15, rowsPerImagePaddingInBlocks: 17 }, // both paddings
      ])
      .combine([
        // In the two cases below, for (WriteTexture, PartialCopyB2T) and (CopyB2T, FullCopyT2B)
        // sets of methods we will have bytesPerRow = 256 and copyDepth % 2 == { 0, 1 }
        // respectively. This covers a special code path for D3D12.
        { copyWidthInBlocks: 3, copyHeightInBlocks: 4, copyDepth: 5 }, // standard copy
        { copyWidthInBlocks: 5, copyHeightInBlocks: 4, copyDepth: 2 }, // standard copy

        { copyWidthInBlocks: 256, copyHeightInBlocks: 3, copyDepth: 2 }, // copyWidth is 256-aligned
        { copyWidthInBlocks: 0, copyHeightInBlocks: 4, copyDepth: 5 }, // empty copy because of width
        { copyWidthInBlocks: 3, copyHeightInBlocks: 0, copyDepth: 5 }, // empty copy because of height
        { copyWidthInBlocks: 3, copyHeightInBlocks: 4, copyDepth: 0 }, // empty copy because of depth
        { copyWidthInBlocks: 1, copyHeightInBlocks: 3, copyDepth: 5 }, // copyWidth = 1

        // The two cases below cover another special code path for D3D12.
        //   - For (WriteTexture, FullCopyT2B) with r8unorm:
        //         bytesPerRow = 15 = 3 * 5 = bytesInACompleteCopyImage.
        { copyWidthInBlocks: 32, copyHeightInBlocks: 1, copyDepth: 8 }, // copyHeight = 1
        //   - For (CopyB2T, FullCopyT2B) and (WriteTexture, PartialCopyT2B) with r8unorm:
        //         bytesPerRow = 256 = 8 * 32 = bytesInACompleteCopyImage.
        { copyWidthInBlocks: 5, copyHeightInBlocks: 4, copyDepth: 1 }, // copyDepth = 1

        { copyWidthInBlocks: 7, copyHeightInBlocks: 1, copyDepth: 1 }, // copyHeight = 1 and copyDepth = 1
      ])
      .combine(poptions('format', kSizedTextureFormats))
      .filter(formatCanBeTested)
  )
  .fn(async t => {
    const {
      bytesPerRowPadding,
      rowsPerImagePaddingInBlocks,
      copyWidthInBlocks,
      copyHeightInBlocks,
      copyDepth,
      format,
      initMethod,
      checkMethod,
    } = t.params;

    const info = kSizedTextureFormatInfo[format];

    // For CopyB2T and CopyT2B we need to have bytesPerRow 256-aligned,
    // to make this happen we align the bytesInACompleteRow value and multiply
    // bytesPerRowPadding by 256.
    const bytesPerRowAlignment =
      initMethod === 'WriteTexture' && checkMethod === 'FullCopyT2B' ? 1 : 256;

    const copyWidth = copyWidthInBlocks * info.blockWidth;
    const copyHeight = copyHeightInBlocks * info.blockHeight;
    const rowsPerImage = copyHeight + rowsPerImagePaddingInBlocks * info.blockHeight;
    const bytesPerRow =
      align(t.bytesInACompleteRow(copyWidth, format), bytesPerRowAlignment) +
      bytesPerRowPadding * bytesPerRowAlignment;
    const copySize = { width: copyWidth, height: copyHeight, depth: copyDepth };

    const minDataSize = t.requiredBytesInCopy(
      { offset: 0, bytesPerRow, rowsPerImage },
      format,
      copySize
    );

    t.uploadTextureAndVerifyCopy({
      textureDataLayout: { offset: 0, bytesPerRow, rowsPerImage },
      copySize,
      dataSize: minDataSize,
      textureSize: [
        Math.max(copyWidth, info.blockWidth),
        Math.max(copyHeight, info.blockHeight),
        Math.max(copyDepth, 1),
      ],
      /* making sure the texture is non-empty */ format,
      initMethod,
      checkMethod,
    });
  });

// Test that copying data with various offset values and additional data paddings
// works for every format with 2d and 2d-array textures.
// Covers two special code paths for D3D12:
//     offset + bytesInCopyExtentPerRow { ==, > } bytesPerRow
//     offset > bytesInACompleteCopyImage
g.test('copy_with_various_offsets_and_data_sizes')
  .params(
    params()
      .combine(kMethodsToTest)
      .combine([
        { offsetInBlocks: 0, dataPaddingInBytes: 0 }, // no offset and no padding
        { offsetInBlocks: 1, dataPaddingInBytes: 0 }, // offset = 1
        { offsetInBlocks: 2, dataPaddingInBytes: 0 }, // offset = 2
        { offsetInBlocks: 15, dataPaddingInBytes: 0 }, // offset = 15
        { offsetInBlocks: 16, dataPaddingInBytes: 0 }, // offset = 16
        { offsetInBlocks: 242, dataPaddingInBytes: 0 }, // for rgba8unorm format: offset + bytesInCopyExtentPerRow = 242 + 12 = 256 = bytesPerRow
        { offsetInBlocks: 243, dataPaddingInBytes: 0 }, // for rgba8unorm format: offset + bytesInCopyExtentPerRow = 243 + 12 > 256 = bytesPerRow
        { offsetInBlocks: 768, dataPaddingInBytes: 0 }, // for copyDepth = 1, blockWidth = 1 and bytesPerBlock = 1: offset = 768 = 3 * 256 = bytesInACompleteCopyImage
        { offsetInBlocks: 769, dataPaddingInBytes: 0 }, // for copyDepth = 1, blockWidth = 1 and bytesPerBlock = 1: offset = 769 > 768 = bytesInACompleteCopyImage
        { offsetInBlocks: 0, dataPaddingInBytes: 1 }, // dataPaddingInBytes > 0
        { offsetInBlocks: 1, dataPaddingInBytes: 8 }, // offset > 0 and dataPaddingInBytes > 0
      ])
      .combine(poptions('copyDepth', [1, 2])) // 2d and 2d-array textures
      .combine(poptions('format', kSizedTextureFormats))
      .filter(formatCanBeTested)
  )
  .fn(async t => {
    const {
      offsetInBlocks,
      dataPaddingInBytes,
      copyDepth,
      format,
      initMethod,
      checkMethod,
    } = t.params;

    const info = kSizedTextureFormatInfo[format];

    const offset = offsetInBlocks * info.bytesPerBlock;
    const copySize = {
      width: 3 * info.blockWidth,
      height: 3 * info.blockHeight,
      depth: copyDepth,
    };

    const rowsPerImage = copySize.height;
    const bytesPerRow = 256;

    const dataSize =
      offset +
      t.requiredBytesInCopy({ offset, bytesPerRow, rowsPerImage }, format, copySize) +
      dataPaddingInBytes;

    // We're copying a (3 x 3 x copyDepth) (in texel blocks) part of a (4 x 4 x copyDepth)
    // (in texel blocks) texture with no origin.
    t.uploadTextureAndVerifyCopy({
      textureDataLayout: { offset, bytesPerRow, rowsPerImage },
      copySize,
      dataSize,
      textureSize: [4 * info.blockWidth, 4 * info.blockHeight, copyDepth],
      format,
      initMethod,
      checkMethod,
    });
  });

// Test that copying slices of a texture works with various origin and copyExtent values
// for all formats. We pass origin and copyExtent as [number, number, number].
g.test('copy_with_various_origins_and_copy_extents')
  .params(
    params()
      .combine(kMethodsToTest)
      .combine(poptions('originValueInBlocks', [0, 7, 8]))
      .combine(poptions('copySizeValueInBlocks', [0, 7, 8]))
      .combine(poptions('textureSizePaddingValueInBlocks', [0, 7, 8]))
      .unless(
        p =>
          // we can't create an empty texture
          p.copySizeValueInBlocks + p.originValueInBlocks + p.textureSizePaddingValueInBlocks === 0
      )
      .combine(poptions('coordinateToTest', ['width', 'height', 'depth']))
      .combine(poptions('format', kSizedTextureFormats))
      .filter(formatCanBeTested)
  )
  .fn(async t => {
    const {
      coordinateToTest,
      originValueInBlocks,
      copySizeValueInBlocks,
      textureSizePaddingValueInBlocks,
      format,
      initMethod,
      checkMethod,
    } = t.params;

    const info = kSizedTextureFormatInfo[format];

    const origin = { x: info.blockWidth, y: info.blockHeight, z: 1 };
    const copySize = { width: 2 * info.blockWidth, height: 2 * info.blockHeight, depth: 2 };
    const textureSize = [3 * info.blockWidth, 3 * info.blockHeight, 3];

    switch (coordinateToTest) {
      case 'width': {
        origin.x = originValueInBlocks * info.blockWidth;
        copySize.width = copySizeValueInBlocks * info.blockWidth;
        textureSize[0] =
          origin.x + copySize.width + textureSizePaddingValueInBlocks * info.blockWidth;
        break;
      }
      case 'height': {
        origin.y = originValueInBlocks * info.blockHeight;
        copySize.height = copySizeValueInBlocks * info.blockHeight;
        textureSize[1] =
          origin.y + copySize.height + textureSizePaddingValueInBlocks * info.blockHeight;
        break;
      }
      case 'depth': {
        origin.z = originValueInBlocks;
        copySize.depth = copySizeValueInBlocks;
        textureSize[2] = origin.z + copySize.depth + textureSizePaddingValueInBlocks;
        break;
      }
    }

    const rowsPerImage = copySize.height;
    const bytesPerRow = align(copySize.width, 256);
    const dataSize = t.requiredBytesInCopy(
      { offset: 0, bytesPerRow, rowsPerImage },
      format,
      copySize
    );

    // For testing width: we copy a (_ x 2 x 2) (in texel blocks) part of a (_ x 3 x 3)
    // (in texel blocks) texture with origin (_, 1, 1) (in texel blocks).
    // Similarly for other coordinates.
    t.uploadTextureAndVerifyCopy({
      textureDataLayout: { offset: 0, bytesPerRow, rowsPerImage },
      copySize,
      dataSize,
      origin,
      textureSize,
      format,
      initMethod,
      checkMethod,
      changeBeforePass: 'arrays',
    });
  });

/**
 * Generates textureSizes which correspond to the same physicalSizeAtMipLevel including virtual
 * sizes at mip level different from the physical ones.
 */
function* textureSizeExpander({ format, mipLevel, _texturePhysicalSizeAtMipLevelInBlocks }) {
  const info = kSizedTextureFormatInfo[format];

  const widthAtThisLevel = _texturePhysicalSizeAtMipLevelInBlocks.width * info.blockWidth;
  const heightAtThisLevel = _texturePhysicalSizeAtMipLevelInBlocks.height * info.blockHeight;
  const textureSize = [
    widthAtThisLevel << mipLevel,
    heightAtThisLevel << mipLevel,
    _texturePhysicalSizeAtMipLevelInBlocks.depth,
  ];

  yield {
    textureSize,
  };

  // We choose width and height of the texture so that the values are divisible by blockWidth and
  // blockHeight respectively and so that the virtual size at mip level corresponds to the same
  // physical size.
  // Virtual size at mip level with modified width has width = (physical size width) - (blockWidth / 2).
  // Virtual size at mip level with modified height has height = (physical size height) - (blockHeight / 2).
  const widthAtPrevLevel = widthAtThisLevel << 1;
  const heightAtPrevLevel = heightAtThisLevel << 1;
  assert(mipLevel > 0);
  assert(widthAtPrevLevel >= info.blockWidth && heightAtPrevLevel >= info.blockHeight);
  const modifiedWidth = (widthAtPrevLevel - info.blockWidth) << (mipLevel - 1);
  const modifiedHeight = (heightAtPrevLevel - info.blockHeight) << (mipLevel - 1);

  const modifyWidth = info.blockWidth > 1 && modifiedWidth !== textureSize[0];
  const modifyHeight = info.blockHeight > 1 && modifiedHeight !== textureSize[1];

  if (modifyWidth) {
    yield {
      textureSize: [modifiedWidth, textureSize[1], textureSize[2]],
    };
  }
  if (modifyHeight) {
    yield {
      textureSize: [textureSize[0], modifiedHeight, textureSize[2]],
    };
  }
  if (modifyWidth && modifyHeight) {
    yield {
      textureSize: [modifiedWidth, modifiedHeight, textureSize[2]],
    };
  }
}

// Test that copying various mip levels works.
// Covers two special code paths:
//   - the physical size of the subresouce is not equal to the logical size
//   - bufferSize - offset < bytesPerImage * copyExtent.depth and copyExtent needs to be clamped for all block formats */
g.test('copy_various_mip_levels')
  .params(
    params()
      .combine(kMethodsToTest)
      .combine([
        // origin + copySize = texturePhysicalSizeAtMipLevel for all coordinates, 2d texture */
        {
          copySizeInBlocks: { width: 5, height: 4, depth: 1 },
          originInBlocks: { x: 3, y: 2, z: 0 },
          _texturePhysicalSizeAtMipLevelInBlocks: { width: 8, height: 6, depth: 1 },
          mipLevel: 1,
        },

        // origin + copySize = texturePhysicalSizeAtMipLevel for all coordinates, 2d-array texture
        {
          copySizeInBlocks: { width: 5, height: 4, depth: 2 },
          originInBlocks: { x: 3, y: 2, z: 1 },
          _texturePhysicalSizeAtMipLevelInBlocks: { width: 8, height: 6, depth: 3 },
          mipLevel: 2,
        },

        // origin.x + copySize.width = texturePhysicalSizeAtMipLevel.width
        {
          copySizeInBlocks: { width: 5, height: 4, depth: 2 },
          originInBlocks: { x: 3, y: 2, z: 1 },
          _texturePhysicalSizeAtMipLevelInBlocks: { width: 8, height: 7, depth: 4 },
          mipLevel: 3,
        },

        // origin.y + copySize.height = texturePhysicalSizeAtMipLevel.height
        {
          copySizeInBlocks: { width: 5, height: 4, depth: 2 },
          originInBlocks: { x: 3, y: 2, z: 1 },
          _texturePhysicalSizeAtMipLevelInBlocks: { width: 9, height: 6, depth: 4 },
          mipLevel: 4,
        },

        // origin.z + copySize.depth = texturePhysicalSizeAtMipLevel.depth
        {
          copySizeInBlocks: { width: 5, height: 4, depth: 2 },
          originInBlocks: { x: 3, y: 2, z: 1 },
          _texturePhysicalSizeAtMipLevelInBlocks: { width: 9, height: 7, depth: 3 },
          mipLevel: 5,
        },

        // origin + copySize < texturePhysicalSizeAtMipLevel for all coordinates
        {
          copySizeInBlocks: { width: 5, height: 4, depth: 2 },
          originInBlocks: { x: 3, y: 2, z: 1 },
          _texturePhysicalSizeAtMipLevelInBlocks: { width: 9, height: 7, depth: 4 },
          mipLevel: 6,
        },
      ])
      .combine(poptions('format', kSizedTextureFormats))
      .filter(formatCanBeTested)
      .expand(textureSizeExpander)
  )
  .fn(async t => {
    const {
      copySizeInBlocks,
      originInBlocks,
      textureSize,
      mipLevel,
      format,
      initMethod,
      checkMethod,
    } = t.params;

    const info = kSizedTextureFormatInfo[format];

    const origin = {
      x: originInBlocks.x * info.blockWidth,
      y: originInBlocks.y * info.blockHeight,
      z: originInBlocks.z,
    };

    const copySize = {
      width: copySizeInBlocks.width * info.blockWidth,
      height: copySizeInBlocks.height * info.blockHeight,
      depth: copySizeInBlocks.depth,
    };

    const rowsPerImage = copySize.height + info.blockHeight;
    const bytesPerRow = align(copySize.width, 256);
    const dataSize = t.requiredBytesInCopy(
      { offset: 0, bytesPerRow, rowsPerImage },
      format,
      copySize
    );

    t.uploadTextureAndVerifyCopy({
      textureDataLayout: { offset: 0, bytesPerRow, rowsPerImage },
      copySize,
      dataSize,
      origin,
      mipLevel,
      textureSize,
      format,
      initMethod,
      checkMethod,
    });
  });

// Test that when copying a single row we can set any bytesPerRow value and when copying a single
// slice we can set rowsPerImage to 0. Origin, offset, mipLevel and rowsPerImage values will be set
// to undefined when appropriate.
g.test('copy_with_no_image_or_slice_padding_and_undefined_values')
  .params(
    params()
      .combine(kMethodsToTest)
      .combine([
        // copying one row: bytesPerRow and rowsPerImage can be set to 0
        {
          bytesPerRow: 0,
          rowsPerImage: 0,
          copySize: { width: 3, height: 1, depth: 1 },
          origin: { x: 0, y: 0, z: 0 },
        },

        // copying one row: bytesPerRow can be < bytesInACompleteRow = 400
        {
          bytesPerRow: 256,
          rowsPerImage: 0,
          copySize: { width: 100, height: 1, depth: 1 },
          origin: { x: 0, y: 0, z: 0 },
        },

        // copying one slice: rowsPerImage = 0 will be set to undefined
        {
          bytesPerRow: 256,
          rowsPerImage: 0,
          copySize: { width: 3, height: 3, depth: 1 },
          origin: { x: 0, y: 0, z: 0 },
        },

        // copying one slice: rowsPerImage = 2 is valid
        {
          bytesPerRow: 256,
          rowsPerImage: 2,
          copySize: { width: 3, height: 3, depth: 1 },
          origin: { x: 0, y: 0, z: 0 },
        },

        // origin.x = 0 will be set to undefined
        {
          bytesPerRow: 0,
          rowsPerImage: 0,
          copySize: { width: 1, height: 1, depth: 1 },
          origin: { x: 0, y: 1, z: 1 },
        },

        // origin.y = 0 will be set to undefined
        {
          bytesPerRow: 0,
          rowsPerImage: 0,
          copySize: { width: 1, height: 1, depth: 1 },
          origin: { x: 1, y: 0, z: 1 },
        },

        // origin.z = 0 will be set to undefined
        {
          bytesPerRow: 0,
          rowsPerImage: 0,
          copySize: { width: 1, height: 1, depth: 1 },
          origin: { x: 1, y: 1, z: 0 },
        },
      ])
  )
  .fn(async t => {
    const { bytesPerRow, rowsPerImage, copySize, origin, initMethod, checkMethod } = t.params;

    t.uploadTextureAndVerifyCopy({
      textureDataLayout: { offset: 0, bytesPerRow, rowsPerImage },
      copySize,
      dataSize: 100 * 3 * 4,
      textureSize: [100, 3, 2],
      origin,
      format: 'rgba8unorm',
      initMethod,
      checkMethod,
      changeBeforePass: 'undefined',
    });
  });
