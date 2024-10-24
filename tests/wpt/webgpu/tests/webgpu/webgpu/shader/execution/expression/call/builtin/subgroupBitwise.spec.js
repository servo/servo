/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution tests for subgroupAny.

Note: There is a lack of portability for non-uniform execution so these tests
restrict themselves to uniform control flow.
Note: There is no guaranteed mapping between subgroup_invocation_id and
local_invocation_index. Tests should avoid assuming there is.
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { keysOf, objectsToRecord } from '../../../../../../common/util/data_tables.js';
import { iterRange } from '../../../../../../common/util/util.js';
import { kTextureFormatInfo } from '../../../../../format_info.js';
import {
  kConcreteSignedIntegerScalarsAndVectors,
  kConcreteUnsignedIntegerScalarsAndVectors,
  scalarTypeOf,

  VectorType } from
'../../../../../util/conversion.js';
import { align } from '../../../../../util/math.js';
import { PRNG } from '../../../../../util/prng.js';

import {
  kWGSizes,
  kPredicateCases,
  SubgroupTest,
  kDataSentinel,
  runComputeTest,
  runFragmentTest,
  kFramebufferSizes } from
'./subgroup_util.js';

export const g = makeTestGroup(SubgroupTest);

const kNumCases = 15;
const kOps = ['subgroupAnd', 'subgroupOr', 'subgroupXor'];
const kTypes = objectsToRecord([
...kConcreteSignedIntegerScalarsAndVectors,
...kConcreteUnsignedIntegerScalarsAndVectors]
);

/**
 * Performs the appropriate bitwise operation on v1 and v2.
 *
 * @param op The subgroup operation
 * @param v1 The first value
 * @param v2 The second value
 */
function bitwise(op, v1, v2) {
  switch (op) {
    case 'subgroupAnd':
      return v1 & v2;
    case 'subgroupOr':
      return v1 | v2;
    case 'subgroupXor':
      return v1 ^ v2;
  }
}

/**
 * Returns the identity value for the subgroup operations
 *
 * @param op The subgroup operation
 */
function identity(op) {
  switch (op) {
    case 'subgroupAnd':
      return ~0;
    case 'subgroupOr':
    case 'subgroupXor':
      return 0;
  }
}

/**
 * Checks the results for data type test
 *
 * The shader generate a unique subgroup id for each subgroup (avoiding 0).
 * The check calculates the expected result for all subgroups and then compares that
 * to the actual results.
 * @param metadata An array of integers divided as follows:
 *                 * first half subgroup invocation id
 *                 * second half unique subgroup id
 * @param output An array of output values
 * @param type The type being tested
 * @param op The subgroup operation
 * @param offset A constant offset added to subgroup invocation id to form the
 *               the input to the subgroup operation
 */
function checkDataTypes(
metadata,
output,
type,
op,
offset)
{
  const expected = new Map();
  for (let i = 0; i < Math.floor(metadata.length / 2); i++) {
    const group_id = metadata[i + Math.floor(metadata.length / 2)];
    let expect = expected.get(group_id) ?? identity(op);
    expect = bitwise(op, expect, i + offset);
    expected.set(group_id, expect);
  }

  let numEles = 1;
  let stride = 1;
  if (type instanceof VectorType) {
    numEles = type.width;
    stride = numEles === 3 ? 4 : numEles;
  }
  for (let inv = 0; inv < Math.floor(output.length / stride); inv++) {
    const group_id = metadata[inv + Math.floor(metadata.length / 2)];
    const expect = expected.get(group_id) ?? 0;
    for (let ele = 0; ele < numEles; ele++) {
      const res = output[inv * stride + ele];
      if (res !== expect) {
        return new Error(`Invocation ${inv}, component ${ele}: incorrect result
- expected: ${expect}
-      got: ${res}`);
      }
    }
  }

  return undefined;
}

