/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution tests for subgroupShuffle, subgroupShuffleUp, subgroupShuffleDown, and subgroupShuffleXor.

Note: There is a lack of portability for non-uniform execution so these tests
restrict themselves to uniform control flow.
Note: There is no guaranteed mapping between subgroup_invocation_id and
local_invocation_index. Tests should avoid assuming there is.
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { keysOf, objectsToRecord } from '../../../../../../common/util/data_tables.js';
import { assert, iterRange } from '../../../../../../common/util/util.js';
import {
  kConcreteNumericScalarsAndVectors,

  VectorType } from
'../../../../../util/conversion.js';
import { PRNG } from '../../../../../util/prng.js';

import {
  kWGSizes,
  kPredicateCases,
  SubgroupTest,
  runComputeTest,
  runFragmentTest,
  kFramebufferSizes,
  generateTypedInputs,
  getUintsPerFramebuffer } from
'./subgroup_util.js';

export const g = makeTestGroup(SubgroupTest);







const kUpDownOps = ['subgroupShuffleUp', 'subgroupShuffleDown'];

const kOps = ['subgroupShuffle', 'subgroupShuffleXor', ...kUpDownOps];

const kNumCases = 16;

const kTypes = objectsToRecord(kConcreteNumericScalarsAndVectors);

// This size is selected to guarantee a single subgroup.
const kSize = 4;
const kShuffleCases = {
  no_shuffle: {
    id: `id`,
    expected: (input, id) => {
      return input[id];
    }
  },
  broadcast: {
    id: `input[2]`,
    expected: (input, id) => {
      return input[2];
    }
  },
  rotate_1_up: {
    id: `select(id - 1, ${kSize} - 1, id == 0)`,
    expected: (input, id) => {
      const idx = id === 0 ? kSize - 1 : id - 1;
      return input[idx];
    }
  },
  rotate_2_down: {
    id: `(id + 2) % ${kSize}`,
    expected: (input, id) => {
      const idx = (id + 2) % kSize;
      return input[idx];
    }
  },
  reversed: {
    id: `${kSize} - id - 1`,
    expected: (input, id) => {
      return input[kSize - id - 1];
    }
  },
  clamped: {
    id: `clamp(id + 2, 1, 3)`,
    expected: (input, id) => {
      const idx = Math.max(Math.min(id + 2, 3), 1);
      return input[idx];
    }
  }
};

function checkShuffleId(
metadata, // unused
output,
input,
expected)
{
  for (let i = 0; i < kSize; i++) {
    const expect = expected(input, i);
    const res = output[i];
    if (res !== expect) {
      return new Error(`Invocation ${i}: incorrect results
- expected: ${expect}
-      got: ${res}`);
    }
  }

  return undefined;
}

g.test('shuffle,id').
desc(`Tests various ways to shuffle invocations`).
params((u) => u.combine('case', keysOf(kShuffleCases))).
fn(async (t) => {
  t.skipIfDeviceDoesNotHaveFeature('subgroups');
  const testcase = kShuffleCases[t.params.case];

  const wgsl = `
enable subgroups;

@group(0) @binding(0)
var<storage> input : array<u32>;

@group(0) @binding(1)
var<storage, read_write> output : array<u32>;

@group(0) @binding(2)
var<storage, read_write> metadata : array<u32>; // unused

@compute @workgroup_size(${kSize})
fn main(
  @builtin(subgroup_invocation_id) id : u32,
) {
  // Force usage
  _ = metadata[0];

  output[id] = subgroupShuffle(input[id], ${testcase.id});
}`;

  const inputData = new Uint32Array([...iterRange(kSize, (x) => x)]);
  const uintsPerOutput = 1;
  await runComputeTest(
    t,
    wgsl,
    [kSize, 1, 1],
    uintsPerOutput,
    inputData,
    (metadata, output) => {
      return checkShuffleId(metadata, output, inputData, testcase.expected);
    }
  );
});







