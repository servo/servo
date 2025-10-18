/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution tests for subgroupMin and subgroupMax.

Note: There is a lack of portability for non-uniform execution so these tests
restrict themselves to uniform control flow.
Note: There is no guaranteed mapping between subgroup_invocation_id and
local_invocation_index. Tests should avoid assuming there is.
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { keysOf, objectsToRecord } from '../../../../../../common/util/data_tables.js';
import { assert, iterRange } from '../../../../../../common/util/util.js';
import { kValue } from '../../../../../util/constants.js';
import {
  kConcreteNumericScalarsAndVectors,

  VectorType } from
'../../../../../util/conversion.js';
import { FP } from '../../../../../util/floating_point.js';
import { PRNG } from '../../../../../util/prng.js';

import {
  kNumCases,
  kStride,
  kWGSizes,
  kPredicateCases,
  runAccuracyTest,
  runComputeTest,
  generateTypedInputs,
  getUintsPerFramebuffer,
  kFramebufferSizes,
  runFragmentTest,
  SubgroupTest } from
'./subgroup_util.js';

export const g = makeTestGroup(SubgroupTest);

const kDataTypes = objectsToRecord(kConcreteNumericScalarsAndVectors);



const kOps = ['subgroupMin', 'subgroupMax'];

/**
 * Returns an identity value for the given operation and type.
 *
 * This function should use positive or negative infinity for min and max
 * identities respectively, but implementations may assume infinities are not
 * present so max value is used instead.
 * @param op Min or max
 * @param type The type (f16 or f32)
 */
function identity(op, type) {
  assert(type === 'f16' || type === 'f32');
  if (op === 'subgroupMin') {
    return type === 'f16' ? kValue.f16.positive.max : kValue.f32.positive.max;
  } else {
    return type === 'f16' ? kValue.f16.negative.min : kValue.f32.negative.min;
  }
}

/**
 * Returns the interval generator for the given operation and type.
 *
 * @param op Min or max
 * @param type The type (f16 or f32)
 */
function interval(
op,
type)
{
  assert(type === 'f16' || type === 'f32');
  if (op === 'subgroupMin') {
    return type === 'f16' ? FP.f16.minInterval : FP.f32.minInterval;
  } else {
    return type === 'f16' ? FP.f16.maxInterval : FP.f32.maxInterval;
  }
}

g.test('fp_accuracy').
desc(
  `Tests the accuracy of floating-point addition.

The order of operations is implementation defined, most threads are filled with
the identity value and two receive random values.
Subgroup sizes are not known ahead of time so some cases may not perform any
interesting operations. The test biases towards checking subgroup sizes under 64.
These tests only check two values in order to reuse more of the existing infrastructure
and limit the number of permutations needed to calculate the final result.`
).
params((u) =>
u.
combine('case', [...iterRange(kNumCases, (x) => x)]).
combine('type', ['f32', 'f16']).
combine('op', kOps).
combine('wgSize', [
[kStride, 1, 1],
[kStride / 2, 2, 1]]
)
).
fn(async (t) => {
  t.skipIfDeviceDoesNotHaveFeature('subgroups');
  if (t.params.type === 'f16') {
    t.skipIfDeviceDoesNotHaveFeature('shader-f16');
  }

  await runAccuracyTest(
    t,
    t.params.case,
    [t.params.wgSize[0], t.params.wgSize[1], t.params.wgSize[2]],
    t.params.op,
    t.params.type,
    identity(t.params.op, t.params.type),
    interval(t.params.op, t.params.type)
  );
});

/**
 * Checks the results of subgroupMin and subgroupMax for allowed data types.
 *
 * The shader performs a subgroup operation and equivalent single invocation
 * operation and the results are compared. Since the subgroup operation is a
 * reduction all invocations should have the same expected result.
 * @param metadata The expected reduction results
 * @param output The subgroup operation outputs
 * @param type The data type
 */
