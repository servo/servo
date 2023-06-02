/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Test validation of pushDebugGroup, popDebugGroup, and insertDebugMarker.
`;
import { makeTestGroup } from '../../../common/framework/test_group.js';

import { ValidationTest } from './validation_test.js';

class F extends ValidationTest {
  beginRenderPass(commandEncoder) {
    const attachmentTexture = this.device.createTexture({
      format: 'rgba8unorm',
      size: { width: 16, height: 16, depthOrArrayLayers: 1 },
      usage: GPUTextureUsage.RENDER_ATTACHMENT,
    });
    this.trackForCleanup(attachmentTexture);
    return commandEncoder.beginRenderPass({
      colorAttachments: [
        {
          view: attachmentTexture.createView(),
          clearValue: { r: 1.0, g: 0.0, b: 0.0, a: 1.0 },
          loadOp: 'clear',
          storeOp: 'store',
        },
      ],
    });
  }
}

export const g = makeTestGroup(F);

g.test('push_pop_call_count_unbalance,command_encoder')
  .desc(
    `
  Test that a validation error is generated if {push,pop} debug group call count is not paired.
  `
  )
  .paramsSubcasesOnly(u =>
    u //
      .combine('pushCount', [1, 2, 3])
      .combine('popCount', [1, 2, 3])
  )
  .fn(t => {
    const { pushCount, popCount } = t.params;

    const encoder = t.device.createCommandEncoder();

    for (let i = 0; i < pushCount; ++i) {
      encoder.pushDebugGroup('EventStart');
    }

    encoder.insertDebugMarker('Marker');

    for (let i = 0; i < popCount; ++i) {
      encoder.popDebugGroup();
    }

    t.expectValidationError(() => {
      encoder.finish();
    }, pushCount !== popCount);
  });

g.test('push_pop_call_count_unbalance,render_compute_pass')
  .desc(
    `
  Test that a validation error is generated if {push,pop} debug group call count is not paired in
  ComputePassEncoder and RenderPassEncoder.
  `
  )
  .params(u =>
    u //
      .combine('passType', ['compute', 'render'])
      .beginSubcases()
      .combine('pushCount', [1, 2, 3])
      .combine('popCount', [1, 2, 3])
  )
  .fn(t => {
    const { passType, pushCount, popCount } = t.params;

    const encoder = t.device.createCommandEncoder();

    const pass = passType === 'compute' ? encoder.beginComputePass() : t.beginRenderPass(encoder);

    for (let i = 0; i < pushCount; ++i) {
      pass.pushDebugGroup('EventStart');
    }

    pass.insertDebugMarker('Marker');

    for (let i = 0; i < popCount; ++i) {
      pass.popDebugGroup();
    }

    t.expectValidationError(() => {
      pass.end();
      encoder.finish();
    }, pushCount !== popCount);
  });
