/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution tests for quadBroadcast.

Note: There is a lack of portability for non-uniform execution so these tests
restrict themselves to uniform control flow.
Note: There is no guaranteed mapping between subgroup_invocation_id and
local_invocation_index. Tests should avoid assuming there is.
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { keysOf, objectsToRecord } from '../../../../../../common/util/data_tables.js';
import { assert, unreachable } from '../../../../../../common/util/util.js';
import { kTextureFormatInfo } from '../../../../../format_info.js';
import { kBit } from '../../../../../util/constants.js';
import {
  kConcreteNumericScalarsAndVectors,
  Type,
  VectorType,
  scalarTypeOf } from
'../../../../../util/conversion.js';
import { align } from '../../../../../util/math.js';

import {
  kWGSizes,
  kDataSentinel,
  kPredicateCases,
  runComputeTest,
  SubgroupTest,
  kFramebufferSizes,
  runFragmentTest } from
'./subgroup_util.js';

export const g = makeTestGroup(SubgroupTest);

const kTypes = objectsToRecord(kConcreteNumericScalarsAndVectors);

/**
 * Generates scalar values for type
 *
 * Generates 4 32-bit values whose bit patterns represent
 * interesting values of the data type.
 * @param type The data type
 */
function generateScalarValues(type) {
  const scalarTy = scalarTypeOf(type);
  switch (scalarTy) {
    case Type.u32:
      return [kBit.u32.min, kBit.u32.max, 1111, 2222];
    case Type.i32:
      return [
      kBit.i32.positive.min,
      kBit.i32.positive.max,
      kBit.i32.negative.min,
      0xffffffff // -1
      ];
    case Type.f32:
      return [
      kBit.f32.positive.zero,
      kBit.f32.positive.nearest_max,
      kBit.f32.negative.nearest_min,
      0xbf800000 // -1
      ];
    case Type.f16:
      return [
      kBit.f16.positive.zero,
      kBit.f16.positive.nearest_max,
      kBit.f16.negative.nearest_min,
      0xbc00 // -1
      ];
    default:
      unreachable(`Unsupported type: ${type.toString()}`);
  }
  return [0, 0, 0, 0];
}

/**
 * Generates input bit patterns for the input type
 *
 * Generates 4 values of type in a Uint32Array.
 * 16-bit types are appropriately packed.
 * @param type The data type
 */
function generateTypedInputs(type) {
  const scalarValues = generateScalarValues(type);
  let elements = 1;
  if (type instanceof VectorType) {
    elements = type.width;
  }
  if (type.requiresF16()) {
    switch (elements) {
      case 1:
        return new Uint32Array([
        scalarValues[0] | scalarValues[1] << 16,
        scalarValues[2] | scalarValues[3] << 16]
        );
      case 2:
        return new Uint32Array([
        scalarValues[0] | scalarValues[0] << 16,
        scalarValues[1] | scalarValues[1] << 16,
        scalarValues[2] | scalarValues[2] << 16,
        scalarValues[3] | scalarValues[3] << 16]
        );
      case 3:
        return new Uint32Array([
        scalarValues[0] | scalarValues[0] << 16,
        scalarValues[0] | kDataSentinel << 16,
        scalarValues[1] | scalarValues[1] << 16,
        scalarValues[1] | kDataSentinel << 16,
        scalarValues[2] | scalarValues[2] << 16,
        scalarValues[2] | kDataSentinel << 16,
        scalarValues[3] | scalarValues[3] << 16,
        scalarValues[3] | kDataSentinel << 16]
        );
      case 4:
        return new Uint32Array([
        scalarValues[0] | scalarValues[0] << 16,
        scalarValues[0] | scalarValues[0] << 16,
        scalarValues[1] | scalarValues[1] << 16,
        scalarValues[1] | scalarValues[1] << 16,
        scalarValues[2] | scalarValues[2] << 16,
        scalarValues[2] | scalarValues[2] << 16,
        scalarValues[3] | scalarValues[3] << 16,
        scalarValues[3] | scalarValues[3] << 16]
        );
      default:
        unreachable(`Unsupported type: ${type.toString()}`);
    }
    return new Uint32Array([0]);
  } else {
    const bound = elements === 3 ? 4 : elements;
    const values = [];
    for (let i = 0; i < 4; i++) {
      for (let j = 0; j < bound; j++) {
        if (j < elements) {
          values.push(scalarValues[i]);
        } else {
          values.push(kDataSentinel);
        }
      }
    }
    return new Uint32Array(values);
  }
}

