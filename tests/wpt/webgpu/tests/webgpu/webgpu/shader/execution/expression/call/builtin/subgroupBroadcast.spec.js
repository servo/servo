/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution tests for subgroupBroadcast

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
  elementTypeOf } from
'../../../../../util/conversion.js';

export const g = makeTestGroup(GPUTest);

const kDataTypes = objectsToRecord(kConcreteNumericScalarsAndVectors);

const kWGSizes = [
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
[3, 3, 15]];


g.test('data_types').
desc('Tests broadcast of data types').
params((u) =>
u.
combine('type', keysOf(kDataTypes)).
filter((t) => {
  // Skip vec3h for simplicity
  const type = kDataTypes[t.type];
  return type !== Type['vec3h'];
}).
beginSubcases().
combine('id', [0, 1, 2, 3]).
combine('wgSize', kWGSizes)
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
  let enables = 'enable subgroups;\n';
  if (type.requiresF16()) {
    enables += 'enable f16;\nenable subgroups_f16;\n';
  }

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

  let outputType = Type.u32;
  if (type instanceof VectorType) {
    outputType = Type['vec'](type.width, outputType);
  }

  let updates = `_ = atomicAdd(&output[b], 1);`;
  if (type instanceof VectorType) {
    updates = ``;
    for (let i = 0; i < type.width; i++) {
      updates += `_ = atomicAdd(&output[b[${i}]], 1);\n`;
    }
  }

  // This test should be expanded to cover large input values for each type instead of just conversions to u32.
  const wgsl = `
${enables}

@group(0) @binding(0)
var<storage, read_write> size : u32;

@group(0) @binding(1)
var<storage, read_write> output : array<atomic<u32>>;

@compute @workgroup_size(${t.params.wgSize[0]}, ${t.params.wgSize[1]}, ${t.params.wgSize[2]})
fn main(@builtin(local_invocation_index) lid : u32,
        @builtin(subgroup_invocation_id) id : u32,
        @builtin(subgroup_size) subgroupSize : u32) {
  let scalar = ${elementTypeOf(type).toString()}(id);
  let v = ${type.toString()}(scalar);
  let b = ${outputType.toString()}(subgroupBroadcast(v, ${t.params.id}));
  ${updates}
  if (lid == 0) {
    size = subgroupSize;
  }
}`;

  const sizeBuffer = t.makeBufferWithContents(
    new Uint32Array([0]),
    GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST | GPUBufferUsage.STORAGE
  );
  t.trackForCleanup(sizeBuffer);

  const outputNumInts = wgThreads;
  const outputBuffer = t.makeBufferWithContents(
    new Uint32Array([...iterRange(outputNumInts, (x) => 0)]),
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
        buffer: sizeBuffer
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

  const sizeReadback = await t.readGPUBufferRangeTyped(sizeBuffer, {
    srcByteOffset: 0,
    type: Uint32Array,
    typedLength: 1,
    method: 'copy'
  });
  const subgroupSize = sizeReadback.data[0];

  let width = 1;
  if (type instanceof VectorType) {
    width = type.width;
  }

  const expect = Array(wgThreads);
  expect.fill(0);

  const numFullSubgroups = Math.floor(wgThreads / subgroupSize);
  const partialSize = wgThreads % subgroupSize;
  const numSubgroups = Math.ceil(wgThreads / subgroupSize);
  for (let i = 0; i < numSubgroups; i++) {
    if (i < numFullSubgroups) {
      expect[t.params.id] += width * subgroupSize;
    } else {
      expect[t.params.id] += width * partialSize;
    }
  }
  t.expectGPUBufferValuesEqual(outputBuffer, new Uint32Array(expect));
});

g.test('workgroup_uniform_load').
desc('Tests a workgroup uniform load equivalent').
params((u) =>
u.
combine('wgSize', kWGSizes).
beginSubcases().
combine('inputId', [1, 2, 3])
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('subgroups');
}).
fn((t) => {
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

  const wgsl = `
enable subgroups;

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
  v = subgroupBroadcast(v, 0);
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

g.test('fragment').unimplemented();