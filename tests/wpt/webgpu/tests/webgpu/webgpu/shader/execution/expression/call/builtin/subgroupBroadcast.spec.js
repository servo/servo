/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution tests for subgroupBroadcast and subgroupBroadcastFirst

Note: There is a lack of portability for non-uniform execution so these tests
restrict themselves to uniform control flow.
Note: There is no guaranteed mapping between subgroup_invocation_id and
local_invocation_index. Tests should avoid assuming there is.
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { keysOf, objectsToRecord } from '../../../../../../common/util/data_tables.js';
import { iterRange } from '../../../../../../common/util/util.js';
import {
  kConcreteNumericScalarsAndVectors,

  VectorType } from
'../../../../../util/conversion.js';

import {
  kWGSizes,
  runComputeTest,
  kDataSentinel,
  generateTypedInputs,
  kPredicateCases,
  runFragmentTest,
  getUintsPerFramebuffer,
  SubgroupTest,
  kFramebufferSizes } from
'./subgroup_util.js';

export const g = makeTestGroup(SubgroupTest);

const kDataTypes = objectsToRecord(kConcreteNumericScalarsAndVectors);

/**
 * Checks the results of the data types test
 *
 * The outputs for a given index are expected to match the input values
 * for the given broadcast.
 * @param metadata An unused parameter
 * @param output The output data
 * @param id The broadcast id
 * @param type The data type
 */
function checkDataTypes(
metadata,
output,
input,
id,
type)
{
  if (type.requiresF16() && !(type instanceof VectorType)) {
    for (let i = 0; i < 4; i++) {
      const expectIdx = Math.floor(id / 2);
      const expectShift = id % 2 === 1;
      let expect = input[expectIdx];
      if (expectShift) {
        expect >>= 16;
      }
      expect &= 0xffff;

      const resIdx = Math.floor(i / 2);
      const resShift = i % 2 === 1;
      let res = output[resIdx];
      if (resShift) {
        res >>= 16;
      }
      res &= 0xffff;

      if (res !== expect) {
        return new Error(`${i}: incorrect result
- expected: ${expect}
-      got: ${res}`);
      }
    }
  } else {
    let uints = 1;
    if (type instanceof VectorType) {
      uints = type.width === 3 ? 4 : type.width;
      if (type.requiresF16()) {
        uints = Math.floor(uints / 2);
      }
    }
    for (let i = 0; i < 4; i++) {
      for (let j = 0; j < uints; j++) {
        const expect = input[id * uints + j];
        const res = output[i * uints + j];
        if (res !== expect) {
          return new Error(`${uints * i + j}: incorrect result
- expected: ${expect}
-      got: ${res}`);
        }
      }
    }
  }

  return undefined;
}

g.test('data_types').
desc('Tests broadcast of data types').
params((u) =>
u.
combine('type', keysOf(kDataTypes)).
beginSubcases().
combine('id', [0, 1, 2, 3])
).
fn(async (t) => {
  const wgSize = [4, 1, 1];
  const type = kDataTypes[t.params.type];
  t.skipIfDeviceDoesNotHaveFeature('subgroups');
  if (type.requiresF16()) {
    t.skipIfDeviceDoesNotHaveFeature('shader-f16');
  }
  let enables = 'enable subgroups;\n';
  if (type.requiresF16()) {
    enables += 'enable f16;\n';
  }

  const broadcast =
  t.params.id === 0 ?
  `subgroupBroadcastFirst(input[id])` :
  `subgroupBroadcast(input[id], ${t.params.id})`;

  const wgsl = `
${enables}

@group(0) @binding(0)
var<storage, read_write> input : array<${type.toString()}>;

@group(0) @binding(1)
var<storage, read_write> output : array<${type.toString()}>;

@group(0) @binding(2)
var<storage, read_write> metadata : array<u32>; // unused

@compute @workgroup_size(${wgSize[0]}, ${wgSize[1]}, ${wgSize[2]})
fn main(
  @builtin(subgroup_invocation_id) id : u32,
) {
  // Force usage.
  _ = metadata[0];

  output[id] = ${broadcast};
}`;

  const inputData = generateTypedInputs(type);
  let uintsPerOutput = 1;
  if (type instanceof VectorType) {
    uintsPerOutput = type.width === 3 ? 4 : type.width;
    if (type.requiresF16()) {
      uintsPerOutput = Math.floor(uintsPerOutput / 2);
    }
  }
  await runComputeTest(
    t,
    wgsl,
    wgSize,
    uintsPerOutput,
    inputData,
    (metadata, output) => {
      return checkDataTypes(metadata, output, inputData, t.params.id, type);
    }
  );
});

