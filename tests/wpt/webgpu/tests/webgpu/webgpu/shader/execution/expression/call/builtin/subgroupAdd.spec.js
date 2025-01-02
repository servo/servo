/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution tests for subgroupAdd, subgroupExclusiveAdd, and subgroupInclusiveAdd

Note: There is a lack of portability for non-uniform execution so these tests
restrict themselves to uniform control flow.
Note: There is no guaranteed mapping between subgroup_invocation_id and
local_invocation_index. Tests should avoid assuming there is.
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { keysOf, objectsToRecord } from '../../../../../../common/util/data_tables.js';
import { iterRange } from '../../../../../../common/util/util.js';
import { GPUTest } from '../../../../../gpu_test.js';
import {
  kConcreteNumericScalarsAndVectors,
  Type,
  VectorType,
  numberToFloatBits,
  floatBitsToNumber,
  kFloat32Format,
  kFloat16Format,
  scalarTypeOf } from
'../../../../../util/conversion.js';
import { FP } from '../../../../../util/floating_point.js';

import {
  kNumCases,
  kStride,
  kWGSizes,
  kPredicateCases,
  runAccuracyTest,
  runComputeTest } from
'./subgroup_util.js';

export const g = makeTestGroup(GPUTest);

const kIdentity = 0;

const kDataTypes = objectsToRecord(kConcreteNumericScalarsAndVectors);

const kOperations = ['subgroupAdd', 'subgroupExclusiveAdd', 'subgroupInclusiveAdd'];

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
combine('wgSize', [
[kStride, 1, 1],
[kStride / 2, 2, 1]]
)
).
beforeAllSubcases((t) => {
  const features = ['subgroups'];
  if (t.params.type === 'f16') {
    features.push('shader-f16');
    features.push('subgroups-f16');
  }
  t.selectDeviceOrSkipTestCase(features);
}).
fn(async (t) => {
  await runAccuracyTest(
    t,
    t.params.case,
    [t.params.wgSize[0], t.params.wgSize[1], t.params.wgSize[2]],
    'subgroupAdd',
    t.params.type,
    kIdentity,
    t.params.type === 'f16' ? FP.f16.additionInterval : FP.f32.additionInterval
  );
});

/**
 * Checks subgroup additions
 *
 * Expected results:
 * - subgroupAdd: each invocation should have result equal to real subgroup size
 * - subgroupExclusiveAdd: each invocation should have result equal to its subgroup invocation id
 * - subgroupInclusiveAdd: each invocation should be equal to the result of subgroupExclusiveAdd plus the fill value
 * @param metadata An array containing actual subgroup size per invocation followed by
 *                 subgroup invocation id per invocation
 * @param output An array of additions
 * @param type The data type
 * @param operation Type of addition
 * @param expectedfillValue The original value used to fill the test array
 */
function checkAddition(
metadata,
output,
type,
operation,
expectedfillValue)
{
  let numEles = 1;
  if (type instanceof VectorType) {
    numEles = type.width;
  }
  const scalarTy = scalarTypeOf(type);
  const expectedOffset = operation === 'subgroupAdd' ? 0 : metadata.length / 2;
  for (let i = 0; i < metadata.length / 2; i++) {
    let expected = metadata[i + expectedOffset];
    if (operation === 'subgroupInclusiveAdd') {
      expected += expectedfillValue;
    }

    for (let j = 0; j < numEles; j++) {
      let idx = i * numEles + j;
      const isOdd = idx & 0x1;
      if (scalarTy === Type.f16) {
        idx = Math.floor(idx / 2);
      }
      let val = output[idx];
      if (scalarTy === Type.f32) {
        val = floatBitsToNumber(val, kFloat32Format);
      } else if (scalarTy === Type.f16) {
        if (isOdd) {
          val = val >> 16;
        }
        val = floatBitsToNumber(val & 0xffff, kFloat16Format);
      }
      if (expected !== val) {
        return new Error(`Invocation ${i}, component ${j}: incorrect result
- expected: ${expected}
-      got: ${val}`);
      }
    }
  }

  return undefined;
}

