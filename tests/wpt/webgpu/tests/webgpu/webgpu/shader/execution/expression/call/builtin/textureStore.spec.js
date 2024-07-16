/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Writes a single texel to a texture.

The channel format T depends on the storage texel format F.
See the texel format table for the mapping of texel format to channel format.

Note: An out-of-bounds access occurs if:
 * any element of coords is outside the range [0, textureDimensions(t)) for the corresponding element, or
 * array_index is outside the range of [0, textureNumLayers(t))

If an out-of-bounds access occurs, the built-in function should not be executed.
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { unreachable, iterRange } from '../../../../../../common/util/util.js';
import { GPUTest, TextureTestMixin } from '../../../../../gpu_test.js';
import {
  kFloat32Format,
  kFloat16Format,
  numberToFloatBits,
  pack4x8unorm,
  pack4x8snorm } from
'../../../../../util/conversion.js';
import { virtualMipSize } from '../../../../../util/texture/base.js';
import { TexelFormats } from '../../../../types.js';

import { generateCoordBoundaries } from './utils.js';

export const g = makeTestGroup(TextureTestMixin(GPUTest));

g.test('store_1d_coords').
specURL('https://www.w3.org/TR/WGSL/#texturestore').
desc(
  `
C is i32 or u32

fn textureStore(t: texture_storage_1d<F,write>, coords: C, value: vec4<T>)

Parameters:
 * t  The sampled, depth, or external texture to sample.
 * s  The sampler type.
 * coords The texture coordinates used for sampling.
 * value The new texel value
`
).
params((u) =>
u.
combineWithParams(TexelFormats).
beginSubcases().
combine('coords', generateCoordBoundaries(1)).
combine('C', ['i32', 'u32'])
).
unimplemented();

g.test('store_2d_coords').
specURL('https://www.w3.org/TR/WGSL/#texturestore').
desc(
  `
C is i32 or u32

fn textureStore(t: texture_storage_2d<F,write>, coords: vec2<C>, value: vec4<T>)

Parameters:
 * t  The sampled, depth, or external texture to sample.
 * s  The sampler type.
 * coords The texture coordinates used for sampling.
 * value The new texel value
`
).
params((u) =>
u.
combineWithParams(TexelFormats).
beginSubcases().
combine('coords', generateCoordBoundaries(2)).
combine('C', ['i32', 'u32'])
).
unimplemented();

g.test('store_array_2d_coords').
specURL('https://www.w3.org/TR/WGSL/#texturestore').
desc(
  `
C is i32 or u32

fn textureStore(t: texture_storage_2d_array<F,write>, coords: vec2<C>, array_index: C, value: vec4<T>)

Parameters:
 * t  The sampled, depth, or external texture to sample.
 * s  The sampler type.
 * array_index The 0-based texture array index
 * coords The texture coordinates used for sampling.
 * value The new texel value
`
).
params(
  (u) =>
  u.
  combineWithParams(TexelFormats).
  beginSubcases().
  combine('coords', generateCoordBoundaries(2)).
  combine('C', ['i32', 'u32']).
  combine('C_value', [-1, 0, 1, 2, 3, 4])
  /* array_index not param'd as out-of-bounds is implementation specific */
).
unimplemented();

g.test('store_3d_coords').
specURL('https://www.w3.org/TR/WGSL/#texturestore').
desc(
  `
C is i32 or u32

fn textureStore(t: texture_storage_3d<F,write>, coords: vec3<C>, value: vec4<T>)

Parameters:
 * t  The sampled, depth, or external texture to sample.
 * s  The sampler type.
 * coords The texture coordinates used for sampling.
 * value The new texel value
`
).
params((u) =>
u.
combineWithParams(TexelFormats).
beginSubcases().
combine('coords', generateCoordBoundaries(3)).
combine('C', ['i32', 'u32'])
).
unimplemented();