g.test('workgroup_uniform_load').
desc('Tests a workgroup uniform load equivalent').
params((u) =>
u.
combine('wgSize', kWGSizes).
beginSubcases().
combine('inputId', [1, 2, 3]).
combine('first', [false, true])
).
fn((t) => {
  t.skipIfDeviceDoesNotHaveFeature('subgroups');
  // Compatibility mode has lower workgroup limits.
  const wgThreads = t.params.wgSize[0] * t.params.wgSize[1] * t.params.wgSize[2];
  const {
    maxComputeInvocationsPerWorkgroup,
    maxComputeWorkgroupSizeX,
    maxComputeWorkgroupSizeY,
    maxComputeWorkgroupSizeZ
  } = t.device.limits;
  t.skipIf(
    maxComputeInvocationsPerWorkgroup < wgThreads ||
    maxComputeWorkgroupSizeX < t.params.wgSize[0] ||
    maxComputeWorkgroupSizeY < t.params.wgSize[1] ||
    maxComputeWorkgroupSizeZ < t.params.wgSize[2],
    'Workgroup size too large'
  );

  const broadcast = t.params.first ? `subgroupBroadcastFirst(v)` : `subgroupBroadcast(v, 0)`;

  const wgsl = `
enable subgroups;

diagnostic(off, subgroup_branching);

var<workgroup> wgmem : u32;

@group(0) @binding(0)
var<storage, read> inputs : array<u32>;

@group(0) @binding(1)
var<storage, read_write> output : array<u32>;

@compute @workgroup_size(${t.params.wgSize[0]}, ${t.params.wgSize[1]}, ${t.params.wgSize[2]})
fn main(@builtin(subgroup_invocation_id) id : u32,
        @builtin(local_invocation_index) lid : u32) {
  if (lid == ${t.params.inputId}) {
    wgmem = inputs[lid];
  }
  workgroupBarrier();
  var v = 0u;
  if (id == 0) {
    v = wgmem;
  }
  v = ${broadcast};
  output[lid] = v;
}`;

  const values = [1, 13, 33, 125];
  const inputBuffer = t.makeBufferWithContents(
    new Uint32Array(values),
    GPUBufferUsage.COPY_SRC | GPUBufferUsage.STORAGE
  );
  t.trackForCleanup(inputBuffer);

  const outputBuffer = t.makeBufferWithContents(
    new Uint32Array([...iterRange(wgThreads, (x) => 0)]),
    GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST | GPUBufferUsage.STORAGE
  );
  t.trackForCleanup(outputBuffer);

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
        buffer: inputBuffer
      }
    },
    {
      binding: 1,
      resource: {
        buffer: outputBuffer
      }
    }]

  });

  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginComputePass();
  pass.setPipeline(pipeline);
  pass.setBindGroup(0, bg);
  pass.dispatchWorkgroups(1, 1, 1);
  pass.end();
  t.queue.submit([encoder.finish()]);

  const expect = Array(wgThreads);
  expect.fill(values[t.params.inputId]);
  t.expectGPUBufferValuesEqual(outputBuffer, new Uint32Array(expect));
});

/**
 * Checks the results of broadcast in compute shaders.
 *
 * @param metadata An array of uint32s
 *                 * first half is subgroup_invocation_id
 *                 * second half is subgroup_size
 * @param output An array uint32s containing the broadcast results
 * @param numInvs The number of invocations
 * @param broadcast The broadcast invocation (or 'first' to indicate the lowest active)
 * @param filter A functor indicating whether the invocation participates in the broadcast
 */
