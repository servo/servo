/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests for external textures from HTMLVideoElement (and other video-type sources?).

- videos with various encodings/formats (webm vp8, webm vp9, ogg theora, mp4), video color spaces
  (bt.601, bt.709, bt.2020) and dst color spaces(display-p3, srgb)

TODO: consider whether external_texture and copyToTexture video tests should be in the same file
TODO(#3193): Test video in BT.2020 color space
`;import { makeTestGroup } from '../../../common/framework/test_group.js';
import { GPUTest, TextureTestMixin } from '../../gpu_test.js';
import { createCanvas } from '../../util/create_elements.js';
import {
  startPlayingAndWaitForVideo,
  getVideoFrameFromVideoElement,
  getVideoElement,
  convertToUnorm8,
  kPredefinedColorSpace,
  kVideoNames,
  kVideoInfo,
  kVideoExpectedColors } from
'../../web_platform/util.js';

const kHeight = 16;
const kWidth = 16;
const kFormat = 'rgba8unorm';

export const g = makeTestGroup(TextureTestMixin(GPUTest));

function createExternalTextureSamplingTestPipeline(
t,
colorAttachmentFormat = kFormat)
{
  const pipeline = t.device.createRenderPipeline({
    layout: 'auto',
    vertex: {
      module: t.device.createShaderModule({
        code: `
        @vertex fn main(@builtin(vertex_index) VertexIndex : u32) -> @builtin(position) vec4<f32> {
            var pos = array<vec4<f32>, 6>(
              vec4<f32>( 1.0,  1.0, 0.0, 1.0),
              vec4<f32>( 1.0, -1.0, 0.0, 1.0),
              vec4<f32>(-1.0, -1.0, 0.0, 1.0),
              vec4<f32>( 1.0,  1.0, 0.0, 1.0),
              vec4<f32>(-1.0, -1.0, 0.0, 1.0),
              vec4<f32>(-1.0,  1.0, 0.0, 1.0)
            );
            return pos[VertexIndex];
        }
        `
      }),
      entryPoint: 'main'
    },
    fragment: {
      module: t.device.createShaderModule({
        code: `
        @group(0) @binding(0) var s : sampler;
        @group(0) @binding(1) var t : texture_external;

        @fragment fn main(@builtin(position) FragCoord : vec4<f32>)
                                 -> @location(0) vec4<f32> {
            return textureSampleBaseClampToEdge(t, s, FragCoord.xy / vec2<f32>(16.0, 16.0));
        }
        `
      }),
      entryPoint: 'main',
      targets: [
      {
        format: colorAttachmentFormat
      }]

    },
    primitive: { topology: 'triangle-list' }
  });

  return pipeline;
}

function createExternalTextureSamplingTestBindGroup(
t,
checkNonStandardIsZeroCopy,
source,
pipeline,
dstColorSpace)
{
  const linearSampler = t.device.createSampler();

  const externalTexture = t.device.importExternalTexture({
    source,
    colorSpace: dstColorSpace
  });

  if (checkNonStandardIsZeroCopy) {
    expectZeroCopyNonStandard(t, externalTexture);
  }
  const bindGroup = t.device.createBindGroup({
    layout: pipeline.getBindGroupLayout(0),
    entries: [
    {
      binding: 0,
      resource: linearSampler
    },
    {
      binding: 1,
      resource: externalTexture
    }]

  });

  return bindGroup;
}

/**
 * Expect the non-standard `externalTexture.isZeroCopy` is true.
 */
function expectZeroCopyNonStandard(t, externalTexture) {

  t.expect(externalTexture.isZeroCopy, '0-copy import failed.');
}

/**
 * `externalTexture.isZeroCopy` is a non-standard Chrome API for testing only.
 * It is exposed by enabling chrome://flags/#enable-webgpu-developer-features
 *
 * If the API is available, this function adds a parameter `checkNonStandardIsZeroCopy`.
 * Cases with that parameter set to `true` will fail if `externalTexture.isZeroCopy` is not true.
 */
function checkNonStandardIsZeroCopyIfAvailable() {
  if (
  typeof GPUExternalTexture !== 'undefined' &&

  GPUExternalTexture.prototype.hasOwnProperty('isZeroCopy'))
  {
    return [{}, { checkNonStandardIsZeroCopy: true }];
  } else {
    return [{}];
  }
}

g.test('importExternalTexture,sample').
desc(
  `
Tests that we can import an HTMLVideoElement/VideoFrame into a GPUExternalTexture, sample from it
for several combinations of video format, video color spaces and dst color spaces.
`
).
params((u) =>
u //
.combineWithParams(checkNonStandardIsZeroCopyIfAvailable()).
combine('videoName', kVideoNames).
combine('sourceType', ['VideoElement', 'VideoFrame']).
combine('dstColorSpace', kPredefinedColorSpace)
).
fn(async (t) => {
  const { videoName, sourceType, dstColorSpace } = t.params;

  if (sourceType === 'VideoFrame' && typeof VideoFrame === 'undefined') {
    t.skip('WebCodec is not supported');
  }

  const videoElement = getVideoElement(t, videoName);

  await startPlayingAndWaitForVideo(videoElement, async () => {
    const source =
    sourceType === 'VideoFrame' ?
    await getVideoFrameFromVideoElement(t, videoElement) :
    videoElement;

    const colorAttachment = t.createTextureTracked({
      format: kFormat,
      size: { width: kWidth, height: kHeight, depthOrArrayLayers: 1 },
      usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT
    });

    const pipeline = createExternalTextureSamplingTestPipeline(t);
    const bindGroup = createExternalTextureSamplingTestBindGroup(
      t,
      t.params.checkNonStandardIsZeroCopy,
      source,
      pipeline,
      dstColorSpace
    );

    const commandEncoder = t.device.createCommandEncoder();
    const passEncoder = commandEncoder.beginRenderPass({
      colorAttachments: [
      {
        view: colorAttachment.createView(),
        clearValue: { r: 0.0, g: 0.0, b: 0.0, a: 1.0 },
        loadOp: 'clear',
        storeOp: 'store'
      }]

    });
    passEncoder.setPipeline(pipeline);
    passEncoder.setBindGroup(0, bindGroup);
    passEncoder.draw(6);
    passEncoder.end();
    t.device.queue.submit([commandEncoder.finish()]);

    const srcColorSpace = kVideoInfo[videoName].colorSpace;
    const presentColors = kVideoExpectedColors[srcColorSpace][dstColorSpace];

    // visible rect is whole frame, no clipping.
    const expect = kVideoInfo[videoName].display;

    // For validation, we sample a few pixels away from the edges to avoid compression
    // artifacts.
    t.expectSinglePixelComparisonsAreOkInTexture({ texture: colorAttachment }, [
    // Top-left.
    {
      coord: { x: kWidth * 0.25, y: kHeight * 0.25 },
      exp: convertToUnorm8(presentColors[expect.topLeftColor])
    },
    // Top-right.
    {
      coord: { x: kWidth * 0.75, y: kHeight * 0.25 },
      exp: convertToUnorm8(presentColors[expect.topRightColor])
    },
    // Bottom-left.
    {
      coord: { x: kWidth * 0.25, y: kHeight * 0.75 },
      exp: convertToUnorm8(presentColors[expect.bottomLeftColor])
    },
    // Bottom-right.
    {
      coord: { x: kWidth * 0.75, y: kHeight * 0.75 },
      exp: convertToUnorm8(presentColors[expect.bottomRightColor])
    }]
    );
  });
});

g.test('importExternalTexture,sample_non_YUV_video_frame').
desc(
  `
Tests that we can import an VideoFrame with non-YUV pixel format into a GPUExternalTexture and sample it.
`
).
params((u) =>
u //
.combine('videoFrameFormat', ['RGBA', 'RGBX', 'BGRA', 'BGRX'])
).
fn((t) => {
  const { videoFrameFormat } = t.params;

  if (typeof VideoFrame === 'undefined') {
    t.skip('WebCodec is not supported');
  }

  const canvas = createCanvas(t, 'onscreen', kWidth, kHeight);

  const canvasContext = canvas.getContext('2d');

  if (canvasContext === null) {
    t.skip(' onscreen canvas 2d context not available');
  }

  const ctx = canvasContext;

  const rectWidth = Math.floor(kWidth / 2);
  const rectHeight = Math.floor(kHeight / 2);

  // Red
  ctx.fillStyle = `rgba(255, 0, 0, 1.0)`;
  ctx.fillRect(0, 0, rectWidth, rectHeight);
  // Lime
  ctx.fillStyle = `rgba(0, 255, 0, 1.0)`;
  ctx.fillRect(rectWidth, 0, kWidth - rectWidth, rectHeight);
  // Blue
  ctx.fillStyle = `rgba(0, 0, 255, 1.0)`;
  ctx.fillRect(0, rectHeight, rectWidth, kHeight - rectHeight);
  // Fuchsia
  ctx.fillStyle = `rgba(255, 0, 255, 1.0)`;
  ctx.fillRect(rectWidth, rectHeight, kWidth - rectWidth, kHeight - rectHeight);

  const imageData = ctx.getImageData(0, 0, kWidth, kHeight);

  // Create video frame with default color space 'srgb'
  const frameInit = {
    format: videoFrameFormat,
    codedWidth: kWidth,
    codedHeight: kHeight,
    timestamp: 0
  };

  const frame = new VideoFrame(imageData.data.buffer, frameInit);
  let textureFormat = 'rgba8unorm';

  if (videoFrameFormat === 'BGRA' || videoFrameFormat === 'BGRX') {
    textureFormat = 'bgra8unorm';
  }

  const colorAttachment = t.createTextureTracked({
    format: textureFormat,
    size: { width: kWidth, height: kHeight, depthOrArrayLayers: 1 },
    usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT
  });

  const pipeline = createExternalTextureSamplingTestPipeline(t, textureFormat);
  const bindGroup = createExternalTextureSamplingTestBindGroup(
    t,
    undefined /* checkNonStandardIsZeroCopy */,
    frame,
    pipeline,
    'srgb'
  );

  const commandEncoder = t.device.createCommandEncoder();
  const passEncoder = commandEncoder.beginRenderPass({
    colorAttachments: [
    {
      view: colorAttachment.createView(),
      clearValue: { r: 0.0, g: 0.0, b: 0.0, a: 1.0 },
      loadOp: 'clear',
      storeOp: 'store'
    }]

  });
  passEncoder.setPipeline(pipeline);
  passEncoder.setBindGroup(0, bindGroup);
  passEncoder.draw(6);
  passEncoder.end();
  t.device.queue.submit([commandEncoder.finish()]);

  const expected = {
    topLeft: new Uint8Array([255, 0, 0, 255]),
    topRight: new Uint8Array([0, 255, 0, 255]),
    bottomLeft: new Uint8Array([0, 0, 255, 255]),
    bottomRight: new Uint8Array([255, 0, 255, 255])
  };

  // For validation, we sample a few pixels away from the edges to avoid compression
  // artifacts.
  t.expectSinglePixelComparisonsAreOkInTexture({ texture: colorAttachment }, [
  // Top-left.
  {
    coord: { x: kWidth * 0.25, y: kHeight * 0.25 },
    exp: expected.topLeft
  },
  // Top-right.
  {
    coord: { x: kWidth * 0.75, y: kHeight * 0.25 },
    exp: expected.topRight
  },
  // Bottom-left.
  {
    coord: { x: kWidth * 0.25, y: kHeight * 0.75 },
    exp: expected.bottomLeft
  },
  // Bottom-right.
  {
    coord: { x: kWidth * 0.75, y: kHeight * 0.75 },
    exp: expected.bottomRight
  }]
  );
});

g.test('importExternalTexture,sampleWithVideoFrameWithVisibleRectParam').
desc(
  `
Tests that we can import VideoFrames and sample the correct sub-rectangle when visibleRect
parameters are present.
`
).
params((u) =>
u //
.combineWithParams(checkNonStandardIsZeroCopyIfAvailable()).
combine('videoName', kVideoNames).
combine('dstColorSpace', kPredefinedColorSpace)
).
fn(async (t) => {
  const { videoName, dstColorSpace } = t.params;

  const videoElement = getVideoElement(t, videoName);

  await startPlayingAndWaitForVideo(videoElement, async () => {
    const source = await getVideoFrameFromVideoElement(t, videoElement);

    // All tested videos are derived from an image showing yellow, red, blue or green in each
    // quadrant. In this test we crop the video to each quadrant and check that desired color
    // is sampled from each corner of the cropped image.
    // visible rect clip applies on raw decoded frame, which defines based on video frame coded size.
    const srcVideoHeight = source.codedHeight;
    const srcVideoWidth = source.codedWidth;

    const srcColorSpace = kVideoInfo[videoName].colorSpace;
    const presentColors = kVideoExpectedColors[srcColorSpace][dstColorSpace];

    // The test crops raw decoded videos first and then apply transform. Expectation should
    // use coded colors as reference.
    const expect = kVideoInfo[videoName].coded;

    const cropParams = [
    // Top left
    {
      subRect: { x: 0, y: 0, width: srcVideoWidth / 2, height: srcVideoHeight / 2 },
      color: convertToUnorm8(presentColors[expect.topLeftColor])
    },
    // Top right
    {
      subRect: {
        x: srcVideoWidth / 2,
        y: 0,
        width: srcVideoWidth / 2,
        height: srcVideoHeight / 2
      },
      color: convertToUnorm8(presentColors[expect.topRightColor])
    },
    // Bottom left
    {
      subRect: {
        x: 0,
        y: srcVideoHeight / 2,
        width: srcVideoWidth / 2,
        height: srcVideoHeight / 2
      },
      color: convertToUnorm8(presentColors[expect.bottomLeftColor])
    },
    // Bottom right
    {
      subRect: {
        x: srcVideoWidth / 2,
        y: srcVideoHeight / 2,
        width: srcVideoWidth / 2,
        height: srcVideoHeight / 2
      },
      color: convertToUnorm8(presentColors[expect.bottomRightColor])
    }];


    for (const cropParam of cropParams) {
      const subRect = new VideoFrame(source, { visibleRect: cropParam.subRect });

      const colorAttachment = t.createTextureTracked({
        format: kFormat,
        size: { width: kWidth, height: kHeight, depthOrArrayLayers: 1 },
        usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT
      });

      const pipeline = createExternalTextureSamplingTestPipeline(t);
      const bindGroup = createExternalTextureSamplingTestBindGroup(
        t,
        t.params.checkNonStandardIsZeroCopy,
        subRect,
        pipeline,
        dstColorSpace
      );

      const commandEncoder = t.device.createCommandEncoder();
      const passEncoder = commandEncoder.beginRenderPass({
        colorAttachments: [
        {
          view: colorAttachment.createView(),
          clearValue: { r: 0.0, g: 0.0, b: 0.0, a: 1.0 },
          loadOp: 'clear',
          storeOp: 'store'
        }]

      });
      passEncoder.setPipeline(pipeline);
      passEncoder.setBindGroup(0, bindGroup);
      passEncoder.draw(6);
      passEncoder.end();
      t.device.queue.submit([commandEncoder.finish()]);

      // For validation, we sample a few pixels away from the edges to avoid compression
      // artifacts.
      t.expectSinglePixelComparisonsAreOkInTexture({ texture: colorAttachment }, [
      { coord: { x: kWidth * 0.1, y: kHeight * 0.1 }, exp: cropParam.color },
      { coord: { x: kWidth * 0.9, y: kHeight * 0.1 }, exp: cropParam.color },
      { coord: { x: kWidth * 0.1, y: kHeight * 0.9 }, exp: cropParam.color },
      { coord: { x: kWidth * 0.9, y: kHeight * 0.9 }, exp: cropParam.color }]
      );

      subRect.close();
    }

    source.close();
  });
});
g.test('importExternalTexture,compute').
desc(
  `
Tests that we can import an HTMLVideoElement/VideoFrame into a GPUExternalTexture and use it in a
compute shader, for several combinations of video format, video color spaces and dst color spaces.
`
).
params((u) =>
u //
.combineWithParams(checkNonStandardIsZeroCopyIfAvailable()).
combine('videoName', kVideoNames).
combine('sourceType', ['VideoElement', 'VideoFrame']).
combine('dstColorSpace', kPredefinedColorSpace)
).
fn(async (t) => {
  const { videoName, sourceType, dstColorSpace } = t.params;

  if (sourceType === 'VideoFrame' && typeof VideoFrame === 'undefined') {
    t.skip('WebCodec is not supported');
  }

  const videoElement = getVideoElement(t, videoName);

  await startPlayingAndWaitForVideo(videoElement, async () => {
    const source =
    sourceType === 'VideoFrame' ?
    await getVideoFrameFromVideoElement(t, videoElement) :
    videoElement;
    const externalTexture = t.device.importExternalTexture({
      source,
      colorSpace: dstColorSpace
    });
    if (t.params.checkNonStandardIsZeroCopy) {
      expectZeroCopyNonStandard(t, externalTexture);
    }
    const outputTexture = t.createTextureTracked({
      format: 'rgba8unorm',
      size: [2, 2, 1],
      usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.STORAGE_BINDING
    });

    // Use display size of VideoFrame and video size of HTMLVideoElement as frame size. These sizes are presenting size which
    // apply transformation in video metadata if any.

    const pipeline = t.device.createComputePipeline({
      layout: 'auto',
      compute: {
        // Shader loads 4 pixels, and then store them in a storage texture.
        module: t.device.createShaderModule({
          code: `
              override frameWidth : i32 = 0;
              override frameHeight : i32 = 0;
              @group(0) @binding(0) var t : texture_external;
              @group(0) @binding(1) var outImage : texture_storage_2d<rgba8unorm, write>;

              @compute @workgroup_size(1) fn main() {
                let coordTopLeft = vec2<i32>(frameWidth / 4, frameHeight / 4);
                let coordTopRight = vec2<i32>(frameWidth / 4 * 3, frameHeight / 4);
                let coordBottomLeft = vec2<i32>(frameWidth / 4, frameHeight / 4 * 3);
                let coordBottomRight = vec2<i32>(frameWidth / 4 * 3, frameHeight / 4 * 3);
                var yellow : vec4<f32> = textureLoad(t, coordTopLeft);
                textureStore(outImage, vec2<i32>(0, 0), yellow);
                var red : vec4<f32> = textureLoad(t, coordTopRight);
                textureStore(outImage, vec2<i32>(0, 1), red);
                var blue : vec4<f32> = textureLoad(t, coordBottomLeft);
                textureStore(outImage, vec2<i32>(1, 0), blue);
                var green : vec4<f32> = textureLoad(t, coordBottomRight);
                textureStore(outImage, vec2<i32>(1, 1), green);
                return;
              }
            `
        }),
        entryPoint: 'main',

        // Use display size of VideoFrame and video size of HTMLVideoElement as frame size. These sizes are presenting size which
        // apply transformation in video metadata if any.
        constants: {
          frameWidth:
          sourceType === 'VideoFrame' ?
          source.displayWidth :
          source.videoWidth,
          frameHeight:
          sourceType === 'VideoFrame' ?
          source.displayHeight :
          source.videoHeight
        }
      }
    });

    const bg = t.device.createBindGroup({
      entries: [
      { binding: 0, resource: externalTexture },
      { binding: 1, resource: outputTexture.createView() }],

      layout: pipeline.getBindGroupLayout(0)
    });

    const encoder = t.device.createCommandEncoder();
    const pass = encoder.beginComputePass();
    pass.setPipeline(pipeline);
    pass.setBindGroup(0, bg);
    pass.dispatchWorkgroups(1);
    pass.end();
    t.device.queue.submit([encoder.finish()]);

    const srcColorSpace = kVideoInfo[videoName].colorSpace;
    const presentColors = kVideoExpectedColors[srcColorSpace][dstColorSpace];

    // visible rect is whole frame, no clipping.
    const expect = kVideoInfo[videoName].display;

    t.expectSinglePixelComparisonsAreOkInTexture({ texture: outputTexture }, [
    // Top-left.
    { coord: { x: 0, y: 0 }, exp: convertToUnorm8(presentColors[expect.topLeftColor]) },
    // Top-right.
    { coord: { x: 0, y: 1 }, exp: convertToUnorm8(presentColors[expect.topRightColor]) },
    // Bottom-left.
    { coord: { x: 1, y: 0 }, exp: convertToUnorm8(presentColors[expect.bottomLeftColor]) },
    // Bottom-right.
    { coord: { x: 1, y: 1 }, exp: convertToUnorm8(presentColors[expect.bottomRightColor]) }]
    );
  });
});