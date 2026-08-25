/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution tests for subgroupElect
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { keysOf } from '../../../../../../common/util/data_tables.js';
import { iterRange } from '../../../../../../common/util/util.js';

import {
  SubgroupTest,
  kFramebufferSizes,
  getUintsPerFramebuffer,
  kWGSizes,
  kDataSentinel,
  runComputeTest,
  runFragmentTest,
  kPredicateCases } from
'./subgroup_util.js';

export const g = makeTestGroup(SubgroupTest);

/**
 * Checks subgroupElect compute shader results
 *
 * @param metadata An array of uint32s containing:
 *                 * subgroup_invocation_id in first half
 *                 * subgroup_size in second half
 * @param output An array of uint32s containing elect results
 * @param filter A functor to determine active invocations
 */
function checkCompute(
metadata,
output,
filter)
{
  const size = metadata[output.length];
  let elected = 129;
  for (let i = 0; i < 128; i++) {
    if (filter(i, size)) {
      elected = i;
      break;
    }
  }

  for (let i = 0; i < output.length; i++) {
    const res = output[i];
    const id = metadata[i];
    let expected = kDataSentinel;
    if (filter(id, size)) {
      expected = elected === id ? 1 : 0;
    }
    if (res !== expected) {
      return new Error(`Invocation ${i}: incorrect result
- expected: ${expected}
-      got: ${res}`);
    }
  }

  return undefined;
}

g.test('compute,all_active').
desc('Test subgroupElect in compute shader with all active invocations').
params((u) => u.combine('wgSize', kWGSizes)).
fn(async (t) => {
  t.skipIfDeviceDoesNotHaveFeature('subgroups');
  const wgThreads = t.params.wgSize[0] * t.params.wgSize[1] * t.params.wgSize[2];

  const wgsl = `
enable subgroups;

@group(0) @binding(0)
var<storage> inputs : array<u32>; // unused

@group(0) @binding(1)
var<storage, read_write> outputs : array<u32>;

struct Metadata {
  id : array<u32, ${wgThreads}>,
  size : array<u32, ${wgThreads}>,
}

@group(0) @binding(2)
var<storage, read_write> metadata : Metadata;

@compute @workgroup_size(${t.params.wgSize[0]}, ${t.params.wgSize[1]}, ${t.params.wgSize[2]})
fn main(
  @builtin(subgroup_invocation_id) id : u32,
  @builtin(subgroup_size) subgroupSize : u32,
  @builtin(local_invocation_index) lid : u32,
) {
  // Force usage.
  _ = inputs[0];

  let e = subgroupElect();
  outputs[lid] = select(0u, 1u, e);
  metadata.id[lid] = id;
  metadata.size[lid] = subgroupSize;
}`;

  const uintsPerOutput = 1;
  await runComputeTest(
    t,
    wgsl,
    [t.params.wgSize[0], t.params.wgSize[1], t.params.wgSize[2]],
    uintsPerOutput,
    new Uint32Array([0]), // unused
    (metadata, output) => {
      return checkCompute(metadata, output, (id, size) => {
        return true;
      });
    }
  );
});

g.test('compute,split').
desc('Test subgroupElect in compute shader with partially active invocations').
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
var<storage> inputs : array<u32>; // unused

@group(0) @binding(1)
var<storage, read_write> outputs : array<u32>;

struct Metadata {
  id : array<u32, ${wgThreads}>,
  size : array<u32, ${wgThreads}>,
}

@group(0) @binding(2)
var<storage, read_write> metadata : Metadata;

@compute @workgroup_size(${t.params.wgSize[0]}, ${t.params.wgSize[1]}, ${t.params.wgSize[2]})
fn main(
  @builtin(subgroup_invocation_id) id : u32,
  @builtin(subgroup_size) subgroupSize : u32,
  @builtin(local_invocation_index) lid : u32,
) {
  // Force usage.
  _ = inputs[0];

  metadata.id[lid] = id;
  metadata.size[lid] = subgroupSize;
  if ${testcase.cond} {
    let e = subgroupElect();
    outputs[lid] = select(0u, 1u, e);
  } else {
    return;
  }
}`;

  const uintsPerOutput = 1;
  await runComputeTest(
    t,
    wgsl,
    [t.params.wgSize[0], t.params.wgSize[1], t.params.wgSize[2]],
    uintsPerOutput,
    new Uint32Array([0]), // unused
    (metadata, output) => {
      return checkCompute(metadata, output, testcase.filter);
    }
  );
});

g.test('compute,each_invocation').
desc('Test subgroupElect in compute shader to elect each possible invocation').
params((u) =>
u.
combine('id', [...iterRange(128, (x) => x)]).
beginSubcases().
combine('wgSize', kWGSizes)
).
fn(async (t) => {
  t.skipIfDeviceDoesNotHaveFeature('subgroups');
  const wgThreads = t.params.wgSize[0] * t.params.wgSize[1] * t.params.wgSize[2];




  const { subgroupMaxSize } = t.device.adapterInfo;
  t.skipIf(subgroupMaxSize <= t.params.id, 'No invocation selected');

  const wgsl = `