g.test('data_types').
desc('Tests allowed data types').
params((u) =>
u.
combine('type', keysOf(kTypes)).
beginSubcases().
combine('wgSize', kWGSizes).
combine('op', kOps)
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('subgroups');
}).
fn(async (t) => {
  const type = kTypes[t.params.type];
  let numEles = 1;
  if (type instanceof VectorType) {
    numEles = type.width === 3 ? 4 : type.width;
  }

  const scalarTy = scalarTypeOf(type);

  const wgThreads = t.params.wgSize[0] * t.params.wgSize[1] * t.params.wgSize[2];

  const wgsl = `
enable subgroups;

@group(0) @binding(0)
var<storage> inputs : array<u32>;

@group(0) @binding(1)
var<storage, read_write> outputs : array<${type.toString()}>;

struct Metadata {
  id : array<u32, ${wgThreads}>,
  group_id : array<u32, ${wgThreads}>
}

@group(0) @binding(2)
var<storage, read_write> metadata : Metadata;

@compute @workgroup_size(${t.params.wgSize[0]}, ${t.params.wgSize[1]}, ${t.params.wgSize[2]})
fn main(
  @builtin(local_invocation_index) lid : u32,
  @builtin(subgroup_invocation_id) id : u32,
) {

  // Record subgroup invocation id for this invocation.
  metadata.id[lid] = id;

  // Record a unique id for this subgroup (avoid 0).
  let group_id = subgroupBroadcastFirst(lid + 1);
  metadata.group_id[lid] = group_id;

  outputs[lid] = ${t.params.op}(${type.toString()}(${scalarTy.toString()}(lid + inputs[0])));
}`;

  const magicOffset = 0x7fff000f;
  await runComputeTest(
    t,
    wgsl,
    [t.params.wgSize[0], t.params.wgSize[1], t.params.wgSize[2]],
    numEles,
    new Uint32Array([magicOffset]),
    (metadata, output) => {
      return checkDataTypes(metadata, output, type, t.params.op, magicOffset);
    }
  );
});

/**
 * Generates randomized input data
 *
 * Case 0: All 0s
 * Case 1: All 0xffffs
 * Case 2-9: All identity values except an inverted value randomly every 32 values.
 *           All values capped to 0xffff
 * Case 10+: Random values in the range [0, 2 ** 30]
 * @param seed The PRNG seed
 * @param num The number of values to generate
 * @param identity The identity value for the operation
 */
function generateInputData(seed, num, identity) {
  const prng = new PRNG(seed);

  const bound = Math.min(num, 32);
  const index = prng.uniformInt(bound);

  return new Uint32Array([
  ...iterRange(num, (x) => {
    if (seed === 0) {
      return 0;
    } else if (seed === 1) {
      return 0xffff;
    } else if (seed < 10) {
      const bounded = x % bound;
      let val = bounded === index ? ~identity : identity;
      val &= 0xffff;
      return val;
    }
    return prng.uniformInt(1 << 30);
  })]
  );
}

/**
 * Checks the result of compute tests
 *
 * Calculates the expected results for each subgroup and compares against
 * the actual output.
 * @param metadata An array divided as follows:
 *                 * first half: subgroup invocation id in lower 16 bits
 *                               subgroup size in upper 16 bits
 *                 * second half: unique subgroup id
 * @param output The outputs
 * @param input The input data
 * @param op The subgroup operation
 * @param filter A predicate used to filter invocations.
 */
function checkBitwiseCompute(
metadata,
output,
input,
op,
filter)
{
  const expected = new Map();
  for (let i = 0; i < output.length; i++) {
    const group_id = metadata[i + output.length];
    const combo = metadata[i];
    const id = combo & 0xffff;
    const size = combo >> 16 & 0xffff;
    if (filter(id, size)) {
      let expect = expected.get(group_id) ?? identity(op);
      expect = bitwise(op, expect, input[i]);
      expected.set(group_id, expect);
    }
  }

  for (let i = 0; i < output.length; i++) {
    const group_id = metadata[i + output.length];
    const combo = metadata[i];
    const id = combo & 0xffff;
    const size = combo >> 16 & 0xffff;
    const res = output[i];
    if (filter(id, size)) {
      const expect = expected.get(group_id) ?? 0;
      if (res !== expect) {
        return new Error(`Invocation ${i}: incorrect result
- expected: ${expect}
-      got: ${res}`);
      }
    } else {
      if (res !== kDataSentinel) {
        return new Error(`Invocation ${i}: unexpected write`);
      }
    }
  }

  return undefined;
}

