/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Buffer tests in createBindGroup.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { AllFeaturesMaxLimitsGPUTest } from '../../../gpu_test.js';

export const g = makeTestGroup(AllFeaturesMaxLimitsGPUTest);

g.test('buffer_binding_resource').
desc(
  `Validate the correctness of the buffer binding resource by filling the buffer with
    testable data, clearing buffer in shader, and verifying the content of the whole buffer:
  - covers the whole buffer
  - covers the beginning of the buffer
  - covers the end of the buffer
  - covers neither the beginning nor the end of the buffer`
).
paramsSubcasesOnly((u) =>
u //
.combine('bindBufferResource', [false, true]).
combine('offset', [0, 256, undefined]).
combine('size', [4, 8, undefined]).
combine('extraBufferSize', [0, 8])
// offset and size don't matter if bindBufferResource is true
.filter((p) => !p.bindBufferResource || p.offset === undefined && p.size === undefined)
).
fn((t) => {
  const { bindBufferResource, offset, size, extraBufferSize } = t.params;

  const bufferSize = (offset ?? 0) + (size ?? 16) + extraBufferSize;
  const bufferData = new Uint8Array(bufferSize);
  for (let i = 0; i < bufferSize; ++i) {
    bufferData[i] = i + 1;
  }

  const buffer = t.makeBufferWithContents(
    bufferData,
    GPUBufferUsage.COPY_DST | GPUBufferUsage.COPY_SRC | GPUBufferUsage.STORAGE
  );

  const pipeline = t.device.createComputePipeline({
    layout: 'auto',
    compute: {
      module: t.device.createShaderModule({
        code: `
            @group(0) @binding(0) var<storage, read_write> buffer : array<u32>;

            @compute @workgroup_size(1) fn main() {
              for (var i = 0u; i < arrayLength(&buffer); i = i + 1u) {
                buffer[i] = 0;
              }
              return;
            }`
      })
    }
  });

  const resource = bindBufferResource ? buffer : { buffer, offset, size };
  const bg = t.device.createBindGroup({
    entries: [{ binding: 0, resource }],
    layout: pipeline.getBindGroupLayout(0)
  });

  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginComputePass();
  pass.setPipeline(pipeline);
  pass.setBindGroup(0, bg);
  pass.dispatchWorkgroups(1);
  pass.end();
  t.device.queue.submit([encoder.finish()]);

  const expectOffset = bindBufferResource ? 0 : offset ?? 0;
  const expectSize = bindBufferResource ? bufferSize : size ?? bufferSize - expectOffset;

  for (let i = 0; i < expectSize; ++i) {
    bufferData[expectOffset + i] = 0;
  }

  t.expectGPUBufferValuesEqual(buffer, bufferData);
});