/**
 * Checks results from data types test
 *
 * The output is expected to match the input values corresponding to the
 * id being broadcast (assuming a linear mapping).
 * @param metadata An unused parameter
 * @param output The output data
 * @param input The input data
 * @param broadcast The id being broadcast
 * @param type The data type being tested
 */
function checkDataTypes(
metadata, // unused
output,
input,
broadcast,
type)
{
  if (type.requiresF16() && !(type instanceof VectorType)) {
    const expectIdx = Math.floor(broadcast / 2);
    const expectShift = broadcast % 2 === 1;
    let expect = input[expectIdx];
    if (expectShift) {
      expect >>= 16;
    }
    expect &= 0xffff;

    for (let i = 0; i < 4; i++) {
      const index = Math.floor(i / 2);
      const shift = i % 2 === 1;
      let res = output[index];
      if (shift) {
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
        const expect = input[broadcast * uints + j];
        const res = output[i * uints + j];
        if (res !== expect) {
          return new Error(`${i * uints + j}: incorrect result
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
combine('type', keysOf(kTypes)).
beginSubcases().
combine('id', [0, 1, 2, 3])
).
beforeAllSubcases((t) => {
  const features = ['subgroups'];
  const type = kTypes[t.params.type];
  if (type.requiresF16()) {
    features.push('subgroups-f16');
    features.push('shader-f16');
  }
  t.selectDeviceOrSkipTestCase(features);
}).
fn(async (t) => {
  const wgSize = [4, 1, 1];
  const type = kTypes[t.params.type];
  let enables = `enable subgroups;\n`;
  if (type.requiresF16()) {
    enables += `enable f16;\nenable subgroups_f16;`;
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

  output[id] = quadBroadcast(input[id], ${t.params.id});
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

/**
 * Checks quadBroadcast in compute shaders
 *
 * Assumes that quads are linear within a subgroup.
 *
 * @param metadata An array of integers divided as follows:
 *                 * first half subgroup invocation ids
 *                 * second half subgroup sizes
 * @param output An array of integers divided as follows:
 *               * first half results of quad broadcast
 *               * second half generated unique subgroup ids
 * @param broadcast The id being broadcast in the range [0, 3]
 * @param filter A functor to filter active invocations
 */
function checkBroadcastCompute(
metadata,
output,
broadcast,
filter)
{
  assert(broadcast === Math.trunc(broadcast));
  assert(broadcast >= 0 && broadcast <= 3);

  const bound = Math.floor(output.length / 2);
  for (let i = 0; i < bound; i++) {
    const subgroup_id = output[bound + i];
    const id = metadata[i];
    const size = metadata[bound + i];
    if (!filter(id, size)) {
      if (output[i] !== kDataSentinel) {
        return new Error(`Unexpected write for invocation ${i}`);
      }
      continue;
    }

    const quad_id = Math.floor(id / 4);
    const quad = [-1, -1, -1, -1];
    for (let j = 0; j < bound; j++) {
      const other_id = metadata[j];
      const other_quad_id = Math.floor(other_id / 4);
      const other_quad_index = other_id % 4;
      const other_subgroup_id = output[bound + j];
      if (other_subgroup_id === subgroup_id && quad_id === other_quad_id) {
        quad[other_quad_index] = j;
      }
    }
    for (let j = 0; j < 4; j++) {
      if (quad[j] === -1) {
        return new Error(`Invocation ${i}: missing quad index ${j}`);
      }
    }
    for (let j = 0; j < 4; j++) {
      if (output[quad[j]] !== output[quad[broadcast]]) {
        return new Error(`Incorrect result for quad: base invocation = ${
        quad[broadcast]
        }, invocation = ${quad[j]}
- expected: ${output[quad[broadcast]]}
-      got: ${output[quad[j]]}`);
      }
    }
  }

  return undefined;
}

g.test('compute,all_active').
desc(
  `Tests broadcast with all active invocations

Quad operations require a full quad so workgroup sizes are limited to multiples of 4.
  `
).
params((u) =>
u.
combine('wgSize', kWGSizes).
filter((t) => {
  const wgThreads = t.wgSize[0] * t.wgSize[1] * t.wgSize[2];
  return wgThreads % 4 === 0;
}).
beginSubcases().
combine('id', [0, 1, 2, 3])
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('subgroups');
}).
fn(async (t) => {
  const wgThreads = t.params.wgSize[0] * t.params.wgSize[1] * t.params.wgSize[2];

  const wgsl = `
enable subgroups;

@group(0) @binding(0)
var<storage> inputs : u32; // unused

struct Output {
  results : array<u32, ${wgThreads}>,
  subgroup_size : array<u32, ${wgThreads}>,
}

@group(0) @binding(1)
var<storage, read_write> output : Output;

struct Metadata {
  id : array<u32, ${wgThreads}>,
  subgroup_size : array<u32, ${wgThreads}>,
}

@group(0) @binding(2)
var<storage, read_write> metadata : Metadata;

@compute @workgroup_size(${t.params.wgSize[0]}, ${t.params.wgSize[1]}, ${t.params.wgSize[2]})
fn main(
  @builtin(local_invocation_index) lid : u32,
  @builtin(subgroup_invocation_id) id : u32,
  @builtin(subgroup_size) subgroupSize : u32,
) {
  // Force usage
  _ = inputs;

  let b = quadBroadcast(lid, ${t.params.id});
  output.results[lid] = b;
  output.subgroup_size[lid] = subgroupBroadcastFirst(lid + 1);
  metadata.id[lid] = id;
  metadata.subgroup_size[lid] = subgroupSize;
}`;

  const uintsPerOutput = 2;
  await runComputeTest(
    t,
    wgsl,
    [t.params.wgSize[0], t.params.wgSize[1], t.params.wgSize[2]],
    uintsPerOutput,
    new Uint32Array([0]), // unused
    (metadata, output) => {
      return checkBroadcastCompute(metadata, output, t.params.id, (id, size) => {
        return true;
      });
    }
  );
});

g.test('compute,split').
desc(
  `Tests broadcast with predicated invocations

Quad operations require a full quad so workgroup sizes are limited to multiples of 4.
Quad operations require a fully active quad to operate correctly so several of the
predication filters are skipped.
  `
).
params((u) =>
u.
combine('predicate', keysOf(kPredicateCases)).
filter((t) => {
  return t.predicate === 'lower_half' || t.predicate === 'upper_half';
}).
combine('wgSize', kWGSizes).
filter((t) => {
  const wgThreads = t.wgSize[0] * t.wgSize[1] * t.wgSize[2];
  return wgThreads % 4 === 0;
}).
beginSubcases().
combine('id', [0, 1, 2, 3])
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('subgroups');
}).
fn(async (t) => {
  const wgThreads = t.params.wgSize[0] * t.params.wgSize[1] * t.params.wgSize[2];
  const testcase = kPredicateCases[t.params.predicate];

  const wgsl = `
enable subgroups;

@group(0) @binding(0)
var<storage> inputs : u32; // unused

struct Output {
  results : array<u32, ${wgThreads}>,
  subgroup_size : array<u32, ${wgThreads}>,
}

@group(0) @binding(1)
var<storage, read_write> output : Output;

struct Metadata {
  id : array<u32, ${wgThreads}>,
  subgroup_size : array<u32, ${wgThreads}>,
}

@group(0) @binding(2)
var<storage, read_write> metadata : Metadata;

@compute @workgroup_size(${t.params.wgSize[0]}, ${t.params.wgSize[1]}, ${t.params.wgSize[2]})
fn main(
  @builtin(local_invocation_index) lid : u32,
  @builtin(subgroup_invocation_id) id : u32,
  @builtin(subgroup_size) subgroupSize : u32,
) {
  // Force usage
  _ = inputs;

  output.subgroup_size[lid] = subgroupBroadcastFirst(lid + 1);
  metadata.id[lid] = id;
  metadata.subgroup_size[lid] = subgroupSize;

  if ${testcase.cond} {
    let b = quadBroadcast(lid, ${t.params.id});
    output.results[lid] = b;
  }
}`;

  const uintsPerOutput = 2;
  await runComputeTest(
    t,
    wgsl,
    [t.params.wgSize[0], t.params.wgSize[1], t.params.wgSize[2]],
    uintsPerOutput,
    new Uint32Array([0]), // unused
    (metadata, output) => {
      return checkBroadcastCompute(metadata, output, t.params.id, testcase.filter);
    }
  );
});

/**
 * Checks results of quadBroadcast in fragment shaders.
 *
 * @param data The framebuffer output
 *             * component 0 is the broadcast of the integer x position
 *             * component 1 is the broadcast of the integer y position
 * @param format The framebuffer format
 * @param width Framebuffer width
 * @param height Framebuffer height
 * @param broadcast The quad id being broadcast
 */
function checkFragment(
data,
format,
width,
height,
broadcast)
{
  assert(broadcast === Math.trunc(broadcast));
  assert(broadcast >= 0 && broadcast <= 3);

  if (width < 3 || height < 3) {
    return new Error(
      `Insufficient framebuffer size [${width}w x ${height}h]. Minimum is [3w x 3h].`
    );
  }

  const { blockWidth, blockHeight, bytesPerBlock } = kTextureFormatInfo[format];
  const blocksPerRow = width / blockWidth;
  // 256 minimum comes from image copy requirements.
  const bytesPerRow = align(blocksPerRow * (bytesPerBlock ?? 1), 256);
  const uintsPerRow = bytesPerRow / 4;
  const uintsPerTexel = (bytesPerBlock ?? 1) / blockWidth / blockHeight / 4;

  const coordToIndex = (row, col) => {
    return uintsPerRow * row + col * uintsPerTexel;
  };

  // Iteration skips last row and column to avoid helper invocations because it is not
  // guaranteed whether or not they participate in the subgroup operation.
  for (let row = 0; row < height - 1; row++) {
    for (let col = 0; col < width - 1; col++) {
      const offset = coordToIndex(row, col);

      const row_is_odd = row % 2 === 1;
      const col_is_odd = col % 2 === 1;

      // Skip checking quads that extend into potential helper invocations.
      const max_row = row_is_odd ? row : row + 1;
      const max_col = col_is_odd ? col : col + 1;
      if (max_row === height - 1 || max_col === width - 1) {
        continue;
      }

      let expect_row = row;
      let expect_col = col;
      switch (broadcast) {
        case 0:
          expect_row = row_is_odd ? row - 1 : row;
          expect_col = col_is_odd ? col - 1 : col;
          break;
        case 1:
          expect_row = row_is_odd ? row - 1 : row;
          expect_col = col_is_odd ? col : col + 1;
          break;
        case 2:
          expect_row = row_is_odd ? row : row + 1;
          expect_col = col_is_odd ? col - 1 : col;
          break;
        case 3:
          expect_row = row_is_odd ? row : row + 1;
          expect_col = col_is_odd ? col : col + 1;
          break;
      }

      const row_broadcast = data[offset + 1];
      const col_broadcast = data[offset];
      if (expect_row !== row_broadcast) {
        return new Error(`Row ${row}, col ${col}: incorrect row results:
- expected: ${expect_row}
-      got: ${row_broadcast}`);
      }

      if (expect_col !== col_broadcast) {
        return new Error(`Row ${row}, col ${col}: incorrect col results:
- expected: ${expect_row}
-      got: ${col_broadcast}`);
      }
    }
  }

  return undefined;
}

g.test('fragment,all_active').
desc(`Tests quadBroadcast in fragment shaders`).
params((u) =>
u.
combine('size', kFramebufferSizes).
beginSubcases().
combine('id', [0, 1, 2, 3]).
combineWithParams([{ format: 'rgba32uint' }])
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('subgroups');
}).
fn(async (t) => {
  const fsShader = `
enable subgroups;

@group(0) @binding(0)
var<storage, read_write> inputs : array<u32>; // unused

@fragment
fn main(
  @builtin(position) pos : vec4f,
) -> @location(0) vec4u {
  // Force usage
  _ = inputs[0];

  let linear = u32(pos.x) + u32(pos.y) * ${t.params.size[0]};

  // Filter out possible helper invocations.
  let x_in_range = u32(pos.x) < (${t.params.size[0]} - 1);
  let y_in_range = u32(pos.y) < (${t.params.size[1]} - 1);
  let in_range = x_in_range && y_in_range;

  var x_broadcast = select(1001, u32(pos.x), in_range);
  var y_broadcast = select(1001, u32(pos.y), in_range);

  x_broadcast = quadBroadcast(x_broadcast, ${t.params.id});
  y_broadcast = quadBroadcast(y_broadcast, ${t.params.id});

  return vec4u(x_broadcast, y_broadcast, 0, 0);
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
        t.params.id
      );
    }
  );
});

g.test('fragment,split').unimplemented();