g.test('compute,all_active').
desc('Test bitwise operations with randomized inputs').
params((u) =>
u.
combine('case', [...iterRange(kNumCases, (x) => x)]).
beginSubcases().
combine('wgSize', kWGSizes).
combine('op', kOps)
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('subgroups');
}).
fn(async (t) => {
  const wgThreads = t.params.wgSize[0] * t.params.wgSize[1] * t.params.wgSize[2];

  const wgsl = `
enable subgroups;

@group(0) @binding(0)
var<storage> inputs : array<u32>;

@group(0) @binding(1)
var<storage, read_write> outputs : array<u32>;

struct Metadata {
  id_and_size : array<u32, ${wgThreads}>,
  group_id : array<u32, ${wgThreads}>
}

@group(0) @binding(2)
var<storage, read_write> metadata : Metadata;

@compute @workgroup_size(${t.params.wgSize[0]}, ${t.params.wgSize[1]}, ${t.params.wgSize[2]})
fn main(
  @builtin(local_invocation_index) lid : u32,
  @builtin(subgroup_invocation_id) id : u32,
  @builtin(subgroup_size) sg_size : u32,
) {

  // Record both subgroup invocation id and subgroup size in the same u32.
  // Subgroups sizes are in the range [4, 128] so both values fit.
  metadata.id_and_size[lid] = id | (sg_size << 16);

  // Record a unique id for this subgroup (avoid 0).
  let group_id = subgroupBroadcastFirst(lid + 1);
  metadata.group_id[lid] = group_id;

  outputs[lid] = ${t.params.op}(inputs[lid]);
}`;

  const inputData = generateInputData(t.params.case, wgThreads, identity(t.params.op));
  const uintsPerOutput = 1;
  await runComputeTest(
    t,
    wgsl,
    [t.params.wgSize[0], t.params.wgSize[1], t.params.wgSize[2]],
    uintsPerOutput,
    inputData,
    (metadata, output) => {
      return checkBitwiseCompute(
        metadata,
        output,
        inputData,
        t.params.op,
        (id, size) => {
          return true;
        }
      );
    }
  );
});

g.test('compute,split').
desc('Test that only active invocations participate').
params((u) =>
u.
combine('predicate', keysOf(kPredicateCases)).
beginSubcases().
combine('wgSize', kWGSizes).
combine('op', kOps).
combine('case', [...iterRange(kNumCases, (x) => x)])
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('subgroups');
}).
fn(async (t) => {
  const testcase = kPredicateCases[t.params.predicate];
  const wgThreads = t.params.wgSize[0] * t.params.wgSize[1] * t.params.wgSize[2];

  const wgsl = `
enable subgroups;

@group(0) @binding(0)
var<storage> inputs : array<u32>;

@group(0) @binding(1)
var<storage, read_write> outputs : array<u32>;

struct Metadata {
  id_and_size : array<u32, ${wgThreads}>,
  group_id : array<u32, ${wgThreads}>
}

@group(0) @binding(2)
var<storage, read_write> metadata : Metadata;

@compute @workgroup_size(${t.params.wgSize[0]}, ${t.params.wgSize[1]}, ${t.params.wgSize[2]})
fn main(
  @builtin(local_invocation_index) lid : u32,
  @builtin(subgroup_invocation_id) id : u32,
  @builtin(subgroup_size) subgroupSize : u32,
) {

  // Record both subgroup invocation id and subgroup size in the same u32.
  // Subgroups sizes are in the range [4, 128] so both values fit.
  metadata.id_and_size[lid] = id | (subgroupSize << 16);

  // Record a unique id for this subgroup (avoid 0).
  let group_id = subgroupBroadcastFirst(lid + 1);
  metadata.group_id[lid] = group_id;

  if ${testcase.cond} {
    outputs[lid] = ${t.params.op}(inputs[lid]);
  } else {
    return;
  }
}`;

  const inputData = generateInputData(t.params.case, wgThreads, identity(t.params.op));
  const uintsPerOutput = 1;
  await runComputeTest(
    t,
    wgsl,
    [t.params.wgSize[0], t.params.wgSize[1], t.params.wgSize[2]],
    uintsPerOutput,
    inputData,
    (metadata, output) => {
      return checkBitwiseCompute(metadata, output, inputData, t.params.op, testcase.filter);
    }
  );
});

