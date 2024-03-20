/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Performs the following steps atomically:
 * Load the original value pointed to by atomic_ptr.
 * Compare the original value to the value v using an equality operation.
 * Store the value v only if the result of the equality comparison was true.

Returns a two member structure, where the first member, old_value, is the original
value of the atomic object and the second member, exchanged, is whether or not
the comparison succeeded.

Note: the equality comparison may spuriously fail on some implementations.
That is, the second component of the result vector may be false even if the first
component of the result vector equals cmp.
`;import { makeTestGroup } from '../../../../../../../common/framework/test_group.js';
import { keysOf } from '../../../../../../../common/util/data_tables.js';
import { assert } from '../../../../../../../common/util/util.js';
import { GPUTest } from '../../../../../../gpu_test.js';

import {
  dispatchSizes,
  workgroupSizes,
  typedArrayCtor,
  kMapId,
  onlyWorkgroupSizes } from
'./harness.js';

export const g = makeTestGroup(GPUTest);

g.test('compare_exchange_weak_storage_basic').
specURL('https://www.w3.org/TR/WGSL/#atomic-rmw').
desc(
  `
AS is storage or workgroup
T is i32 or u32

fn atomicCompareExchangeWeak(atomic_ptr: ptr<AS, atomic<T>, read_write>, cmp: T, v: T) -> __atomic_compare_exchange_result<T>

struct __atomic_compare_exchange_result<T> {
  old_value : T,    // old value stored in the atomic
  exchanged : bool, // true if the exchange was done
}
`
).
params((u) =>
u.
combine('workgroupSize', workgroupSizes).
combine('dispatchSize', dispatchSizes).
combine('mapId', keysOf(kMapId)).
combine('scalarType', ['u32', 'i32'])
).
fn(async (t) => {
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

      @group(0) @binding(2)
      var<storage, read_write> exchanged : array<${scalarType}>;

      @compute @workgroup_size(${t.params.workgroupSize})
      fn main(
          @builtin(global_invocation_id) global_invocation_id : vec3<u32>,
          ) {
        let id = ${scalarType}(global_invocation_id[0]);

        // Exchange every third value
        var comp = id + 1;
        if (id % 3 == 0) {
          comp = id;
        }
        let r = atomicCompareExchangeWeak(&input[id], comp, map_id(id * 2));

        // Store results
            output[id] = r.old_value;
        if (r.exchanged) {
          exchanged[id] = 1;
        } else {
          exchanged[id] = 0;
        }
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

  // Create input buffer with values [0..n]
  const inputBuffer = t.device.createBuffer({
    size: bufferNumElements * arrayType.BYTES_PER_ELEMENT,
    usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC,
    mappedAtCreation: true
  });
  t.trackForCleanup(inputBuffer);
  const data = new arrayType(inputBuffer.getMappedRange());
  data.forEach((_, i) => data[i] = i);
  inputBuffer.unmap();

  const outputBuffer = t.device.createBuffer({
    size: bufferNumElements * arrayType.BYTES_PER_ELEMENT,
    usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC
  });
  t.trackForCleanup(outputBuffer);

  const exchangedBuffer = t.device.createBuffer({
    size: bufferNumElements * arrayType.BYTES_PER_ELEMENT,
    usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC
  });
  t.trackForCleanup(exchangedBuffer);

  const bindGroup = t.device.createBindGroup({
    layout: pipeline.getBindGroupLayout(0),
    entries: [
    { binding: 0, resource: { buffer: inputBuffer } },
    { binding: 1, resource: { buffer: outputBuffer } },
    { binding: 2, resource: { buffer: exchangedBuffer } }]

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
  // values returned from atomicCompareExchangeWeak
  const outputExpected = new (typedArrayCtor(t.params.scalarType))(bufferNumElements);
  outputExpected.forEach((_, i) => outputExpected[i] = i);
  t.expectGPUBufferValuesEqual(outputBuffer, outputExpected);

  // Read back exchanged buffer
  const exchangedBufferResult = await t.readGPUBufferRangeTyped(exchangedBuffer, {
    type: arrayType,
    typedLength: exchangedBuffer.size / arrayType.BYTES_PER_ELEMENT
  });

  // The input buffer should have been modified to a computed value for every third value,
  // unless the comparison spuriously failed.
  const inputExpected = new (typedArrayCtor(t.params.scalarType))(bufferNumElements);
  inputExpected.forEach((_, i) => {
    if (i % 3 === 0 && exchangedBufferResult.data[i]) {
      inputExpected[i] = mapId.f(i * 2, numInvocations);
    } else {
      inputExpected[i] = i; // No change
    }
  });
  t.expectGPUBufferValuesEqual(inputBuffer, inputExpected);
});

g.test('compare_exchange_weak_workgroup_basic').
specURL('https://www.w3.org/TR/WGSL/#atomic-rmw').
desc(
  `
