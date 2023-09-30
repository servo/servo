/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Atomically stores the value v in the atomic object pointed to by atomic_ptr.
`;
import { makeTestGroup } from '../../../../../../../common/framework/test_group.js';
import { keysOf } from '../../../../../../../common/util/data_tables.js';
import { GPUTest } from '../../../../../../gpu_test.js';

import {
  dispatchSizes,
  workgroupSizes,
  runStorageVariableTest,
  runWorkgroupVariableTest,
  typedArrayCtor,
  kMapId,
} from './harness.js';

export const g = makeTestGroup(GPUTest);

g.test('store_storage_basic')
  .specURL('https://www.w3.org/TR/WGSL/#atomic-store')
  .desc(
    `
AS is storage or workgroup
T is i32 or u32

fn atomicStore(atomic_ptr: ptr<AS, atomic<T>, read_write>, v: T)
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
    const mapId = kMapId[t.params.mapId];
    const extra = mapId.wgsl(numInvocations, t.params.scalarType); // Defines map_id()

    const initValue = 0;
    const op = `atomicStore(&output[id], map_id(id))`;
    const expected = new (typedArrayCtor(t.params.scalarType))(bufferNumElements);
    expected.forEach((_, i) => (expected[i] = mapId.f(i, numInvocations)));

    runStorageVariableTest({
      t,
      workgroupSize: t.params.workgroupSize,
      dispatchSize: t.params.dispatchSize,
      bufferNumElements,
      initValue,
      op,
      expected,
      extra,
    });
  });

g.test('store_workgroup_basic')
  .specURL('https://www.w3.org/TR/WGSL/#atomic-store')
  .desc(
    `
AS is storage or workgroup
T is i32 or u32

fn atomicStore(atomic_ptr: ptr<AS, atomic<T>, read_write>, v: T)
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
    const mapId = kMapId[t.params.mapId];
    const extra = mapId.wgsl(numInvocations, t.params.scalarType); // Defines map_id()

    const initValue = 0;
    const op = `atomicStore(&wg[id], map_id(global_id))`;
    const expected = new (typedArrayCtor(t.params.scalarType))(
      wgNumElements * t.params.dispatchSize
    );

    expected.forEach((_, i) => (expected[i] = mapId.f(i, numInvocations)));

    runWorkgroupVariableTest({
      t,
      workgroupSize: t.params.workgroupSize,
      dispatchSize: t.params.dispatchSize,
      wgNumElements,
      initValue,
      op,
      expected,
      extra,
    });
  });

g.test('store_storage_advanced')
  .specURL('https://www.w3.org/TR/WGSL/#atomic-store')
  .desc(
    `
AS is storage or workgroup
T is i32 or u32

fn atomicStore(atomic_ptr: ptr<AS, atomic<T>, read_write>, v: T)

Tests that multiple invocations of atomicStore to the same location returns
one of the values written.
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
    const scalarType = t.params.scalarType;
    const mapId = kMapId[t.params.mapId];
    const extra = mapId.wgsl(numInvocations, t.params.scalarType); // Defines map_id()

    const wgsl =
      `
      @group(0) @binding(0)
      var<storage, read_write> output : array<atomic<${scalarType}>>;

      @compute @workgroup_size(${t.params.workgroupSize})
      fn main(
          @builtin(global_invocation_id) global_invocation_id : vec3<u32>,
          ) {
        let id = ${scalarType}(global_invocation_id[0]);

        // All invocations store to the same location
        atomicStore(&output[0], map_id(id));
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

    // Output buffer has only 1 element
    const outputBuffer = t.device.createBuffer({
      size: 1 * arrayType.BYTES_PER_ELEMENT,
      usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC,
    });
    t.trackForCleanup(outputBuffer);

    const bindGroup = t.device.createBindGroup({
      layout: pipeline.getBindGroupLayout(0),
      entries: [{ binding: 0, resource: { buffer: outputBuffer } }],
    });

    // Run the shader.
    const encoder = t.device.createCommandEncoder();
    const pass = encoder.beginComputePass();
    pass.setPipeline(pipeline);
    pass.setBindGroup(0, bindGroup);
    pass.dispatchWorkgroups(t.params.dispatchSize);
    pass.end();
    t.queue.submit([encoder.finish()]);

    // Read back the buffer
    const outputBufferResult = (
      await t.readGPUBufferRangeTyped(outputBuffer, {
        type: arrayType,
        typedLength: outputBuffer.size / arrayType.BYTES_PER_ELEMENT,
      })
    ).data;

    // All invocations wrote to the output[0], so validate that it contains one
    // of the possible computed values.
    const expected_one_of = new arrayType(numInvocations);
    expected_one_of.forEach((_, i) => (expected_one_of[i] = mapId.f(i, numInvocations)));

    if (!expected_one_of.includes(outputBufferResult[0])) {
      t.fail(
        `Unexpected value in output[0]: '${outputBufferResult[0]}, expected value to be one of: ${expected_one_of}`
      );
    }
  });

