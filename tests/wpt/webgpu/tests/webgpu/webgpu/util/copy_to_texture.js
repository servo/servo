/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { assert, memcpy } from '../../common/util/util.js';import { GPUTest, TextureTestMixin } from '../gpu_test.js';
import { reifyExtent3D, reifyOrigin3D } from '../util/unions.js';

import { makeInPlaceColorConversion } from './color_space_conversion.js';
import { TexelView } from './texture/texel_view.js';


/**
 * Predefined copy sub rect meta infos.
 */
export const kCopySubrectInfo = [
{
  srcOrigin: { x: 2, y: 2 },
  dstOrigin: { x: 0, y: 0, z: 0 },
  srcSize: { width: 16, height: 16 },
  dstSize: { width: 4, height: 4 },
  copyExtent: { width: 4, height: 4, depthOrArrayLayers: 1 }
},
{
  srcOrigin: { x: 10, y: 2 },
  dstOrigin: { x: 0, y: 0, z: 0 },
  srcSize: { width: 16, height: 16 },
  dstSize: { width: 4, height: 4 },
  copyExtent: { width: 4, height: 4, depthOrArrayLayers: 1 }
},
{
  srcOrigin: { x: 2, y: 10 },
  dstOrigin: { x: 0, y: 0, z: 0 },
  srcSize: { width: 16, height: 16 },
  dstSize: { width: 4, height: 4 },
  copyExtent: { width: 4, height: 4, depthOrArrayLayers: 1 }
},
{
  srcOrigin: { x: 10, y: 10 },
  dstOrigin: { x: 0, y: 0, z: 0 },
  srcSize: { width: 16, height: 16 },
  dstSize: { width: 4, height: 4 },
  copyExtent: { width: 4, height: 4, depthOrArrayLayers: 1 }
},
{
  srcOrigin: { x: 2, y: 2 },
  dstOrigin: { x: 2, y: 2, z: 0 },
  srcSize: { width: 16, height: 16 },
  dstSize: { width: 16, height: 16 },
  copyExtent: { width: 4, height: 4, depthOrArrayLayers: 1 }
},
{
  srcOrigin: { x: 10, y: 2 },
  dstOrigin: { x: 2, y: 2, z: 0 },
  srcSize: { width: 16, height: 16 },
  dstSize: { width: 16, height: 16 },
  copyExtent: { width: 4, height: 4, depthOrArrayLayers: 1 }
}];


export class CopyToTextureUtils extends TextureTestMixin(GPUTest) {
  doFlipY(
  sourcePixels,
  width,
  height,
  bytesPerPixel)
  {
    const dstPixels = new Uint8ClampedArray(width * height * bytesPerPixel);
    for (let i = 0; i < height; ++i) {
      for (let j = 0; j < width; ++j) {
        const srcPixelPos = i * width + j;
        // WebGL readPixel returns pixels from bottom-left origin. Using CopyExternalImageToTexture
        // to copy from WebGL Canvas keeps top-left origin. So the expectation from webgl.readPixel should
        // be flipped.
        const dstPixelPos = (height - i - 1) * width + j;

        memcpy(
          { src: sourcePixels, start: srcPixelPos * bytesPerPixel, length: bytesPerPixel },
          { dst: dstPixels, start: dstPixelPos * bytesPerPixel }
        );
      }
    }

    return dstPixels;
  }

  getExpectedDstPixelsFromSrcPixels({
    srcPixels,
    srcOrigin,
    srcSize,
    dstOrigin,
    dstSize,
    subRectSize,
    format,
    flipSrcBeforeCopy,
    srcDoFlipYDuringCopy,
    conversion
















  }) {
    const applyConversion = makeInPlaceColorConversion(conversion);

    const reifySrcOrigin = reifyOrigin3D(srcOrigin);
    const reifySrcSize = reifyExtent3D(srcSize);
    const reifyDstOrigin = reifyOrigin3D(dstOrigin);
    const reifyDstSize = reifyExtent3D(dstSize);
    const reifySubRectSize = reifyExtent3D(subRectSize);

    assert(
      reifyDstOrigin.x + reifySubRectSize.width <= reifyDstSize.width &&
      reifyDstOrigin.y + reifySubRectSize.height <= reifyDstSize.height,
      'subrect is out of bounds'
    );

    const divide = 255.0;
    return TexelView.fromTexelsAsColors(
      format,
      (coords) => {
        assert(
          coords.x >= reifyDstOrigin.x &&
          coords.y >= reifyDstOrigin.y &&
          coords.x < reifyDstOrigin.x + reifySubRectSize.width &&
          coords.y < reifyDstOrigin.y + reifySubRectSize.height &&
          coords.z === 0,
          'out of bounds'
        );
        // Map dst coords to get candidate src pixel position in y.
        let yInSubRect = coords.y - reifyDstOrigin.y;

        // If srcDoFlipYDuringCopy is true, a flipY op has been applied to src during copy.
        // WebGPU spec requires origin option relative to the top-left corner of the source image,
        // increasing downward consistently.
        // https://www.w3.org/TR/webgpu/#dom-gpuimagecopyexternalimage-flipy
        // Flip only happens in copy rect contents and src origin always top-left.
        // Get candidate src pixel position in y by mirroring in copy sub rect.
        if (srcDoFlipYDuringCopy) yInSubRect = reifySubRectSize.height - 1 - yInSubRect;

        let src_y = yInSubRect + reifySrcOrigin.y;

        // Test might generate flipped source based on srcPixels, e.g. Create ImageBitmap based on srcPixels but set orientation to 'flipY'
        // Get candidate src pixel position in y by mirroring in source.
        if (flipSrcBeforeCopy) src_y = reifySrcSize.height - src_y - 1;

        const pixelPos =
        src_y * reifySrcSize.width + (coords.x - reifyDstOrigin.x) + reifySrcOrigin.x;

        const rgba = {
          R: srcPixels[pixelPos * 4] / divide,
          G: srcPixels[pixelPos * 4 + 1] / divide,
          B: srcPixels[pixelPos * 4 + 2] / divide,
          A: srcPixels[pixelPos * 4 + 3] / divide
        };
        applyConversion(rgba);
        return rgba;
      },
      { clampToFormatRange: true }
    );
  }

  doTestAndCheckResult(
  imageCopyExternalImage,
  dstTextureCopyView,
  expTexelView,
  copySize,
  texelCompareOptions)
  {
    this.device.queue.copyExternalImageToTexture(
      imageCopyExternalImage,
      dstTextureCopyView,
      copySize
    );

    this.expectTexelViewComparisonIsOkInTexture(
      { texture: dstTextureCopyView.texture, origin: dstTextureCopyView.origin },
      expTexelView,
      copySize,
      texelCompareOptions
    );
    this.trackForCleanup(dstTextureCopyView.texture);
  }
}