AS is storage or workgroup
T is i32 or u32

fn atomicCompareExchangeWeak(atomic_ptr: ptr<AS, atomic<T>, read_write>, cmp: T, v: T) -> __atomic_compare_exchange_result<T>

struct __atomic_compare_exchange_result<T> {
  old_value : T,    // old value stored in the atomic
  exchanged : bool, // true if the exchange was done
}
`
).
params((u) =>
u.
combine('workgroupSize', workgroupSizes).
combine('dispatchSize', dispatchSizes).
combine('mapId', keysOf(kMapId)).
combine('scalarType', ['u32', 'i32'])
).
fn(async (t) => {
  const numInvocations = t.params.workgroupSize;
  const wgNumElements = numInvocations;
  const scalarType = t.params.scalarType;
  const dispatchSize = t.params.dispatchSize;
  const mapId = kMapId[t.params.mapId];
  const extra = mapId.wgsl(numInvocations, t.params.scalarType); // Defines map_id()

  const wgsl =
  `
      var<workgroup> wg: array<atomic<${scalarType}>, ${wgNumElements}>;

      @group(0) @binding(0)
      var<storage, read_write> output: array<${scalarType}, ${wgNumElements * dispatchSize}>;

      @group(0) @binding(1)
      var<storage, read_write> exchanged: array<${scalarType}, ${wgNumElements * dispatchSize}>;

      // Result of each workgroup is written to output[workgroup_id.x]
      @group(0) @binding(2)
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

        // Exchange every third value
        var comp = global_id + 1;
        if (global_id % 3 == 0) {
          comp = global_id;
        }
        let r = atomicCompareExchangeWeak(&wg[id], comp, map_id(global_id * 2));

        // Store results
        output[global_id] = r.old_value;
        if (r.exchanged) {
          exchanged[global_id] = 1;
        } else {
          exchanged[global_id] = 0;
        }

        // Copy new value into wg_copy
        wg_copy[global_id] = atomicLoad(&wg[id]);
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

  const outputBuffer = t.device.createBuffer({
    size: wgNumElements * dispatchSize * arrayType.BYTES_PER_ELEMENT,
    usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC
  });
  t.trackForCleanup(outputBuffer);

  const wgCopyBuffer = t.device.createBuffer({
    size: wgNumElements * dispatchSize * arrayType.BYTES_PER_ELEMENT,
    usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC
  });
  t.trackForCleanup(outputBuffer);

  const exchangedBuffer = t.device.createBuffer({
    size: wgNumElements * dispatchSize * arrayType.BYTES_PER_ELEMENT,
    usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC
  });
  t.trackForCleanup(exchangedBuffer);

  const bindGroup = t.device.createBindGroup({
    layout: pipeline.getBindGroupLayout(0),
    entries: [
    { binding: 0, resource: { buffer: outputBuffer } },
    { binding: 1, resource: { buffer: exchangedBuffer } },
    { binding: 2, resource: { buffer: wgCopyBuffer } }]

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
  // values returned from atomicCompareExchangeWeak
  const outputExpected = new (typedArrayCtor(t.params.scalarType))(wgNumElements * dispatchSize);
  outputExpected.forEach((_, i) => outputExpected[i] = i);
  t.expectGPUBufferValuesEqual(outputBuffer, outputExpected);

  // Read back exchanged buffer
  const exchangedBufferResult = await t.readGPUBufferRangeTyped(exchangedBuffer, {
    type: arrayType,
    typedLength: exchangedBuffer.size / arrayType.BYTES_PER_ELEMENT
  });

  // And the wg copy buffer should have been modified to a computed value for every third value,
  // unless the comparison spuriously failed.
  const wgCopyBufferExpected = new (typedArrayCtor(t.params.scalarType))(
    wgNumElements * dispatchSize
  );
  wgCopyBufferExpected.forEach((_, i) => {
    if (i % 3 === 0 && exchangedBufferResult.data[i]) {
      wgCopyBufferExpected[i] = mapId.f(i * 2, numInvocations);
    } else {
      wgCopyBufferExpected[i] = i; // No change
    }
  });
  t.expectGPUBufferValuesEqual(wgCopyBuffer, wgCopyBufferExpected);
});