// Returns shader input values for texel format tests.
// Values are intentionally simple to avoid rounding issues.
function inputArray(format) {
  switch (format) {
    case 'rgba8snorm':
      return [-1.1, 1.0, -0.6, -0.3, 0, 0.3, 0.6, 1.0, 1.1];
    case 'rgba8unorm':
    case 'bgra8unorm':
      return [-0.1, 0, 0.2, 0.4, 0.6, 0.8, 1.0, 1.1];
    case 'rgba8uint':
    case 'rgba16uint':
    case 'rgba32uint':
    case 'r32uint':
    case 'rg32uint':
      // Stick within 8-bit ranges for simplicity.
      return [0, 8, 16, 24, 32, 64, 100, 128, 200, 255];
    case 'rgba8sint':
    case 'rgba16sint':
    case 'rgba32sint':
    case 'r32sint':
    case 'rg32sint':
      // Stick within 8-bit ranges for simplicity.
      return [-128, -100, -64, -32, -16, -8, 0, 8, 16, 32, 64, 100, 127];
    case 'rgba16float':
    case 'rgba32float':
    case 'r32float':
    case 'rg32float':
      // Stick with simple values.
      return [-100, -50, -32, -16, -8, -1, 0, 1, 8, 16, 32, 50, 100];
    default:
      unreachable(`unhandled format ${format}`);
      break;
  }
  return [];
}