// Delta must be dynamically uniform
const kUpDownCases = {
  no_shuffle: {
    delta: `0`,
    expected: (input, id, op) => {
      return input[id];
    },
    diagnostic: `error`
  },
  dynamic_1: {
    delta: `input[1]`,
    expected: (input, id, op) => {
      let idx = id;
      if (op === 'subgroupShuffleUp') {
        idx = id - 1;
        if (idx < 0) {
          return undefined;
        }
        return input[idx];
      } else {
        idx = id + 1;
        if (idx > 3) {
          return undefined;
        }
      }
      return input[idx];
    },
    diagnostic: `off`
  },
  override_2: {
    delta: `override_idx`,
    expected: (input, id, op) => {
      let idx = id;
      if (op === 'subgroupShuffleUp') {
        idx = id - 2;
        if (idx < 0) {
          return undefined;
        }
        return input[idx];
      } else {
        idx = id + 2;
        if (idx > 3) {
          return undefined;
        }
      }
      return input[idx];
    },
    diagnostic: `error`
  }
};

function checkShuffleUpDownDelta(
metadata, // unused
output,
input,
op,
expected)
{
  assert(op === 'subgroupShuffleUp' || op === 'subgroupShuffleDown');

  for (let i = 0; i < kSize; i++) {
    const expect = expected(input, i, op);
    const res = output[i];
    if (expect && expect !== res) {
      return new Error(`Invocation ${i}: incorrect results
- expected: ${expect}
-      got: ${res}`);
    }
  }

  return undefined;
}

g.test('shuffleUpDown,delta').
desc(`Test ShuffleUp and ShuffleDown deltas`).
params((u) => u.combine('op', kUpDownOps).combine('case', keysOf(kUpDownCases))).
fn(async (t) => {
  t.skipIfDeviceDoesNotHaveFeature('subgroups');
  const testcase = kUpDownCases[t.params.case];

  const wgsl = `
enable subgroups;
diagnostic(${testcase.diagnostic}, subgroup_uniformity);

override override_idx = 2u;

@group(0) @binding(0)
var<storage> input : array<u32>;

@group(0) @binding(1)
var<storage, read_write> output : array<u32>;

@group(0) @binding(2)
var<storage, read_write> metadata : array<u32>; // unused

@compute @workgroup_size(${kSize})
fn main(
  @builtin(subgroup_invocation_id) id : u32,
) {
  // Force usage
  _ = metadata[0];

  output[id] = ${t.params.op}(input[id], ${testcase.delta});
}`;

  const inputData = new Uint32Array([...iterRange(kSize, (x) => x)]);
  const uintsPerOutput = 1;
  await runComputeTest(
    t,
    wgsl,
    [kSize, 1, 1],
    uintsPerOutput,
    inputData,
    (metadata, output) => {
      return checkShuffleUpDownDelta(metadata, output, inputData, t.params.op, testcase.expected);
    }
  );
});

const kMaskCases = {
  no_shuffle: {
    mask: `0`,
    value: 0,
    diagnostic: `error`
  },
  dynamic_1: {
    mask: `input[1]`,
    value: 1,
    diagnostic: `off`
  },
  override_2: {
    mask: `override_idx`,
    value: 2,
    diagnostic: `error`
  },
  expr_3: {
    mask: `input[1] + input[2]`,
    value: 3,
    diagnostic: `off`
  }
};

function checkShuffleMask(
metadata, // unused
output,
input,
mask)
{
  assert(mask === Math.trunc(mask));
  assert(0 <= mask && mask <= 3);

  for (let i = 0; i < kSize; i++) {
    const expect = input[i ^ mask];
    const res = output[i];
    if (res !== expect) {
      return new Error(`Invocation ${i}: incorrect result
- expected: ${expect}
-      got: ${res}`);
    }
  }

  return undefined;
}

