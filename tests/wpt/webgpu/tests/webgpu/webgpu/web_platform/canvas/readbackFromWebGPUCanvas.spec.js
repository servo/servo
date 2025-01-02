/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests for readback from WebGPU Canvas.

This includes testing that colorSpace makes it through from the WebGPU canvas
to the form of copy (toDataURL, toBlob, ImageBitmap, drawImage)

The color space support is tested by drawing the readback form of the WebGPU
canvas into a 2D canvas of a different color space via drawImage (A). Another
2D canvas is created with the same source data and color space as the WebGPU
canvas and also drawn into another 2D canvas of a different color space (B).
The contents of A and B should match.

TODO: implement all canvas types, see TODO on kCanvasTypes.
`;import { makeTestGroup } from '../../../common/framework/test_group.js';
import {
  ErrorWithExtra,
  assert,
  raceWithRejectOnTimeout,
  unreachable } from
'../../../common/util/util.js';
import {
  kCanvasAlphaModes,
  kCanvasColorSpaces,
  kCanvasTextureFormats } from
'../../capability_info.js';
import { GPUTest } from '../../gpu_test.js';
import { checkElementsEqual } from '../../util/check_contents.js';
import {
  kAllCanvasTypes,

  createCanvas,
  createOnscreenCanvas,
  createOffscreenCanvas } from
'../../util/create_elements.js';
import { TexelView } from '../../util/texture/texel_view.js';
import { findFailedPixels } from '../../util/texture/texture_ok.js';

export const g = makeTestGroup(GPUTest);

// We choose 0x66 as the value for each color and alpha channel
// 0x66 / 0xff = 0.4
// Given a pixel value of RGBA = (0x66, 0, 0, 0x66) in the source WebGPU canvas,
// For alphaMode = opaque, the copy output should be RGBA = (0x66, 0, 0, 0xff)
// For alphaMode = premultiplied, the copy output should be RGBA = (0xff, 0, 0, 0x66)
const kPixelValue = 0x66;
const kPixelValueFloat = 0x66 / 0xff; // 0.4

// Use four pixels rectangle for the test:
// blue: top-left;
// green: top-right;
// red: bottom-left;
// yellow: bottom-right;
const expect = {

  'opaque': new Uint8ClampedArray([
  0x00, 0x00, kPixelValue, 0xff, // blue
  0x00, kPixelValue, 0x00, 0xff, // green
  kPixelValue, 0x00, 0x00, 0xff, // red
  kPixelValue, kPixelValue, 0x00, 0xff // yellow
  ]),

  'premultiplied': new Uint8ClampedArray([
  0x00, 0x00, 0xff, kPixelValue, // blue
  0x00, 0xff, 0x00, kPixelValue, // green
  0xff, 0x00, 0x00, kPixelValue, // red
  0xff, 0xff, 0x00, kPixelValue // yellow
  ])
};

/**
 * Given 4 pixels in rgba8unorm format, puts them into an ImageData
 * of the specified color space and then puts them into an srgb color space
 * canvas (the default). If the color space is different there will be a
 * conversion. Returns the resulting 4 pixels in rgba8unorm format.
 */
function convertRGBA8UnormBytesToColorSpace(
expected,
srcColorSpace,
dstColorSpace)
{
  const srcImgData = new ImageData(2, 2, { colorSpace: srcColorSpace });
  srcImgData.data.set(expected);
  const dstCanvas = new OffscreenCanvas(2, 2);
  const dstCtx = dstCanvas.getContext('2d', { colorSpace: dstColorSpace });
  assert(dstCtx !== null);
  dstCtx.putImageData(srcImgData, 0, 0);
  return dstCtx.getImageData(0, 0, 2, 2).data;
}

function initWebGPUCanvasContent(
t,
format,
alphaMode,
colorSpace,
canvasType)
{
  const canvas = createCanvas(t, canvasType, 2, 2);
  const ctx = canvas.getContext('webgpu');
  assert(ctx instanceof GPUCanvasContext, 'Failed to get WebGPU context from canvas');

  ctx.configure({
    device: t.device,
    format,
    usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.COPY_DST,
    alphaMode,
    colorSpace
  });

  const canvasTexture = ctx.getCurrentTexture();
  const tempTexture = t.createTextureTracked({
    size: { width: 1, height: 1, depthOrArrayLayers: 1 },
    format,
    usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT
  });
  const tempTextureView = tempTexture.createView();
  const encoder = t.device.createCommandEncoder();

  const clearOnePixel = (origin, color) => {
    const pass = encoder.beginRenderPass({
      colorAttachments: [
      { view: tempTextureView, clearValue: color, loadOp: 'clear', storeOp: 'store' }]

    });
    pass.end();
    encoder.copyTextureToTexture(
      { texture: tempTexture },
      { texture: canvasTexture, origin },
      { width: 1, height: 1 }
    );
  };

  clearOnePixel([0, 0], [0, 0, kPixelValueFloat, kPixelValueFloat]);
  clearOnePixel([1, 0], [0, kPixelValueFloat, 0, kPixelValueFloat]);
  clearOnePixel([0, 1], [kPixelValueFloat, 0, 0, kPixelValueFloat]);
  clearOnePixel([1, 1], [kPixelValueFloat, kPixelValueFloat, 0, kPixelValueFloat]);

  t.device.queue.submit([encoder.finish()]);
  tempTexture.destroy();

  return canvas;
}

function drawImageSourceIntoCanvas(
t,
image,
colorSpace)
{
  const canvas = createOffscreenCanvas(t, 2, 2);
  const ctx = canvas.getContext('2d', { colorSpace });
  assert(ctx !== null);
  ctx.drawImage(image, 0, 0);
  return ctx;
}

function checkImageResultWithSameColorSpaceCanvas(
t,
image,
sourceColorSpace,
expect)
{
  const ctx = drawImageSourceIntoCanvas(t, image, sourceColorSpace);
  readPixelsFrom2DCanvasAndCompare(t, ctx, expect);
}

function checkImageResultWithDifferentColorSpaceCanvas(
t,
image,
sourceColorSpace,
sourceData)
{
  const destinationColorSpace = sourceColorSpace === 'srgb' ? 'display-p3' : 'srgb';

  // draw the WebGPU derived data into a canvas
  const fromWebGPUCtx = drawImageSourceIntoCanvas(t, image, destinationColorSpace);

  const expect = convertRGBA8UnormBytesToColorSpace(
    sourceData,
    sourceColorSpace,
    destinationColorSpace
  );

  readPixelsFrom2DCanvasAndCompare(t, fromWebGPUCtx, expect, 2);
}

function checkImageResult(
t,
image,
sourceColorSpace,
expect)
{
  // canvas(colorSpace)->img(colorSpace)->canvas(colorSpace).drawImage->canvas(colorSpace).getImageData->actual
  // hard coded data->expected
  checkImageResultWithSameColorSpaceCanvas(t, image, sourceColorSpace, expect);

  // canvas(colorSpace)->img(colorSpace)->canvas(diffColorSpace).drawImage->canvas(diffColorSpace).getImageData->actual
  // hard coded data->ImageData(colorSpace)->canvas(diffColorSpace).putImageData->canvas(diffColorSpace).getImageData->expected
  checkImageResultWithDifferentColorSpaceCanvas(t, image, sourceColorSpace, expect);
}

function readPixelsFrom2DCanvasAndCompare(
t,
ctx,
expect,
maxDiffULPsForNormFormat = 0)
{
  const { width, height } = ctx.canvas;
  const actual = ctx.getImageData(0, 0, width, height).data;

  const subrectOrigin = [0, 0, 0];
  const subrectSize = [width, height, 1];

  const areaDesc = {
    bytesPerRow: width * 4,
    rowsPerImage: height,
    subrectOrigin,
    subrectSize
  };

  const format = 'rgba8unorm';
  const actTexelView = TexelView.fromTextureDataByReference(format, actual, areaDesc);
  const expTexelView = TexelView.fromTextureDataByReference(format, expect, areaDesc);

  const failedPixelsMessage = findFailedPixels(
    format,
    { x: 0, y: 0, z: 0 },
    { width, height, depthOrArrayLayers: 1 },
    { actTexelView, expTexelView },
    { maxDiffULPsForNormFormat }
  );

  if (failedPixelsMessage !== undefined) {
    const msg = 'Canvas had unexpected contents:\n' + failedPixelsMessage;
    t.expectOK(
      new ErrorWithExtra(msg, () => ({
        expTexelView,
        actTexelView
      }))
    );
  }
}

g.test('onscreenCanvas,snapshot').
desc(
  `
    Ensure snapshot of canvas with WebGPU context is correct with
    - various WebGPU canvas texture formats
    - WebGPU canvas alpha mode = {"opaque", "premultiplied"}
    - colorSpace = {"srgb", "display-p3"}
    - snapshot methods = {convertToBlob, transferToImageBitmap, createImageBitmap}

    TODO: Snapshot canvas to jpeg, webp and other mime type and
          different quality. Maybe we should test them in reftest.
    `
).
params((u) =>
u //
.combine('format', kCanvasTextureFormats).
combine('alphaMode', kCanvasAlphaModes).
combine('colorSpace', kCanvasColorSpaces).
combine('snapshotType', ['toDataURL', 'toBlob', 'imageBitmap'])
).
fn(async (t) => {
  const canvas = initWebGPUCanvasContent(
    t,
    t.params.format,
    t.params.alphaMode,
    t.params.colorSpace,
    'onscreen'
  );

  let snapshot;
  switch (t.params.snapshotType) {
    case 'toDataURL':{
        const url = canvas.toDataURL();
        const img = new Image(canvas.width, canvas.height);
        img.src = url;
        await raceWithRejectOnTimeout(img.decode(), 5000, 'load image timeout');
        snapshot = img;
        break;
      }
    case 'toBlob':{
        const blobFromCanvas = new Promise((resolve) => {
          canvas.toBlob((blob) => resolve(blob));
        });
        const blob = await blobFromCanvas;
        const url = URL.createObjectURL(blob);
        const img = new Image(canvas.width, canvas.height);
        img.src = url;
        await raceWithRejectOnTimeout(img.decode(), 5000, 'load image timeout');
        snapshot = img;
        break;
      }
    case 'imageBitmap':{
        snapshot = await createImageBitmap(canvas);
        break;
      }
    default:
      unreachable();
  }

  checkImageResult(t, snapshot, t.params.colorSpace, expect[t.params.alphaMode]);
});

g.test('offscreenCanvas,snapshot').
desc(
  `
    Ensure snapshot of offscreenCanvas with WebGPU context is correct with
    - various WebGPU canvas texture formats
    - WebGPU canvas alpha mode = {"opaque", "premultiplied"}
    - colorSpace = {"srgb", "display-p3"}
    - snapshot methods = {convertToBlob, transferToImageBitmap, createImageBitmap}

    TODO: Snapshot offscreenCanvas to jpeg, webp and other mime type and
          different quality. Maybe we should test them in reftest.
    `
).
params((u) =>
u //
.combine('format', kCanvasTextureFormats).
combine('alphaMode', kCanvasAlphaModes).
combine('colorSpace', kCanvasColorSpaces).
combine('snapshotType', ['convertToBlob', 'transferToImageBitmap', 'imageBitmap'])
).
fn(async (t) => {
  const offscreenCanvas = initWebGPUCanvasContent(
    t,
    t.params.format,
    t.params.alphaMode,
    t.params.colorSpace,
    'offscreen'
  );

  let snapshot;
  switch (t.params.snapshotType) {
    case 'convertToBlob':{
        if (typeof offscreenCanvas.convertToBlob === 'undefined') {
          t.skip("Browser doesn't support OffscreenCanvas.convertToBlob");
          return;
        }
        const blob = await offscreenCanvas.convertToBlob();
        snapshot = await createImageBitmap(blob);
        break;
      }
    case 'transferToImageBitmap':{
        if (typeof offscreenCanvas.transferToImageBitmap === 'undefined') {
          t.skip("Browser doesn't support OffscreenCanvas.transferToImageBitmap");
          return;
        }
        snapshot = offscreenCanvas.transferToImageBitmap();
        break;
      }
    case 'imageBitmap':{
        snapshot = await createImageBitmap(offscreenCanvas);
        break;
      }
    default:
      unreachable();
  }

  checkImageResult(t, snapshot, t.params.colorSpace, expect[t.params.alphaMode]);
});

g.test('onscreenCanvas,uploadToWebGL').
desc(
  `
    Ensure upload WebGPU context canvas to webgl texture is correct with
    - various WebGPU canvas texture formats
    - WebGPU canvas alpha mode = {"opaque", "premultiplied"}
    - upload methods = {texImage2D, texSubImage2D}
    `
).
params((u) =>
u //
.combine('format', kCanvasTextureFormats).
combine('alphaMode', kCanvasAlphaModes).
combine('webgl', ['webgl', 'webgl2']).
combine('upload', ['texImage2D', 'texSubImage2D'])
).
fn((t) => {
  const { format, webgl, upload } = t.params;
  const canvas = initWebGPUCanvasContent(t, format, t.params.alphaMode, 'srgb', 'onscreen');

  const expectCanvas = createOnscreenCanvas(t, canvas.width, canvas.height);
  const gl = expectCanvas.getContext(webgl);
  if (gl === null) {
    return;
  }


  const texture = gl.createTexture();
  gl.bindTexture(gl.TEXTURE_2D, texture);
  switch (upload) {
    case 'texImage2D':{
        gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, canvas);
        break;
      }
    case 'texSubImage2D':{
        gl.texImage2D(
          gl.TEXTURE_2D,
          0,
          gl.RGBA,
          canvas.width,
          canvas.height,
          0,
          gl.RGBA,
          gl.UNSIGNED_BYTE,
          null
        );
        gl.texSubImage2D(gl.TEXTURE_2D, 0, 0, 0, gl.RGBA, gl.UNSIGNED_BYTE, canvas);
        break;
      }
    default:
      unreachable();
  }

  const fb = gl.createFramebuffer();

  gl.bindFramebuffer(gl.FRAMEBUFFER, fb);
  gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, texture, 0);

  const pixels = new Uint8Array(canvas.width * canvas.height * 4);
  gl.readPixels(0, 0, 2, 2, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
  const actual = new Uint8ClampedArray(pixels);

  t.expectOK(checkElementsEqual(actual, expect[t.params.alphaMode]));
});

g.test('drawTo2DCanvas').
desc(
  `
    Ensure draw WebGPU context canvas to 2d context canvas/offscreenCanvas is correct with
    - various WebGPU canvas texture formats
    - WebGPU canvas alpha mode = {"opaque", "premultiplied"}
    - colorSpace = {"srgb", "display-p3"}
    - WebGPU canvas type = {"onscreen", "offscreen"}
    - 2d canvas type = {"onscreen", "offscreen"}


    * makes a webgpu canvas with the given colorSpace and puts data in via copy convoluted
      copy process
    * makes a 2d canvas with 'srgb' colorSpace (the default)
    * draws the webgpu canvas into the 2d canvas so if the color spaces do not match
      there will be a conversion.
    * gets the pixels from the 2d canvas via getImageData
    * compares them to hard coded values that are converted to expected values by copying
      to an ImageData of the given color space, and then using putImageData into an srgb canvas.

      canvas(colorSpace) -> canvas(srgb).drawImage -> canvas(srgb).getImageData -> actual
      ImageData(colorSpace) -> canvas(srgb).putImageData -> canvas(srgb).getImageData -> expected
    `
).
params((u) =>
u //
.combine('format', kCanvasTextureFormats).
combine('alphaMode', kCanvasAlphaModes).
combine('colorSpace', kCanvasColorSpaces).
combine('webgpuCanvasType', kAllCanvasTypes).
combine('canvas2DType', kAllCanvasTypes)
).
fn((t) => {
  const { format, webgpuCanvasType, alphaMode, colorSpace, canvas2DType } = t.params;

  const webgpuCanvas = initWebGPUCanvasContent(
    t,
    format,
    alphaMode,
    colorSpace,
    webgpuCanvasType
  );

  const actualCanvas = createCanvas(t, canvas2DType, webgpuCanvas.width, webgpuCanvas.height);
  const ctx = actualCanvas.getContext('2d');
  if (ctx === null) {
    t.skip(canvas2DType + ' canvas cannot get 2d context');
    return;
  }

  ctx.drawImage(webgpuCanvas, 0, 0);

  readPixelsFrom2DCanvasAndCompare(
    t,
    ctx,
    convertRGBA8UnormBytesToColorSpace(expect[t.params.alphaMode], colorSpace, 'srgb')
  );
});

g.test('transferToImageBitmap_unconfigured_nonzero_size').
desc(
  `Regression test for a crash when calling transferImageBitmap on an unconfigured. Case where the canvas is not empty`
).
params((u) => u.combine('readbackCanvasType', ['onscreen', 'offscreen'])).
fn((t) => {
  const kWidth = 2;
  const kHeight = 3;
  const canvas = createCanvas(t, 'offscreen', kWidth, kHeight);
  canvas.getContext('webgpu');

  // Transferring gives an ImageBitmap of the correct size filled with transparent black.
  const ib = canvas.transferToImageBitmap();
  t.expect(ib.width === kWidth);
  t.expect(ib.height === kHeight);

  const readbackCanvas = createCanvas(t, t.params.readbackCanvasType, kWidth, kHeight);
  const readbackContext = readbackCanvas.getContext('2d', {
    alpha: true
  });
  if (readbackContext === null) {
    t.skip('Cannot get a 2D canvas context');
    return;
  }

  // Since there isn't a configuration we expect the ImageBitmap to have the default alphaMode of "opaque".
  const expected = new Uint8ClampedArray(kWidth * kHeight * 4);
  for (let i = 0; i < expected.byteLength; i += 4) {
    expected[i + 0] = 0;
    expected[i + 1] = 0;
    expected[i + 2] = 0;
    expected[i + 3] = 255;
  }

  readbackContext.drawImage(ib, 0, 0);
  readPixelsFrom2DCanvasAndCompare(t, readbackContext, expected);
});

g.test('transferToImageBitmap_zero_size').
desc(
  `Regression test for a crash when calling transferImageBitmap on an unconfigured. Case where the canvas is empty.

    TODO: Spec and expect a particular Exception type here.`
).
params((u) => u.combine('configure', [true, false])).
fn((t) => {
  const { configure } = t.params;
  const canvas = createCanvas(t, 'offscreen', 0, 1);
  const ctx = canvas.getContext('webgpu');

  if (configure) {
    t.expectValidationError(() => ctx.configure({ device: t.device, format: 'bgra8unorm' }));
  }

  // Transferring would give an empty ImageBitmap which is not possible, so an Exception is thrown.
  t.shouldThrow(true, () => {
    canvas.transferToImageBitmap();
  });
});

g.test('transferToImageBitmap_huge_size').
desc(`Regression test for a crash when calling transferImageBitmap on a HUGE canvas.`).
fn((t) => {
  const canvas = createCanvas(t, 'offscreen', 1000000, 1000000);
  canvas.getContext('webgpu');

  // Transferring to such a HUGE image bitmap would not be possible, so an Exception is thrown.
  t.shouldThrow(true, () => {
    canvas.transferToImageBitmap();
  });
});