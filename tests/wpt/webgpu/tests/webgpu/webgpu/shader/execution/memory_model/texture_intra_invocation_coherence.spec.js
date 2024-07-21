/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Test that read/write storage textures are coherent within an invocation.

Each invocation is assigned several random writing indices and a single
read index from among those. Writes are randomly predicated (except the
one corresponding to the read). Checks that an invocation can read data
it has written to the texture previously.
Does not test coherence between invocations

Some platform (e.g. Metal) require a fence call to make writes visible
to reads performed by the same invocation. These tests attempt to ensure
WebGPU implementations emit correct fence calls.`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { unreachable, iterRange } from '../../../../common/util/util.js';
import { GPUTest } from '../../../gpu_test.js';
import { PRNG } from '../../../util/prng.js';

const kRWStorageFormats = ['r32uint', 'r32sint', 'r32float'];
const kDimensions = ['1d', '2d', '2d-array', '3d'];

export const g = makeTestGroup(GPUTest);

function indexToCoord(dim) {
  switch (dim) {
    case '1d':{
        return `
fn indexToCoord(idx : u32) -> u32 {
  return idx;
}`;
      }
    case '2d':
    case '2d-array':{
        return `
fn indexToCoord(idx : u32) -> vec2u {
  return vec2u(idx % (wgx * num_wgs_x), idx / (wgx * num_wgs_x));
}`;
      }
    case '3d':{
        return `
fn indexToCoord(idx : u32) -> vec3u {
  return vec3u(idx % (wgx * num_wgs_x), idx / (wgx * num_wgs_x), 0);
}`;
      }
    default:{
        unreachable(`unhandled dimension: ${dim}`);
      }
  }
  return ``;
}

function textureType(format, dim) {
  let t = `texture_storage_`;
  switch (dim) {
    case '1d':{
        t += '1d';
        break;
      }
    case '2d':{
        t += '2d';
        break;
      }
    case '2d-array':{
        t += '2d_array';
        break;
      }
    case '3d':{
        t += '3d';
        break;
      }
    default:{
        unreachable(`unhandled dim: ${dim}`);
      }
  }
  t += `<${format}, read_write>`;
  return t;
}

function textureStore(dim, index) {
  let code = `textureStore(t, indexToCoord(${index}), `;
  if (dim === '2d-array') {
    code += `0, `;
  }
  code += `texel)`;
  return code;
}

function textureLoad(dim, format) {
  let code = `textureLoad(t, indexToCoord(read_index[global_index])`;
  if (dim === '2d-array') {
    code += `, 0`;
  }
  code += `).x`;
  if (format !== 'r32uint') {
    code = `u32(${code})`;
  }
  return code;
}

function texel(format) {
  switch (format) {
    case 'r32uint':{
        return 'vec4u(global_index,0,0,0)';
      }
    case 'r32sint':{
        return 'vec4i(i32(global_index),0,0,0)';
      }
    case 'r32float':{
        return 'vec4f(f32(global_index),0,0,0)';
      }
    default:{
        unreachable('unhandled format: ${format}');
      }
  }
  return '';
}

function getTextureSize(numTexels, dim) {
  const size = { width: 1, height: 1, depthOrArrayLayers: 1 };
  switch (dim) {
    case '1d':{
        size.width = numTexels;
        break;
      }
    case '2d':
    case '2d-array':
    case '3d':{
        size.width = numTexels / 2;
        size.height = numTexels / 2;
        // depthOrArrayLayers defaults to 1
        break;
      }
    default:{
        unreachable(`unhandled dim: ${dim}`);
      }
  }
  return size;
}

g.test('texture_intra_invocation_coherence').
desc(`Tests writes from an invocation are visible to reads from the same invocation`).
params((u) => u.combine('format', kRWStorageFormats).combine('dim', kDimensions)).
beforeAllSubcases((t) => {
  t.selectDeviceForTextureFormatOrSkipTestCase(t.params.format);
}).
fn((t) => {
  t.skipIfLanguageFeatureNotSupported('readonly_and_readwrite_storage_textures');

  const wgx = 16;
  const wgy = t.device.limits.maxComputeInvocationsPerWorkgroup / wgx;
  const num_wgs_x = 2;
  const num_wgs_y = 2;
  const invocations = wgx * wgy * num_wgs_x * num_wgs_y;
  const num_writes_per_invocation = 4;

  const code = `
requires readonly_and_readwrite_storage_textures;

@group(0) @binding(0)
var t : ${textureType(t.params.format, t.params.dim)};

@group(1) @binding(0)
var<storage> write_indices : array<vec4u>;

@group(1) @binding(1)
var<storage> read_index : array<u32>;

@group(1) @binding(2)
var<storage> write_mask : array<vec4u>;

@group(1) @binding(3)
var<storage, read_write> output : array<u32>;

const wgx = ${wgx}u;
const wgy = ${wgy}u;
const num_wgs_x = ${num_wgs_x}u;
const num_wgs_y = ${num_wgs_y}u;