g.test('shuffleXor,mask').
desc(`Test ShuffleXor masks`).
params((u) => u.combine('case', keysOf(kMaskCases))).
fn(async (t) => {
  t.skipIfDeviceDoesNotHaveFeature('subgroups');
  const testcase = kMaskCases[t.params.case];

  const wgsl = `
enable subgroups;
diagnostic(${testcase.diagnostic}, subgroup_uniformity);

override override_idx = 2u;

@group(0) @binding(0)
var<storage> input : array<u32>;

@group(0) @binding(1)
var<storage, read_write> output : array<u32>;

@group(0) @binding(2)
var<storage, read_write> metadata : array<u32>; // unused

@compute @workgroup_size(${kSize})
fn main(
  @builtin(subgroup_invocation_id) id : u32,
) {
  // Force usage
  _ = metadata[0];

  output[id] = subgroupShuffleXor(input[id], ${testcase.mask});
}`;

  const inputData = new Uint32Array([...iterRange(kSize, (x) => x)]);
  const uintsPerOutput = 1;
  await runComputeTest(
    t,
    wgsl,
    [kSize, 1, 1],
    uintsPerOutput,
    inputData,
    (metadata, output) => {
      return checkShuffleMask(metadata, output, inputData, testcase.value);
    }
  );
});

/**
 * Generate randomized inputs for testing shuffles
 *
 * 1/4 of the cases will be bounded to values in the range [0, 8)
 * 1/4 of the cases will be bounded to values in the range [0, 16)
 * 1/4 of the cases will be bounded to values in the range [0, 32)
 * 1/4 of the cases will be bounded to values in the range [0, 128)
 * @param seed The seed for the PRNG
 * @param numInputs The number of inputs to generate
 */
function generateInputs(seed, numInputs) {
  const prng = new PRNG(seed);

  let bound = 128;
  if (seed < Math.floor(kNumCases / 4)) {
    bound = 8;
  } else if (seed < Math.floor(kNumCases / 2)) {
    bound = 16;
  } else if (seed < 3 * Math.floor(kNumCases / 4)) {
    bound = 32;
  }
  return new Uint32Array([
  ...iterRange(numInputs, (x) => {
    return prng.uniformInt(bound);
  })]
  );
}

/**
 * Returns the subgroup invocation id of requested shuffle
 *
 * @param id The invocation's subgroup_invocation_id
 * @param value The shuffle value
 * @param size The subgroup size
 * @param op The shuffle operation
 */
function getShuffledId(id, value, op) {
  switch (op) {
    case 'subgroupShuffle':
      return value;
    case 'subgroupShuffleUp':
      return id - value;
    case 'subgroupShuffleDown':
      return id + value;
    case 'subgroupShuffleXor':
      return id ^ value;
  }
  assert(false);
  return 0;
}

/**
 * Checks results of compute passes
 *
 * @param metadata An array of uint32 values
 *                 * first half is subgroup_invocation_id
 *                 * second half is unique generated subgroup id
 * @param output An array of uint32 values
 *               * first half is shuffle results
 *               * second half is subgroup_size
 * @param input An array of uint32 input values
 * @param op The shuffle
 * @param numInvs The number of invocations
 * @param filter A predicate to filter invocations
 */
function checkCompute(
metadata,
output,
input,
op,
numInvs,
filter)
{
  const sub_unique_id_to_inv_idx = new Map();
  const empty = [...iterRange(128, (x) => -1)];
  for (let inv = 0; inv < numInvs; inv++) {
    const id = metadata[inv];
    const subgroup_unique_id = metadata[inv + numInvs];
    const v = sub_unique_id_to_inv_idx.get(subgroup_unique_id) ?? Array.from(empty);
    v[id] = inv;
    sub_unique_id_to_inv_idx.set(subgroup_unique_id, v);
  }

  for (let inv = 0; inv < numInvs; inv++) {
    const id = metadata[inv];
    const subgroup_unique_id = metadata[inv + numInvs];
    const sub_inv_id_to_inv_idx = sub_unique_id_to_inv_idx.get(subgroup_unique_id) ?? empty;

    const res = output[inv];
    const size = output[inv + numInvs];

    // subgroup id predicated in shader
    if (!filter(id, size)) {
      continue;
    }

    let inputValue = input[inv];
    if (op !== 'subgroupShuffle') {
      // Because we use 'subgroupBroadcastFirst' without predication.
      const first_subgroup_inv_id = 0;
      inputValue = input[sub_inv_id_to_inv_idx[first_subgroup_inv_id]];
    }

    const shuffled_target_id = getShuffledId(id, inputValue, op);
    if (
    shuffled_target_id < 0 ||
    shuffled_target_id >= 128 ||
    sub_inv_id_to_inv_idx[shuffled_target_id] === -1)
    {
      continue;
    }

    // subgroup id predicated in shader
    if (!filter(shuffled_target_id, size)) {
      continue;
    }

    if (res !== sub_inv_id_to_inv_idx[shuffled_target_id]) {
      return new Error(`Invocation ${inv}: unexpected result
- expected: ${sub_inv_id_to_inv_idx[shuffled_target_id]}
-      got: ${res}
-      id = ${id}
-      size = ${size}
-      inputValue = ${inputValue}
-      shuffled_target_id = ${shuffled_target_id}
-      subgroup_unique_id = ${subgroup_unique_id}`);
    }
  }

  return undefined;
}

