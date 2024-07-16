/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Validation tests for indexed draws accessing the index buffer.
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { ValidationTest } from '../../validation_test.js';

class F extends ValidationTest {
  createIndexBuffer(indexData) {
    return this.makeBufferWithContents(new Uint32Array(indexData), GPUBufferUsage.INDEX);
  }

  createRenderPipeline() {
    return this.device.createRenderPipeline({
      layout: 'auto',
      vertex: {
        module: this.device.createShaderModule({
          code: `
            @vertex fn main() -> @builtin(position) vec4<f32> {
              return vec4<f32>(0.0, 0.0, 0.0, 1.0);
            }`
        }),
        entryPoint: 'main'
      },
      fragment: {
        module: this.device.createShaderModule({
          code: `
            @fragment fn main() -> @location(0) vec4<f32> {
              return vec4<f32>(0.0, 1.0, 0.0, 1.0);
            }`
        }),
        entryPoint: 'main',
        targets: [{ format: 'rgba8unorm' }]
      },
      primitive: {
        topology: 'triangle-strip',
        stripIndexFormat: 'uint32'
      }
    });
  }

  beginRenderPass(encoder) {
    const colorAttachment = this.createTextureTracked({
      format: 'rgba8unorm',
      size: { width: 1, height: 1, depthOrArrayLayers: 1 },
      usage: GPUTextureUsage.RENDER_ATTACHMENT
    });

    return encoder.beginRenderPass({
      colorAttachments: [
      {
        view: colorAttachment.createView(),
        clearValue: { r: 0.0, g: 0.0, b: 0.0, a: 1.0 },
        loadOp: 'clear',
        storeOp: 'store'
      }]

    });
  }

  drawIndexed(
  indexBuffer,
  indexCount,
  instanceCount,
  firstIndex,
  baseVertex,
  firstInstance,
  isSuccess)
  {
    const pipeline = this.createRenderPipeline();

    const encoder = this.device.createCommandEncoder();
    const pass = this.beginRenderPass(encoder);
    pass.setPipeline(pipeline);
    pass.setIndexBuffer(indexBuffer, 'uint32');
    pass.drawIndexed(indexCount, instanceCount, firstIndex, baseVertex, firstInstance);
    pass.end();

    if (isSuccess) {
      this.device.queue.submit([encoder.finish()]);
    } else {
      this.expectValidationError(() => {
        encoder.finish();
      });
    }
  }
}

export const g = makeTestGroup(F);

g.test('out_of_bounds').
desc(
  `Test drawing with out of bound index access to make sure encoder validation catch the
    following indexCount and firstIndex OOB conditions
    - either is within bound but indexCount + firstIndex is out of bound
    - only firstIndex is out of bound
    - only indexCount is out of bound
    - firstIndex much larger than indexCount
    - indexCount much larger than firstIndex
    - max uint32 value for both to make sure the sum doesn't overflow
    - max uint32 indexCount and small firstIndex
    - max uint32 firstIndex and small indexCount
    Together with normal and large instanceCount`
).
params(
  (u) =>
  u.
  combineWithParams([
  { indexCount: 6, firstIndex: 0 }, // draw all 6 out of 6 index
  { indexCount: 5, firstIndex: 1 }, // draw the last 5 out of 6 index
  { indexCount: 1, firstIndex: 5 }, // draw the last 1 out of 6 index
  { indexCount: 0, firstIndex: 6 }, // firstIndex point to the one after last, but (indexCount + firstIndex) * stride <= bufferSize, valid
  { indexCount: 0, firstIndex: 7 }, // (indexCount + firstIndex) * stride > bufferSize, invalid
  { indexCount: 7, firstIndex: 0 }, // only indexCount out of bound
  { indexCount: 6, firstIndex: 1 }, // indexCount + firstIndex out of bound
  { indexCount: 1, firstIndex: 6 }, // indexCount valid, but (indexCount + firstIndex) out of bound
  { indexCount: 6, firstIndex: 10000 }, // firstIndex much larger than the bound
  { indexCount: 10000, firstIndex: 0 }, // indexCount much larger than the bound
  { indexCount: 0xffffffff, firstIndex: 0xffffffff }, // max uint32 value
  { indexCount: 0xffffffff, firstIndex: 2 }, // max uint32 indexCount and small firstIndex
  { indexCount: 2, firstIndex: 0xffffffff } // small indexCount and max uint32 firstIndex
  ]).
  combine('instanceCount', [1, 10000]) // normal and large instanceCount
).
fn((t) => {
  const { indexCount, firstIndex, instanceCount } = t.params;

  const indexBuffer = t.createIndexBuffer([0, 1, 2, 3, 1, 2]);
  const isSuccess = indexCount + firstIndex <= 6;

  t.drawIndexed(indexBuffer, indexCount, instanceCount, firstIndex, 0, 0, isSuccess);
});

g.test('out_of_bounds_zero_sized_index_buffer').
desc(
  `Test drawing with an empty index buffer to make sure the encoder validation catch the
    following indexCount and firstIndex conditions
    - indexCount + firstIndex is out of bound
    - indexCount is 0 but firstIndex is out of bound
    - only indexCount is out of bound
    - both are 0s (not out of bound) but index buffer size is 0
    Together with normal and large instanceCount`
).
params(
  (u) =>
  u.
  combineWithParams([
  { indexCount: 3, firstIndex: 1 }, // indexCount + firstIndex out of bound
  { indexCount: 0, firstIndex: 1 }, // indexCount is 0 but firstIndex out of bound
  { indexCount: 3, firstIndex: 0 }, // only indexCount out of bound
  { indexCount: 0, firstIndex: 0 } // just zeros, valid
  ]).
  combine('instanceCount', [1, 10000]) // normal and large instanceCount
).
fn((t) => {
  const { indexCount, firstIndex, instanceCount } = t.params;

  const indexBuffer = t.createIndexBuffer([]);
  const isSuccess = indexCount + firstIndex <= 0;

  t.drawIndexed(indexBuffer, indexCount, instanceCount, firstIndex, 0, 0, isSuccess);
});