${indexToCoord(t.params.dim)}

@compute @workgroup_size(wgx, wgy, 1)
fn main(@builtin(global_invocation_id) gid : vec3u) {
  let global_index = gid.x + gid.y * num_wgs_x * wgx;

  let write_index = write_indices[global_index];
  let mask = write_mask[global_index];
  let texel = ${texel(t.params.format)};

  if mask.x != 0 {
    ${textureStore(t.params.dim, 'write_index.x')};
  }
  if mask.y != 0 {
    ${textureStore(t.params.dim, 'write_index.y')};
  }
  if mask.z != 0 {
    ${textureStore(t.params.dim, 'write_index.z')};
  }
  if mask.w != 0 {
    ${textureStore(t.params.dim, 'write_index.w')};
  }
  output[global_index] = ${textureLoad(t.params.dim, t.params.format)};
}`;

  // To get a variety of testing, seed the random number generator based on which case this is.
  // This means subcases will not execute the same code.
  const seed =
  kRWStorageFormats.indexOf(t.params.format) * kRWStorageFormats.length +
  kDimensions.indexOf(t.params.dim);
  const prng = new PRNG(seed);

  const num_write_indices = invocations * num_writes_per_invocation;
  const write_indices = new Uint32Array([...iterRange(num_write_indices, (x) => x)]);
  const write_masks = new Uint32Array([...iterRange(num_write_indices, (x) => 0)]);
  // Shuffle the indices.
  for (let i = 0; i < num_write_indices; i++) {
    const remaining = num_write_indices - i;
    const swapIdx = prng.randomU32() % remaining + i;
    const tmp = write_indices[swapIdx];
    write_indices[swapIdx] = write_indices[i];
    write_indices[i] = tmp;

    // Assign random write masks
    const mask = prng.randomU32() % 2;
    write_masks[i] = mask;
  }
  const num_read_indices = invocations;
  const read_indices = new Uint32Array(num_read_indices);
  for (let i = 0; i < num_read_indices; i++) {
    // Pick a random index from index from this invocation's writes to read from.
    // Ensure that write is not masked out.
    const readIdx = prng.randomU32() % num_writes_per_invocation;
    read_indices[i] = write_indices[num_writes_per_invocation * i + readIdx];
    write_masks[num_writes_per_invocation * i + readIdx] = 1;
  }

  // Buffers
  const write_index_buffer = t.makeBufferWithContents(
    write_indices,
    GPUBufferUsage.COPY_SRC | GPUBufferUsage.STORAGE
  );
  const read_index_buffer = t.makeBufferWithContents(
    read_indices,
    GPUBufferUsage.COPY_SRC | GPUBufferUsage.STORAGE
  );
  const write_mask_buffer = t.makeBufferWithContents(
    write_masks,
    GPUBufferUsage.COPY_SRC | GPUBufferUsage.STORAGE
  );
  const output_buffer = t.makeBufferWithContents(
    new Uint32Array([...iterRange(invocations, (x) => 0)]),
    GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST | GPUBufferUsage.STORAGE
  );

  // Texture
  const texture_size = getTextureSize(invocations * num_writes_per_invocation, t.params.dim);
  const texture = t.createTextureTracked({
    format: t.params.format,
    dimension: t.params.dim === '2d-array' ? '2d' : t.params.dim,
    size: texture_size,
    usage: GPUTextureUsage.STORAGE_BINDING
  });

  const pipeline = t.device.createComputePipeline({
    layout: 'auto',
    compute: {
      module: t.device.createShaderModule({
        code
      }),
      entryPoint: 'main'
    }
  });

  const bg0 = t.device.createBindGroup({
    layout: pipeline.getBindGroupLayout(0),
    entries: [
    {
      binding: 0,
      resource: texture.createView({
        format: t.params.format,
        dimension: t.params.dim,
        mipLevelCount: 1,
        arrayLayerCount: 1
      })
    }]

  });
  const bg1 = t.device.createBindGroup({
    layout: pipeline.getBindGroupLayout(1),
    entries: [
    {
      binding: 0,
      resource: {
        buffer: write_index_buffer
      }
    },
    {
      binding: 1,
      resource: {
        buffer: read_index_buffer
      }
    },
    {
      binding: 2,
      resource: {
        buffer: write_mask_buffer
      }
    },
    {
      binding: 3,
      resource: {
        buffer: output_buffer
      }
    }]

  });

  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginComputePass();
  pass.setPipeline(pipeline);
  pass.setBindGroup(0, bg0);
  pass.setBindGroup(1, bg1);
  pass.dispatchWorkgroups(num_wgs_x, num_wgs_y, 1);
  pass.end();
  t.queue.submit([encoder.finish()]);

  const expectedOutput = new Uint32Array([...iterRange(num_read_indices, (x) => x)]);
  t.expectGPUBufferValuesEqual(output_buffer, expectedOutput);
});