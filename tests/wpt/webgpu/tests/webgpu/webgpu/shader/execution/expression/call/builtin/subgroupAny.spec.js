/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution tests for subgroupAny.

Note: There is a lack of portability for non-uniform execution so these tests
restrict themselves to uniform control flow.
Note: There is no guaranteed mapping between subgroup_invocation_id and
local_invocation_index. Tests should avoid assuming there is.
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { keysOf } from '../../../../../../common/util/data_tables.js';
import { iterRange } from '../../../../../../common/util/util.js';
import { kTextureFormatInfo } from '../../../../../format_info.js';
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

/**
 * Generate input data for testing.
 *
 * Data is generated in the following categories:
 * Seed 0 generates all 0 data
 * Seed 1 generates all 1 data
 * Seeds 2-9 generates all 0s except for a one randomly once per 32 elements
 * Seeds 10+ generate all random data
 * @param seed The seed for the PRNG
 * @param num The number of data items to generate
 */
function generateInputData(seed, num) {
  const prng = new PRNG(seed);

  const bound = Math.min(num, 32);
  const index = prng.uniformInt(bound);

  return new Uint32Array([
  ...iterRange(num, (x) => {
    if (seed === 0) {
      return 0;
    } else if (seed === 1) {
      return 1;
    } else if (seed < 10) {
      const bounded = x % bound;
      return bounded === index ? 1 : 0;
    }
    return prng.uniformInt(2);
  })]
  );
}

/**
 * Checks the result of a subgroupAny operation
 *
 * Since subgroup size depends on the pipeline compile, we calculate the expected
 * results after execution. The shader generates a subgroup id and records it for
 * each invocation. The check first calculates the expected result for each subgroup
 * and then compares to the actual result for each invocation. The filter functor
 * ensures only the correct invocations contribute to the calculation.
 * @param metadata An array of uints:
 *                 * first half containing subgroup sizes (from builtin value)
 *                 * second half subgroup invocation id
 * @param output An array of uints containing:
 *               * first half is the outputs of subgroupAny
 *               * second half is a generated subgroup id
 * @param numInvs Number of invocations executed
 * @param input The input data (equal size to output)
 * @param filter A functor to filter active invocations
 */
function checkAny(
metadata, // unused
output,
numInvs,
input,
filter)
{
  // First, generate expected results.
  const expected = new Map();
  for (let inv = 0; inv < numInvs; inv++) {
    const size = metadata[inv];
    const id = metadata[inv + numInvs];
    if (!filter(id, size)) {
      continue;
    }
    const subgroup_id = output[numInvs + inv];
    let v = expected.get(subgroup_id) ?? 0;
    v |= input[inv];
    expected.set(subgroup_id, v);
  }

  // Second, check against actual results.
  for (let inv = 0; inv < numInvs; inv++) {
    const size = metadata[inv];
    const id = metadata[inv + numInvs];
    const res = output[inv];
    if (filter(id, size)) {
      const subgroup_id = output[numInvs + inv];
      const expected_v = expected.get(subgroup_id) ?? 0;
      if (expected_v !== res) {
        return new Error(`Invocation ${inv}:
- expected: ${expected_v}
-      got: ${res}`);
      }
    } else {
      if (res !== kDataSentinel) {
        return new Error(`Invocation ${inv} unexpected write:
- subgroup invocation id: ${id}
-          subgroup size: ${size}`);
      }
    }
  }

  return undefined;
}

g.test('compute,all_active').
desc(`Test compute subgroupAny`).
params((u) =>
u.
combine('wgSize', kWGSizes).
beginSubcases().
combine('case', [...iterRange(kNumCases, (x) => x)])
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
  subgroup_size: array<u32, ${wgThreads}>,
  subgroup_invocation_id: array<u32, ${wgThreads}>,
}

@group(0) @binding(2)
var<storage, read_write> metadata : Metadata;