g.test('texel_formats').
desc(`Test storage of texel formats`).
params((u) => u.combineWithParams([...TexelFormats, { format: 'bgra8unorm', _shaderType: 'f32' }])).
beforeAllSubcases((t) => {
  if (t.params.format === 'bgra8unorm') {
    t.selectDeviceOrSkipTestCase('bgra8unorm-storage');
  } else {
    t.skipIfTextureFormatNotUsableAsStorageTexture(t.params.format);
  }
}).
fn((t) => {
  const { format, _shaderType } = t.params;
  const values = inputArray(format);

  let numChannels = 4;
  switch (format) {
    case 'r32uint':
    case 'r32sint':
    case 'r32float':
      numChannels = 1;
      break;
    case 'rg32uint':
    case 'rg32sint':
    case 'rg32float':
      numChannels = 2;
      break;
    default:
      break;
  }

  let zeroVal = ``;
  if (numChannels > 1) {
    zeroVal = `val[idx % ${numChannels}] = 0;`;
  }

  let wgsl = `
const range = array(`;
  for (const v of values) {
    wgsl += `${v},\n`;
  }

  wgsl += `
);

@group(0) @binding(0)
var tex : texture_storage_1d<${format}, write>;

@compute @workgroup_size(${values.length})
fn main(@builtin(global_invocation_id) gid : vec3u) {
  let idx = gid.x;
  let scalarVal = range[idx];
  let vecVal = vec4(scalarVal);
  var val = vec4<${_shaderType}>(vecVal);
  ${zeroVal}
  textureStore(tex, gid.x, val);
}
`;

  const numTexels = values.length;
  const textureSize = { width: numTexels, height: 1, depthOrArrayLayers: 1 };
  const texture = t.createTextureTracked({
    format: format,
    dimension: '1d',
    size: textureSize,
    mipLevelCount: 1,
    usage: GPUTextureUsage.STORAGE_BINDING | GPUTextureUsage.COPY_SRC
  });

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
      resource: texture.createView({
        format: format,
        dimension: '1d'
      })
    }]

  });

  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginComputePass();
  pass.setPipeline(pipeline);
  pass.setBindGroup(0, bg);
  pass.dispatchWorkgroups(1, 1, 1);
  pass.end();
  t.queue.submit([encoder.finish()]);

  let bytesPerTexel = 4;
  switch (format) {
    case 'rgba16uint':
    case 'rgba16sint':
    case 'rgba16float':
    case 'rg32uint':
    case 'rg32sint':
    case 'rg32float':
      bytesPerTexel = 8;
      break;
    case 'rgba32uint':
    case 'rgba32sint':
    case 'rgba32float':
      bytesPerTexel = 16;
      break;
    default:
      break;
  }

  let zeroChannel = 0;
  const buffer = t.copyWholeTextureToNewBufferSimple(texture, 0);
  const uintsPerTexel = bytesPerTexel / 4;
  const expected = new Uint32Array([
  ...iterRange(numTexels * uintsPerTexel, (x) => {
    const idx = Math.floor(x / uintsPerTexel);
    const channel = idx % numChannels;
    zeroChannel = zeroChannel % numChannels;
    const shaderVal = values[idx];
    switch (format) {
      case 'rgba8unorm':{
          const vals = [shaderVal, shaderVal, shaderVal, shaderVal];
          vals[zeroChannel++] = 0;
          return pack4x8unorm(vals[0], vals[1], vals[2], vals[3]);
        }
      case 'bgra8unorm':{
          const vals = [shaderVal, shaderVal, shaderVal, shaderVal];
          vals[zeroChannel++] = 0;
          return pack4x8unorm(vals[2], vals[1], vals[0], vals[3]);
        }
      case 'rgba8snorm':{
          const vals = [shaderVal, shaderVal, shaderVal, shaderVal];
          vals[zeroChannel++] = 0;
          return pack4x8snorm(vals[0], vals[1], vals[2], vals[3]);
        }
      case 'r32uint':
      case 'r32sint':
        return shaderVal;
      case 'rg32uint':
      case 'rgba32uint':
      case 'rg32sint':
      case 'rgba32sint':{
          const maskedVal = channel === zeroChannel++ ? 0 : shaderVal;
          return maskedVal;
        }
      case 'rgba8uint':
      case 'rgba8sint':{
          const vals = [shaderVal, shaderVal, shaderVal, shaderVal];
          vals[zeroChannel++] = 0;
          return (
            (vals[3] & 0xff) << 24 |
            (vals[2] & 0xff) << 16 |
            (vals[1] & 0xff) << 8 |
            vals[0] & 0xff);

        }
      case 'rgba16uint':
      case 'rgba16sint':{
          // 4 channels split over 2 uint32s.
          // Determine if this pair has the zero channel.
          const vals = [shaderVal, shaderVal];
          const lowChannels = (x & 0x1) === 0;
          if (lowChannels) {
            if (zeroChannel < 2) {
              vals[zeroChannel] = 0;
            }
          } else {
            if (zeroChannel >= 2) {
              vals[zeroChannel - 2] = 0;
            }
            zeroChannel++;
          }
          return (vals[1] & 0xffff) << 16 | vals[0] & 0xffff;
        }
      case 'r32float':{
          return numberToFloatBits(shaderVal, kFloat32Format);
        }
      case 'rg32float':
      case 'rgba32float':{
          const maskedVal = channel === zeroChannel++ ? 0 : shaderVal;
          return numberToFloatBits(maskedVal, kFloat32Format);
        }
      case 'rgba16float':{
          // 4 channels split over 2 uint32s.
          // Determine if this pair has the zero channel.
          const bits = numberToFloatBits(shaderVal, kFloat16Format);
          const vals = [bits, bits];
          const lowChannels = (x & 0x1) === 0;
          if (lowChannels) {
            if (zeroChannel < 2) {
              vals[zeroChannel] = 0;
            }
          } else {
            if (zeroChannel >= 2) {
              vals[zeroChannel - 2] = 0;
            }
            zeroChannel++;
          }
          return (vals[1] & 0xffff) << 16 | vals[0] & 0xffff;
        }
      default:
        unreachable(`unhandled format ${format}`);
        break;
    }
    return 0;
  })]
  );
  t.expectGPUBufferValuesEqual(buffer, expected);
});

