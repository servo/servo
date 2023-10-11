/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Atomically stores the value v in the atomic object pointed to atomic_ptr and returns the original value stored in the atomic object.
`;
import { makeTestGroup } from '../../../../../../../common/framework/test_group.js';
import { keysOf } from '../../../../../../../common/util/data_tables.js';
import { GPUTest } from '../../../../../../gpu_test.js';
import { checkElementsEqual } from '../../../../../../util/check_contents.js';

import { dispatchSizes, workgroupSizes, typedArrayCtor, kMapId } from './harness.js';

export const g = makeTestGroup(GPUTest);

g.test('exchange_storage_basic')
  .specURL('https://www.w3.org/TR/WGSL/#atomic-rmw')
  .desc(
    `
AS is storage or workgroup
T is i32 or u32

fn atomicExchange(atomic_ptr: ptr<AS, atomic<T>, read_write>, v: T) -> T
`
  )
  .params(u =>
    u
      .combine('workgroupSize', workgroupSizes)
      .combine('dispatchSize', dispatchSizes)
      .combine('mapId', keysOf(kMapId))
      .combine('scalarType', ['u32', 'i32'])
  )
  .fn(t => {
    const numInvocations = t.params.workgroupSize * t.params.dispatchSize;
    const bufferNumElements = numInvocations;
    const scalarType = t.params.scalarType;
    const mapId = kMapId[t.params.mapId];
    const extra = mapId.wgsl(numInvocations, t.params.scalarType); // Defines map_id()

    const wgsl =
      `
      @group(0) @binding(0)
      var<storage, read_write> input : array<atomic<${scalarType}>>;

      @group(0) @binding(1)
      var<storage, read_write> output : array<${scalarType}>;

      @compute @workgroup_size(${t.params.workgroupSize})
      fn main(
          @builtin(global_invocation_id) global_invocation_id : vec3<u32>,
          ) {
        let id = ${scalarType}(global_invocation_id[0]);

        output[id] = atomicExchange(&input[id], map_id(id * 2));
      }
    ` + extra;

    const pipeline = t.device.createComputePipeline({
      layout: 'auto',
      compute: {
        module: t.device.createShaderModule({ code: wgsl }),
        entryPoint: 'main',
      },
    });

    const arrayType = typedArrayCtor(scalarType);

    // Create input buffer with values [0..n]
    const inputBuffer = t.device.createBuffer({
      size: bufferNumElements * arrayType.BYTES_PER_ELEMENT,
      usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC,
      mappedAtCreation: true,
    });
    t.trackForCleanup(inputBuffer);
    const data = new arrayType(inputBuffer.getMappedRange());
    data.forEach((_, i) => (data[i] = i));
    inputBuffer.unmap();

    const outputBuffer = t.device.createBuffer({
      size: bufferNumElements * arrayType.BYTES_PER_ELEMENT,
      usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC,
    });
    t.trackForCleanup(outputBuffer);

    const bindGroup = t.device.createBindGroup({
      layout: pipeline.getBindGroupLayout(0),
      entries: [
        { binding: 0, resource: { buffer: inputBuffer } },
        { binding: 1, resource: { buffer: outputBuffer } },
      ],
    });

    // Run the shader.
    const encoder = t.device.createCommandEncoder();
    const pass = encoder.beginComputePass();
    pass.setPipeline(pipeline);
    pass.setBindGroup(0, bindGroup);
    pass.dispatchWorkgroups(t.params.dispatchSize);
    pass.end();
    t.queue.submit([encoder.finish()]);

    // Output buffer should be the same as the initial input buffer as it contains
    // values returned from atomicExchange
    const outputExpected = new (typedArrayCtor(t.params.scalarType))(bufferNumElements);
    outputExpected.forEach((_, i) => (outputExpected[i] = i));
    t.expectGPUBufferValuesEqual(outputBuffer, outputExpected);

    // And the input buffer should have been modified to a computed value
    const inputExpected = new (typedArrayCtor(t.params.scalarType))(bufferNumElements);
    inputExpected.forEach((_, i) => (inputExpected[i] = mapId.f(i * 2, numInvocations)));
    t.expectGPUBufferValuesEqual(inputBuffer, inputExpected);
  });

g.test('exchange_workgroup_basic')
  .specURL('https://www.w3.org/TR/WGSL/#atomic-load')
  .desc(
    `
