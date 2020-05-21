/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

export const description = `
copy imageBitmap To texture tests.
`;
import { poptions, params } from '../../common/framework/params_builder.js';
import { makeTestGroup } from '../../common/framework/test_group.js';
import { GPUTest } from '../gpu_test.js';

function calculateRowPitch(width, bytesPerPixel) {
  const bytesPerRow = width * bytesPerPixel; // Rounds up to a multiple of 256 according to WebGPU requirements.

  return (bytesPerRow - 1 >> 8) + 1 << 8;
}

class F extends GPUTest {
  checkCopyImageBitmapResult(src, expected, width, height, bytesPerPixel) {
    const exp = new Uint8Array(expected.buffer, expected.byteOffset, expected.byteLength);
    const rowPitch = calculateRowPitch(width, bytesPerPixel);
    const dst = this.createCopyForMapRead(src, rowPitch * height);
    this.eventualAsyncExpectation(async niceStack => {
      const actual = new Uint8Array((await dst.mapReadAsync()));
      const check = this.checkBufferWithRowPitch(actual, exp, width, height, rowPitch, bytesPerPixel);

      if (check !== undefined) {
        niceStack.message = check;
        this.rec.expectationFailed(niceStack);
      }

      dst.destroy();
    });
  }

  checkBufferWithRowPitch(actual, exp, width, height, rowPitch, bytesPerPixel) {
    const failedByteIndices = [];
    const failedByteExpectedValues = [];
    const failedByteActualValues = [];

    iLoop: for (let i = 0; i < height; ++i) {
      const bytesPerRow = width * bytesPerPixel;

      for (let j = 0; j < bytesPerRow; ++j) {
        const indexExp = j + i * bytesPerRow;
        const indexActual = j + rowPitch * i;

        if (actual[indexActual] !== exp[indexExp]) {
          if (failedByteIndices.length >= 4) {
            failedByteIndices.push('...');
            failedByteExpectedValues.push('...');
            failedByteActualValues.push('...');
            break iLoop;
          }

          failedByteIndices.push(`(${i},${j})`);
          failedByteExpectedValues.push(exp[indexExp].toString());
          failedByteActualValues.push(actual[indexActual].toString());
        }
      }
    }

    if (failedByteIndices.length > 0) {
      return `at [${failedByteIndices.join(', ')}], \
expected [${failedByteExpectedValues.join(', ')}], \
got [${failedByteActualValues.join(', ')}]`;
    }

    return undefined;
  }

  doTestAndCheckResult(imageBitmapCopyView, dstTextureCopyView, copySize, bytesPerPixel, expectedData) {
    this.device.defaultQueue.copyImageBitmapToTexture(imageBitmapCopyView, dstTextureCopyView, copySize);
    const imageBitmap = imageBitmapCopyView.imageBitmap;
    const dstTexture = dstTextureCopyView.texture;
    const bytesPerRow = calculateRowPitch(imageBitmap.width, bytesPerPixel);
    const testBuffer = this.device.createBuffer({
      size: bytesPerRow * imageBitmap.height,
      usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
    });
    const encoder = this.device.createCommandEncoder();
    encoder.copyTextureToBuffer({
      texture: dstTexture,
      mipLevel: 0,
      origin: {
        x: 0,
        y: 0,
        z: 0
      }
    }, {
      buffer: testBuffer,
      bytesPerRow
    }, {
      width: imageBitmap.width,
      height: imageBitmap.height,
      depth: 1
    });
    this.device.defaultQueue.submit([encoder.finish()]);
    this.checkCopyImageBitmapResult(testBuffer, expectedData, imageBitmap.width, imageBitmap.height, bytesPerPixel);
  }

}

