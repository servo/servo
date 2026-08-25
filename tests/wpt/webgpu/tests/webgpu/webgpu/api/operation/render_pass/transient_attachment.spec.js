/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `API Operation Tests for transient attachment in render passes.`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { AllFeaturesMaxLimitsGPUTest } from '../../../gpu_test.js';

const kSize = 4;

export const g = makeTestGroup(AllFeaturesMaxLimitsGPUTest);

g.test('increasing_attachments_count').
desc(
  `
    Use multiple render passes with increasing amounts of transient attachments.
  `
).
fn((t) => {
  // MAINTENANCE_TODO(#4509): Remove this after all implementations have TRANSIENT_ATTACHMENT.
  t.skipIfTransientAttachmentNotSupported();

  const maxAttachments = t.device.limits.maxColorAttachments;
  const encoder = t.device.createCommandEncoder();

  for (let count = 1; count <= maxAttachments; count++) {
    const colorAttachments = [];

    for (let i = 0; i < count; i++) {
      const texture = t.createTextureTracked({
        size: [kSize, kSize, 1],
        format: 'rgba8unorm',
        usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.TRANSIENT_ATTACHMENT
      });

      colorAttachments.push({
        view: texture.createView(),
        clearValue: { r: 0.0, g: 0.0, b: 0.0, a: 0.0 },
        loadOp: 'clear',
        storeOp: 'discard'
      });
    }

    const pass = encoder.beginRenderPass({ colorAttachments });
    pass.end();
  }
  t.device.queue.submit([encoder.finish()]);
});

g.test('overlapping_transient_attachments').
desc(
  `
    Use multiple render passes with transient attachments in a circular overlap pattern:
    Pass 1: (T1, T2)
    Pass 2: (T2, T3)
    Pass 3: (T3, T1)

    This stresses the driver's transient memory allocator. If the driver naively reuses
    T1's memory for T3 during Pass 2 (because T1 isn't active in Pass 2),
    Pass 3 will fail or corrupt because T1 and T3 are both needed simultaneously again.
  `
).
fn((t) => {
  // MAINTENANCE_TODO(#4509): Remove this after all implementations have TRANSIENT_ATTACHMENT.
  t.skipIfTransientAttachmentNotSupported();

  const encoder = t.device.createCommandEncoder();

  const desc = {
    size: [kSize, kSize, 1],
    format: 'rgba8unorm',
    usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.TRANSIENT_ATTACHMENT
  };
  const t1 = t.createTextureTracked(desc);
  const t2 = t.createTextureTracked(desc);
  const t3 = t.createTextureTracked(desc);

  const passes = [
  [t1, t2], // Pass 1
  [t2, t3], // Pass 2
  [t3, t1] // Pass 3
  ];

  for (const attachments of passes) {
    const colorAttachments = attachments.map((texture) => ({
      view: texture.createView(),
      clearValue: { r: 0.0, g: 0.0, b: 0.0, a: 0.0 },
      loadOp: 'clear',
      storeOp: 'discard'
    }));

    const pass = encoder.beginRenderPass({ colorAttachments });
    pass.end();
  }

  t.device.queue.submit([encoder.finish()]);
});