AS is storage or workgroup
T is i32 or u32

fn atomicLoad(atomic_ptr: ptr<AS, atomic<T>, read_write>) -> T

`
  )
  .params(u =>
    u
      .combine('workgroupSize', workgroupSizes)
      .combine('dispatchSize', dispatchSizes)
      .combine('mapId', keysOf(kMapId))
      .combine('scalarType', ['u32', 'i32'])
  )
  .fn(t => {
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

      @group(0) @binding(1)
      var<storage, read_write> wg_copy: array<${scalarType}, ${wgNumElements * dispatchSize}>;

      @compute @workgroup_size(${t.params.workgroupSize})
      fn main(
          @builtin(local_invocation_index) local_invocation_index: u32,
          @builtin(workgroup_id) workgroup_id : vec3<u32>
          ) {
        let id = ${scalarType}(local_invocation_index);
        let global_id = ${scalarType}(workgroup_id.x * ${wgNumElements} + local_invocation_index);

        // Initialize wg[id] with this invocations global id
        atomicStore(&wg[id], global_id);
        workgroupBarrier();

        // Test atomicExchange, storing old value into output
        output[global_id] = atomicExchange(&wg[id], map_id(global_id * 2));

        // Copy new value into wg_copy
        wg_copy[global_id] = atomicLoad(&wg[id]);
      }
      ` + extra;

    const pipeline = t.device.createComputePipeline({
      layout: 'auto',
      compute: {
        module: t.device.createShaderModule({ code: wgsl }),
        entryPoint: 'main',
      },
    });

    const arrayType = typedArrayCtor(scalarType);

    const outputBuffer = t.device.createBuffer({
      size: wgNumElements * dispatchSize * arrayType.BYTES_PER_ELEMENT,
      usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC,
    });
    t.trackForCleanup(outputBuffer);

    const wgCopyBuffer = t.device.createBuffer({
      size: wgNumElements * dispatchSize * arrayType.BYTES_PER_ELEMENT,
      usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC,
    });
    t.trackForCleanup(outputBuffer);

    const bindGroup = t.device.createBindGroup({
      layout: pipeline.getBindGroupLayout(0),
      entries: [
        { binding: 0, resource: { buffer: outputBuffer } },
        { binding: 1, resource: { buffer: wgCopyBuffer } },
      ],
    });

    // Run the shader.
    const encoder = t.device.createCommandEncoder();
    const pass = encoder.beginComputePass();
    pass.setPipeline(pipeline);
    pass.setBindGroup(0, bindGroup);
    pass.dispatchWorkgroups(dispatchSize);
    pass.end();
    t.queue.submit([encoder.finish()]);

    // Output buffer should be the same as the initial wg buffer as it contains
    // values returned from atomicExchange
    const outputExpected = new (typedArrayCtor(t.params.scalarType))(wgNumElements * dispatchSize);
    outputExpected.forEach((_, i) => (outputExpected[i] = i));
    t.expectGPUBufferValuesEqual(outputBuffer, outputExpected);

    // And the wg copy buffer should have been modified to a computed value
    const wgCopyBufferExpected = new (typedArrayCtor(t.params.scalarType))(
      wgNumElements * dispatchSize
    );

    wgCopyBufferExpected.forEach(
      (_, i) => (wgCopyBufferExpected[i] = mapId.f(i * 2, numInvocations))
    );

    t.expectGPUBufferValuesEqual(wgCopyBuffer, wgCopyBufferExpected);
  });

