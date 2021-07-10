/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
copy imageBitmap To texture tests.
`;
import { poptions, params } from '../../common/framework/params_builder.js';
import { makeTestGroup } from '../../common/framework/test_group.js';
import { unreachable } from '../../common/framework/util/util.js';
import { kUncompressedTextureFormatInfo } from '../capability_info.js';
import { GPUTest } from '../gpu_test.js';
import { getTexelDataRepresentation } from '../util/texture/texelData.js';

function calculateRowPitch(width, bytesPerPixel) {
  const bytesPerRow = width * bytesPerPixel;
  // Rounds up to a multiple of 256 according to WebGPU requirements.
  return (((bytesPerRow - 1) >> 8) + 1) << 8;
}
var Color;

// Cache for generated pixels.
(function (Color) {
  Color[(Color['Red'] = 0)] = 'Red';
  Color[(Color['Green'] = 1)] = 'Green';
  Color[(Color['Blue'] = 2)] = 'Blue';
  Color[(Color['White'] = 3)] = 'White';
  Color[(Color['OpaqueBlack'] = 4)] = 'OpaqueBlack';
  Color[(Color['TransparentBlack'] = 5)] = 'TransparentBlack';
})(Color || (Color = {}));
const generatedPixelCache = new Map();

class F extends GPUTest {
  checkCopyImageBitmapResult(src, expected, width, height, bytesPerPixel) {
    const exp = new Uint8Array(expected.buffer, expected.byteOffset, expected.byteLength);
    const rowPitch = calculateRowPitch(width, bytesPerPixel);
    const dst = this.createCopyForMapRead(src, 0, rowPitch * height);

    this.eventualAsyncExpectation(async niceStack => {
      await dst.mapAsync(GPUMapMode.READ);
      const actual = new Uint8Array(dst.getMappedRange());
      const check = this.checkBufferWithRowPitch(
        actual,
        exp,
        width,
        height,
        rowPitch,
        bytesPerPixel
      );

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

  doTestAndCheckResult(
    imageBitmapCopyView,
    dstTextureCopyView,
    copySize,
    bytesPerPixel,
    expectedData
  ) {
    this.device.defaultQueue.copyImageBitmapToTexture(
      imageBitmapCopyView,
      dstTextureCopyView,
      copySize
    );

    const imageBitmap = imageBitmapCopyView.imageBitmap;
    const dstTexture = dstTextureCopyView.texture;

    const bytesPerRow = calculateRowPitch(imageBitmap.width, bytesPerPixel);
    const testBuffer = this.device.createBuffer({
      size: bytesPerRow * imageBitmap.height,
      usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST,
    });

    const encoder = this.device.createCommandEncoder();

    encoder.copyTextureToBuffer(
      { texture: dstTexture, mipLevel: 0, origin: { x: 0, y: 0, z: 0 } },
      { buffer: testBuffer, bytesPerRow },
      { width: imageBitmap.width, height: imageBitmap.height, depth: 1 }
    );

    this.device.defaultQueue.submit([encoder.finish()]);

    this.checkCopyImageBitmapResult(
      testBuffer,
      expectedData,
      imageBitmap.width,
      imageBitmap.height,
      bytesPerPixel
    );
  }

  generatePixel(color, format) {
    var _generatedPixelCache$, _generatedPixelCache$3;
    if (!generatedPixelCache.get(format)) {
      generatedPixelCache.set(format, new Map());
    }

    // None of the dst texture format is 'uint' or 'sint', so we can always use float value.
    if (
      !((_generatedPixelCache$ = generatedPixelCache.get(format)) === null ||
      _generatedPixelCache$ === void 0
        ? void 0
        : _generatedPixelCache$.has(color))
    ) {
      var _generatedPixelCache$2;
      let pixels;
      switch (color) {
        case Color.Red:
          pixels = new Uint8Array(
            getTexelDataRepresentation(format).getBytes({ R: 1.0, G: 0, B: 0, A: 1.0 })
          );

          break;
        case Color.Green:
          pixels = new Uint8Array(
            getTexelDataRepresentation(format).getBytes({ R: 0, G: 1.0, B: 0, A: 1.0 })
          );

          break;
        case Color.Blue:
          pixels = new Uint8Array(
            getTexelDataRepresentation(format).getBytes({ R: 0, G: 0, B: 1.0, A: 1.0 })
          );

          break;
        case Color.White:
          pixels = new Uint8Array(
            getTexelDataRepresentation(format).getBytes({ R: 0, G: 0, B: 0, A: 1.0 })
          );

          break;
        case Color.OpaqueBlack:
          pixels = new Uint8Array(
            getTexelDataRepresentation(format).getBytes({ R: 1.0, G: 1.0, B: 1.0, A: 1.0 })
          );

          break;
        case Color.TransparentBlack:
          pixels = new Uint8Array(
            getTexelDataRepresentation(format).getBytes({ R: 1.0, G: 1.0, B: 1.0, A: 0 })
          );

          break;
        default:
          unreachable();
      }

      (_generatedPixelCache$2 = generatedPixelCache.get(format)) === null ||
      _generatedPixelCache$2 === void 0
        ? void 0
        : _generatedPixelCache$2.set(color, pixels);
    }

    return (_generatedPixelCache$3 = generatedPixelCache.get(format)) === null ||
      _generatedPixelCache$3 === void 0
      ? void 0
      : _generatedPixelCache$3.get(color);
  }
}

export const g = makeTestGroup(F);

g.test('from_ImageData')
  .params(
    params()
      .combine(poptions('width', [1, 2, 4, 15, 255, 256]))
      .combine(poptions('height', [1, 2, 4, 15, 255, 256]))
      .combine(poptions('alpha', ['none', 'premultiply']))
      .combine(poptions('orientation', ['none', 'flipY']))
      .combine(
        poptions('dstColorFormat', [
          'rgba8unorm',
          'bgra8unorm',
          'rgba8unorm-srgb',
          'bgra8unorm-srgb',
          'rgb10a2unorm',
          'rgba16float',
          'rgba32float',
          'rg8unorm',
          'rg16float',
        ])
      )
  )
  .fn(async t => {
    const { width, height, alpha, orientation, dstColorFormat } = t.params;

    const format = 'rgba8unorm';
    const srcBytesPerPixel = kUncompressedTextureFormatInfo[format].bytesPerBlock;

    // Generate input contents by iterating 'Color' enum
    const imagePixels = new Uint8ClampedArray(srcBytesPerPixel * width * height);
    const startPixel = Color.Red;
    for (let i = 0, currentPixel = startPixel; i < width * height; ++i) {
      const pixels = t.generatePixel(currentPixel, format);
      if (currentPixel === Color.TransparentBlack) {
        currentPixel = Color.Red;
      } else {
        ++currentPixel;
      }
      for (let j = 0; j < srcBytesPerPixel; ++j) {
        imagePixels[i * srcBytesPerPixel + j] = pixels[j];
      }
    }

    // Generate correct expected values
    const imageData = new ImageData(imagePixels, width, height);

    const imageBitmap = await createImageBitmap(imageData, {
      premultiplyAlpha: alpha,
      imageOrientation: orientation,
    });

    const dst = t.device.createTexture({
      size: {
        width: imageBitmap.width,
        height: imageBitmap.height,
        depth: 1,
      },

      format: dstColorFormat,
      usage: GPUTextureUsage.COPY_DST | GPUTextureUsage.COPY_SRC,
    });

    // Construct expected value for different dst color format
    const dstBytesPerPixel = kUncompressedTextureFormatInfo[dstColorFormat].bytesPerBlock;
    const dstPixels = new Uint8ClampedArray(dstBytesPerPixel * width * height);
    let expectedPixels = new Uint8ClampedArray(dstBytesPerPixel * width * height);
    for (let i = 0, currentPixel = startPixel; i < width * height; ++i) {
      const pixels = t.generatePixel(currentPixel, dstColorFormat);
      for (let j = 0; j < dstBytesPerPixel; ++j) {
        // All pixels are 0 due to premultiply alpha
        if (alpha === 'premultiply' && currentPixel === Color.TransparentBlack) {
          dstPixels[i * dstBytesPerPixel + j] = 0;
        } else {
          dstPixels[i * dstBytesPerPixel + j] = pixels[j];
        }
      }

      if (currentPixel === Color.TransparentBlack) {
        currentPixel = Color.Red;
      } else {
        ++currentPixel;
      }
    }

    if (orientation === 'flipY') {
      for (let i = 0; i < height; ++i) {
        for (let j = 0; j < width * dstBytesPerPixel; ++j) {
          const posImagePixel = (height - i - 1) * width * dstBytesPerPixel + j;
          const posExpectedValue = i * width * dstBytesPerPixel + j;
          expectedPixels[posExpectedValue] = dstPixels[posImagePixel];
        }
      }
    } else {
      expectedPixels = dstPixels;
    }

    t.doTestAndCheckResult(
      { imageBitmap, origin: { x: 0, y: 0 } },
      { texture: dst },
      { width: imageBitmap.width, height: imageBitmap.height, depth: 1 },
      dstBytesPerPixel,
      expectedPixels
    );
  });

g.test('from_canvas')
  .params(
    params()
      .combine(poptions('width', [1, 2, 4, 15, 255, 256]))
      .combine(poptions('height', [1, 2, 4, 15, 255, 256]))
  )
  .fn(async t => {
    const { width, height } = t.params;

    // CTS sometimes runs on worker threads, where document is not available.
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
    }

    // The texture format is rgba8unorm, so the bytes per pixel is 4.
    const bytesPerPixel = 4;

    // Generate original data.
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
        depth: 1,
      },

      format: 'rgba8unorm',
      usage: GPUTextureUsage.COPY_DST | GPUTextureUsage.COPY_SRC,
    });

    // This will get origin data and even it has premultiplied-alpha
    const expectedData = imageCanvasContext.getImageData(
      0,
      0,
      imageBitmap.width,
      imageBitmap.height
    ).data;

    t.doTestAndCheckResult(
      { imageBitmap, origin: { x: 0, y: 0 } },
      { texture: dst },
      { width: imageBitmap.width, height: imageBitmap.height, depth: 1 },
      bytesPerPixel,
      expectedData
    );
  });
