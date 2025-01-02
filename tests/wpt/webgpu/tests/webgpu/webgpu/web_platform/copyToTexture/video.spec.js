/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
copyToTexture with HTMLVideoElement and VideoFrame.

- videos with various encodings/formats (webm vp8, webm vp9, ogg theora, mp4), video color spaces
  (bt.601, bt.709, bt.2020) and dst color spaces(display-p3, srgb).

  TODO: Test video in BT.2020 color space
`;import { makeTestGroup } from '../../../common/framework/test_group.js';
import { GPUTest, TextureTestMixin } from '../../gpu_test.js';
import {
  startPlayingAndWaitForVideo,
  getVideoElement,
  getVideoFrameFromVideoElement,
  convertToUnorm8,
  kPredefinedColorSpace,
  kVideoNames,
  kVideoInfo,
  kVideoExpectedColors } from
'../../web_platform/util.js';

const kFormat = 'rgba8unorm';

export const g = makeTestGroup(TextureTestMixin(GPUTest));

g.test('copy_from_video').
desc(
  `
Test HTMLVideoElement and VideoFrame can be copied to WebGPU texture correctly.

It creates HTMLVideoElement with videos under Resource folder.

  Then call copyExternalImageToTexture() to do a full copy to the 0 mipLevel
  of dst texture, and read the contents out to compare with the ImageBitmap contents.

  If 'flipY' in 'GPUImageCopyExternalImage' is set to 'true', copy will ensure the result
  is flipped.

  The tests covers:
  - Video comes from different color spaces.
  - Valid 'flipY' config in 'GPUImageCopyExternalImage' (named 'srcDoFlipYDuringCopy' in cases)
  - TODO: partial copy tests should be added
  - TODO: all valid dstColorFormat tests should be added.
`
).
params((u) =>
u //
.combine('videoName', kVideoNames).
combine('sourceType', ['VideoElement', 'VideoFrame']).
combine('srcDoFlipYDuringCopy', [true, false]).
combine('dstColorSpace', kPredefinedColorSpace)
).
fn(async (t) => {
  const { videoName, sourceType, srcDoFlipYDuringCopy, dstColorSpace } = t.params;

  if (sourceType === 'VideoFrame' && typeof VideoFrame === 'undefined') {
    t.skip('WebCodec is not supported');
  }

  const videoElement = getVideoElement(t, videoName);

  await startPlayingAndWaitForVideo(videoElement, async () => {
    let source, width, height;
    if (sourceType === 'VideoFrame') {
      source = await getVideoFrameFromVideoElement(t, videoElement);
      width = source.displayWidth;
      height = source.displayHeight;
    } else {
      source = videoElement;
      width = source.videoWidth;
      height = source.videoHeight;
    }

    const dstTexture = t.createTextureTracked({
      format: kFormat,
      size: { width, height, depthOrArrayLayers: 1 },
      usage:
      GPUTextureUsage.COPY_SRC | GPUTextureUsage.COPY_DST | GPUTextureUsage.RENDER_ATTACHMENT
    });

    t.queue.copyExternalImageToTexture(
      {
        source,
        origin: { x: 0, y: 0 },
        flipY: srcDoFlipYDuringCopy
      },
      {
        texture: dstTexture,
        origin: { x: 0, y: 0 },
        colorSpace: dstColorSpace,
        premultipliedAlpha: true
      },
      { width, height, depthOrArrayLayers: 1 }
    );

    const srcColorSpace = kVideoInfo[videoName].colorSpace;
    const presentColors = kVideoExpectedColors[srcColorSpace][dstColorSpace];

    // visible rect is whole frame, no clipping.
    const expect = kVideoInfo[videoName].display;

    if (srcDoFlipYDuringCopy) {
      t.expectSinglePixelComparisonsAreOkInTexture({ texture: dstTexture }, [
      // Flipped top-left.
      {
        coord: { x: width * 0.25, y: height * 0.25 },
        exp: convertToUnorm8(presentColors[expect.bottomLeftColor])
      },
      // Flipped top-right.
      {
        coord: { x: width * 0.75, y: height * 0.25 },
        exp: convertToUnorm8(presentColors[expect.bottomRightColor])
      },
      // Flipped bottom-left.
      {
        coord: { x: width * 0.25, y: height * 0.75 },
        exp: convertToUnorm8(presentColors[expect.topLeftColor])
      },
      // Flipped bottom-right.
      {
        coord: { x: width * 0.75, y: height * 0.75 },
        exp: convertToUnorm8(presentColors[expect.topRightColor])
      }]
      );
    } else {
      t.expectSinglePixelComparisonsAreOkInTexture({ texture: dstTexture }, [
      // Top-left.
      {
        coord: { x: width * 0.25, y: height * 0.25 },
        exp: convertToUnorm8(presentColors[expect.topLeftColor])
      },
      // Top-right.
      {
        coord: { x: width * 0.75, y: height * 0.25 },
        exp: convertToUnorm8(presentColors[expect.topRightColor])
      },
      // Bottom-left.
      {
        coord: { x: width * 0.25, y: height * 0.75 },
        exp: convertToUnorm8(presentColors[expect.bottomLeftColor])
      },
      // Bottom-right.
      {
        coord: { x: width * 0.75, y: height * 0.75 },
        exp: convertToUnorm8(presentColors[expect.bottomRightColor])
      }]
      );
    }

    if (source instanceof VideoFrame) {
      source.close();
    }
  });
});