/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `Test compute shader builtin variables`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { iterRange } from '../../../../common/util/util.js';
import { GPUTest } from '../../../gpu_test.js';

export const g = makeTestGroup(GPUTest);

// Test that the values for each input builtin are correct.
g.test('inputs').
desc(`Test compute shader builtin inputs values`).
params((u) =>
u.
combine('method', ['param', 'struct', 'mixed']).
combine('dispatch', ['direct', 'indirect']).
combineWithParams([
{
  groupSize: { x: 1, y: 1, z: 1 },
  numGroups: { x: 1, y: 1, z: 1 }
},
{
  groupSize: { x: 8, y: 4, z: 2 },
  numGroups: { x: 1, y: 1, z: 1 }
},
{
  groupSize: { x: 1, y: 1, z: 1 },
  numGroups: { x: 8, y: 4, z: 2 }
},
{
  groupSize: { x: 3, y: 7, z: 5 },
  numGroups: { x: 13, y: 9, z: 11 }
}]
).
beginSubcases()
).
fn((t) => {
  const invocationsPerGroup = t.params.groupSize.x * t.params.groupSize.y * t.params.groupSize.z;
  const totalInvocations =
  invocationsPerGroup * t.params.numGroups.x * t.params.numGroups.y * t.params.numGroups.z;

  // Generate the structures, parameters, and builtin expressions used in the shader.
  let params = '';
  let structures = '';
  let local_id = '';
  let local_index = '';
  let global_id = '';
  let group_id = '';
  let num_groups = '';
  switch (t.params.method) {
    case 'param':
      params = `
          @builtin(local_invocation_id) local_id : vec3<u32>,
          @builtin(local_invocation_index) local_index : u32,
          @builtin(global_invocation_id) global_id : vec3<u32>,
          @builtin(workgroup_id) group_id : vec3<u32>,
          @builtin(num_workgroups) num_groups : vec3<u32>,
        `;
      local_id = 'local_id';
      local_index = 'local_index';
      global_id = 'global_id';
      group_id = 'group_id';
      num_groups = 'num_groups';
      break;
    case 'struct':
      structures = `struct Inputs {
            @builtin(local_invocation_id) local_id : vec3<u32>,
            @builtin(local_invocation_index) local_index : u32,
            @builtin(global_invocation_id) global_id : vec3<u32>,
            @builtin(workgroup_id) group_id : vec3<u32>,
            @builtin(num_workgroups) num_groups : vec3<u32>,
          };`;
      params = `inputs : Inputs`;
      local_id = 'inputs.local_id';
      local_index = 'inputs.local_index';
      global_id = 'inputs.global_id';
      group_id = 'inputs.group_id';
      num_groups = 'inputs.num_groups';
      break;
    case 'mixed':
      structures = `struct InputsA {
          @builtin(local_invocation_index) local_index : u32,
          @builtin(global_invocation_id) global_id : vec3<u32>,
        };
        struct InputsB {
          @builtin(workgroup_id) group_id : vec3<u32>
        };`;
      params = `@builtin(local_invocation_id) local_id : vec3<u32>,
                  inputsA : InputsA,
                  inputsB : InputsB,
                  @builtin(num_workgroups) num_groups : vec3<u32>,`;
      local_id = 'local_id';
      local_index = 'inputsA.local_index';
      global_id = 'inputsA.global_id';
      group_id = 'inputsB.group_id';
      num_groups = 'num_groups';
      break;
  }

  // WGSL shader that stores every builtin value to a buffer, for every invocation in the grid.
  const wgsl = `
      struct Outputs {
        local_id: vec3u,
        local_index: u32,
        global_id: vec3u,
        group_id: vec3u,
        num_groups: vec3u,
      };
      @group(0) @binding(0) var<storage, read_write> outputs : array<Outputs>;

      ${structures}

      const group_width = ${t.params.groupSize.x}u;
      const group_height = ${t.params.groupSize.y}u;
      const group_depth = ${t.params.groupSize.z}u;

      @compute @workgroup_size(group_width, group_height, group_depth)
      fn main(
        ${params}
        ) {
        let group_index = ((${group_id}.z * ${num_groups}.y) + ${group_id}.y) * ${num_groups}.x + ${group_id}.x;
        let global_index = group_index * ${invocationsPerGroup}u + ${local_index};
        var o: Outputs;
        o.local_id = ${local_id};
        o.local_index = ${local_index};
        o.global_id = ${global_id};
        o.group_id = ${group_id};
        o.num_groups = ${num_groups};
        outputs[global_index] = o;
      }
    `;

  const pipeline = t.device.createComputePipeline({
    layout: 'auto',
    compute: {
      module: t.device.createShaderModule({
        code: wgsl
      }),
      entryPoint: 'main'
    }
  });

  // Offsets are in u32 size units
  const kLocalIdOffset = 0;
  const kLocalIndexOffset = 3;
  const kGlobalIdOffset = 4;
  const kGroupIdOffset = 8;
  const kNumGroupsOffset = 12;
  const kOutputElementSize = 16;

  // Create the output buffers.
  const outputBuffer = t.createBufferTracked({
    size: totalInvocations * kOutputElementSize * 4,
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
  switch (t.params.dispatch) {
    case 'direct':
      pass.dispatchWorkgroups(t.params.numGroups.x, t.params.numGroups.y, t.params.numGroups.z);
      break;
    case 'indirect':{
        const dispatchBuffer = t.createBufferTracked({
          size: 3 * Uint32Array.BYTES_PER_ELEMENT,
          usage: GPUBufferUsage.INDIRECT,
          mappedAtCreation: true
        });
        const dispatchData = new Uint32Array(dispatchBuffer.getMappedRange());
        dispatchData[0] = t.params.numGroups.x;
        dispatchData[1] = t.params.numGroups.y;
        dispatchData[2] = t.params.numGroups.z;
        dispatchBuffer.unmap();
        pass.dispatchWorkgroupsIndirect(dispatchBuffer, 0);
        break;
      }
  }
  pass.end();
  t.queue.submit([encoder.finish()]);



  // Helper to check that the vec3<u32> value at each index of the provided `output` buffer
  // matches the expected value for that invocation, as generated by the `getBuiltinValue`
  // function. The `name` parameter is the builtin name, used for error messages.
  const checkEachIndex = (output) => {
    // Loop over workgroups.
    for (let gz = 0; gz < t.params.numGroups.z; gz++) {
      for (let gy = 0; gy < t.params.numGroups.y; gy++) {
        for (let gx = 0; gx < t.params.numGroups.x; gx++) {
          // Loop over invocations within a group.
          for (let lz = 0; lz < t.params.groupSize.z; lz++) {
            for (let ly = 0; ly < t.params.groupSize.y; ly++) {
              for (let lx = 0; lx < t.params.groupSize.x; lx++) {
                const groupIndex = (gz * t.params.numGroups.y + gy) * t.params.numGroups.x + gx;
                const localIndex = (lz * t.params.groupSize.y + ly) * t.params.groupSize.x + lx;
                const globalIndex = groupIndex * invocationsPerGroup + localIndex;
                const globalOffset = globalIndex * kOutputElementSize;

                const expectEqual = (name, expected, actual) => {
                  if (actual !== expected) {
                    return new Error(
                      `${name} failed at group(${gx},${gy},${gz}) local(${lx},${ly},${lz}))\n` +
                      `    expected: ${expected}\n` +
                      `    got:      ${actual}`
                    );
                  }
                  return undefined;
                };

                const checkVec3Value = (name, fieldOffset, expected) => {
                  const offset = globalOffset + fieldOffset;
                  return (
                    expectEqual(`${name}.x`, expected.x, output[offset + 0]) ||
                    expectEqual(`${name}.y`, expected.y, output[offset + 1]) ||
                    expectEqual(`${name}.z`, expected.z, output[offset + 2]));

                };

                const error =
                checkVec3Value('local_id', kLocalIdOffset, { x: lx, y: ly, z: lz }) ||
                checkVec3Value('global_id', kGlobalIdOffset, {
                  x: gx * t.params.groupSize.x + lx,
                  y: gy * t.params.groupSize.y + ly,
                  z: gz * t.params.groupSize.z + lz
                }) ||
                checkVec3Value('group_id', kGroupIdOffset, { x: gx, y: gy, z: gz }) ||
                checkVec3Value('num_groups', kNumGroupsOffset, t.params.numGroups) ||
                expectEqual(
                  'local_index',
                  localIndex,
                  output[globalOffset + kLocalIndexOffset]
                );
                if (error) {
                  return error;
                }
              }
            }
          }
        }
      }
    }
    return undefined;
  };

  t.expectGPUBufferValuesPassCheck(outputBuffer, (outputData) => checkEachIndex(outputData), {
    type: Uint32Array,
    typedLength: outputBuffer.size / 4
  });
});

/**
 * @returns The population count of input.
 *
 * @param input Treated as an unsigned 32-bit integer
 */
function popcount(input) {
  let n = input;
  n = n - (n >> 1 & 0x55555555);
  n = (n & 0x33333333) + (n >> 2 & 0x33333333);
  return (n + (n >> 4) & 0xf0f0f0f) * 0x1010101 >> 24;
}

function ErrorMsg(msg, got, expected) {
  return `${msg}:\n-      got: ${got}\n- expected: ${expected}`;
}

/**
 * Checks that the subgroup size and ballot buffers are consistent.
 *
 * This function checks that all invocations see a consistent value for
 * subgroup_size and that all ballots are less than or equal to that value.
 *
 * @param subgroupSizes The subgroup_size buffer
 * @param ballotSizes The ballot buffer
 * @param min The minimum subgroup size allowed
 * @param max The maximum subgroup size allowed
 * @param invocations The number of invocations in a workgroup
 */
function checkSubgroupSizeConsistency(
subgroupSizes,
ballotSizes,
min,
max,
invocations)
{
  const subgroupSize = subgroupSizes[0];
  if (popcount(subgroupSize) !== 1) {
    return new Error(`Subgroup size '${subgroupSize}' is not a power of two`);
  }
  if (subgroupSize < min) {
    return new Error(`Subgroup size '${subgroupSize}' is less than minimum '${min}'`);
  }
  if (max < subgroupSize) {
    return new Error(`Subgroup size '${subgroupSize}' is greater than maximum '${max}'`);
  }

  // Check that remaining invocations record a consistent subgroup size.
  for (let i = 1; i < subgroupSizes.length; i++) {
    if (subgroupSizes[i] !== subgroupSize) {
      return new Error(
        ErrorMsg(`Invocation ${i}: subgroup size inconsistency`, subgroupSizes[i], subgroupSize)
      );
    }
  }

  for (let i = 0; i < ballotSizes.length; i++) {
    if (ballotSizes[i] > subgroupSize) {
      return new Error(
        `Invocation ${i}, subgroup size, ${ballotSizes[i]}, is greater than built-in value, ${subgroupSize}`
      );
    }
  }

  return undefined;
}

/**
 * Returns a WGSL function that generates linear local id
 *
 * Using (0,1,2) generates the standard local linear id.
 * Changing the order of p0, p1, and p2 changes the linearity.
 *
 * Assumes p0, p1, and p2 are not repeated and in range [0, 2].
 * @param p0 The index used for the x-dimension
 * @param p1 The index used for the y-dimension
 * @param p2 The index used for the z-dimension
 * @param sizes An array of workgroup sizes for each dimension.
 */
function genLID(p0, p1, p2, sizes) {
  return `
fn getLID(lid : vec3u) -> u32 {
  let p0 = lid[${p0}];
  let p1 = lid[${p1}] * ${sizes[p0]};
  let p2 = lid[${p2}] * ${sizes[p0]} * ${sizes[p1]};
  return p0 + p1 + p2;
}`;
}

const kWGSizes = [
[1, 1, 1],
[4, 1, 1],
[8, 1, 1],
[16, 1, 1],
[32, 1, 1],
[64, 1, 1],
[128, 1, 1],
[256, 1, 1],
[1, 4, 1],
[1, 8, 1],
[1, 16, 1],
[1, 32, 1],
[1, 64, 1],
[1, 128, 1],
[1, 256, 1],
[1, 1, 4],
[1, 1, 8],
[1, 1, 16],
[1, 1, 32],
[1, 1, 64],
[3, 3, 3],
[4, 4, 4],
[16, 16, 1],
[16, 1, 16],
[1, 16, 16],
[15, 3, 3],
[3, 15, 3],
[3, 3, 15],
[17, 5, 2],
[17, 4, 2],
[15, 2, 8]];


g.test('subgroup_size').
desc('Tests subgroup_size values').
params((u) =>
u.
combine('sizes', kWGSizes).
beginSubcases().
combine('numWGs', [1, 2]).
combine('lid', [
[0, 1, 2],
[0, 2, 1],
[1, 0, 2],
[1, 2, 0],
[2, 0, 1],
[2, 1, 0]]
)
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('subgroups');
}).
fn(async (t) => {




  const { minSubgroupSize, maxSubgroupSize } = t.device.limits;

  const wgx = t.params.sizes[0];
  const wgy = t.params.sizes[1];
  const wgz = t.params.sizes[2];
  const lid = t.params.lid;
  const wgThreads = wgx * wgy * wgz;

  // Compatibility mode has lower workgroup limits.
  const {
    maxComputeInvocationsPerWorkgroup,
    maxComputeWorkgroupSizeX,
    maxComputeWorkgroupSizeY,
    maxComputeWorkgroupSizeZ
  } = t.device.limits;
  t.skipIf(
    maxComputeInvocationsPerWorkgroup < wgThreads ||
    maxComputeWorkgroupSizeX < t.params.sizes[0] ||
    maxComputeWorkgroupSizeY < t.params.sizes[1] ||
    maxComputeWorkgroupSizeZ < t.params.sizes[2],
    'Workgroup size too large'
  );

  const wgsl = `
enable subgroups;

const stride = ${wgThreads};

${genLID(lid[0], lid[1], lid[2], t.params.sizes)}

@group(0) @binding(0)
var<storage, read_write> output : array<u32>;

@group(0) @binding(1)
var<storage, read_write> compare : array<u32>;

@compute @workgroup_size(${wgx}, ${wgy}, ${wgz})
fn main(@builtin(subgroup_size) size : u32,
        @builtin(workgroup_id) wgid : vec3u,
        @builtin(local_invocation_id) local_id : vec3u) {
  // Remap local ids according to test linearity.
  let lid = getLID(local_id);

  output[lid + wgid.x * stride] = size;
  let ballot = countOneBits(subgroupBallot(true));
  let ballotSize = ballot[0] + ballot[1] + ballot[2] + ballot[3];
  compare[lid + wgid.x * stride] = ballotSize;
}`;

  const numInvocations = wgThreads * t.params.numWGs;
  const sizesBuffer = t.makeBufferWithContents(
    new Uint32Array([...iterRange(numInvocations, (x) => 0)]),
    GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  );
  t.trackForCleanup(sizesBuffer);
  const compareBuffer = t.makeBufferWithContents(
    new Uint32Array([...iterRange(numInvocations, (x) => 0)]),
    GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  );
  t.trackForCleanup(compareBuffer);

  const pipeline = t.device.createComputePipeline({
    layout: 'auto',
    compute: {
      module: t.device.createShaderModule({
        code: wgsl
      }),
      entryPoint: 'main'
    }
  });
  const bg = t.device.createBindGroup({
    layout: pipeline.getBindGroupLayout(0),
    entries: [
    {
      binding: 0,
      resource: {
        buffer: sizesBuffer
      }
    },
    {
      binding: 1,
      resource: {
        buffer: compareBuffer
      }
    }]

  });

  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginComputePass();
  pass.setPipeline(pipeline);
  pass.setBindGroup(0, bg);
  pass.dispatchWorkgroups(t.params.numWGs, 1, 1);
  pass.end();
  t.queue.submit([encoder.finish()]);

  const sizesReadback = await t.readGPUBufferRangeTyped(sizesBuffer, {
    srcByteOffset: 0,
    type: Uint32Array,
    typedLength: numInvocations,
    method: 'copy'
  });
  const sizesData = sizesReadback.data;

  const compareReadback = await t.readGPUBufferRangeTyped(compareBuffer, {
    srcByteOffset: 0,
    type: Uint32Array,
    typedLength: numInvocations,
    method: 'copy'
  });
  const compareData = compareReadback.data;

  t.expectOK(
    checkSubgroupSizeConsistency(
      sizesData,
      compareData,
      minSubgroupSize,
      maxSubgroupSize,
      wgThreads
    )
  );
});

/**
 * Checks the consistency of subgroup_invocation_id builtin values
 *
 * Creates a ballot out consisting of all invocations sharing the same generated
 * subgroup id. Checks that the ballot contains at most subgroupSize invocations
 * and that the invocations are tightly packed from the lowest id.
 * @param data The subgroup_invocation_id output data
 * @param ids The representative subgroup ids for each invocation
 * @param subgroupSize The subgroup size
 * @param invocations The number of invocations per workgroup
 * @param numWGs The number of workgroups
 */
function checkSubgroupInvocationIdConsistency(
data,
ids,
subgroupSize,
invocations,
numWGs)
{
  for (let wg = 0; wg < numWGs; wg++) {
    // Tracks an effective ballot of each subgroup based on the representative id
    // (global id of invocation 0 in the subgroup).
    const mappings = new Map();
    for (let i = 0; i < invocations; i++) {
      const idx = i + invocations * wg;
      const subgroup_id = ids[idx];
      if (subgroup_id === 999) {
        return new Error(`Invocation ${i}: no data`);
      }
      let v = mappings.get(subgroup_id) ?? 0n;
      v |= 1n << BigInt(data[idx]);
      mappings.set(subgroup_id, v);
    }

    for (const value of mappings.entries()) {
      const id = value[0];
      const ballot = value[1];

      let onebits = popcount(Number(BigInt.asUintN(32, ballot)));
      onebits += popcount(Number(BigInt.asUintN(32, ballot >> 32n)));
      onebits += popcount(Number(BigInt.asUintN(32, ballot >> 64n)));
      onebits += popcount(Number(BigInt.asUintN(32, ballot >> 96n)));
      if (onebits > subgroupSize) {
        return new Error(
          `Subgroup including invocation ${id} is too large, ${onebits}, for subgroup size, ${subgroupSize}`
        );
      }

      const ballotP1 = ballot + 1n;
      if ((ballot & ballotP1) !== 0n) {
        return new Error(
          `Subgroup including invocation ${id} has non-continguous ids: ${ballot.toString(2)}`
        );
      }
    }
  }

  return undefined;
}

g.test('subgroup_invocation_id').
desc(
  'Tests subgroup_invocation_id values. No mapping between local_invocation_index and subgroup_invocation_id can be relied upon.'
).
params((u) =>
u.
combine('sizes', kWGSizes).
beginSubcases().
combine('numWGs', [1, 2]).
combine('lid', [
[0, 1, 2],
[0, 2, 1],
[1, 0, 2],
[1, 2, 0],
[2, 0, 1],
[2, 1, 0]]
)
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('subgroups');
}).
fn(async (t) => {
  const wgx = t.params.sizes[0];
  const wgy = t.params.sizes[1];
  const wgz = t.params.sizes[2];
  const lid = t.params.lid;
  const wgThreads = wgx * wgy * wgz;

  // Compatibility mode has lower workgroup limits.
  const {
    maxComputeInvocationsPerWorkgroup,
    maxComputeWorkgroupSizeX,
    maxComputeWorkgroupSizeY,
    maxComputeWorkgroupSizeZ
  } = t.device.limits;
  t.skipIf(
    maxComputeInvocationsPerWorkgroup < wgThreads ||
    maxComputeWorkgroupSizeX < wgx ||
    maxComputeWorkgroupSizeY < wgy ||
    maxComputeWorkgroupSizeZ < wgz,
    'Workgroup size too large'
  );

  const wgsl = `
enable subgroups;

const stride = ${wgThreads};

${genLID(lid[0], lid[1], lid[2], t.params.sizes)}

@group(0) @binding(0)
var<storage, read_write> output : array<u32>;

// This var stores the global id of invocation 0 in the subgroup.
@group(0) @binding(1)
var<storage, read_write> subgroup_ids : array<u32>;

@group(0) @binding(2)
var<storage, read_write> sizes : array<u32>;

@compute @workgroup_size(${wgx}, ${wgy}, ${wgz})
fn main(@builtin(subgroup_size) size : u32,
        @builtin(subgroup_invocation_id) id : u32,
        @builtin(workgroup_id) wgid : vec3u,
        @builtin(local_invocation_id) local_id : vec3u) {
  // Remap local ids according to test linearity.
  let lid = getLID(local_id);

  // Representative subgroup_id value.
  let gid = lid + stride * wgid.x;

  let b = subgroupBroadcast(gid, 0);
  output[gid] = id;
  subgroup_ids[gid] = b;
  if (lid == 0) {
    sizes[wgid.x] = size;
  }
}`;

  const numInvocations = wgThreads * t.params.numWGs;
  const outputBuffer = t.makeBufferWithContents(
    new Uint32Array([...iterRange(numInvocations, (x) => 0)]),
    GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  );
  t.trackForCleanup(outputBuffer);
  const placeholderValue = 999;
  const idsBuffer = t.makeBufferWithContents(
    new Uint32Array([...iterRange(numInvocations, (x) => placeholderValue)]),
    GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  );
  t.trackForCleanup(idsBuffer);
  const sizeBuffer = t.makeBufferWithContents(
    new Uint32Array([...iterRange(t.params.numWGs, (x) => 0)]),
    GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  );
  t.trackForCleanup(sizeBuffer);

  const pipeline = t.device.createComputePipeline({
    layout: 'auto',
    compute: {
      module: t.device.createShaderModule({
        code: wgsl
      }),
      entryPoint: 'main'
    }
  });
  const bg = t.device.createBindGroup({
    layout: pipeline.getBindGroupLayout(0),
    entries: [
    {
      binding: 0,
      resource: {
        buffer: outputBuffer
      }
    },
    {
      binding: 1,
      resource: {
        buffer: idsBuffer
      }
    },
    {
      binding: 2,
      resource: {
        buffer: sizeBuffer
      }
    }]

  });

  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginComputePass();
  pass.setPipeline(pipeline);
  pass.setBindGroup(0, bg);
  pass.dispatchWorkgroups(t.params.numWGs, 1, 1);
  pass.end();
  t.queue.submit([encoder.finish()]);

  const sizeReadback = await t.readGPUBufferRangeTyped(sizeBuffer, {
    srcByteOffset: 0,
    type: Uint32Array,
    typedLength: 1,
    method: 'copy'
  });
  const sizeData = sizeReadback.data;

  const outputReadback = await t.readGPUBufferRangeTyped(outputBuffer, {
    srcByteOffset: 0,
    type: Uint32Array,
    typedLength: numInvocations,
    method: 'copy'
  });
  const outputData = outputReadback.data;

  const idsReadback = await t.readGPUBufferRangeTyped(idsBuffer, {
    srcByteOffset: 0,
    type: Uint32Array,
    typedLength: numInvocations,
    method: 'copy'
  });
  const idsData = idsReadback.data;

  t.expectOK(
    checkSubgroupInvocationIdConsistency(
      outputData,
      idsData,
      sizeData[0],
      wgThreads,
      t.params.numWGs
    )
  );
});