g.test('compute,all_active').
desc(`Test randomized inputs across many workgroup sizes`).
params((u) =>
u.
combine('wgSize', kWGSizes).
combine('op', kOps).
beginSubcases().
combine('case', [...iterRange(kNumCases, (x) => x)])
).
fn(async (t) => {
  t.skipIfDeviceDoesNotHaveFeature('subgroups');
  const wgThreads = t.params.wgSize[0] * t.params.wgSize[1] * t.params.wgSize[2];

  let selectValue = `input[lid]`;
  if (t.params.op !== 'subgroupShuffle') {
    // delta and mask operands must be subgroup uniform
    selectValue = `subgroupBroadcastFirst(input[lid])`;
  }

  const wgsl = `
enable subgroups;
diagnostic(off, subgroup_uniformity);

@group(0) @binding(0)
var<storage> input : array<u32, ${wgThreads}>;

struct Output {
  res : array<u32, ${wgThreads}>,
  size : array<u32, ${wgThreads}>
}

@group(0) @binding(1)
var<storage, read_write> output : Output;

struct Metadata {
  id : array<u32, ${wgThreads}>,
  subgroup_id : array<u32, ${wgThreads}>
}

@group(0) @binding(2)
var<storage, read_write> metadata : Metadata;

@compute @workgroup_size(${t.params.wgSize[0]}, ${t.params.wgSize[1]}, ${t.params.wgSize[2]})
fn main(
  @builtin(local_invocation_index) lid : u32,
  @builtin(subgroup_invocation_id) id : u32,
  @builtin(subgroup_size) subgroupSize : u32,
) {
  metadata.id[lid] = id;
  metadata.subgroup_id[lid] = subgroupBroadcastFirst(lid + 1); // avoid 0

  output.size[lid] = subgroupSize;
  output.res[lid] = ${t.params.op}(lid, ${selectValue});
}`;

  const inputArray = generateInputs(t.params.case, wgThreads);
  const numUintsPerOutput = 2;
  await runComputeTest(
    t,
    wgsl,
    [t.params.wgSize[0], t.params.wgSize[1], t.params.wgSize[2]],
    numUintsPerOutput,
    inputArray,
    (metadata, output) => {
      return checkCompute(
        metadata,
        output,
        inputArray,
        t.params.op,
        wgThreads,
        (id, size) => {
          return true;
        }
      );
    }
  );
});

