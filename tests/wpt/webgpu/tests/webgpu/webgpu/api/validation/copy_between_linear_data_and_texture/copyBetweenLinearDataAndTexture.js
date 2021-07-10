/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ import { poptions } from '../../../../common/framework/params_builder.js';
import { assert } from '../../../../common/framework/util/util.js';
import { kSizedTextureFormatInfo } from '../../../capability_info.js';
import { ValidationTest } from '../validation_test.js';

export const kAllTestMethods = ['WriteTexture', 'CopyBufferToTexture', 'CopyTextureToBuffer'];

export class CopyBetweenLinearDataAndTextureTest extends ValidationTest {
  bytesInACompleteRow(copyWidth, format) {
    const info = kSizedTextureFormatInfo[format];
    assert(copyWidth % info.blockWidth === 0);
    return (info.bytesPerBlock * copyWidth) / info.blockWidth;
  }

  requiredBytesInCopy(layout, format, copyExtent) {
    const info = kSizedTextureFormatInfo[format];
    assert(layout.rowsPerImage % info.blockHeight === 0);
    assert(copyExtent.height % info.blockHeight === 0);
    assert(copyExtent.width % info.blockWidth === 0);
    if (copyExtent.width === 0 || copyExtent.height === 0 || copyExtent.depth === 0) {
      return 0;
    } else {
      const texelBlockRowsPerImage = layout.rowsPerImage / info.blockHeight;
      const bytesPerImage = layout.bytesPerRow * texelBlockRowsPerImage;
      const bytesInLastSlice =
        layout.bytesPerRow * (copyExtent.height / info.blockHeight - 1) +
        (copyExtent.width / info.blockWidth) * info.bytesPerBlock;
      return bytesPerImage * (copyExtent.depth - 1) + bytesInLastSlice;
    }
  }

  testRun(
    textureCopyView,
    textureDataLayout,
    size,
    {
      dataSize,
      method,
      success,
      submit = false, // If submit is true, the validaton error is expected to come from the submit and encoding should succeed.
    }
  ) {
    switch (method) {
      case 'WriteTexture': {
        const data = new Uint8Array(dataSize);

        this.expectValidationError(() => {
          this.device.defaultQueue.writeTexture(textureCopyView, data, textureDataLayout, size);
        }, !success);

        break;
      }
      case 'CopyBufferToTexture': {
        const buffer = this.device.createBuffer({
          size: dataSize,
          usage: GPUBufferUsage.COPY_SRC,
        });

        const encoder = this.device.createCommandEncoder();
        encoder.copyBufferToTexture({ buffer, ...textureDataLayout }, textureCopyView, size);

        if (submit) {
          const cmd = encoder.finish();
          this.expectValidationError(() => {
            this.device.defaultQueue.submit([cmd]);
          }, !success);
        } else {
          this.expectValidationError(() => {
            encoder.finish();
          }, !success);
        }

        break;
      }
      case 'CopyTextureToBuffer': {
        const buffer = this.device.createBuffer({
          size: dataSize,
          usage: GPUBufferUsage.COPY_DST,
        });

        const encoder = this.device.createCommandEncoder();
        encoder.copyTextureToBuffer(textureCopyView, { buffer, ...textureDataLayout }, size);

        if (submit) {
          const cmd = encoder.finish();
          this.expectValidationError(() => {
            this.device.defaultQueue.submit([cmd]);
          }, !success);
        } else {
          this.expectValidationError(() => {
            encoder.finish();
          }, !success);
        }

        break;
      }
    }
  }

  // This is a helper function used for creating a texture when we don't have to be very
  // precise about its size as long as it's big enough and properly aligned.
  createAlignedTexture(
    format,
    copySize = { width: 1, height: 1, depth: 1 },
    origin = { x: 0, y: 0, z: 0 }
  ) {
    const info = kSizedTextureFormatInfo[format];
    return this.device.createTexture({
      size: {
        width: Math.max(1, copySize.width + origin.x) * info.blockWidth,
        height: Math.max(1, copySize.height + origin.y) * info.blockHeight,
        depth: Math.max(1, copySize.depth + origin.z),
      },

      format,
      usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.COPY_DST,
    });
  }
}

// For testing divisibility by a number we test all the values returned by this function:
function valuesToTestDivisibilityBy(number) {
  const values = [];
  for (let i = 0; i <= 2 * number; ++i) {
    values.push(i);
  }
  values.push(3 * number);
  return values;
}

// This is a helper function used for expanding test parameters for texel block alignment tests on offset
export function texelBlockAlignmentTestExpanderForOffset({ format }) {
  return poptions(
    'offset',
    valuesToTestDivisibilityBy(kSizedTextureFormatInfo[format].bytesPerBlock)
  );
}

// This is a helper function used for expanding test parameters for texel block alignment tests on rowsPerImage
export function texelBlockAlignmentTestExpanderForRowsPerImage({ format }) {
  return poptions(
    'rowsPerImage',
    valuesToTestDivisibilityBy(kSizedTextureFormatInfo[format].blockHeight)
  );
}

// This is a helper function used for expanding test parameters for texel block alignment tests on origin and size
export function texelBlockAlignmentTestExpanderForValueToCoordinate({ format, coordinateToTest }) {
  switch (coordinateToTest) {
    case 'x':
    case 'width':
      return poptions(
        'valueToCoordinate',
        valuesToTestDivisibilityBy(kSizedTextureFormatInfo[format].blockWidth)
      );

    case 'y':
    case 'height':
      return poptions(
        'valueToCoordinate',
        valuesToTestDivisibilityBy(kSizedTextureFormatInfo[format].blockHeight)
      );

    case 'z':
    case 'depth':
      return poptions('valueToCoordinate', valuesToTestDivisibilityBy(1));
  }
}

// This is a helper function used for filtering test parameters
export function formatCopyableWithMethod({ format, method }) {
  if (method === 'CopyTextureToBuffer') {
    return kSizedTextureFormatInfo[format].copySrc;
  } else {
    return kSizedTextureFormatInfo[format].copyDst;
  }
}
