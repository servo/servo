/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

export const description = `
copy imageBitmap To texture tests.
`;
import { TestGroup, pcombine, poptions } from '../../framework/index.js';
import { GPUTest } from './gpu_test.js';

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
        this.rec.fail(niceStack);
      }

      dst.destroy();
    });
  }

  checkBufferWithRowPitch(actual, exp, width, height, rowPitch, bytesPerPixel) {
    const lines = [];
    let failedPixels = 0;

    for (let i = 0; i < height; ++i) {
      const bytesPerRow = width * bytesPerPixel;

      for (let j = 0; j < bytesPerRow; ++j) {
        const indexExp = j + i * bytesPerRow;
        const indexActual = j + rowPitch * i;

        if (actual[indexActual] !== exp[indexExp]) {
          if (failedPixels > 4) {
            break;
          }

          failedPixels++;
          lines.push(`at [${indexExp}], expected ${exp[indexExp]}, got ${actual[indexActual]}`);
        }
      }

      if (failedPixels > 4) {
        lines.push('... and more');
        break;
      }
    }

    return failedPixels > 0 ? lines.join('\n') : undefined;
  } // Using drawImage to extract imageBitmap content.


  imageBitmapToData(imageBitmap) {
    const imageCanvas = document.createElement('canvas');
    imageCanvas.width = imageBitmap.width;
    imageCanvas.height = imageBitmap.height;
    const imageCanvasContext = imageCanvas.getContext('2d');

    if (!imageCanvasContext) {
      throw new Error('Cannot create canvas context for reading back contents from imageBitmap.');
    }

    imageCanvasContext.drawImage(imageBitmap, 0, 0, imageBitmap.width, imageBitmap.height);
    return imageCanvasContext.getImageData(0, 0, imageBitmap.width, imageBitmap.height).data;
  }

}

export const g = new TestGroup(F);
g.test('from ImageData', async t => {
  const {
    width,
    height
  } = t.params; // The texture format is rgba8uint, so the bytes per pixel is 4.

  const bytesPerPixel = 4;
  const imagePixels = new Uint8ClampedArray(bytesPerPixel * width * height);

  for (let i = 0; i < width * height * bytesPerPixel; ++i) {
    imagePixels[i] = i % 4 === 3 ? 255 : i % 256;
  }

  const imageData = new ImageData(imagePixels, width, height);
  const imageBitmap = await createImageBitmap(imageData);
  const dst = t.device.createTexture({
    size: {
      width: imageBitmap.width,
      height: imageBitmap.height,
      depth: 1
    },
    format: 'rgba8uint',
    usage: GPUTextureUsage.COPY_DST | GPUTextureUsage.COPY_SRC
  });
  t.device.defaultQueue.copyImageBitmapToTexture({
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
  });
  const data = t.imageBitmapToData(imageBitmap);
  const rowPitchValue = calculateRowPitch(imageBitmap.width, bytesPerPixel);
  const testBuffer = t.device.createBuffer({
    size: rowPitchValue * imageBitmap.height,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  });
  const encoder = t.device.createCommandEncoder();
  encoder.copyTextureToBuffer({
    texture: dst,
    mipLevel: 0,
    origin: {
      x: 0,
      y: 0,
      z: 0
    }
  }, {
    buffer: testBuffer,
    rowPitch: rowPitchValue,
    imageHeight: 0
  }, {
    width: imageBitmap.width,
    height: imageBitmap.height,
    depth: 1
  });
  t.device.defaultQueue.submit([encoder.finish()]);
  t.checkCopyImageBitmapResult(testBuffer, data, imageBitmap.width, imageBitmap.height, bytesPerPixel);
}).params(pcombine(poptions('width', [1, 2, 4, 15, 255, 256]), //
poptions('height', [1, 2, 4, 15, 255, 256])));
//# sourceMappingURL=copyImageBitmapToTexture.spec.js.map