g.test('exchange_storage_advanced')
  .specURL('https://www.w3.org/TR/WGSL/#atomic-rmw')
  .desc(
    `
AS is storage or workgroup
T is i32 or u32

fn atomicExchange(atomic_ptr: ptr<AS, atomic<T>, read_write>, v: T) -> T
`
  )
  .params(u =>
    u
      .combine('workgroupSize', workgroupSizes)
      .combine('dispatchSize', dispatchSizes)
      .combine('mapId', keysOf(kMapId))
      .combine('scalarType', ['u32', 'i32'])
  )
  .fn(async t => {
    const numInvocations = t.params.workgroupSize * t.params.dispatchSize;
    const bufferNumElements = numInvocations;
    const scalarType = t.params.scalarType;
    const mapId = kMapId[t.params.mapId];
    const extra = mapId.wgsl(numInvocations, t.params.scalarType); // Defines map_id()

    const wgsl =
      `
      @group(0) @binding(0)
      var<storage, read_write> input : atomic<${scalarType}>;

      @group(0) @binding(1)
      var<storage, read_write> output : array<${scalarType}>;

      @compute @workgroup_size(${t.params.workgroupSize})
      fn main(
          @builtin(global_invocation_id) global_invocation_id : vec3<u32>,
          ) {
        let id = ${scalarType}(global_invocation_id[0]);

        // All invocations exchange with same single memory address, and we store
        // the old value at the current invocation's location in the output buffer.
        output[id] = atomicExchange(&input, map_id(id));
      }
    ` + extra;

    const pipeline = t.device.createComputePipeline({
      layout: 'auto',
      compute: {
        module: t.device.createShaderModule({ code: wgsl }),
        entryPoint: 'main',
      },
    });

    const arrayType = typedArrayCtor(scalarType);

    // Create input buffer of size 1 with initial value 0
    const inputBuffer = t.device.createBuffer({
      size: 1 * arrayType.BYTES_PER_ELEMENT,
      usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC,
    });
    t.trackForCleanup(inputBuffer);

    const outputBuffer = t.device.createBuffer({
      size: bufferNumElements * arrayType.BYTES_PER_ELEMENT,
      usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC,
    });
    t.trackForCleanup(outputBuffer);

    const bindGroup = t.device.createBindGroup({
      layout: pipeline.getBindGroupLayout(0),
      entries: [
        { binding: 0, resource: { buffer: inputBuffer } },
        { binding: 1, resource: { buffer: outputBuffer } },
      ],
    });

    // Run the shader.
    const encoder = t.device.createCommandEncoder();
    const pass = encoder.beginComputePass();
    pass.setPipeline(pipeline);
    pass.setBindGroup(0, bindGroup);
    pass.dispatchWorkgroups(t.params.dispatchSize);
    pass.end();
    t.queue.submit([encoder.finish()]);

    // Read back buffers
    const inputBufferResult = await t.readGPUBufferRangeTyped(inputBuffer, {
      type: arrayType,
      typedLength: inputBuffer.size / arrayType.BYTES_PER_ELEMENT,
    });
    const outputBufferResult = await t.readGPUBufferRangeTyped(outputBuffer, {
      type: arrayType,
      typedLength: outputBuffer.size / arrayType.BYTES_PER_ELEMENT,
    });

    // The one value in the input buffer plus all values in the output buffer
    // should contain initial value 0 plus map_id(0..n), unsorted.
    const values = new arrayType([...inputBufferResult.data, ...outputBufferResult.data]);

    const expected = new arrayType(values.length);
    expected.forEach((_, i) => {
      if (i === 0) {
        expected[0] = 0;
      } else {
        expected[i] = mapId.f(i - 1, numInvocations);
      }
    });

    // Sort both arrays and compare
    values.sort();
    expected.sort(); // Sort because we store hashed results when mapId == 'remap'
    t.expectOK(checkElementsEqual(values, expected));
  });