g.test('bgra8unorm_swizzle').
desc('Test bgra8unorm swizzling').
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('bgra8unorm-storage');
}).
fn((t) => {
  const values = [
  { r: -1.1, g: 0.6, b: 0.4, a: 1 },
  { r: 1.1, g: 0.6, b: 0.4, a: 1 },
  { r: 0.4, g: -1.1, b: 0.6, a: 1 },
  { r: 0.4, g: 1.1, b: 0.6, a: 1 },
  { r: 0.6, g: 0.4, b: -1.1, a: 1 },
  { r: 0.6, g: 0.4, b: 1.1, a: 1 },
  { r: 0.2, g: 0.4, b: 0.6, a: 1 },
  { r: -0.2, g: -0.4, b: -0.6, a: 1 }];

  let wgsl = `
@group(0) @binding(0) var tex : texture_storage_1d<bgra8unorm, write>;

const values = array(`;
  for (const v of values) {
    wgsl += `vec4(${v.r},${v.g},${v.b},${v.a}),\n`;
  }
  wgsl += `);

@compute @workgroup_size(${values.length})
fn main(@builtin(global_invocation_id) gid : vec3u) {
  let value = values[gid.x];
  textureStore(tex, gid.x, value);
}`;

  const numTexels = values.length;
  const textureSize = { width: numTexels, height: 1, depthOrArrayLayers: 1 };
  const texture = t.createTextureTracked({
    format: 'bgra8unorm',
    dimension: '1d',
    size: textureSize,
    mipLevelCount: 1,
    usage: GPUTextureUsage.STORAGE_BINDING | GPUTextureUsage.COPY_SRC
  });

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
      resource: texture.createView({
        format: 'bgra8unorm',
        dimension: '1d'
      })
    }]

  });

  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginComputePass();
  pass.setPipeline(pipeline);
  pass.setBindGroup(0, bg);
  pass.dispatchWorkgroups(1, 1, 1);
  pass.end();
  t.queue.submit([encoder.finish()]);

  const buffer = t.copyWholeTextureToNewBufferSimple(texture, 0);
  const expected = new Uint32Array([
  ...iterRange(numTexels, (x) => {
    const { r, g, b, a } = values[x];
    return pack4x8unorm(b, g, r, a);
  })]
  );
  t.expectGPUBufferValuesEqual(buffer, expected);
});

// Texture width for dimensions >1D.
// Sized such that mip level 2 will be at least 256 bytes/row.
const kWidth = 256;

// Returns the texture geometry based on a given number of texels.
function getTextureSize(numTexels, dim, array) {
  const size = { width: 1, height: 1, depthOrArrayLayers: 1 };
  switch (dim) {
    case '1d':
      size.width = numTexels;
      break;
    case '2d':{
        const texelsPerArray = numTexels / array;
        size.width = kWidth;
        size.height = texelsPerArray / kWidth;
        size.depthOrArrayLayers = array;
        break;
      }
    case '3d':
      size.width = kWidth;
      size.height = numTexels / (2 * kWidth);
      size.depthOrArrayLayers = 2;
      break;
  }
  return size;
}

// WGSL declaration type for the texture.
function textureType(dim) {
  return `texture_storage_${dim}<r32uint, write>`;
}

// Defines a function to convert linear global id into a texture coordinate.
function indexToCoord(dim, type) {
  switch (dim) {
    case '1d':
      return `
fn indexToCoord(id : u32) -> ${type} {
  return ${type}(id);
}`;
      break;
    case '2d':
      return `
fn indexToCoord(id : u32) -> vec2<${type}> {
  return vec2<${type}>(${type}(id % width), ${type}(id / width));
}`;
      break;
    case '3d':
      return `
fn indexToCoord(id : u32) -> vec3<${type}> {
  const half = numTexels / depth;
  let half_id = id % half;
  return vec3<${type}>(${type}(half_id % width), ${type}(half_id / width), ${type}(id / half));
}`;
      break;
  }
  return ``;
}

// Mutates 'coords' to produce an out-of-bounds value.
// 1D workgroups are launched so 'gid.x' is the linear id.
//
// This code is only executed for odd global ids (gid.x % 2 == 1).
// All the values are chosen such they will further divide the odd invocations.
function outOfBoundsValue(dim, type) {
  switch (dim) {
    case '1d':{
        if (type === 'i32') {
          return `if gid.x % 3 == 0 {
          coords = -coords;
        } else {
          coords = coords + numTexels;
        }`;
        } else {
          return `coords = coords + numTexels;`;
        }
        break;
      }
    case '2d':{
        if (type === 'i32') {
          return `if gid.x % 3 == 0 {
          coords.x = -coords.x;
        } else {
          coords.y = coords.y + height;
        }`;
        } else {
          return `if gid.x % 3 == 1 {
          coords.x = coords.x + width;
        } else {
          coords.y = coords.y + height;
        }`;
        }
        break;
      }
    case '3d':{
        if (type === 'i32') {
          return `if gid.x % 3 == 0 {
          coords.x = -coords.x;
        } else if gid.x % 5 == 0 {
          coords.y = coords.y + height;
        } else {
          coords.z = coords.z + depth;
        }`;
        } else {
          return `if gid.x % 3 == 1 {
          coords.x = coords.x + width;
        } else if gid.x % 5 == 1 {
          coords.y = coords.y + height;
        } else {
          coords.z = 2 * depth;
        }`;
        }
        break;
      }
  }
  return ``;
}

