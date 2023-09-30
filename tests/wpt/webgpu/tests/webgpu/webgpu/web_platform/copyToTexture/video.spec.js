/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
copyToTexture with HTMLVideoElement and VideoFrame.

- videos with various encodings/formats (webm vp8, webm vp9, ogg theora, mp4), color spaces
  (bt.601, bt.709, bt.2020)
`;
import { makeTestGroup } from '../../../common/framework/test_group.js';
import { GPUTest, TextureTestMixin } from '../../gpu_test.js';
import {
  startPlayingAndWaitForVideo,
  getVideoElement,
  getVideoFrameFromVideoElement,
  kVideoExpectations,
} from '../../web_platform/util.js';

const kFormat = 'rgba8unorm';

export const g = makeTestGroup(TextureTestMixin(GPUTest));

g.test('copy_from_video')
  .desc(
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
  - TODO: dst color space tests need to be added
`
  )
  .params(u =>
    u //
      .combineWithParams(kVideoExpectations)
      .combine('sourceType', ['VideoElement', 'VideoFrame'])
      .combine('srcDoFlipYDuringCopy', [true, false])
  )
  .fn(async t => {
    const { videoName, sourceType, srcDoFlipYDuringCopy } = t.params;

    if (sourceType === 'VideoFrame' && typeof VideoFrame === 'undefined') {
      t.skip('WebCodec is not supported');
    }

    const videoElement = getVideoElement(t, videoName);

    await startPlayingAndWaitForVideo(videoElement, async () => {
      let source, width, height;
      if (sourceType === 'VideoFrame') {
        source = await getVideoFrameFromVideoElement(t, videoElement);
        width = source.codedWidth;
        height = source.codedHeight;
      } else {
        source = videoElement;
        width = source.videoWidth;
        height = source.videoHeight;
      }

      const dstTexture = t.device.createTexture({
        format: kFormat,
        size: { width, height, depthOrArrayLayers: 1 },
        usage:
          GPUTextureUsage.COPY_SRC | GPUTextureUsage.COPY_DST | GPUTextureUsage.RENDER_ATTACHMENT,
      });

      t.queue.copyExternalImageToTexture(
        {
          source,
          origin: { x: 0, y: 0 },
          flipY: srcDoFlipYDuringCopy,
        },
        {
          texture: dstTexture,
          origin: { x: 0, y: 0 },
          colorSpace: 'srgb',
          premultipliedAlpha: true,
        },
        { width, height, depthOrArrayLayers: 1 }
      );

      if (srcDoFlipYDuringCopy) {
        t.expectSinglePixelComparisonsAreOkInTexture({ texture: dstTexture }, [
          // Top-left should be blue.
          { coord: { x: width * 0.25, y: height * 0.25 }, exp: t.params._blueExpectation },
          // Top-right should be green.
          { coord: { x: width * 0.75, y: height * 0.25 }, exp: t.params._greenExpectation },
          // Bottom-left should be yellow.
          { coord: { x: width * 0.25, y: height * 0.75 }, exp: t.params._yellowExpectation },
          // Bottom-right should be red.
          { coord: { x: width * 0.75, y: height * 0.75 }, exp: t.params._redExpectation },
        ]);
      } else {
        t.expectSinglePixelComparisonsAreOkInTexture({ texture: dstTexture }, [
          // Top-left should be yellow.
          { coord: { x: width * 0.25, y: height * 0.25 }, exp: t.params._yellowExpectation },
          // Top-right should be red.
          { coord: { x: width * 0.75, y: height * 0.25 }, exp: t.params._redExpectation },
          // Bottom-left should be blue.
          { coord: { x: width * 0.25, y: height * 0.75 }, exp: t.params._blueExpectation },
          // Bottom-right should be green.
          { coord: { x: width * 0.75, y: height * 0.75 }, exp: t.params._greenExpectation },
        ]);
      }

      if (source instanceof VideoFrame) {
        source.close();
      }
    });
  });