g.test('compare_exchange_weak_storage_advanced').
specURL('https://www.w3.org/TR/WGSL/#atomic-rmw').
desc(
  `
AS is storage or workgroup
T is i32 or u32

fn atomicCompareExchangeWeak(atomic_ptr: ptr<AS, atomic<T>, read_write>, cmp: T, v: T) -> __atomic_compare_exchange_result<T>

struct __atomic_compare_exchange_result<T> {
  old_value : T,    // old value stored in the atomic
  exchanged : bool, // true if the exchange was done
}
`
).
params((u) =>
u.
combine('workgroupSize', onlyWorkgroupSizes) //
.combine('scalarType', ['u32', 'i32'])
).
fn(async (t) => {
  const numInvocations = t.params.workgroupSize;
  const scalarType = t.params.scalarType;

  t.skipIf(
    numInvocations > t.device.limits.maxComputeWorkgroupSizeX,
    `${numInvocations} > maxComputeWorkgroupSizeX(${t.device.limits.maxComputeWorkgroupSizeX})`
  );

  // Number of times each workgroup attempts to exchange the same value to the same memory address
  const numWrites = 4;

  const bufferNumElements = numInvocations * numWrites;
  const pingPongValues = [24, 68];

  const wgsl = `
      @group(0) @binding(0)
      var<storage, read_write> data : atomic<${scalarType}>;

      @group(0) @binding(1)
      var<storage, read_write> old_values : array<${scalarType}>;

      @group(0) @binding(2)
      var<storage, read_write> exchanged : array<${scalarType}>;

      fn ping_pong_value(i: u32) -> ${scalarType} {
        if (i % 2 == 0) {
          return ${pingPongValues[0]};
        } else {
          return ${pingPongValues[1]};
        }
      }

      @compute @workgroup_size(${t.params.workgroupSize})
      fn main(
          @builtin(global_invocation_id) global_invocation_id : vec3<u32>,
          ) {
        let id = ${scalarType}(global_invocation_id[0]);

        // Each invocation attempts to write an alternating (ping-pong) value, once per loop.
        // The data value is initialized with the first of the two ping-pong values.
        // Only one invocation per loop iteration should succeed. Note the workgroupBarrier() used
        // to synchronize each invocation in the loop.
        // The reason we alternate is in case atomicCompareExchangeWeak spurioulsy fails:
        // If all invocations of one iteration spuriously fail, the very next iteration will also
        // fail since the value will not have been exchanged; however, the subsequent one will succeed
        // (assuming not all iterations spuriously fail yet again).

        for (var i = 0u; i < ${numWrites}u; i++) {
          let compare = ping_pong_value(i);
          let next = ping_pong_value(i + 1);

          let r = atomicCompareExchangeWeak(&data, compare, next);

          let slot = i * ${numInvocations}u + u32(id);
          old_values[slot] = r.old_value;
          if (r.exchanged) {
            exchanged[slot] = 1;
          } else {
            exchanged[slot] = 0;
          }

          workgroupBarrier();
        }
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
  const defaultValue = 99999999;

  // Create single-value data buffer initialized to the first ping-pong value
  const dataBuffer = t.device.createBuffer({
    size: 1 * arrayType.BYTES_PER_ELEMENT,
    usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC,
    mappedAtCreation: true
  });
  {
    const data = new arrayType(dataBuffer.getMappedRange());
    data[0] = pingPongValues[0];
    dataBuffer.unmap();
  }
  t.trackForCleanup(dataBuffer);

  const oldValuesBuffer = t.device.createBuffer({
    size: bufferNumElements * arrayType.BYTES_PER_ELEMENT,
    usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC,
    mappedAtCreation: true
  });
  t.trackForCleanup(oldValuesBuffer);
  {
    const data = new arrayType(oldValuesBuffer.getMappedRange());
    data.fill(defaultValue);
    oldValuesBuffer.unmap();
  }

  const exchangedBuffer = t.device.createBuffer({
    size: bufferNumElements * arrayType.BYTES_PER_ELEMENT,
    usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC,
    mappedAtCreation: true
  });
  t.trackForCleanup(exchangedBuffer);
  {
    const data = new arrayType(exchangedBuffer.getMappedRange());
    data.fill(defaultValue);
    exchangedBuffer.unmap();
  }

  const bindGroup = t.device.createBindGroup({
    layout: pipeline.getBindGroupLayout(0),
    entries: [
    { binding: 0, resource: { buffer: dataBuffer } },
    { binding: 1, resource: { buffer: oldValuesBuffer } },
    { binding: 2, resource: { buffer: exchangedBuffer } }]

  });

  // Run the shader.
  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginComputePass();
  pass.setPipeline(pipeline);
  pass.setBindGroup(0, bindGroup);
  pass.dispatchWorkgroups(1);
  pass.end();
  t.queue.submit([encoder.finish()]);

  // Read back buffers
  const oldValuesBufferResult = (
  await t.readGPUBufferRangeTyped(oldValuesBuffer, {
    type: arrayType,
    typedLength: oldValuesBuffer.size / arrayType.BYTES_PER_ELEMENT
  })).
  data;
  const exchangedBufferResult = (
  await t.readGPUBufferRangeTyped(exchangedBuffer, {
    type: arrayType,
    typedLength: exchangedBuffer.size / arrayType.BYTES_PER_ELEMENT
  })).
  data;

  for (let w = 0; w < numWrites; ++w) {
    const offset = w * numInvocations;
    const exchanged = exchangedBufferResult.subarray(offset, offset + numInvocations);
    const oldValues = oldValuesBufferResult.subarray(offset, offset + numInvocations);

    const dumpValues = () => {
      return `
        For write: ${w}
        exchanged: ${exchanged}
        oldValues: ${oldValues}`;
    };

    // Only one of the invocations should have succeeded to exchange - or none if spurious failures occured
    const noExchanges = exchanged.every((v) => v === 0);
    if (noExchanges) {
      // Spurious failure, all values in oldValues should be the default value
      if (!oldValues.every((v) => v === defaultValue)) {
        t.fail(
          `Spurious failure detected, expected only default value of ${defaultValue} in oldValues buffer.${dumpValues()}`
        );
        return;
      }
    } else {
      // Only one invocation should have exchanged its value
      if (exchanged.filter((v) => v === 1).length !== 1) {
        t.fail(`More than one invocation exchanged its value.${dumpValues()}`);
        return;
      }

      // Get its index
      const idx = exchanged.findIndex((v) => v === 1);
      assert(idx !== -1);

      // Its output should contain the old value after exchange
      const oldValue = pingPongValues[w % 2];
      if (oldValues[idx] !== oldValue) {
        t.fail(
          `oldValues[${idx}] expected to contain old value from exchange: ${oldValue}.${dumpValues()}'`
        );
        return;
      }

      // The rest of oldValues should either contain the old value or the newly exchanged value,
      // depending on whether they executed atomicCompareExchangWeak before or after invocation 'idx'.
      const oldValuesRest = oldValues.filter((_, i) => i !== idx);
      if (!oldValuesRest.every((v) => pingPongValues.includes(v))) {
        t.fail(
          `Values in oldValues buffer should be one of '${pingPongValues}', except at index '${idx} where it is '${oldValue}'.${dumpValues()}`
        );
        return;
      }
    }
  }
});