// Returns the number of texels for a given mip level.
//
// 1D textures cannot have multiple mip levels so always return the input number of texels.
function getMipTexels(numTexels, dim, mip) {
  let texels = numTexels;
  if (mip === 0) {
    return texels;
  }
  if (dim === '2d') {
    texels /= 1 << mip;
    texels /= 1 << mip;
  } else if (dim === '3d') {
    texels /= 1 << mip;
    texels /= 1 << mip;
    texels /= 1 << mip;
  }
  return texels;
}

const kDims = ['1d', '2d', '3d'];

g.test('out_of_bounds').
desc('Test that textureStore on out-of-bounds coordinates have no effect').
params((u) =>
u.
combine('dim', kDims).
combine('coords', ['i32', 'u32']).
combine('mipCount', [1, 2, 3]).
combine('mip', [0, 1, 2]).
filter((t) => {
  if (t.dim === '1d') {
    return t.mipCount === 1 && t.mip === 0;
  }
  if (t.dim === '3d') {
    return t.mipCount <= 2 && t.mip < t.mipCount;
  }
  return t.mip < t.mipCount;
})
).
fn((t) => {
  const texel_format = 'r32uint';
  // Chosen such that the even at higher mip counts,
  // the texture is laid out without padding.
  // This simplifies the checking code below.
  //
  // Mip level | 1d   | 2d       | 3d
  // -----------------------------------------
  // 0         | 4096 | 256 x 16 | 256 x 8 x 2
  // 1         | -    | 128 x 8  | 128 x 4 x 1
  // 2         | -    | 64  x 4  | -
  const num_texels = 4096;
  const view_texels = getMipTexels(num_texels, t.params.dim, t.params.mip);

  const texture_size = getTextureSize(num_texels, t.params.dim, 1);
  const mip_size = virtualMipSize(t.params.dim, texture_size, t.params.mip);
  const texture = t.createTextureTracked({
    format: texel_format,
    dimension: t.params.dim,
    size: texture_size,
    mipLevelCount: t.params.mipCount,
    usage: GPUTextureUsage.STORAGE_BINDING | GPUTextureUsage.COPY_SRC
  });

  const oob_value = outOfBoundsValue(t.params.dim, t.params.coords);
  const wgx_size = 32;
  const num_wgs_x = view_texels / wgx_size;

  const wgsl = `
@group(0) @binding(0) var tex : ${textureType(t.params.dim)};

const numTexels = ${view_texels};
const width = ${mip_size[0]};
const height = ${mip_size[1]};
const depth = ${mip_size[2]};

${indexToCoord(t.params.dim, t.params.coords)}

@compute @workgroup_size(${wgx_size})
fn main(@builtin(global_invocation_id) gid : vec3u) {
  var coords = indexToCoord(gid.x);
  if gid.x % 2 == 1 {
    ${oob_value}
  }
  textureStore(tex, coords, vec4u(gid.x));
}`;

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
      resource: texture.createView({
        format: texel_format,
        dimension: t.params.dim,
        baseArrayLayer: 0,
        arrayLayerCount: 1,
        baseMipLevel: t.params.mip,
        mipLevelCount: 1
      })
    }]

  });

  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginComputePass();
  pass.setPipeline(pipeline);
  pass.setBindGroup(0, bg);
  pass.dispatchWorkgroups(num_wgs_x, 1, 1);
  pass.end();
  t.queue.submit([encoder.finish()]);

  for (let m = 0; m < t.params.mipCount; m++) {
    const buffer = t.copyWholeTextureToNewBufferSimple(texture, m);
    if (m === t.params.mip) {
      const expectedOutput = new Uint32Array([
      ...iterRange(view_texels, (x) => {
        if (x >= view_texels) {
          return 0;
        }
        if (x % 2 === 1) {
          return 0;
        }
        return x;
      })]
      );
      t.expectGPUBufferValuesEqual(buffer, expectedOutput);
    } else {
      const expectedOutput = new Uint32Array([
      ...iterRange(getMipTexels(num_texels, t.params.dim, m), (x) => 0)]
      );
      t.expectGPUBufferValuesEqual(buffer, expectedOutput);
    }
  }
});