enable subgroups;

diagnostic(off, subgroup_uniformity);

@group(0) @binding(0)
var<storage> inputs : array<u32>; // unused

@group(0) @binding(1)
var<storage, read_write> outputs : array<u32>;

struct Metadata {
  id : array<u32, ${wgThreads}>,
  size : array<u32, ${wgThreads}>,
}

@group(0) @binding(2)
var<storage, read_write> metadata : Metadata;

@compute @workgroup_size(${t.params.wgSize[0]}, ${t.params.wgSize[1]}, ${t.params.wgSize[2]})
fn main(
  @builtin(subgroup_invocation_id) id : u32,
  @builtin(subgroup_size) subgroupSize : u32,
  @builtin(local_invocation_index) lid : u32,
) {
  // Force usage.
  _ = inputs[0];

  metadata.id[lid] = id;
  metadata.size[lid] = subgroupSize;
  if id >= ${t.params.id} {
    let e = subgroupElect();
    outputs[lid] = select(0u, 1u, e);
  } else {
    return;
  }
}`;

  const uintsPerOutput = 1;
  await runComputeTest(
    t,
    wgsl,
    [t.params.wgSize[0], t.params.wgSize[1], t.params.wgSize[2]],
    uintsPerOutput,
    new Uint32Array([0]), // unused
    (metadata, output) => {
      return checkCompute(metadata, output, (id, size) => {
        return id >= t.params.id;
      });
    }
  );
});

/**
 * Checks subgroupElect results from a fragment shader.
 *
 * Avoids subgroups in last row or column to skip potential helper invocations.
 * @param data Framebuffer output
 *             * component 0 is result
 *             * component 1 is generated subgroup_invocation_id
 *             * component 2 is generated subgroup id
 * @param format The framebuffer format
 * @param width Framebuffer width
 * @param height Framebuffer height
 */
function checkFragment(
data,
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
      const subgroup_id = data[offset + 2];
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

  // Iteration skips subgroups in the last row or column to avoid helper
  // invocations because it is not guaranteed whether or not they participate
  // in the subgroup operation.
  for (let row = 0; row < height; row++) {
    for (let col = 0; col < width; col++) {
      const offset = uintsPerRow * row + col * uintsPerTexel;
      const subgroup_id = data[offset + 2];

      if (subgroup_id === 0) {
        return new Error(`Internal error: helper invocation at (${col}, ${row})`);
      }

      const subgroupInBounds = inBounds.get(subgroup_id) ?? true;
      if (!subgroupInBounds) {
        continue;
      }

      const res = data[offset];
      const id = data[offset + 1];
      const expected = id === 0 ? 0x55555555 : 0xaaaaaaaa;
      if (res !== expected) {
        return new Error(`Row ${row}, col ${col}: incorrect result
- expected: 0x${expected.toString(16)}
-      got: 0x${res.toString(16)}`);
      }
    }
  }

  return undefined;
}

g.test('fragment').
desc('Tests subgroupElect in fragment shaders').
params((u) =>
u.
combine('size', kFramebufferSizes).
beginSubcases().
combineWithParams([{ format: 'rgba32uint' }])
).
fn(async (t) => {
  t.skipIfDeviceDoesNotHaveFeature('subgroups');



  const { subgroupMinSize } = t.device.adapterInfo;
  const innerTexels = (t.params.size[0] - 1) * (t.params.size[1] - 1);
  t.skipIf(innerTexels < subgroupMinSize, 'Too few texels to be reliable');

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

  // Generate a subgroup id based on linearized position, avoid 0.
  let linear = u32(pos.x) + u32(pos.y) * ${t.params.size[0]};
  let subgroup_id = subgroupBroadcastFirst(linear + 1);

  let e = subgroupElect();
  let res = select(0xaaaaaaaau, 0x55555555u, e);
  return vec4u(res, id, subgroup_id, 0);
}`;

  await runFragmentTest(
    t,
    t.params.format,
    fsShader,
    t.params.size[0],
    t.params.size[1],
    new Uint32Array([0]), // unused,
    (data) => {
      return checkFragment(data, t.params.format, t.params.size[0], t.params.size[1]);
    }
  );
});