g.test('data_types').
desc(
  `Tests subgroup addition for valid data types

Tests a simple addition of all 1 values.
Reductions expect result to be equal to actual subgroup size.
Exclusice scans expect result to be equal subgroup invocation id.

TODO: support vec3 types.
  `
).
params((u) =>
u.
combine('type', keysOf(kDataTypes)).
filter((t) => {
  const type = kDataTypes[t.type];
  if (type instanceof VectorType) {
    return type.width !== 3;
  }
  return true;
}).
beginSubcases().
combine('wgSize', kWGSizes).
combine('operation', kOperations)
).
beforeAllSubcases((t) => {
  const features = ['subgroups'];
  const type = kDataTypes[t.params.type];
  if (type.requiresF16()) {
    features.push('shader-f16');
    features.push('subgroups-f16');
  }
  t.selectDeviceOrSkipTestCase(features);
}).
fn(async (t) => {
  const type = kDataTypes[t.params.type];
  let numEles = 1;
  if (type instanceof VectorType) {
    numEles = type.width;
  }
  const scalarType = scalarTypeOf(type);
  let enables = 'enable subgroups;\n';
  if (type.requiresF16()) {
    enables += 'enable f16;\nenable subgroups_f16;\n';
  }

  const wgThreads = t.params.wgSize[0] * t.params.wgSize[1] * t.params.wgSize[2];

  const wgsl = `
${enables}

@group(0) @binding(0)
var<storage> inputs : array<${type.toString()}>;

@group(0) @binding(1)
var<storage, read_write> outputs : array<${type.toString()}>;

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
) {
  // Record the actual subgroup size for this invocation.
  // Note: subgroup_size builtin is always a power-of-2 and might be larger
  // if the subgroup is not full.
  let ballot = subgroupBallot(true);
  var size = countOneBits(ballot.x);
  size += countOneBits(ballot.y);
  size += countOneBits(ballot.z);
  size += countOneBits(ballot.w);
  metadata.subgroup_size[lid] = size;

  // Record subgroup invocation id for this invocation.
  metadata.subgroup_invocation_id[lid] = id;

  outputs[lid] = ${t.params.operation}(inputs[lid]);
}`;
  const expectedFillValue = 1;
  let fillValue = expectedFillValue;
  let numUints = wgThreads * numEles;
  if (scalarType === Type.f32) {
    fillValue = numberToFloatBits(1, kFloat32Format);
  } else if (scalarType === Type.f16) {
    const f16 = numberToFloatBits(1, kFloat16Format);
    fillValue = f16 | f16 << 16;
    numUints = Math.ceil(numUints / 2);
  }
  await runComputeTest(
    t,
    wgsl,
    [t.params.wgSize[0], t.params.wgSize[1], t.params.wgSize[2]],
    numUints,
    new Uint32Array([...iterRange(numUints, (x) => fillValue)]),
    (metadata, output) => {
      return checkAddition(metadata, output, type, t.params.operation, expectedFillValue);
    }
  );
});

g.test('fragment').unimplemented();

/**
 * Performs correctness checking for predicated additions
 *
 * Assumes the shader performs a predicated subgroup addition with the
 * subgroup_invocation_id as the data.
 *
 * @param metadata An array containing subgroup sizes and subgroup invocation ids
 * @param output An array containing the output results
 * @param operation The type of addition
 * @param filter A functor that mirrors the predication in the shader
 */
function checkPredicatedAddition(
metadata,
output,
operation,
filter)
{
  for (let i = 0; i < output.length; i++) {
    const size = metadata[i];
    const id = metadata[output.length + i];
    let expected = 0;
    if (filter(id, size)) {
      const bound =
      operation === 'subgroupInclusiveAdd' ? id + 1 : operation === 'subgroupAdd' ? size : id;
      for (let j = 0; j < bound; j++) {
        if (filter(j, size)) {
          expected += j;
        }
      }
    } else {
      expected = 999;
    }
    if (expected !== output[i]) {
      return new Error(`Invocation ${i}: incorrect result
- expected: ${expected}
-      got: ${output[i]}`);
    }
  }
  return undefined;
}

g.test('compute,split').
desc('Tests that only active invocations contribute to the operation').
params((u) =>
u.
combine('case', keysOf(kPredicateCases)).
beginSubcases().
combine('operation', kOperations).
combine('wgSize', kWGSizes)
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('subgroups');
}).
fn(async (t) => {
  const testcase = kPredicateCases[t.params.case];
  const outputUintsPerElement = 1;
  const inputData = new Uint32Array([0]); // no input data
  const wgThreads = t.params.wgSize[0] * t.params.wgSize[1] * t.params.wgSize[2];

  const wgsl = `
enable subgroups;

@group(0) @binding(0)
var<storage> input : array<u32>;

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
) {
  _ = input[0];

  // Record the actual subgroup size for this invocation.
  // Note: subgroup_size builtin is always a power-of-2 and might be larger
  // if the subgroup is not full.
  let ballot = subgroupBallot(true);
  var subgroupSize = countOneBits(ballot.x);
  subgroupSize += countOneBits(ballot.y);
  subgroupSize += countOneBits(ballot.z);
  subgroupSize += countOneBits(ballot.w);
  metadata.subgroup_size[lid] = subgroupSize;

  // Record subgroup invocation id for this invocation.
  metadata.subgroup_invocation_id[lid] = id;

  if ${testcase.cond} {
    outputs[lid] = ${t.params.operation}(id);
  } else {
    return;
  }
}`;

  await runComputeTest(
    t,
    wgsl,
    [t.params.wgSize[0], t.params.wgSize[1], t.params.wgSize[2]],
    outputUintsPerElement,
    inputData,
    (metadata, output) => {
      return checkPredicatedAddition(metadata, output, t.params.operation, testcase.filter);
    }
  );
});