@compute @workgroup_size(${t.params.wgSize[0]}, ${t.params.wgSize[1]}, ${t.params.wgSize[2]})
fn main(
  @builtin(local_invocation_index) lid : u32,
  @builtin(subgroup_invocation_id) id : u32,
  @builtin(subgroup_size) subgroupSize : u32,
) {
  metadata.subgroup_size[lid] = subgroupSize;

  metadata.subgroup_invocation_id[lid] = id;

  // Record a representative subgroup id.
  outputs[lid + ${wgThreads}] = subgroupBroadcastFirst(lid);

  let res = select(0u, 1u, subgroupAny(bool(inputs[lid])));
  outputs[lid] = res;
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
      return checkAny(metadata, output, wgThreads, inputData, (id, size) => {
        return true;
      });
    }
  );
});

g.test('compute,split').
desc('Test that only active invocation participate').
params((u) =>
u.
combine('predicate', keysOf(kPredicateCases)).
beginSubcases().
combine('wgSize', kWGSizes).
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
  subgroup_size : array<u32, ${wgThreads}>,
  subgroup_invocation_id : array<u32, ${wgThreads}>,
}

@group(0) @binding(2)
var<storage, read_write> metadata : Metadata;

@compute @workgroup_size(${t.params.wgSize[0]}, ${t.params.wgSize[1]}, ${t.params.wgSize[2]})
fn main(
  @builtin(local_invocation_index) lid : u32,
  @builtin(subgroup_invocation_id) id : u32,
  @builtin(subgroup_size) subgroupSize : u32,
) {
  metadata.subgroup_size[lid] = subgroupSize;

  // Record subgroup invocation id for this invocation.
  metadata.subgroup_invocation_id[lid] = id;

  // Record a generated subgroup id.
  outputs[${wgThreads} + lid] = subgroupBroadcastFirst(lid);

  if ${testcase.cond} {
    outputs[lid] = select(0u, 1u, subgroupAny(bool(inputs[lid])));
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
      return checkAny(metadata, output, wgThreads, inputData, testcase.filter);
    }
  );
});

/**
 * Checks subgroupAny results from a fragment shader.
 *
 * @param data Framebuffer output
 *             * component 0 is result
 *             * component 1 is generated subgroup id
 * @param input An array of input data
 * @param format The framebuffer format
 * @param width Framebuffer width
 * @param height Framebuffer height
 */
function checkFragmentAny(
data,
input,
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

      let v = expected.get(subgroup_id) ?? 0;
      // First index of input is an atomic counter.
      v |= input[row * width + col];
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
desc('Tests subgroupAny in fragment shaders').
params((u) =>
u.
combine('size', kFramebufferSizes).
beginSubcases().
combine('case', [...iterRange(kNumCases, (x) => x)]).
combineWithParams([{ format: 'rg32uint' }])
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('subgroups');
}).
fn(async (t) => {
  const numInputs = t.params.size[0] * t.params.size[1];
  const inputData = generateInputData(t.params.case, numInputs);

  const fsShader = `
enable subgroups;

@group(0) @binding(0)
var<storage, read_write> inputs : array<u32>;

@fragment
fn main(
  @builtin(position) pos : vec4f,
) -> @location(0) vec2u {
  // Generate a subgroup id based on linearized position, but avoid 0.
  let linear = u32(pos.x) + u32(pos.y) * ${t.params.size[0]};
  var subgroup_id = linear + 1;
  subgroup_id = subgroupBroadcastFirst(subgroup_id);

  // Filter out possible helper invocations.
  let x_in_range = u32(pos.x) < (${t.params.size[0]} - 1);
  let y_in_range = u32(pos.y) < (${t.params.size[1]} - 1);
  let in_range = x_in_range && y_in_range;
  let input = select(0u, inputs[linear], in_range);

  let res = select(0u, 1u, subgroupAny(bool(input)));
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
      return checkFragmentAny(
        data,
        inputData,
        t.params.format,
        t.params.size[0],
        t.params.size[1]
      );
    }
  );
});

// Using subgroup operations in control with fragment shaders
// quickly leads to unportable behavior.
g.test('fragment,split').unimplemented();