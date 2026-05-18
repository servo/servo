/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Compute dispatch validation tests.
`;import { AllFeaturesMaxLimitsGPUTest } from '../.././gpu_test.js';
import { makeTestGroup } from '../../../common/framework/test_group.js';

export const g = makeTestGroup(AllFeaturesMaxLimitsGPUTest);

g.test('dispatch,linear_indexing_range').
desc('Tests validation of total invocations for linear_indexing built-in values').
params((u) =>
u.
combine('builtin', ['global_invocation_index', 'workgroup_index']).
beginSubcases().
combine('size', ['max', 'valid'])
).
fn((t) => {
  // Other builtins are not tested due to onerous runtimes.
  t.skipIf(!t.hasLanguageFeature('linear_indexing'), 'Missing linear_indexing language feature');

  // Spec limits:
  // - maxComputeWorkgroupsPerDimension = 65535
  const { maxComputeWorkgroupsPerDimension } = t.device.limits;
  const x = t.params.builtin === 'global_invocation_index' ? 2 : 1,
    y = 1,
    z = 1;
  const wgSize = x * y * z;
  const countX = maxComputeWorkgroupsPerDimension;
  const countY = t.params.size === 'max' ? maxComputeWorkgroupsPerDimension : 1;
  const countZ = t.params.builtin === 'workgroup_index' ? 2 : 1;

  const totalInvocations = wgSize * countX * countY * countZ;
  t.skipIf(t.params.size === 'max' && totalInvocations <= 0xffffffff, 'Uninteresting test');

  const code = `
@compute @workgroup_size(${x}, ${y}, ${z})
fn main(@builtin(${t.params.builtin}) input : u32) {
  _ = input;
}`;

  const shaderModule = t.device.createShaderModule({ code });
  const computePipeline = t.device.createComputePipeline({
    layout: 'auto',
    compute: {
      module: shaderModule
    }
  });
  const commandEncoder = t.device.createCommandEncoder();
  const computePassEncoder = commandEncoder.beginComputePass();
  computePassEncoder.setPipeline(computePipeline);
  computePassEncoder.dispatchWorkgroups(countX, countY, countZ);
  computePassEncoder.end();

  t.expectValidationError(() => {
    commandEncoder.finish();
  }, t.params.size === 'max');
});

g.test('dispatchIndirect,linear_indexing_range').
desc('Tests dispatchIndirect skips when linear_indexing is out of range').
params((u) =>
u.
combine('builtin', ['global_invocation_index', 'workgroup_index']).
beginSubcases().
combine('size', ['max', 'valid'])
).
fn((t) => {
  // Other builtins are not tested due to onerous runtimes.
  t.skipIf(!t.hasLanguageFeature('linear_indexing'), 'Missing linear_indexing language feature');

  // Spec limits:
  // - maxComputeWorkgroupsPerDimension = 65535
  const { maxComputeWorkgroupsPerDimension } = t.device.limits;
  const x = t.params.builtin === 'global_invocation_index' ? 2 : 1,
    y = 1,
    z = 1;
  const wgSize = x * y * z;
  const countX = maxComputeWorkgroupsPerDimension;
  const countY = t.params.size === 'max' ? maxComputeWorkgroupsPerDimension : 1;
  const countZ = t.params.builtin === 'workgroup_index' ? 2 : 1;

  const totalInvocations = wgSize * countX * countY * countZ;
  t.skipIf(t.params.size === 'max' && totalInvocations <= 0xffffffff, 'Uninteresting test');

  const kMagic = 0xdeadbeef;
  const code = `
@group(0) @binding(0)
var<storage, read_write> out : u32;

@compute @workgroup_size(${x}, ${y}, ${z})
fn main(@builtin(${t.params.builtin}) input : u32,
        @builtin(global_invocation_id) gid : vec3u) {
  _ = input;
  if (gid.x == 0 && gid.y == 0 && gid.z == 0) {
    out = ${kMagic};
  }
}`;

  const dispatchIndirectCounts = new Uint32Array(3);
  dispatchIndirectCounts[0] = countX;
  dispatchIndirectCounts[1] = countY;
  dispatchIndirectCounts[2] = countZ;
  const indirectBuffer = t.makeBufferWithContents(
    dispatchIndirectCounts,
    GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST | GPUBufferUsage.INDIRECT
  );
  t.trackForCleanup(indirectBuffer);
  const outputBuffer = t.makeBufferWithContents(
    new Uint32Array([0]),
    GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST | GPUBufferUsage.STORAGE
  );
  t.trackForCleanup(outputBuffer);

  const shaderModule = t.device.createShaderModule({ code });
  const computePipeline = t.device.createComputePipeline({
    layout: 'auto',
    compute: {
      module: shaderModule
    }
  });
  const bg = t.device.createBindGroup({
    layout: computePipeline.getBindGroupLayout(0),
    entries: [
    {
      binding: 0,
      resource: {
        buffer: outputBuffer
      }
    }]

  });
  const commandEncoder = t.device.createCommandEncoder();
  const computePassEncoder = commandEncoder.beginComputePass();
  computePassEncoder.setPipeline(computePipeline);
  computePassEncoder.setBindGroup(0, bg);
  computePassEncoder.dispatchWorkgroupsIndirect(indirectBuffer, 0);
  computePassEncoder.end();
  t.queue.submit([commandEncoder.finish()]);

  const expected = t.params.size === 'max' ? 0 : kMagic;
  t.expectGPUBufferValuesEqual(outputBuffer, new Uint32Array([expected]));
});