g.test('store_workgroup_advanced')
  .specURL('https://www.w3.org/TR/WGSL/#atomic-store')
  .desc(
    `
AS is storage or workgroup
T is i32 or u32

fn atomicStore(atomic_ptr: ptr<AS, atomic<T>, read_write>, v: T)

Tests that multiple invocations of atomicStore to the same location returns
one of the values written.
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

      // Result of each workgroup is written to output[workgroup_id.x]
      @group(0) @binding(0)
      var<storage, read_write> output: array<${scalarType}, ${dispatchSize}>;

      @compute @workgroup_size(${t.params.workgroupSize})
      fn main(
          @builtin(local_invocation_index) local_invocation_index: u32,
          @builtin(workgroup_id) workgroup_id : vec3<u32>
          ) {
        let id = ${scalarType}(local_invocation_index);

        // All invocations of a given dispatch store to the same location.
        // In the end, the final value should be randomly equal to one of the ids.
        atomicStore(&wg, map_id(id));

        // Once all invocations have completed, the first one copies the result
        // to output for this dispatch (workgroup_id.x)
        workgroupBarrier();
        if (local_invocation_index == 0u) {
          output[workgroup_id.x] = atomicLoad(&wg);
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
      size: dispatchSize * arrayType.BYTES_PER_ELEMENT,
      usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC,
    });
    t.trackForCleanup(outputBuffer);

    const bindGroup = t.device.createBindGroup({
      layout: pipeline.getBindGroupLayout(0),
      entries: [{ binding: 0, resource: { buffer: outputBuffer } }],
    });

    // Run the shader.
    const encoder = t.device.createCommandEncoder();
    const pass = encoder.beginComputePass();
    pass.setPipeline(pipeline);
    pass.setBindGroup(0, bindGroup);
    pass.dispatchWorkgroups(dispatchSize);
    pass.end();
    t.queue.submit([encoder.finish()]);

    // Read back the buffer
    const outputBufferResult = (
      await t.readGPUBufferRangeTyped(outputBuffer, {
        type: arrayType,
        typedLength: outputBuffer.size / arrayType.BYTES_PER_ELEMENT,
      })
    ).data;

    // Each dispatch wrote to a single atomic workgroup var that was copied
    // to outputBuffer[dispatch]. Validate that each value in the output buffer
    // is one of the possible computed values.
    const expected_one_of = new arrayType(numInvocations);
    expected_one_of.forEach((_, i) => (expected_one_of[i] = mapId.f(i, numInvocations)));

    for (let d = 0; d < dispatchSize; d++) {
      if (!expected_one_of.includes(outputBufferResult[d])) {
        t.fail(
          `Unexpected value in output[d] for dispatch d '${d}': '${outputBufferResult[d]}', expected value to be one of: ${expected_one_of}`
        );
      }
    }
  });