g.test('compute,split').
desc(`Test randomized inputs across many workgroup sizes`).
params((u) =>
u.
combine('predicate', keysOf(kPredicateCases)).
combine('op', kOps).
beginSubcases().
combine('wgSize', kWGSizes)
).
fn(async (t) => {
  t.skipIfDeviceDoesNotHaveFeature('subgroups');
  const testcase = kPredicateCases[t.params.predicate];
  const wgThreads = t.params.wgSize[0] * t.params.wgSize[1] * t.params.wgSize[2];

  let value = `input[lidx]`;
  if (t.params.op !== 'subgroupShuffle') {
    value = `subgroupBroadcastFirst(input[lidx])`;
  }

  const wgsl = `
enable subgroups;
diagnostic(off, subgroup_uniformity);

@group(0) @binding(0)
var<storage> input : array<u32, ${wgThreads}>;

struct Output {
  res : array<u32, ${wgThreads}>,
  size : array<u32, ${wgThreads}>
}

@group(0) @binding(1)
var<storage, read_write> output : Output;

struct Metadata {
  id : array<u32, ${wgThreads}>,
  subgroup_id : array<u32, ${wgThreads}>
}

@group(0) @binding(2)
var<storage, read_write> metadata : Metadata;

@compute @workgroup_size(${t.params.wgSize[0]}, ${t.params.wgSize[1]}, ${t.params.wgSize[2]})
fn main(
  @builtin(local_invocation_index) lidx : u32,
  @builtin(subgroup_invocation_id) id : u32,
  @builtin(subgroup_size) subgroupSize : u32,
) {
  _ = input[0];
  metadata.id[lidx] = id;
  // Made from lidx but not lidx to avoid value confusion.
  var fake_unique_id = lidx + 1000;
  metadata.subgroup_id[lidx] = subgroupBroadcastFirst(fake_unique_id);

  output.size[lidx] = subgroupSize;
  let value = ${value};
  if ${testcase.cond} {
    output.res[lidx] = ${t.params.op}(lidx, value);
  } else {
    return;
  }
}`;

  const inputArray = new Uint32Array([...iterRange(wgThreads, (x) => x % 128)]);
  const numUintsPerOutput = 2;
  await runComputeTest(
    t,
    wgsl,
    [t.params.wgSize[0], t.params.wgSize[1], t.params.wgSize[2]],
    numUintsPerOutput,
    inputArray,
    (metadata, output) => {
      return checkCompute(metadata, output, inputArray, t.params.op, wgThreads, testcase.filter);
    }
  );
});

/**
 * Checks the results of the data types test
 *
 * The outputs for a given index are expected to match the input values
 * for the given shuffle (op and id).
 * @param metadata An unused parameter
 * @param output The output data
 * @param op The shuffle
 * @param id The shuffle id/mask/delta parameter
 * @param type The data type
 */
