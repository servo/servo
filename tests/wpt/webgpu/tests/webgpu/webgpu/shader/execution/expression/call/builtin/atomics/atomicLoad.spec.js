/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Returns the atomically loaded the value pointed to by atomic_ptr. It does not modify the object.
`;import { makeTestGroup } from '../../../../../../../common/framework/test_group.js';
import { keysOf } from '../../../../../../../common/util/data_tables.js';
import { GPUTest } from '../../../../../../gpu_test.js';

import { dispatchSizes, workgroupSizes, typedArrayCtor, kMapId } from './harness.js';

export const g = makeTestGroup(GPUTest);

g.test('load_storage').
specURL('https://www.w3.org/TR/WGSL/#atomic-load').
desc(
  `
AS is storage or workgroup
T is i32 or u32

fn atomicLoad(atomic_ptr: ptr<AS, atomic<T>, read_write>) -> T

`
).
params((u) =>
u.
combine('workgroupSize', workgroupSizes).
combine('dispatchSize', dispatchSizes).
combine('mapId', keysOf(kMapId)).
combine('scalarType', ['u32', 'i32'])
).
fn((t) => {
  const numInvocations = t.params.workgroupSize * t.params.dispatchSize;
  const bufferNumElements = numInvocations;
  const scalarType = t.params.scalarType;
  const mapId = kMapId[t.params.mapId];

  const wgsl = `
      @group(0) @binding(0)
      var<storage, read_write> input : array<atomic<${scalarType}>>;

      @group(0) @binding(1)
      var<storage, read_write> output : array<${scalarType}>;

      @compute @workgroup_size(${t.params.workgroupSize})
      fn main(
          @builtin(global_invocation_id) global_invocation_id : vec3<u32>,
          ) {
        let id = ${scalarType}(global_invocation_id[0]);
        output[id] = atomicLoad(&input[id]);
      }
    `;

  const pipeline = t.device.createComputePipeline({
    layout: 'auto',
    compute: {
      module: t.device.createShaderModule({ code: wgsl }),
      entryPoint: 'main'
    }
  });

  const arrayType = typedArrayCtor(scalarType);

  // Create input buffer with values [map_id(0)..map_id(n)]
  const inputBuffer = t.createBufferTracked({
    size: bufferNumElements * arrayType.BYTES_PER_ELEMENT,
    usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC,
    mappedAtCreation: true
  });
  const data = new arrayType(inputBuffer.getMappedRange());
  data.forEach((_, i) => data[i] = mapId.f(i, numInvocations));
  inputBuffer.unmap();

  const outputBuffer = t.createBufferTracked({
    size: bufferNumElements * arrayType.BYTES_PER_ELEMENT,
    usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC
  });

  const bindGroup = t.device.createBindGroup({
    layout: pipeline.getBindGroupLayout(0),
    entries: [
    { binding: 0, resource: { buffer: inputBuffer } },
    { binding: 1, resource: { buffer: outputBuffer } }]

  });

  // Run the shader.
  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginComputePass();
  pass.setPipeline(pipeline);
  pass.setBindGroup(0, bindGroup);
  pass.dispatchWorkgroups(t.params.dispatchSize);
  pass.end();
  t.queue.submit([encoder.finish()]);

  // Both input and output buffer should be the same now
  const expected = new (typedArrayCtor(t.params.scalarType))(bufferNumElements);
  expected.forEach((_, i) => expected[i] = mapId.f(i, numInvocations));
  t.expectGPUBufferValuesEqual(inputBuffer, expected);
  t.expectGPUBufferValuesEqual(outputBuffer, expected);
});

g.test('load_workgroup').
specURL('https://www.w3.org/TR/WGSL/#atomic-load').
desc(
  `
AS is storage or workgroup
T is i32 or u32

fn atomicLoad(atomic_ptr: ptr<AS, atomic<T>, read_write>) -> T

`
).
params((u) =>
u.
combine('workgroupSize', workgroupSizes).
combine('dispatchSize', dispatchSizes).
combine('mapId', keysOf(kMapId)).
combine('scalarType', ['u32', 'i32'])
).
fn((t) => {
  const numInvocations = t.params.workgroupSize;
  const wgNumElements = numInvocations;
  const scalarType = t.params.scalarType;
  const dispatchSize = t.params.dispatchSize;
  const mapId = kMapId[t.params.mapId];
  const extra = mapId.wgsl(numInvocations, t.params.scalarType); // Defines map_id()

  const wgsl =
  `
      var<workgroup> wg: array<atomic<${scalarType}>, ${wgNumElements}>;

      // Result of each workgroup is written to output[workgroup_id.x]
      @group(0) @binding(0)
      var<storage, read_write> output: array<${scalarType}, ${wgNumElements * dispatchSize}>;

      @compute @workgroup_size(${t.params.workgroupSize})
      fn main(
          @builtin(local_invocation_index) local_invocation_index: u32,
          @builtin(workgroup_id) workgroup_id : vec3<u32>
          ) {
        let id = ${scalarType}(local_invocation_index);
        let global_id = ${scalarType}(workgroup_id.x * ${wgNumElements} + local_invocation_index);

        // Initialize wg[id] with this invocations global id (mapped)
        atomicStore(&wg[id], map_id(global_id));
        workgroupBarrier();

        // Test atomic loading of value at wg[id] and store result in output[global_id]
        output[global_id] = atomicLoad(&wg[id]);
      }
      ` + extra;

  const pipeline = t.device.createComputePipeline({
    layout: 'auto',
    compute: {
      module: t.device.createShaderModule({ code: wgsl }),
      entryPoint: 'main'
    }
  });

  const arrayType = typedArrayCtor(scalarType);

  const outputBuffer = t.createBufferTracked({
    size: wgNumElements * dispatchSize * arrayType.BYTES_PER_ELEMENT,
    usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC
  });

  const bindGroup = t.device.createBindGroup({
    layout: pipeline.getBindGroupLayout(0),
    entries: [{ binding: 0, resource: { buffer: outputBuffer } }]
  });

  // Run the shader.
  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginComputePass();
  pass.setPipeline(pipeline);
  pass.setBindGroup(0, bindGroup);
  pass.dispatchWorkgroups(dispatchSize);
  pass.end();
  t.queue.submit([encoder.finish()]);

  // Expected values should be map_id(0..n)
  const expected = new (typedArrayCtor(t.params.scalarType))(
    wgNumElements * t.params.dispatchSize
  );
  expected.forEach((_, i) => expected[i] = mapId.f(i, numInvocations));

  t.expectGPUBufferValuesEqual(outputBuffer, expected);
});