g.test('compare_exchange_weak_workgroup_advanced').
specURL('https://www.w3.org/TR/WGSL/#atomic-rmw').
desc(
  `
AS is storage or workgroup
T is i32 or u32

fn atomicCompareExchangeWeak(atomic_ptr: ptr<AS, atomic<T>, read_write>, cmp: T, v: T) -> __atomic_compare_exchange_result<T>

struct __atomic_compare_exchange_result<T> {
  old_value : T,    // old value stored in the atomic
  exchanged : bool, // true if the exchange was done
}
`
).
params((u) =>
u.
combine('workgroupSize', onlyWorkgroupSizes) //
.combine('scalarType', ['u32', 'i32'])
).
fn(async (t) => {
  const numInvocations = t.params.workgroupSize;
  const scalarType = t.params.scalarType;

  t.skipIf(
    numInvocations > t.device.limits.maxComputeWorkgroupSizeX,
    `${numInvocations} > maxComputeWorkgroupSizeX(${t.device.limits.maxComputeWorkgroupSizeX})`
  );

  // Number of times each workgroup attempts to exchange the same value to the same memory address
  const numWrites = 4;

  const bufferNumElements = numInvocations * numWrites;
  const pingPongValues = [24, 68];

  const wgsl = `
      var<workgroup> wg: atomic<${scalarType}>;

      @group(0) @binding(0)
      var<storage, read_write> old_values : array<${scalarType}>;

      @group(0) @binding(1)
      var<storage, read_write> exchanged : array<${scalarType}>;

      fn ping_pong_value(i: u32) -> ${scalarType} {
        if (i % 2 == 0) {
          return ${pingPongValues[0]};
        } else {
          return ${pingPongValues[1]};
        }
      }

      @compute @workgroup_size(${t.params.workgroupSize})
      fn main(
        @builtin(local_invocation_index) local_invocation_index: u32,
        @builtin(workgroup_id) workgroup_id : vec3<u32>
        ) {
          let id = ${scalarType}(local_invocation_index);

        // Each invocation attempts to write an alternating (ping-pong) value, once per loop.
        // The input value is initialized with the first of the two ping-pong values.
        // Only one invocation per loop iteration should succeed. Note the workgroupBarrier() used
        // to synchronize each invocation in the loop.
        // The reason we alternate is in case atomicCompareExchangeWeak spurioulsy fails:
        // If all invocations of one iteration spuriously fail, the very next iteration will also
        // fail since the value will not have been exchanged; however, the subsequent one will succeed
        // (assuming not all iterations spuriously fail yet again).

        // Initialize wg
        if (local_invocation_index == 0) {
          atomicStore(&wg, ${pingPongValues[0]});
        }
        workgroupBarrier();

        for (var i = 0u; i < ${numWrites}u; i++) {
          let compare = ping_pong_value(i);
          let next = ping_pong_value(i + 1);

          let r = atomicCompareExchangeWeak(&wg, compare, next);

          let slot = i * ${numInvocations}u + u32(id);
          old_values[slot] = r.old_value;
          if (r.exchanged) {
            exchanged[slot] = 1;
          } else {
            exchanged[slot] = 0;
          }

          workgroupBarrier();
        }
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
  const defaultValue = 99999999;

  const oldValuesBuffer = t.device.createBuffer({
    size: bufferNumElements * arrayType.BYTES_PER_ELEMENT,
    usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC,
    mappedAtCreation: true
  });
  t.trackForCleanup(oldValuesBuffer);
  {
    const data = new arrayType(oldValuesBuffer.getMappedRange());
    data.fill(defaultValue);
    oldValuesBuffer.unmap();
  }

  const exchangedBuffer = t.device.createBuffer({
    size: bufferNumElements * arrayType.BYTES_PER_ELEMENT,
    usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC,
    mappedAtCreation: true
  });
  t.trackForCleanup(exchangedBuffer);
  {
    const data = new arrayType(exchangedBuffer.getMappedRange());
    data.fill(defaultValue);
    exchangedBuffer.unmap();
  }

  const bindGroup = t.device.createBindGroup({
    layout: pipeline.getBindGroupLayout(0),
    entries: [
    { binding: 0, resource: { buffer: oldValuesBuffer } },
    { binding: 1, resource: { buffer: exchangedBuffer } }]

  });

  // Run the shader.
  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginComputePass();
  pass.setPipeline(pipeline);
  pass.setBindGroup(0, bindGroup);
  pass.dispatchWorkgroups(1);
  pass.end();
  t.queue.submit([encoder.finish()]);

  // Read back buffers
  const oldValuesBufferResult = (
  await t.readGPUBufferRangeTyped(oldValuesBuffer, {
    type: arrayType,
    typedLength: oldValuesBuffer.size / arrayType.BYTES_PER_ELEMENT
  })).
  data;
  const exchangedBufferResult = (
  await t.readGPUBufferRangeTyped(exchangedBuffer, {
    type: arrayType,
    typedLength: exchangedBuffer.size / arrayType.BYTES_PER_ELEMENT
  })).
  data;

  for (let w = 0; w < numWrites; ++w) {
    const offset = w * numInvocations;
    const exchanged = exchangedBufferResult.subarray(offset, offset + numInvocations);
    const oldValues = oldValuesBufferResult.subarray(offset, offset + numInvocations);

    const dumpValues = () => {
      return `
        For write: ${w}
        exchanged: ${exchanged}
        oldValues: ${oldValues}`;
    };

    // Only one of the invocations should have succeeded to exchange - or none if spurious failures occured
    const noExchanges = exchanged.every((v) => v === 0);
    if (noExchanges) {
      // Spurious failure, all values in oldValues should be the default value
      if (!oldValues.every((v) => v === defaultValue)) {
        t.fail(
          `Spurious failure detected, expected only default value of ${defaultValue} in oldValues buffer.${dumpValues()}`
        );
        return;
      }
    } else {
      // Only one invocation should have exchanged its value
      if (exchanged.filter((v) => v === 1).length !== 1) {
        t.fail(`More than one invocation exchanged its value.${dumpValues()}`);
        return;
      }

      // Get its index
      const idx = exchanged.findIndex((v) => v === 1);
      assert(idx !== -1);

      // Its output should contain the old value after exchange
      const oldValue = pingPongValues[w % 2];
      if (oldValues[idx] !== oldValue) {
        t.fail(
          `oldValues[${idx}] expected to contain old value from exchange: ${oldValue}.${dumpValues()}'`
        );
        return;
      }

      // The rest of oldValues should either contain the old value or the newly exchanged value,
      // depending on whether they executed atomicCompareExchangWeak before or after invocation 'idx'.
      const oldValuesRest = oldValues.filter((_, i) => i !== idx);
      if (!oldValuesRest.every((v) => pingPongValues.includes(v))) {
        t.fail(
          `Values in oldValues buffer should be one of '${pingPongValues}', except at index '${idx} where it is '${oldValue}'.${dumpValues()}`
        );
        return;
      }
    }
  }
});