function checkCompute(
metadata,
output,
numInvs,
broadcast,
filter)
{
  let broadcastedId = broadcast;
  if (broadcast === 'first') {
    // Subgroup size is uniform in compute shaders so any will do.
    const size = metadata[numInvs];
    for (let i = 0; i < size; i++) {
      if (filter(i, size)) {
        broadcastedId = i;
        break;
      }
    }
  }

  const mapping = new Map();
  const sizes = new Map();
  for (let i = 0; i < numInvs; i++) {
    const id = metadata[i];
    const size = metadata[i + numInvs];

    const res = output[i];

    if (filter(id, size)) {
      let seen = mapping.get(res) ?? 0;
      seen++;
      mapping.set(res, seen);

      if (broadcastedId === id) {
        sizes.set(res, size);
        if (res !== i) {
          return new Error(`Invocation ${i}: incorrect result:
- expected: ${i}
-      got: ${res}`);
        }
      }
    } else {
      if (res !== kDataSentinel) {
        return new Error(`Invocation ${i}: unexpected write (${res})`);
      }
    }
  }

  for (const [key, value] of mapping) {
    const id = Number(key);
    const seen = Number(value);
    const size = sizes.get(id) ?? 0;
    if (size < seen) {
      return new Error(`Unexpected number of invocations for subgroup ${id}
- expected: ${size}
-      got: ${seen}`);
    }
  }

  return undefined;
}

g.test('compute,all_active').
desc('Test broadcasts in compute shaders with all active invocations').
params((u) =>
u.
combine('wgSize', kWGSizes).
beginSubcases()
// Only values < 4 are used because it is a dynamic error to broadcast an inactive invocation.
.combine('id', [0, 1, 2, 3])
).
fn(async (t) => {
  t.skipIfDeviceDoesNotHaveFeature('subgroups');
  const wgThreads = t.params.wgSize[0] * t.params.wgSize[1] * t.params.wgSize[2];

  const broadcast =
  t.params.id === 0 ?
  `subgroupBroadcastFirst(input[lid])` :
  `subgroupBroadcast(input[lid], ${t.params.id})`;

  const wgsl = `
enable subgroups;

@group(0) @binding(0)
var<storage> input : array<u32>;

struct Metadata {
  id : array<u32, ${wgThreads}>,
  size : array<u32, ${wgThreads}>,
}

@group(0) @binding(1)
var<storage, read_write> output : array<u32>;

@group(0) @binding(2)
var<storage, read_write> metadata: Metadata;

@compute @workgroup_size(${t.params.wgSize[0]}, ${t.params.wgSize[1]}, ${t.params.wgSize[2]})
fn main(
  @builtin(local_invocation_index) lid : u32,
  @builtin(subgroup_invocation_id) id : u32,
  @builtin(subgroup_size) subgroupSize : u32,
) {
  metadata.id[lid] = id;
  metadata.size[lid] = subgroupSize;

  output[lid] = ${broadcast};
}`;

  const inputData = new Uint32Array([...iterRange(wgThreads, (x) => x)]);
  const uintsPerOutput = 1;
  await runComputeTest(
    t,
    wgsl,
    [t.params.wgSize[0], t.params.wgSize[1], t.params.wgSize[2]],
    uintsPerOutput,
    inputData,
    (metadata, output) => {
      return checkCompute(
        metadata,
        output,
        wgThreads,
        t.params.id,
        (id, size) => {
          return true;
        }
      );
    }
  );
});

