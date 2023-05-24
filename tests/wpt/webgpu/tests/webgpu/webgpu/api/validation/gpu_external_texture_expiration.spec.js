/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
GPUExternalTexture expiration mechanism validation tests.
`;
import { makeTestGroup } from '../../../common/framework/test_group.js';
import { assert } from '../../../common/util/util.js';
import {
  getVideoElement,
  startPlayingAndWaitForVideo,
  getVideoFrameFromVideoElement,
  waitForNextFrame,
  waitForNextTask,
} from '../../web_platform/util.js';

import { ValidationTest } from './validation_test.js';

class GPUExternalTextureExpireTest extends ValidationTest {
  submitCommandBuffer(bindGroup, success) {
    const kHeight = 16;
    const kWidth = 16;
    const kFormat = 'rgba8unorm';

    const colorAttachment = this.device.createTexture({
      format: kFormat,
      size: { width: kWidth, height: kHeight, depthOrArrayLayers: 1 },
      usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT,
    });
    const passDescriptor = {
      colorAttachments: [
        {
          view: colorAttachment.createView(),
          clearValue: [0, 0, 0, 1],
          loadOp: 'clear',
          storeOp: 'store',
        },
      ],
    };

    const commandEncoder = this.device.createCommandEncoder();
    const passEncoder = commandEncoder.beginRenderPass(passDescriptor);
    passEncoder.setBindGroup(0, bindGroup);
    passEncoder.end();
    const commandBuffer = commandEncoder.finish();
    this.expectValidationError(() => this.device.queue.submit([commandBuffer]), !success);
  }

  getDefaultVideoElementAndCheck() {
    const videoElement = getVideoElement(this, 'four-colors-vp9-bt601.webm');

    if (!('requestVideoFrameCallback' in videoElement)) {
      this.skip('HTMLVideoElement.requestVideoFrameCallback is not supported');
    }

    return videoElement;
  }

  getDefaultBindGroupLayout() {
    return this.device.createBindGroupLayout({
      entries: [{ binding: 0, visibility: GPUShaderStage.FRAGMENT, externalTexture: {} }],
    });
  }
}

export const g = makeTestGroup(GPUExternalTextureExpireTest);

g.test('import_multiple_times_in_same_task_scope')
  .desc(
    `
    Tests that GPUExternalTexture is valid after been imported in the task.
    Tests that in the same task scope, import twice on the same video source should return
    the same GPUExternalTexture and bindGroup doesn't need to be updated.
    `
  )
  .params(u =>
    u //
      .combine('sourceType', ['VideoElement', 'VideoFrame'])
  )
  .fn(async t => {
    const sourceType = t.params.sourceType;
    const videoElement = t.getDefaultVideoElementAndCheck();

    let bindGroup;
    let externalTexture;
    await startPlayingAndWaitForVideo(videoElement, async () => {
      const source =
        sourceType === 'VideoFrame'
          ? await getVideoFrameFromVideoElement(t, videoElement)
          : videoElement;
      externalTexture = t.device.importExternalTexture({
        source: source,
      });

      bindGroup = t.device.createBindGroup({
        layout: t.getDefaultBindGroupLayout(),
        entries: [{ binding: 0, resource: externalTexture }],
      });

      t.submitCommandBuffer(bindGroup, true);

      // Import again in the same task scope should return same object.
      const shouldBeTheSameExternalTexture = t.device.importExternalTexture({
        source: source,
      });
      assert(externalTexture === shouldBeTheSameExternalTexture);

      t.submitCommandBuffer(bindGroup, true);
    });
  });

g.test('import_and_use_in_different_microtask')
  .desc(
    `
    Tests that in the same task scope, imported GPUExternalTexture is valid in
    different microtasks.
    `
  )
  .params(u =>
    u //
      .combine('sourceType', ['VideoElement', 'VideoFrame'])
  )
  .fn(async t => {
    const sourceType = t.params.sourceType;
    const videoElement = t.getDefaultVideoElementAndCheck();

    let bindGroup;
    let externalTexture;
    await startPlayingAndWaitForVideo(videoElement, async () => {
      const source =
        sourceType === 'VideoFrame'
          ? await getVideoFrameFromVideoElement(t, videoElement)
          : videoElement;

      // Import GPUExternalTexture
      queueMicrotask(() => {
        externalTexture = t.device.importExternalTexture({
          source: source,
        });
      });

      // Submit GPUExternalTexture
      queueMicrotask(() => {
        bindGroup = t.device.createBindGroup({
          layout: t.getDefaultBindGroupLayout(),
          entries: [{ binding: 0, resource: externalTexture }],
        });
        t.submitCommandBuffer(bindGroup, true);
      });
    });
  });

g.test('import_and_use_in_different_task')
  .desc(
    `
    Tests that in the different task scope, previous imported GPUExternalTexture
    should be expired.
    `
  )
  .params(u =>
    u //
      .combine('sourceType', ['VideoElement', 'VideoFrame'])
  )
  .fn(async t => {
    const sourceType = t.params.sourceType;
    const videoElement = t.getDefaultVideoElementAndCheck();

    let bindGroup;
    let externalTexture;
    await startPlayingAndWaitForVideo(videoElement, async () => {
      const source =
        sourceType === 'VideoFrame'
          ? await getVideoFrameFromVideoElement(t, videoElement)
          : videoElement;
      externalTexture = t.device.importExternalTexture({
        source: source,
      });

      bindGroup = t.device.createBindGroup({
        layout: t.getDefaultBindGroupLayout(),
        entries: [{ binding: 0, resource: externalTexture }],
      });

      t.submitCommandBuffer(bindGroup, true);
    });

    await waitForNextTask(() => {
      // Enter in another task scope, previous GPUExternalTexture should be expired.
      t.submitCommandBuffer(bindGroup, false);
    });
  });

g.test('use_import_to_refresh')
  .desc(
    `
    Tests that in the different task scope, imported GPUExternalTexture
    again on the same source frame should return the same GPUExternalTexture
    object and refresh it.
    `
  )
  .params(u =>
    u //
      .combine('sourceType', ['VideoElement', 'VideoFrame'])
  )
  .fn(async t => {
    const sourceType = t.params.sourceType;
    const videoElement = t.getDefaultVideoElementAndCheck();

    let bindGroup;
    let externalTexture;
    let source;
    await startPlayingAndWaitForVideo(videoElement, async () => {
      source =
        sourceType === 'VideoFrame'
          ? await getVideoFrameFromVideoElement(t, videoElement)
          : videoElement;
      externalTexture = t.device.importExternalTexture({
        source: source,
      });

      bindGroup = t.device.createBindGroup({
        layout: t.getDefaultBindGroupLayout(),
        entries: [{ binding: 0, resource: externalTexture }],
      });

      t.submitCommandBuffer(bindGroup, true);
    });

    await waitForNextTask(() => {
      // Video frame is not updated, import should return the same GPUExternalTexture object.
      const shouldBeTheSameExternalTexture = t.device.importExternalTexture({
        source: source,
      });
      assert(externalTexture === shouldBeTheSameExternalTexture);

      // ImportExternalTexture should refresh expired GPUExternalTexture.
      t.submitCommandBuffer(bindGroup, true);
    });
  });

g.test('webcodec_video_frame_close_expire_immediately')
  .desc(
    `
    Tests that in the same task scope, imported GPUExternalTexture should be expired
    immediately if webcodec VideoFrame.close() is called.
    `
  )
  .fn(async t => {
    const videoElement = t.getDefaultVideoElementAndCheck();

    let bindGroup;
    let externalTexture;
    await startPlayingAndWaitForVideo(videoElement, async () => {
      const source = await getVideoFrameFromVideoElement(t, videoElement);
      externalTexture = t.device.importExternalTexture({
        source: source,
      });

      bindGroup = t.device.createBindGroup({
        layout: t.getDefaultBindGroupLayout(),
        entries: [{ binding: 0, resource: externalTexture }],
      });

      t.submitCommandBuffer(bindGroup, true);

      source.close();

      t.submitCommandBuffer(bindGroup, false);
    });
  });

g.test('import_from_different_video_frame')
  .desc(
    `
    Tests that imported GPUExternalTexture from different video frame should
    return different GPUExternalTexture objects.
    If the frames are from the same HTMLVideoElement source, GPUExternalTexture
    with old frame should be expired and not been refreshed again.
    `
  )
  .fn(async t => {
    const videoElement = t.getDefaultVideoElementAndCheck();

    let bindGroup;
    let externalTexture;
    await startPlayingAndWaitForVideo(videoElement, () => {
      externalTexture = t.device.importExternalTexture({
        source: videoElement,
      });

      bindGroup = t.device.createBindGroup({
        layout: t.getDefaultBindGroupLayout(),
        entries: [{ binding: 0, resource: externalTexture }],
      });

      t.submitCommandBuffer(bindGroup, true);
    });

    // Update new video frame.
    await waitForNextFrame(videoElement, () => {
      // Import again for the new video frame.
      const newValidExternalTexture = t.device.importExternalTexture({
        source: videoElement,
      });
      assert(externalTexture !== newValidExternalTexture);

      // VideoFrame is updated. GPUExternalTexture imported from old frame should be expired and
      // cannot be refreshed again.
      // Using the GPUExternalTexture should result in an error.
      t.submitCommandBuffer(bindGroup, false);

      // Update bindGroup with updated GPUExternalTexture should work.
      bindGroup = t.device.createBindGroup({
        layout: t.getDefaultBindGroupLayout(),
        entries: [{ binding: 0, resource: newValidExternalTexture }],
      });
      t.submitCommandBuffer(bindGroup, true);
    });
  });