export const g = makeTestGroup(F);
g.test('from_ImageData').params(params().combine(poptions('width', [1, 2, 4, 15, 255, 256])).combine(poptions('height', [1, 2, 4, 15, 255, 256])).combine(poptions('alpha', ['none', 'premultiply'])).combine(poptions('orientation', ['none', 'flipY']))).fn(async t => {
  const {
    width,
    height,
    alpha,
    orientation
  } = t.params; // The texture format is rgba8unorm, so the bytes per pixel is 4.

  const bytesPerPixel = 4;
  const imagePixels = new Uint8ClampedArray(bytesPerPixel * width * height);

  if (alpha === 'premultiply') {
    // Make expected value simple to construct:
    // Input is (255, 255, 255, a), which will be stored into the ImageBitmap
    // as (a, a, a, a).
    for (let i = 0; i < width * height * bytesPerPixel; ++i) {
      imagePixels[i] = i % 4 !== 3 ? 255 : i % 256;
    }
  } else {
    for (let i = 0; i < width * height * bytesPerPixel; ++i) {
      imagePixels[i] = i % 4 === 3 ? 255 : i % 256;
    }
  }

  const imageData = new ImageData(imagePixels, width, height);
  const imageBitmap = await createImageBitmap(imageData, {
    premultiplyAlpha: alpha,
    imageOrientation: orientation
  });
  const dst = t.device.createTexture({
    size: {
      width: imageBitmap.width,
      height: imageBitmap.height,
      depth: 1
    },
    format: 'rgba8unorm',
    usage: GPUTextureUsage.COPY_DST | GPUTextureUsage.COPY_SRC
  }); // Construct expected value

  const expectedPixels = new Uint8ClampedArray(bytesPerPixel * width * height);

  for (let i = 0; i < width * height * bytesPerPixel; ++i) {
    expectedPixels[i] = imagePixels[i];
  }

  if (orientation === 'flipY') {
    for (let i = 0; i < height; ++i) {
      for (let j = 0; j < width * bytesPerPixel; ++j) {
        const pos_image_pixel = (height - i - 1) * width * bytesPerPixel + j;
        const pos_expected_value = i * width * bytesPerPixel + j;
        expectedPixels[pos_expected_value] = imagePixels[pos_image_pixel];
      }
    }
  }

  if (alpha === 'premultiply') {
    for (let i = 0; i < width * height * bytesPerPixel; ++i) {
      const alpha_value_position = 3 - i % 4 + i;

      if (i % 4 !== 3) {
        // Expected value is (a, a, a, a)
        expectedPixels[i] = expectedPixels[alpha_value_position];
      }
    }
  }

  t.doTestAndCheckResult({
    imageBitmap,
    origin: {
      x: 0,
      y: 0
    }
  }, {
    texture: dst
  }, {
    width: imageBitmap.width,
    height: imageBitmap.height,
    depth: 1
  }, bytesPerPixel, expectedPixels);
});
g.test('from_canvas').params(params().combine(poptions('width', [1, 2, 4, 15, 255, 256])).combine(poptions('height', [1, 2, 4, 15, 255, 256]))).fn(async t => {
  const {
    width,
    height
  } = t.params; // CTS sometimes runs on worker threads, where document is not available.
  // In this case, OffscreenCanvas can be used instead of <canvas>.
  // But some browsers don't support OffscreenCanvas, and some don't
  // support '2d' contexts on OffscreenCanvas.
  // In this situation, the case will be skipped.

  let imageCanvas;

  if (typeof document !== 'undefined') {
    imageCanvas = document.createElement('canvas');
    imageCanvas.width = width;
    imageCanvas.height = height;
  } else if (typeof OffscreenCanvas === 'undefined') {
    t.skip('OffscreenCanvas is not supported');
    return;
  } else {
    imageCanvas = new OffscreenCanvas(width, height);
  }

  const imageCanvasContext = imageCanvas.getContext('2d');

  if (imageCanvasContext === null) {
    t.skip('OffscreenCanvas "2d" context not available');
    return;
  } // The texture format is rgba8unorm, so the bytes per pixel is 4.


  const bytesPerPixel = 4; // Generate original data.

  const imagePixels = new Uint8ClampedArray(bytesPerPixel * width * height);

  for (let i = 0; i < width * height * bytesPerPixel; ++i) {
    imagePixels[i] = i % 4 === 3 ? 255 : i % 256;
  }

  const imageData = new ImageData(imagePixels, width, height);
  imageCanvasContext.putImageData(imageData, 0, 0);
  const imageBitmap = await createImageBitmap(imageCanvas);
  const dst = t.device.createTexture({
    size: {
      width: imageBitmap.width,
      height: imageBitmap.height,
      depth: 1
    },
    format: 'rgba8unorm',
    usage: GPUTextureUsage.COPY_DST | GPUTextureUsage.COPY_SRC
  }); // This will get origin data and even it has premultiplied-alpha

  const expectedData = imageCanvasContext.getImageData(0, 0, imageBitmap.width, imageBitmap.height).data;
  t.doTestAndCheckResult({
    imageBitmap,
    origin: {
      x: 0,
      y: 0
    }
  }, {
    texture: dst
  }, {
    width: imageBitmap.width,
    height: imageBitmap.height,
    depth: 1
  }, bytesPerPixel, expectedData);
});
//# sourceMappingURL=copyImageBitmapToTexture.spec.js.map