/**
 * Checks bitwise ops results from a fragment shader.
 *
 * Avoids the last row and column to skip potential helper invocations.
 * @param data Framebuffer output
 *             * component 0 is result
 *             * component 1 is generated subgroup id
 * @param input An array of input data
 * @param op The subgroup operation
 * @param format The framebuffer format
 * @param width Framebuffer width
 * @param height Framebuffer height
 */
function checkBitwiseFragment(
data,
input,
op,
format,
width,
height)
{
  const { blockWidth, blockHeight, bytesPerBlock } = kTextureFormatInfo[format];
  const blocksPerRow = width / blockWidth;
  // 256 minimum comes from image copy requirements.
  const bytesPerRow = align(blocksPerRow * (bytesPerBlock ?? 1), 256);
  const uintsPerRow = bytesPerRow / 4;
  const uintsPerTexel = (bytesPerBlock ?? 1) / blockWidth / blockHeight / 4;

  // Iteration skips last row and column to avoid helper invocations because it is not
  // guaranteed whether or not they participate in the subgroup operation.
  const expected = new Map();
  for (let row = 0; row < height - 1; row++) {
    for (let col = 0; col < width - 1; col++) {
      const offset = uintsPerRow * row + col * uintsPerTexel;
      const subgroup_id = data[offset + 1];

      if (subgroup_id === 0) {
        return new Error(`Internal error: helper invocation at (${col}, ${row})`);
      }

      let v = expected.get(subgroup_id) ?? identity(op);
      v = bitwise(op, v, input[row * width + col]);
      expected.set(subgroup_id, v);
    }
  }

  for (let row = 0; row < height - 1; row++) {
    for (let col = 0; col < width - 1; col++) {
      const offset = uintsPerRow * row + col * uintsPerTexel;
      const res = data[offset];
      const subgroup_id = data[offset + 1];

      if (subgroup_id === 0) {
        // Inactive in the fragment.
        continue;
      }

      const expected_v = expected.get(subgroup_id) ?? 0;
      if (expected_v !== res) {
        return new Error(`Row ${row}, col ${col}: incorrect results:
- expected: ${expected_v}
-      got: ${res}`);
      }
    }
  }

  return undefined;
}

g.test('fragment,all_active').
desc('Tests bitwise operations in fragment shaders').
params((u) =>
u.
combine('size', kFramebufferSizes).
beginSubcases().
combine('case', [...iterRange(kNumCases, (x) => x)]).
combine('op', kOps).
combineWithParams([{ format: 'rg32uint' }])
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('subgroups');
}).
fn(async (t) => {
  const numInputs = t.params.size[0] * t.params.size[1];
  const inputData = generateInputData(t.params.case, numInputs, identity(t.params.op));

  const ident = identity(t.params.op) === 0 ? '0' : '0xffffffff';
  const fsShader = `
enable subgroups;

@group(0) @binding(0)
var<storage, read_write> inputs : array<u32>;

@fragment
fn main(
  @builtin(position) pos : vec4f,
) -> @location(0) vec2u {
  // Generate a subgroup id based on linearized position, avoid 0.
  let linear = u32(pos.x) + u32(pos.y) * ${t.params.size[0]};
  let subgroup_id = subgroupBroadcastFirst(linear + 1);

  // Filter out possible helper invocations.
  let x_in_range = u32(pos.x) < (${t.params.size[0]} - 1);
  let y_in_range = u32(pos.y) < (${t.params.size[1]} - 1);
  let in_range = x_in_range && y_in_range;
  let input = select(${ident}, inputs[linear], in_range);

  let res = ${t.params.op}(input);
  return vec2u(res, subgroup_id);
}`;

  await runFragmentTest(
    t,
    t.params.format,
    fsShader,
    t.params.size[0],
    t.params.size[1],
    inputData,
    (data) => {
      return checkBitwiseFragment(
        data,
        inputData,
        t.params.op,
        t.params.format,
        t.params.size[0],
        t.params.size[1]
      );
    }
  );
});

g.test('fragment,split').unimplemented();