g.test('compute,split').
desc(`Test broadcasts with only some active invocations`).
params((u) =>
u.
combine('predicate', keysOf(kPredicateCases)).
filter((t) => {
  // This case would be UB
  return t.predicate !== 'upper_half';
}).
beginSubcases().
combine('id', [0, 1, 2, 3]).
combine('wgSize', kWGSizes)
).
fn(async (t) => {
  t.skipIfDeviceDoesNotHaveFeature('subgroups');
  const testcase = kPredicateCases[t.params.predicate];
  const wgThreads = t.params.wgSize[0] * t.params.wgSize[1] * t.params.wgSize[2];





  const { subgroupMinSize, subgroupMaxSize } = t.device.adapterInfo;
  for (let size = subgroupMinSize; size <= subgroupMaxSize; size *= 2) {
    t.skipIf(!testcase.filter(t.params.id, size), 'Skipping potential undefined behavior');
  }

  const broadcast =
  t.params.id === 0 ?
  `subgroupBroadcastFirst(input[lid])` :
  `subgroupBroadcast(input[lid], ${t.params.id})`;

  const wgsl = `
enable subgroups;
diagnostic(off, subgroup_uniformity);

@group(0) @binding(0)
var<storage> input : array<u32>;

struct Metadata {
  id : array<u32, ${wgThreads}>,
  size : array<u32, ${wgThreads}>,
}

@group(0) @binding(1)
var<storage, read_write> output : array<u32>;

@group(0) @binding(2)
var<storage, read_write> metadata: Metadata;

@compute @workgroup_size(${t.params.wgSize[0]}, ${t.params.wgSize[1]}, ${t.params.wgSize[2]})
fn main(
  @builtin(local_invocation_index) lid : u32,
  @builtin(subgroup_invocation_id) id : u32,
  @builtin(subgroup_size) subgroupSize : u32,
) {
  metadata.id[lid] = id;
  metadata.size[lid] = subgroupSize;

  if ${testcase.cond} {
    output[lid] = ${broadcast};
  } else {
    return;
  }
}`;

  const inputData = new Uint32Array([...iterRange(wgThreads, (x) => x)]);
  const uintsPerOutput = 1;
  await runComputeTest(
    t,
    wgsl,
    [t.params.wgSize[0], t.params.wgSize[1], t.params.wgSize[2]],
    uintsPerOutput,
    inputData,
    (metadata, output) => {
      return checkCompute(metadata, output, wgThreads, t.params.id, testcase.filter);
    }
  );
});

g.test('broadcastFirst,split').
desc(`Test broadcastFirst with only some active invocations`).
params((u) =>
u.combine('predicate', keysOf(kPredicateCases)).beginSubcases().combine('wgSize', kWGSizes)
).
fn(async (t) => {
  t.skipIfDeviceDoesNotHaveFeature('subgroups');
  const testcase = kPredicateCases[t.params.predicate];
  const wgThreads = t.params.wgSize[0] * t.params.wgSize[1] * t.params.wgSize[2];

  const wgsl = `
enable subgroups;
diagnostic(off, subgroup_uniformity);

@group(0) @binding(0)
var<storage> input : array<u32>;

struct Metadata {
  id : array<u32, ${wgThreads}>,
  size : array<u32, ${wgThreads}>,
}

@group(0) @binding(1)
var<storage, read_write> output : array<u32>;

@group(0) @binding(2)
var<storage, read_write> metadata: Metadata;

@compute @workgroup_size(${t.params.wgSize[0]}, ${t.params.wgSize[1]}, ${t.params.wgSize[2]})
fn main(
  @builtin(local_invocation_index) lid : u32,
  @builtin(subgroup_invocation_id) id : u32,
  @builtin(subgroup_size) subgroupSize : u32,
) {
  metadata.id[lid] = id;
  metadata.size[lid] = subgroupSize;

  if ${testcase.cond} {
    output[lid] = subgroupBroadcastFirst(input[lid]);
  } else {
    return;
  }
}`;

  const inputData = new Uint32Array([...iterRange(wgThreads, (x) => x)]);
  const uintsPerOutput = 1;
  await runComputeTest(
    t,
    wgsl,
    [t.params.wgSize[0], t.params.wgSize[1], t.params.wgSize[2]],
    uintsPerOutput,
    inputData,
    (metadata, output) => {
      return checkCompute(metadata, output, wgThreads, 'first', testcase.filter);
    }
  );
});