function checkDataTypes(
metadata, // unused
output,
input,
op,
id,
type)
{
  if (type.requiresF16() && !(type instanceof VectorType)) {
    for (let i = 0; i < 4; i++) {
      const index = getShuffledId(i, id, op);
      if (index < 0 || index >= 4) {
        continue;
      }

      const expectIdx = Math.floor(index / 2);
      const expectShift = index % 2 === 1;
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
        const index = getShuffledId(i, id, op);
        if (index < 0 || index >= 4) {
          continue;
        }

        const expect = input[index * uints + j];
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
params((u) =>
u.
combine('op', kOps).
combine('type', keysOf(kTypes)).
beginSubcases().
combine('id', [0, 1, 2, 3])
).
fn(async (t) => {
  const wgSize = [4, 1, 1];
  const type = kTypes[t.params.type];
  t.skipIfDeviceDoesNotHaveFeature('subgroups');
  if (type.requiresF16()) {
    t.skipIfDeviceDoesNotHaveFeature('shader-f16');
  }

  let enables = `enable subgroups;\n`;
  if (type.requiresF16()) {
    enables += `enable f16;`;
  }
  const wgsl = `
${enables}

@group(0) @binding(0)
var<storage> input : array<${type.toString()}>;

@group(0) @binding(1)
var<storage, read_write> output : array<${type.toString()}>;

@group(0) @binding(2)
var<storage, read_write> metadata : array<u32>; // unused

@compute @workgroup_size(${wgSize[0]}, ${wgSize[1]}, ${wgSize[2]})
fn main(
  @builtin(subgroup_invocation_id) id : u32,
) {
  // Force usage
  _ = metadata[0];

  output[id] = ${t.params.op}(input[id], ${t.params.id});
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
      return checkDataTypes(metadata, output, inputData, t.params.op, t.params.id, type);
    }
  );
});

/**
 * Check subgroup shuffles in fragment shaders
 *
 * @param data The framebuffer output
 *             * component 0 is the result
 *             * component 1 is the subgroup_invocation_id
 *             * component 2 is a unique generated subgroup_id
 * @param format The framebuffer format
 * @param width Framebuffer width
 * @param height Framebuffer height
 * @param shuffleId The value of the shuffle parameter (e.g. id/mask/delta)
 * @param op The shuffle operation
 */
function checkFragment(
data,
format,
width,
height,
shuffleId,
op)
{
  if (width < 3 || height < 3) {
    return new Error(
      `Insufficient framebuffer size [${width}w x ${height}h]. Minimum is [3w x 3h].`
    );
  }

  const { uintsPerRow, uintsPerTexel } = getUintsPerFramebuffer(format, width, height);

  const coordToIndex = (row, col) => {
    return uintsPerRow * row + col * uintsPerTexel;
  };

  const mapping = new Map();
  const empty = [...iterRange(128, (x) => -1)];

  // Iteration skips last row and column to avoid helper invocations because it is not
  // guaranteed whether or not they participate in the subgroup operation.
  for (let row = 0; row < height - 1; row++) {
    for (let col = 0; col < width - 1; col++) {
      const offset = coordToIndex(row, col);

      const id = data[offset + 1];
      const subgroup_id = data[offset + 2];

      const v = mapping.get(subgroup_id) ?? Array.from(empty);
      v[id] = col + row * width;
      mapping.set(subgroup_id, v);
    }
  }

  for (let row = 0; row < height - 1; row++) {
    for (let col = 0; col < width - 1; col++) {
      const offset = coordToIndex(row, col);

      const res = data[offset];
      const id = data[offset + 1];
      const subgroup_id = data[offset + 2];

      const subgroupMapping = mapping.get(subgroup_id) ?? empty;

      const index = getShuffledId(id, shuffleId, op);
      if (index < 0 || index >= 128 || subgroupMapping[index] === -1) {
        continue;
      }

      const shuffleLinear = subgroupMapping[index];
      const shuffleRow = Math.floor(shuffleLinear / width);
      const shuffleCol = shuffleLinear % width;
      if (shuffleRow === height - 1 || shuffleCol === width - 1) {
        continue;
      }

      if (res !== subgroupMapping[index]) {
        return new Error(`Row ${row}, col ${col}: incorrect results:
- expected: ${subgroupMapping[index]}
-      got: ${res}`);
      }
    }
  }

  return undefined;
}

g.test('fragment').
desc(`Test shuffles in fragment shaders`).
params((u) =>
u.
combine('size', kFramebufferSizes).
beginSubcases().
combine('op', kOps).
combine('id', [0, 1, 2, 3]).
combineWithParams([{ format: 'rgba32uint' }])
).
fn(async (t) => {
  t.skipIfDeviceDoesNotHaveFeature('subgroups');
  const fsShader = `
enable subgroups;

@group(0) @binding(0)
var<uniform> inputs : array<vec4u, 1>; // unused

@fragment
fn main(
  @builtin(position) pos : vec4f,
  @builtin(subgroup_invocation_id) id : u32,
) -> @location(0) vec4u {
  // Force usage
  _ = inputs[0];

  let linear = u32(pos.x) + u32(pos.y) * ${t.params.size[0]};
  let subgroup_id = subgroupBroadcastFirst(linear + 1);

  // Filter out possible helper invocations.
  let x_in_range = u32(pos.x) < (${t.params.size[0]} - 1);
  let y_in_range = u32(pos.y) < (${t.params.size[1]} - 1);
  let in_range = x_in_range && y_in_range;

  return vec4u(${t.params.op}(linear, ${t.params.id}), id, subgroup_id, linear);
}`;

  await runFragmentTest(
    t,
    t.params.format,
    fsShader,
    t.params.size[0],
    t.params.size[1],
    new Uint32Array([0]), // unused,
    (data) => {
      return checkFragment(
        data,
        t.params.format,
        t.params.size[0],
        t.params.size[1],
        t.params.id,
        t.params.op
      );
    }
  );
});