const kArrayLevels = 4;

g.test('out_of_bounds_array').
desc('Test that out-of-bounds array coordinates to textureStore have no effect').
params((u) =>
u.
combine('baseLevel', [0, 1, 2, 3]).
combine('arrayLevels', [1, 2, 3, 4]).
combine('type', ['i32', 'u32']).
filter((t) => {
  if (t.arrayLevels <= t.baseLevel) {
    return false;
  }
  if (kArrayLevels < t.baseLevel + t.arrayLevels) {
    return false;
  }
  return true;
})
).
fn((t) => {
  const dim = '2d';
  const view_dim = '2d-array';
  const texel_format = 'r32uint';
  const width = 64;
  const height = 64;
  const base_texels = width * height;
  const num_texels = base_texels * kArrayLevels;
  const view_texels = base_texels * t.params.arrayLevels;
  const texture_size = { width, height, depthOrArrayLayers: kArrayLevels };
  const view_size = { width, height, depthOrArrayLayers: t.params.arrayLevels };

  const texture = t.createTextureTracked({
    format: texel_format,
    dimension: dim,
    size: texture_size,
    mipLevelCount: 1,
    usage: GPUTextureUsage.STORAGE_BINDING | GPUTextureUsage.COPY_SRC
  });

  const wgx_size = 32;
  const num_wgs_x = num_texels / wgx_size;

  let oob_value = `layer = layer + layers;`;
  if (t.params.type === 'i32') {
    oob_value = `if gid.x % 3 == 0 {
        layer = -(layer + layers);
      } else {
        layer = layer + layers;
      }`;
  }

  const wgsl = `
@group(0) @binding(0) var tex : texture_storage_2d_array<r32uint, write>;

const numTexels = ${view_texels};
const width = ${view_size.width};
const height = ${view_size.height ?? 1};
const layers = ${view_size.depthOrArrayLayers ?? 1};
const layerTexels = numTexels / layers;

@compute @workgroup_size(${wgx_size})
fn main(@builtin(global_invocation_id) gid : vec3u) {
  let layer_id = gid.x % layerTexels;
  var x = ${t.params.type}(layer_id % width);
  var y = ${t.params.type}(layer_id / width);
  var layer = ${t.params.type}(gid.x / layerTexels);
  if gid.x % 2 == 1 {
    ${oob_value}
  }
  textureStore(tex, vec2(x, y), layer, vec4u(gid.x));
}`;

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
      resource: texture.createView({
        format: texel_format,
        dimension: view_dim,
        baseArrayLayer: t.params.baseLevel,
        arrayLayerCount: t.params.arrayLevels,
        baseMipLevel: 0,
        mipLevelCount: 1
      })
    }]

  });

  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginComputePass();
  pass.setPipeline(pipeline);
  pass.setBindGroup(0, bg);
  pass.dispatchWorkgroups(num_wgs_x, 1, 1);
  pass.end();
  t.queue.submit([encoder.finish()]);

  const buffer = t.copyWholeTextureToNewBufferSimple(texture, 0);
  const expectedOutput = new Uint32Array([
  ...iterRange(num_texels, (x) => {
    const baseOffset = base_texels * t.params.baseLevel;
    if (x < baseOffset) {
      return 0;
    }
    if (base_texels * (t.params.baseLevel + t.params.arrayLevels) <= x) {
      return 0;
    }
    if (x % 2 === 1) {
      return 0;
    }
    return x - baseOffset;
  })]
  );
  t.expectGPUBufferValuesEqual(buffer, expectedOutput);
});