function checkDataTypes(metadata, output, type) {
  if (type.requiresF16() && !(type instanceof VectorType)) {
    const expected = metadata[0];
    const expectF16 = expected & 0xffff;
    for (let i = 0; i < 4; i++) {
      const index = Math.floor(i / 2);
      const shift = i % 2 === 1;
      let res = output[index];
      if (shift) {
        res >>= 16;
      }
      res &= 0xffff;

      if (res !== expectF16) {
        return new Error(`Invocation ${i}: incorrect results
- expected: ${expectF16.toString(16)}
-      got: ${res.toString(16)}`);
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
        const expect = metadata[j];
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
desc('Test allowed data types').
params((u) =>
u.
combine('op', kOps).
combine('type', keysOf(kDataTypes)).
beginSubcases().
combine('idx1', [0, 1, 2, 3]).
combine('idx2', [0, 1, 2, 3]).
combine('idx1Id', [0, 1, 2, 3])
).
fn(async (t) => {
  const wgSize = [4, 1, 1];
  const type = kDataTypes[t.params.type];
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
var<storage, read_write> metadata : array<${type.toString()}>;

@compute @workgroup_size(${wgSize[0]}, ${wgSize[1]}, ${wgSize[2]})
fn main(
  @builtin(subgroup_invocation_id) id : u32,
) {
  let value = select(input[${t.params.idx2}], input[${t.params.idx1}], id == ${t.params.idx1Id});
  output[id] = ${t.params.op}(value);

  if (id == 0) {
    metadata[0] = ${t.params.op === 'subgroupMin' ? 'min' : 'max'}(input[${t.params.idx1}], input[${
  t.params.idx2
  }]);
  }
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
      return checkDataTypes(metadata, output, type);
    }
  );
});

/**
 * Returns a Uint32Array of randomized integers in the range [0, 2**30)
 *
 * @param seed The PRNG seed
 * @param num The number of integers to generate
 */
function generateInputData(seed, num) {
  const prng = new PRNG(seed);
  return new Uint32Array([
  ...iterRange(num, (x) => {
    return prng.uniformInt(1 << 30);
  })]
  );
}

/**
 * Checks results from compute shaders
 *
 * @param metadata An array of uint32s containing:
 *                 * subgroup_invocation_id
 *                 * generated unique subgroup id
 * @param output An array of uint32s containing:
 *               * subgroup operation results
 *               * subgroup_size
 * @param input An array of uint32s used as input data
 * @param numInvs The number of invocations
 * @param op The subgroup operation
 * @param filter A functor for filtering active invocations
 */
function checkCompute(
metadata,
output,
input,
numInvs,
op,
filter)
{
  const identity = op === 'subgroupMin' ? 0x7fffffff : 0;
  const func = op === 'subgroupMin' ? Math.min : Math.max;
  const expected = new Map();
  for (let i = 0; i < numInvs; i++) {
    const id = metadata[i];
    const subgroup_id = metadata[numInvs + i];
    const size = output[numInvs + i];
    if (!filter(id, size)) {
      continue;
    }

    let e = expected.get(subgroup_id) ?? identity;
    e = func(e, input[i]);
    expected.set(subgroup_id, e);
  }

  for (let i = 0; i < numInvs; i++) {
    const id = metadata[i];
    const subgroup_id = metadata[numInvs + i];
    const size = output[numInvs + i];
    if (!filter(id, size)) {
      continue;
    }

    const res = output[i];
    const e = expected.get(subgroup_id) ?? identity;
    if (res !== e) {
      return new Error(`Invocation ${i}: incorrect result
- expected: ${e}
-      got: ${res}`);
    }
  }

  return undefined;
}

const kNumRandomCases = 15;

g.test('compute,all_active').
desc(
  'Test subgroupMin/Max in compute shader with all active invocations and varied workgroup sizes'
).
params((u) =>
u.
combine('op', kOps).
combine('wgSize', kWGSizes).
beginSubcases().
combine('case', [...iterRange(kNumRandomCases, (x) => x)])
).
fn(async (t) => {
  t.skipIfDeviceDoesNotHaveFeature('subgroups');
  const wgThreads = t.params.wgSize[0] * t.params.wgSize[1] * t.params.wgSize[2];

  const wgsl = `
enable subgroups;

@group(0) @binding(0)
var<storage> input : array<u32>;

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
  output.res[lid] = ${t.params.op}(input[lid]);
}`;

  const inputData = generateInputData(t.params.case, wgThreads);
  const uintsPerOutput = 2;
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
        inputData,
        wgThreads,
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
combine('op', kOps).
combine('predicate', keysOf(kPredicateCases)).
beginSubcases().
combine('wgSize', kWGSizes).
combine('case', [...iterRange(kNumRandomCases, (x) => x)])
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
  if ${testcase.cond} {
    output.res[lid] = ${t.params.op}(input[lid]);
  } else {
    return;
  }
}`;

  const inputData = generateInputData(t.params.case, wgThreads);
  const uintsPerOutput = 2;
  await runComputeTest(
    t,
    wgsl,
    [t.params.wgSize[0], t.params.wgSize[1], t.params.wgSize[2]],
    uintsPerOutput,
    inputData,
    (metadata, output) => {
      return checkCompute(metadata, output, inputData, wgThreads, t.params.op, testcase.filter);
    }
  );
});

/**
 * Checks min/max ops results from a fragment shader.
 *
 * Avoids subgroups in last row or column to skip potential helper invocations.
 * @param data Framebuffer output
 *             * component 0 is result
 *             * component 1 is generated subgroup id
 * @param input An array of input data
 * @param op The subgroup operation
 * @param format The framebuffer format
 * @param width Framebuffer width
 * @param height Framebuffer height
 */
function checkFragment(
data,
input,
op,
format,
width,
height)
{
  const { uintsPerRow, uintsPerTexel } = getUintsPerFramebuffer(format, width, height);

  // Determine if the subgroup should be included in the checks.
  const inBounds = new Map();
  for (let row = 0; row < height; row++) {
    for (let col = 0; col < width; col++) {
      const offset = uintsPerRow * row + col * uintsPerTexel;
      const subgroup_id = data[offset + 1];
      if (subgroup_id === 0) {
        return new Error(`Internal error: helper invocation at (${col}, ${row})`);
      }

      let ok = inBounds.get(subgroup_id) ?? true;
      ok = ok && row !== height - 1 && col !== width - 1;
      inBounds.set(subgroup_id, ok);
    }
  }

  let anyInBounds = false;
  for (const [_, value] of inBounds) {
    const ok = Boolean(value);
    anyInBounds = anyInBounds || ok;
  }
  if (!anyInBounds) {
    // This variant would not reliably test behavior.
    return undefined;
  }

  const identity = op === 'subgroupMin' ? 0x7fffffff : 0;

  // Iteration skips subgroups in the last row or column to avoid helper
  // invocations because it is not guaranteed whether or not they participate
  // in the subgroup operation.
  const expected = new Map();
  for (let row = 0; row < height; row++) {
    for (let col = 0; col < width; col++) {
      const offset = uintsPerRow * row + col * uintsPerTexel;
      const subgroup_id = data[offset + 1];

      if (subgroup_id === 0) {
        return new Error(`Internal error: helper invocation at (${col}, ${row})`);
      }

      const subgroupInBounds = inBounds.get(subgroup_id) ?? true;
      if (!subgroupInBounds) {
        continue;
      }

      const func = op === 'subgroupMin' ? Math.min : Math.max;
      let v = expected.get(subgroup_id) ?? identity;
      v = func(v, input[row * width + col]);
      expected.set(subgroup_id, v);
    }
  }

  for (let row = 0; row < height; row++) {
    for (let col = 0; col < width; col++) {
      const offset = uintsPerRow * row + col * uintsPerTexel;
      const res = data[offset];
      const subgroup_id = data[offset + 1];

      if (subgroup_id === 0) {
        // Inactive in the fragment.
        continue;
      }

      const subgroupInBounds = inBounds.get(subgroup_id) ?? true;
      if (!subgroupInBounds) {
        continue;
      }

      const expected_v = expected.get(subgroup_id) ?? identity;
      if (expected_v !== res) {
        return new Error(`Row ${row}, col ${col}: incorrect results:
- expected: ${expected_v}
-      got: ${res}`);
      }
    }
  }

  return undefined;
}

g.test('fragment').
desc('Test subgroupMin/Max in fragment shaders').
params((u) =>
u.
combine('size', kFramebufferSizes).
combine('op', kOps).
beginSubcases().
combine('case', [...iterRange(kNumRandomCases, (x) => x)]).
combineWithParams([{ format: 'rg32uint' }])
).
fn(async (t) => {
  t.skipIfDeviceDoesNotHaveFeature('subgroups');
  const numInputs = t.params.size[0] * t.params.size[1];




  const { subgroupMinSize } = t.device.adapterInfo;
  const innerTexels = (t.params.size[0] - 1) * (t.params.size[1] - 1);
  t.skipIf(innerTexels < subgroupMinSize, 'Too few texels to be reliable');

  const inputData = generateInputData(t.params.case, numInputs);

  const identity = t.params.op === 'subgroupMin' ? 0x7fffffff : 0;
  const fsShader = `
enable subgroups;

@group(0) @binding(0)
var<uniform> inputs : array<vec4u, ${inputData.length}>;

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
  let input = select(${identity}, inputs[linear].x, in_range);

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
      return checkFragment(
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