/**
 * Check broadcasts in fragment shaders
 *
 * Only checks subgroups where no invocation is in the last row
 * or column to avoid helper invocations.
 * @param data The framebuffer output
 *             * component 0 is the broadcast result
 *             * component 1 is the subgroup_invocation_id
 *             * component 2 is the subgroup_size
 * @param format The framebuffer format
 * @param width The framebuffer width
 * @param height The framebuffer height
 * @param broadcast The id being broadcast
 */
function checkFragment(
data,
format,
width,
height,
broadcast)
{
  const { uintsPerRow, uintsPerTexel } = getUintsPerFramebuffer(format, width, height);

  const coordToIndex = (row, col) => {
    return uintsPerRow * row + col * uintsPerTexel;
  };

  const inBounds = new Map();
  for (let row = 0; row < height; row++) {
    for (let col = 0; col < width; col++) {
      const offset = coordToIndex(row, col);

      const res = data[offset];

      let bound = inBounds.get(res) ?? true;
      bound = bound && row < height - 1 && col < height - 1;
      inBounds.set(res, bound);
    }
  }

  const seen = new Map();
  const sizes = new Map();
  for (let row = 0; row < height; row++) {
    for (let col = 0; col < width; col++) {
      const offset = coordToIndex(row, col);

      const res = data[offset];
      const bound = inBounds.get(res) ?? true;
      if (!bound) {
        continue;
      }

      const id = data[offset + 1];
      const size = data[offset + 2];

      let s = seen.get(res) ?? 0;
      s++;
      seen.set(res, s);

      if (id === broadcast) {
        const linear = row * width + col;
        if (res !== linear) {
          return new Error(`Row ${row}, col ${col}: incorrect broadcast
- expected: ${linear}
-      got: ${res}`);
        }

        sizes.set(res, size);
      }
    }
  }

  for (const [key, value] of inBounds) {
    const id = Number(key);
    const ok = Boolean(value);
    if (ok) {
      const size = sizes.get(id) ?? 0;
      const seen = sizes.get(id) ?? 0;
      if (size < seen) {
        return new Error(`Unexpected number of invocations for subgroup ${id}
- expected: ${size}
-      got: ${seen}`);
      }
    }
  }

  return undefined;
}

g.test('fragment').
desc('Test broadcast in fragment shaders').
params((u) =>
u.
combine('size', kFramebufferSizes).
beginSubcases().
combine('id', [0, 1, 2, 3]).
combineWithParams([{ format: 'rgba32uint' }])
).
fn(async (t) => {
  t.skipIfDeviceDoesNotHaveFeature('subgroups');
  const innerTexels = (t.params.size[0] - 1) * (t.params.size[1] - 1);



  const { subgroupMaxSize } = t.device.adapterInfo;
  t.skipIf(innerTexels < subgroupMaxSize, 'Too few texels to be reliable');

  const broadcast =
  t.params.id === 0 ?
  `subgroupBroadcastFirst(input[linear].x)` :
  `subgroupBroadcast(input[linear].x, ${t.params.id})`;
  const texels = t.params.size[0] * t.params.size[1];
  const inputData = new Uint32Array([...iterRange(texels, (x) => x)]);

  const fsShader = `
enable subgroups;

@group(0) @binding(0)
var<uniform> input : array<vec4u, ${inputData.length}>;

@fragment
fn main(
  @builtin(position) pos : vec4f,
  @builtin(subgroup_invocation_id) id : u32,
  @builtin(subgroup_size) size : u32,
) -> @location(0) vec4u {
  let linear = u32(pos.x) + u32(pos.y) * ${t.params.size[0]};

  return vec4u(${broadcast}, id, size, linear);
}`;

  await runFragmentTest(
    t,
    t.params.format,
    fsShader,
    t.params.size[0],
    t.params.size[1],
    inputData,
    (data) => {
      return checkFragment(
        data,
        t.params.format,
        t.params.size[0],
        t.params.size[1],
        t.params.id
      );
    }
  );
});