/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
copyToTexture with HTMLCanvasElement and OffscreenCanvas sources.
`;import { makeTestGroup } from '../../../common/framework/test_group.js';
import { skipTestCase } from '../../../common/util/util.js';
import { kCanvasAlphaModes } from '../../capability_info.js';
import {
  kTextureFormatInfo,
  kValidTextureFormatsForCopyE2T } from

'../../format_info.js';
import { CopyToTextureUtils } from '../../util/copy_to_texture.js';
import { kAllCanvasTypes, createCanvas } from '../../util/create_elements.js';


class F extends CopyToTextureUtils {
  init2DCanvasContentWithColorSpace({
    width,
    height,
    colorSpace




  })


  {
    const canvas = createCanvas(this, 'onscreen', width, height);

    let canvasContext = null;
    canvasContext = canvas.getContext('2d', { colorSpace });

    if (canvasContext === null) {
      this.skip('onscreen canvas 2d context not available');
    }

    if (
    typeof canvasContext.getContextAttributes === 'undefined' ||
    typeof canvasContext.getContextAttributes().colorSpace === 'undefined')
    {
      this.skip('color space attr is not supported for canvas 2d context');
    }

    const SOURCE_PIXEL_BYTES = 4;
    const imagePixels = new Uint8ClampedArray(SOURCE_PIXEL_BYTES * width * height);

    const rectWidth = Math.floor(width / 2);
    const rectHeight = Math.floor(height / 2);

    const alphaValue = 153;

    let pixelStartPos = 0;
    // Red;
    for (let i = 0; i < rectHeight; ++i) {
      for (let j = 0; j < rectWidth; ++j) {
        pixelStartPos = (i * width + j) * SOURCE_PIXEL_BYTES;
        imagePixels[pixelStartPos] = 255;
        imagePixels[pixelStartPos + 1] = 0;
        imagePixels[pixelStartPos + 2] = 0;
        imagePixels[pixelStartPos + 3] = alphaValue;
      }
    }

    // Lime;
    for (let i = 0; i < rectHeight; ++i) {
      for (let j = rectWidth; j < width; ++j) {
        pixelStartPos = (i * width + j) * SOURCE_PIXEL_BYTES;
        imagePixels[pixelStartPos] = 0;
        imagePixels[pixelStartPos + 1] = 255;
        imagePixels[pixelStartPos + 2] = 0;
        imagePixels[pixelStartPos + 3] = alphaValue;
      }
    }

    // Blue
    for (let i = rectHeight; i < height; ++i) {
      for (let j = 0; j < rectWidth; ++j) {
        pixelStartPos = (i * width + j) * SOURCE_PIXEL_BYTES;
        imagePixels[pixelStartPos] = 0;
        imagePixels[pixelStartPos + 1] = 0;
        imagePixels[pixelStartPos + 2] = 255;
        imagePixels[pixelStartPos + 3] = alphaValue;
      }
    }

    // Fuchsia
    for (let i = rectHeight; i < height; ++i) {
      for (let j = rectWidth; j < width; ++j) {
        pixelStartPos = (i * width + j) * SOURCE_PIXEL_BYTES;
        imagePixels[pixelStartPos] = 255;
        imagePixels[pixelStartPos + 1] = 0;
        imagePixels[pixelStartPos + 2] = 255;
        imagePixels[pixelStartPos + 3] = alphaValue;
      }
    }

    const imageData = new ImageData(imagePixels, width, height, { colorSpace });
    if (typeof imageData.colorSpace === 'undefined') {
      this.skip('color space attr is not supported for ImageData');
    }

    const ctx = canvasContext;
    ctx.putImageData(imageData, 0, 0);

    return {
      canvas,
      expectedSourceData: this.getExpectedReadbackFor2DCanvas(canvasContext, width, height)
    };
  }

  // MAINTENANCE_TODO: Cache the generated canvas to avoid duplicated initialization.
  init2DCanvasContent({
    canvasType,
    width,
    height




  })


  {
    const canvas = createCanvas(this, canvasType, width, height);

    let canvasContext = null;
    canvasContext = canvas.getContext('2d');

    if (canvasContext === null) {
      this.skip(canvasType + ' canvas 2d context not available');
    }

    const ctx = canvasContext;
    this.paint2DCanvas(ctx, width, height, 0.6);

    return {
      canvas,
      expectedSourceData: this.getExpectedReadbackFor2DCanvas(canvasContext, width, height)
    };
  }

  paint2DCanvas(
  ctx,
  width,
  height,
  alphaValue)
  {
    const rectWidth = Math.floor(width / 2);
    const rectHeight = Math.floor(height / 2);

    // Red
    ctx.fillStyle = `rgba(255, 0, 0, ${alphaValue})`;
    ctx.fillRect(0, 0, rectWidth, rectHeight);
    // Lime
    ctx.fillStyle = `rgba(0, 255, 0, ${alphaValue})`;
    ctx.fillRect(rectWidth, 0, width - rectWidth, rectHeight);
    // Blue
    ctx.fillStyle = `rgba(0, 0, 255, ${alphaValue})`;
    ctx.fillRect(0, rectHeight, rectWidth, height - rectHeight);
    // Fuchsia
    ctx.fillStyle = `rgba(255, 0, 255, ${alphaValue})`;
    ctx.fillRect(rectWidth, rectHeight, width - rectWidth, height - rectHeight);
  }

  // MAINTENANCE_TODO: Cache the generated canvas to avoid duplicated initialization.
  initGLCanvasContent({
    canvasType,
    contextName,
    width,
    height,
    premultiplied






  })


  {
    const canvas = createCanvas(this, canvasType, width, height);

    // MAINTENANCE_TODO: Workaround for @types/offscreencanvas missing an overload of
    // `OffscreenCanvas.getContext` that takes `string` or a union of context types.
    const gl = canvas.getContext(contextName, {
      premultipliedAlpha: premultiplied
    });

    if (gl === null) {
      this.skip(canvasType + ' canvas ' + contextName + ' context not available');
    }
    this.trackForCleanup(gl);

    const rectWidth = Math.floor(width / 2);
    const rectHeight = Math.floor(height / 2);

    const alphaValue = 0.6;
    const colorValue = premultiplied ? alphaValue : 1.0;

    // For webgl/webgl2 context canvas, if the context created with premultipliedAlpha attributes,
    // it means that the value in drawing buffer is premultiplied or not. So we should set
    // premultipliedAlpha value for premultipliedAlpha true gl context and unpremultipliedAlpha value
    // for the premultipliedAlpha false gl context.
    gl.enable(gl.SCISSOR_TEST);
    gl.scissor(0, 0, rectWidth, rectHeight);
    gl.clearColor(colorValue, 0.0, 0.0, alphaValue);
    gl.clear(gl.COLOR_BUFFER_BIT);

    gl.scissor(rectWidth, 0, width - rectWidth, rectHeight);
    gl.clearColor(0.0, colorValue, 0.0, alphaValue);
    gl.clear(gl.COLOR_BUFFER_BIT);

    gl.scissor(0, rectHeight, rectWidth, height - rectHeight);
    gl.clearColor(0.0, 0.0, colorValue, alphaValue);
    gl.clear(gl.COLOR_BUFFER_BIT);

    gl.scissor(rectWidth, rectHeight, width - rectWidth, height - rectHeight);
    gl.clearColor(colorValue, colorValue, colorValue, alphaValue);
    gl.clear(gl.COLOR_BUFFER_BIT);

    return {
      canvas,
      expectedSourceData: this.getExpectedReadbackForWebGLCanvas(gl, width, height)
    };
  }

  getDataToInitSourceWebGPUCanvas(
  width,
  height,
  alphaMode)
  {
    const rectWidth = Math.floor(width / 2);
    const rectHeight = Math.floor(height / 2);

    const alphaValue = 153;
    // Always output [153, 153, 153, 153]. When the alphaMode is...
    //   - premultiplied: the readback is CSS `rgba(255, 255, 255, 60%)`.
    //   - opaque: the readback is CSS `rgba(153, 153, 153, 100%)`.
    // getExpectedReadbackForWebGPUCanvas matches this.
    const colorValue = alphaValue;

    // BGRA8Unorm texture
    const initialData = new Uint8ClampedArray(4 * width * height);
    const maxRectHeightIndex = width * rectHeight;
    for (let pixelIndex = 0; pixelIndex < initialData.length / 4; ++pixelIndex) {
      const index = pixelIndex * 4;

      // Top-half two rectangles
      if (pixelIndex < maxRectHeightIndex) {
        // top-left side rectangle
        if (pixelIndex % width < rectWidth) {
          // top-left side rectangle
          initialData[index] = colorValue;
          initialData[index + 1] = 0;
          initialData[index + 2] = 0;
          initialData[index + 3] = alphaValue;
        } else {
          // top-right side rectangle
          initialData[index] = 0;
          initialData[index + 1] = colorValue;
          initialData[index + 2] = 0;
          initialData[index + 3] = alphaValue;
        }
      } else {
        // Bottom-half two rectangles
        // bottom-left side rectangle
        if (pixelIndex % width < rectWidth) {
          initialData[index] = 0;
          initialData[index + 1] = 0;
          initialData[index + 2] = colorValue;
          initialData[index + 3] = alphaValue;
        } else {
          // bottom-right side rectangle
          initialData[index] = colorValue;
          initialData[index + 1] = colorValue;
          initialData[index + 2] = colorValue;
          initialData[index + 3] = alphaValue;
        }
      }
    }
    return initialData;
  }

  initSourceWebGPUCanvas({
    device,
    canvasType,
    width,
    height,
    alphaMode






  })


  {
    const canvas = createCanvas(this, canvasType, width, height);

    const gpuContext = canvas.getContext('webgpu');

    if (!(gpuContext instanceof GPUCanvasContext)) {
      this.skip(canvasType + ' canvas webgpu context not available');
    }

    gpuContext.configure({
      device,
      format: 'bgra8unorm',
      usage: GPUTextureUsage.COPY_DST | GPUTextureUsage.COPY_SRC,
      alphaMode
    });

    // BGRA8Unorm texture
    const initialData = this.getDataToInitSourceWebGPUCanvas(width, height, alphaMode);
    const canvasTexture = gpuContext.getCurrentTexture();
    device.queue.writeTexture(
      { texture: canvasTexture },
      initialData,
      {
        bytesPerRow: width * 4,
        rowsPerImage: height
      },
      {
        width,
        height,
        depthOrArrayLayers: 1
      }
    );

    return {
      canvas,
      expectedSourceData: this.getExpectedReadbackForWebGPUCanvas(width, height, alphaMode)
    };
  }

  getExpectedReadbackFor2DCanvas(
  context,
  width,
  height)
  {
    // Always read back the raw data from canvas
    return context.getImageData(0, 0, width, height).data;
  }

  getExpectedReadbackForWebGLCanvas(
  gl,
  width,
  height)
  {
    const bytesPerPixel = 4;

    const sourcePixels = new Uint8ClampedArray(width * height * bytesPerPixel);
    gl.readPixels(0, 0, width, height, gl.RGBA, gl.UNSIGNED_BYTE, sourcePixels);

    return this.doFlipY(sourcePixels, width, height, bytesPerPixel);
  }

  getExpectedReadbackForWebGPUCanvas(
  width,
  height,
  alphaMode)
  {
    const bytesPerPixel = 4;

    const rgbaPixels = this.getDataToInitSourceWebGPUCanvas(width, height, alphaMode);

    // The source canvas has bgra8unorm back resource. We
    // swizzle the channels to align with 2d/webgl canvas and
    // clear alpha to 255 (1.0) when context alphaMode
    // is set to opaque (follow webgpu spec).
    for (let i = 0; i < height; ++i) {
      for (let j = 0; j < width; ++j) {
        const pixelPos = i * width + j;
        const r = rgbaPixels[pixelPos * bytesPerPixel + 2];
        if (alphaMode === 'opaque') {
          rgbaPixels[pixelPos * bytesPerPixel + 3] = 255;
        }

        rgbaPixels[pixelPos * bytesPerPixel + 2] = rgbaPixels[pixelPos * bytesPerPixel];
        rgbaPixels[pixelPos * bytesPerPixel] = r;
      }
    }

    return rgbaPixels;
  }

  doCopyContentsTest(
  source,
  expectedSourceImage,
  p)







  {
    const dst = this.createTextureTracked({
      size: {
        width: p.width,
        height: p.height,
        depthOrArrayLayers: 1
      },
      format: p.dstColorFormat,
      usage:
      GPUTextureUsage.COPY_DST | GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT
    });

    // Construct expected value for different dst color format
    const info = kTextureFormatInfo[p.dstColorFormat];
    const expFormat = info.baseFormat ?? p.dstColorFormat;

    // For 2d canvas, get expected pixels with getImageData(), which returns unpremultiplied
    // values.
    const expectedDestinationImage = this.getExpectedDstPixelsFromSrcPixels({
      srcPixels: expectedSourceImage,
      srcOrigin: [0, 0],
      srcSize: [p.width, p.height],
      dstOrigin: [0, 0],
      dstSize: [p.width, p.height],
      subRectSize: [p.width, p.height],
      format: expFormat,
      flipSrcBeforeCopy: false,
      srcDoFlipYDuringCopy: p.srcDoFlipYDuringCopy,
      conversion: {
        srcPremultiplied: p.srcPremultiplied,
        dstPremultiplied: p.dstPremultiplied
      }
    });

    this.doTestAndCheckResult(
      { source, origin: { x: 0, y: 0 }, flipY: p.srcDoFlipYDuringCopy },
      {
        texture: dst,
        origin: { x: 0, y: 0 },
        colorSpace: 'srgb',
        premultipliedAlpha: p.dstPremultiplied
      },
      expectedDestinationImage,
      { width: p.width, height: p.height, depthOrArrayLayers: 1 },
      // 1.0 and 0.6 are representable precisely by all formats except rgb10a2unorm, but
      // allow diffs of 1ULP since that's the generally-appropriate threshold.
      { maxDiffULPsForNormFormat: 1, maxDiffULPsForFloatFormat: 1 }
    );
  }
}

export const g = makeTestGroup(F);

g.test('copy_contents_from_2d_context_canvas').
desc(
  `
  Test HTMLCanvasElement and OffscreenCanvas with 2d context
  can be copied to WebGPU texture correctly.

  It creates HTMLCanvasElement/OffscreenCanvas with '2d'.
  Use fillRect(2d context) to render red rect for top-left,
  green rect for top-right, blue rect for bottom-left and white for bottom-right.

  Then call copyExternalImageToTexture() to do a full copy to the 0 mipLevel
  of dst texture, and read the contents out to compare with the canvas contents.

  Provide premultiplied input if 'premultipliedAlpha' in 'GPUImageCopyTextureTagged'
  is set to 'true' and unpremultiplied input if it is set to 'false'.

  If 'flipY' in 'GPUImageCopyExternalImage' is set to 'true', copy will ensure the result
  is flipped.

  The tests covers:
  - Valid canvas type
  - Valid 2d context type
  - Valid dstColorFormat of copyExternalImageToTexture()
  - Valid dest alphaMode
  - Valid 'flipY' config in 'GPUImageCopyExternalImage' (named 'srcDoFlipYDuringCopy' in cases)
  - TODO(#913): color space tests need to be added

  And the expected results are all passed.
  `
).
params((u) =>
u.
combine('canvasType', kAllCanvasTypes).
combine('dstColorFormat', kValidTextureFormatsForCopyE2T).
combine('dstAlphaMode', kCanvasAlphaModes).
combine('srcDoFlipYDuringCopy', [true, false]).
beginSubcases().
combine('width', [1, 2, 4, 15]).
combine('height', [1, 2, 4, 15])
).
beforeAllSubcases((t) => {
  t.skipIfTextureFormatNotSupported(t.params.dstColorFormat);
}).
fn((t) => {
  const { width, height, canvasType, dstAlphaMode } = t.params;

  const { canvas, expectedSourceData } = t.init2DCanvasContent({
    canvasType,
    width,
    height
  });

  t.doCopyContentsTest(canvas, expectedSourceData, {
    srcPremultiplied: false,
    dstPremultiplied: dstAlphaMode === 'premultiplied',
    ...t.params
  });
});

g.test('copy_contents_from_gl_context_canvas').
desc(
  `
  Test HTMLCanvasElement and OffscreenCanvas with webgl/webgl2 context
  can be copied to WebGPU texture correctly.

  It creates HTMLCanvasElement/OffscreenCanvas with webgl'/'webgl2'.
  Use scissor + clear to render red rect for top-left, green rect
  for top-right, blue rect for bottom-left and white for bottom-right.
  And do premultiply alpha in advance if the webgl/webgl2 context is created
  with premultipliedAlpha : true.

  Then call copyExternalImageToTexture() to do a full copy to the 0 mipLevel
  of dst texture, and read the contents out to compare with the canvas contents.

  Provide premultiplied input if 'premultipliedAlpha' in 'GPUImageCopyTextureTagged'
  is set to 'true' and unpremultiplied input if it is set to 'false'.

  If 'flipY' in 'GPUImageCopyExternalImage' is set to 'true', copy will ensure the result
  is flipped.

  The tests covers:
  - Valid canvas type
  - Valid webgl/webgl2 context type
  - Valid dstColorFormat of copyExternalImageToTexture()
  - Valid source image alphaMode
  - Valid dest alphaMode
  - Valid 'flipY' config in 'GPUImageCopyExternalImage'(named 'srcDoFlipYDuringCopy' in cases)
  - TODO: color space tests need to be added

  And the expected results are all passed.
  `
).
params((u) =>
u.
combine('canvasType', kAllCanvasTypes).
combine('contextName', ['webgl', 'webgl2']).
combine('dstColorFormat', kValidTextureFormatsForCopyE2T).
combine('srcPremultiplied', [true, false]).
combine('dstAlphaMode', kCanvasAlphaModes).
combine('srcDoFlipYDuringCopy', [true, false]).
beginSubcases().
combine('width', [1, 2, 4, 15]).
combine('height', [1, 2, 4, 15])
).
beforeAllSubcases((t) => {
  t.skipIfTextureFormatNotSupported(t.params.dstColorFormat);
}).
fn((t) => {
  const { width, height, canvasType, contextName, srcPremultiplied, dstAlphaMode } = t.params;

  const { canvas, expectedSourceData } = t.initGLCanvasContent({
    canvasType,
    contextName,
    width,
    height,
    premultiplied: srcPremultiplied
  });

  t.doCopyContentsTest(canvas, expectedSourceData, {
    dstPremultiplied: dstAlphaMode === 'premultiplied',
    ...t.params
  });
});

g.test('copy_contents_from_gpu_context_canvas').
desc(
  `
  Test HTMLCanvasElement and OffscreenCanvas with webgpu context
  can be copied to WebGPU texture correctly.

  It creates HTMLCanvasElement/OffscreenCanvas with 'webgpu'.
  Use writeTexture to copy pixels to back buffer. The results are:
  red rect for top-left, green rect for top-right, blue rect for bottom-left
  and white for bottom-right.

  TODO: Actually test alphaMode = opaque.
  And do premultiply alpha in advance if the webgpu context is created
  with alphaMode="premultiplied".

  Then call copyExternalImageToTexture() to do a full copy to the 0 mipLevel
  of dst texture, and read the contents out to compare with the canvas contents.

  Provide premultiplied input if 'premultipliedAlpha' in 'GPUImageCopyTextureTagged'
  is set to 'true' and unpremultiplied input if it is set to 'false'.

  If 'flipY' in 'GPUImageCopyExternalImage' is set to 'true', copy will ensure the result
  is flipped.

  The tests covers:
  - Valid canvas type
  - Source WebGPU Canvas lives in the same GPUDevice or different GPUDevice as test
  - Valid dstColorFormat of copyExternalImageToTexture()
  - TODO: test more source image alphaMode
  - Valid dest alphaMode
  - Valid 'flipY' config in 'GPUImageCopyExternalImage'(named 'srcDoFlipYDuringCopy' in cases)
  - TODO: color space tests need to be added

  And the expected results are all passed.
  `
).
params((u) =>
u.
combine('canvasType', kAllCanvasTypes).
combine('srcAndDstInSameGPUDevice', [true, false]).
combine('dstColorFormat', kValidTextureFormatsForCopyE2T)
// .combine('srcAlphaMode', kCanvasAlphaModes)
.combine('srcAlphaMode', ['premultiplied']).
combine('dstAlphaMode', kCanvasAlphaModes).
combine('srcDoFlipYDuringCopy', [true, false]).
beginSubcases().
combine('width', [1, 2, 4, 15]).
combine('height', [1, 2, 4, 15])
).
beforeAllSubcases((t) => {
  t.skipIfTextureFormatNotSupported(t.params.dstColorFormat);
  t.selectMismatchedDeviceOrSkipTestCase(undefined);
}).
fn((t) => {
  const { width, height, canvasType, srcAndDstInSameGPUDevice, srcAlphaMode, dstAlphaMode } =
  t.params;

  const device = srcAndDstInSameGPUDevice ? t.device : t.mismatchedDevice;
  const { canvas: source, expectedSourceData } = t.initSourceWebGPUCanvas({
    device,
    canvasType,
    width,
    height,
    alphaMode: srcAlphaMode
  });

  t.doCopyContentsTest(source, expectedSourceData, {
    srcPremultiplied: srcAlphaMode === 'premultiplied',
    dstPremultiplied: dstAlphaMode === 'premultiplied',
    ...t.params
  });
});

g.test('copy_contents_from_bitmaprenderer_context_canvas').
desc(
  `
  Test HTMLCanvasElement and OffscreenCanvas with ImageBitmapRenderingContext
  can be copied to WebGPU texture correctly.

  It creates HTMLCanvasElement/OffscreenCanvas with 'bitmaprenderer'.
  First, use fillRect(2d context) to render red rect for top-left,
  green rect for top-right, blue rect for bottom-left and white for bottom-right on a
  2d context canvas and create imageBitmap with that canvas. Use transferFromImageBitmap()
  to render the imageBitmap to source canvas.

  Then call copyExternalImageToTexture() to do a full copy to the 0 mipLevel
  of dst texture, and read the contents out to compare with the canvas contents.

  Provide premultiplied input if 'premultipliedAlpha' in 'GPUImageCopyTextureTagged'
  is set to 'true' and unpremultiplied input if it is set to 'false'.

  If 'flipY' in 'GPUImageCopyExternalImage' is set to 'true', copy will ensure the result
  is flipped.

  The tests covers:
  - Valid canvas type
  - Valid ImageBitmapRendering context type
  - Valid dstColorFormat of copyExternalImageToTexture()
  - Valid dest alphaMode
  - Valid 'flipY' config in 'GPUImageCopyExternalImage' (named 'srcDoFlipYDuringCopy' in cases)
  - TODO(#913): color space tests need to be added

  And the expected results are all passed.
  `
).
params((u) =>
u.
combine('canvasType', kAllCanvasTypes).
combine('dstColorFormat', kValidTextureFormatsForCopyE2T).
combine('dstAlphaMode', kCanvasAlphaModes).
combine('srcDoFlipYDuringCopy', [true, false]).
beginSubcases().
combine('width', [1, 2, 4, 15]).
combine('height', [1, 2, 4, 15])
).
beforeAllSubcases((t) => {
  t.skipIfTextureFormatNotSupported(t.params.dstColorFormat);
}).
fn(async (t) => {
  const { width, height, canvasType, dstAlphaMode } = t.params;

  const canvas = createCanvas(t, canvasType, width, height);

  const imageBitmapRenderingContext = canvas.getContext('bitmaprenderer');

  if (!(imageBitmapRenderingContext instanceof ImageBitmapRenderingContext)) {
    skipTestCase(canvasType + ' canvas imageBitmap rendering context not available');
  }

  const { canvas: sourceContentCanvas, expectedSourceData } = t.init2DCanvasContent({
    canvasType,
    width,
    height
  });

  const imageBitmap = await createImageBitmap(sourceContentCanvas, { premultiplyAlpha: 'none' });
  imageBitmapRenderingContext.transferFromImageBitmap(imageBitmap);

  t.doCopyContentsTest(canvas, expectedSourceData, {
    srcPremultiplied: false,
    dstPremultiplied: dstAlphaMode === 'premultiplied',
    ...t.params
  });
});

g.test('color_space_conversion').
desc(
  `
    Test HTMLCanvasElement with 2d context can created with 'colorSpace' attribute.
    Using CopyExternalImageToTexture to copy from such type of canvas needs
    to do color space converting correctly.

    It creates HTMLCanvasElement/OffscreenCanvas with '2d' and 'colorSpace' attributes.
    Use fillRect(2d context) to render red rect for top-left,
    green rect for top-right, blue rect for bottom-left and white for bottom-right.

    Then call copyExternalImageToTexture() to do a full copy to the 0 mipLevel
    of dst texture, and read the contents out to compare with the canvas contents.

    Provide premultiplied input if 'premultipliedAlpha' in 'GPUImageCopyTextureTagged'
    is set to 'true' and unpremultiplied input if it is set to 'false'.

    If 'flipY' in 'GPUImageCopyExternalImage' is set to 'true', copy will ensure the result
    is flipped.

    If color space from source input and user defined dstTexture color space are different, the
    result must convert the content to user defined color space

    The tests covers:
    - Valid dstColorFormat of copyExternalImageToTexture()
    - Valid dest alphaMode
    - Valid 'flipY' config in 'GPUImageCopyExternalImage' (named 'srcDoFlipYDuringCopy' in cases)
    - Valid 'colorSpace' config in 'dstColorSpace'

    And the expected results are all passed.

    TODO: Enhance test data with colors that aren't always opaque and fully saturated.
    TODO: Consider refactoring src data setup with TexelView.writeTextureData.
  `
).
params((u) =>
u.
combine('srcColorSpace', ['srgb', 'display-p3']).
combine('dstColorSpace', ['srgb', 'display-p3']).
combine('dstColorFormat', kValidTextureFormatsForCopyE2T).
combine('dstPremultiplied', [true, false]).
combine('srcDoFlipYDuringCopy', [true, false]).
beginSubcases().
combine('width', [1, 2, 4, 15, 255, 256]).
combine('height', [1, 2, 4, 15, 255, 256])
).
beforeAllSubcases((t) => {
  t.skipIfTextureFormatNotSupported(t.params.dstColorFormat);
}).
fn((t) => {
  const {
    width,
    height,
    srcColorSpace,
    dstColorSpace,
    dstColorFormat,
    dstPremultiplied,
    srcDoFlipYDuringCopy
  } = t.params;
  const { canvas, expectedSourceData } = t.init2DCanvasContentWithColorSpace({
    width,
    height,
    colorSpace: srcColorSpace
  });

  const dst = t.createTextureTracked({
    size: { width, height },
    format: dstColorFormat,
    usage:
    GPUTextureUsage.COPY_DST | GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT
  });

  const expectedDestinationImage = t.getExpectedDstPixelsFromSrcPixels({
    srcPixels: expectedSourceData,
    srcOrigin: [0, 0],
    srcSize: [width, height],
    dstOrigin: [0, 0],
    dstSize: [width, height],
    subRectSize: [width, height],
    // copyExternalImageToTexture does not perform gamma-encoding into `-srgb` formats.
    format: kTextureFormatInfo[dstColorFormat].baseFormat ?? dstColorFormat,
    flipSrcBeforeCopy: false,
    srcDoFlipYDuringCopy,
    conversion: {
      srcPremultiplied: false,
      dstPremultiplied,
      srcColorSpace,
      dstColorSpace
    }
  });

  const texelCompareOptions = {
    maxFractionalDiff: 0,
    maxDiffULPsForNormFormat: 1
  };
  if (srcColorSpace !== dstColorSpace) {
    // Color space conversion seems prone to errors up to about 0.0003 on f32, 0.0007 on f16.
    texelCompareOptions.maxFractionalDiff = 0.001;
  } else {
    texelCompareOptions.maxDiffULPsForFloatFormat = 1;
  }

  t.doTestAndCheckResult(
    { source: canvas, origin: { x: 0, y: 0 }, flipY: srcDoFlipYDuringCopy },
    {
      texture: dst,
      origin: { x: 0, y: 0 },
      colorSpace: dstColorSpace,
      premultipliedAlpha: dstPremultiplied
    },
    expectedDestinationImage,
    { width, height, depthOrArrayLayers: 1 },
    texelCompareOptions
  );
});