g.test('exchange_workgroup_advanced')
  .specURL('https://www.w3.org/TR/WGSL/#atomic-load')
  .desc(
    `
AS is storage or workgroup
T is i32 or u32

fn atomicLoad(atomic_ptr: ptr<AS, atomic<T>, read_write>) -> T

`
  )
  .params(u =>
    u
      .combine('workgroupSize', workgroupSizes)
      .combine('dispatchSize', dispatchSizes)
      .combine('mapId', keysOf(kMapId))
      .combine('scalarType', ['u32', 'i32'])
  )
  .fn(async t => {
    const numInvocations = t.params.workgroupSize;
    const scalarType = t.params.scalarType;
    const dispatchSize = t.params.dispatchSize;
    const mapId = kMapId[t.params.mapId];
    const extra = mapId.wgsl(numInvocations, t.params.scalarType); // Defines map_id()

    const wgsl =
      `
      var<workgroup> wg: atomic<${scalarType}>;

      // Will contain the atomicExchange result for each invocation at global index
      @group(0) @binding(0)
      var<storage, read_write> output: array<${scalarType}, ${numInvocations * dispatchSize}>;

      // Will contain the final value in wg in wg_copy for this dispatch
      @group(0) @binding(1)
      var<storage, read_write> wg_copy: array<${scalarType}, ${dispatchSize}>;

      @compute @workgroup_size(${t.params.workgroupSize})
      fn main(
          @builtin(local_invocation_index) local_invocation_index: u32,
          @builtin(workgroup_id) workgroup_id : vec3<u32>
          ) {
        let id = ${scalarType}(local_invocation_index);
        let global_id = ${scalarType}(workgroup_id.x * ${numInvocations} + local_invocation_index);

        // All invocations exchange with same single memory address, and we store
        // the old value at the current invocation's location in the output buffer.
        output[global_id] = atomicExchange(&wg, map_id(id));

        // Once all invocations have completed, the first one copies the final exchanged value
        // to wg_copy for this dispatch (workgroup_id.x)
        workgroupBarrier();
        if (local_invocation_index == 0u) {
          wg_copy[workgroup_id.x] = atomicLoad(&wg);
        }
      }
      ` + extra;

    const pipeline = t.device.createComputePipeline({
      layout: 'auto',
      compute: {
        module: t.device.createShaderModule({ code: wgsl }),
        entryPoint: 'main',
      },
    });

    const arrayType = typedArrayCtor(scalarType);

    const outputBuffer = t.device.createBuffer({
      size: numInvocations * dispatchSize * arrayType.BYTES_PER_ELEMENT,
      usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC,
    });
    t.trackForCleanup(outputBuffer);

    const wgCopyBuffer = t.device.createBuffer({
      size: dispatchSize * arrayType.BYTES_PER_ELEMENT,
      usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC,
    });
    t.trackForCleanup(outputBuffer);

    const bindGroup = t.device.createBindGroup({
      layout: pipeline.getBindGroupLayout(0),
      entries: [
        { binding: 0, resource: { buffer: outputBuffer } },
        { binding: 1, resource: { buffer: wgCopyBuffer } },
      ],
    });

    // Run the shader.
    const encoder = t.device.createCommandEncoder();
    const pass = encoder.beginComputePass();
    pass.setPipeline(pipeline);
    pass.setBindGroup(0, bindGroup);
    pass.dispatchWorkgroups(dispatchSize);
    pass.end();
    t.queue.submit([encoder.finish()]);

    // Read back buffers
    const outputBufferResult = await t.readGPUBufferRangeTyped(outputBuffer, {
      type: arrayType,
      typedLength: outputBuffer.size / arrayType.BYTES_PER_ELEMENT,
    });
    const wgCopyBufferResult = await t.readGPUBufferRangeTyped(wgCopyBuffer, {
      type: arrayType,
      typedLength: wgCopyBuffer.size / arrayType.BYTES_PER_ELEMENT,
    });

    // For each dispatch, the one value in wgCopyBuffer plus all values in the output buffer
    // should contain initial value 0 plus map_id(0..n), unsorted.

    // Expected values for each dispatch
    const expected = new arrayType(numInvocations + 1);
    expected.forEach((_, i) => {
      if (i === 0) {
        expected[0] = 0;
      } else {
        expected[i] = mapId.f(i - 1, numInvocations);
      }
    });
    expected.sort(); // Sort because we store hashed results when mapId == 'remap'

    // Test values for each dispatch
    for (let d = 0; d < dispatchSize; ++d) {
      // Get values for this dispatch
      const dispatchOffset = d * numInvocations;
      const values = new arrayType([
        wgCopyBufferResult.data[d], // Last 'wg' value for this dispatch
        ...outputBufferResult.data.subarray(dispatchOffset, dispatchOffset + numInvocations), // Rest of the returned values
      ]);

      values.sort();
      t.expectOK(checkElementsEqual(values, expected));
    }
  });
