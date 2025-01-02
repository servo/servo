/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution tests for the 'textureDimensions' builtin function

The dimensions of the texture in texels.
For textures based on cubes, the results are the dimensions of each face of the cube.
Cube faces are square, so the x and y components of the result are equal.
If level is outside the range [0, textureNumLevels(t)) then any valid value for the return type may be returned.
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import {
  kAllTextureFormats,
  kColorTextureFormats,
  kTextureFormatInfo,
  sampleTypeForFormatAndAspect,
  textureDimensionAndFormatCompatible } from
'../../../../../format_info.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { align } from '../../../../../util/math.js';

export const g = makeTestGroup(GPUTest);

/// The maximum number of texture mipmap levels to test.
/// Keep this small to reduce memory and test permutations.
const kMaxMipsForTest = 3;

/// The maximum number of texture samples to test.
const kMaxSamplesForTest = 4;

/// All the possible GPUTextureViewDimensions.
const kAllViewDimensions = [
'1d',
'2d',
'2d-array',
'3d',
'cube',
'cube-array'];


/** @returns the aspects to test for the given format */
function aspectsForFormat(format) {
  const formatInfo = kTextureFormatInfo[format];
  if (formatInfo.depth !== undefined && formatInfo.stencil !== undefined) {
    return ['depth-only', 'stencil-only'];
  }
  return ['all'];
}

/** @returns the sample counts to test for the given format */
function samplesForFormat(format) {
  const info = kTextureFormatInfo[format];
  return info.multisample ? [1, kMaxSamplesForTest] : [1];
}

/**
 * @returns a list of number of texture mipmap levels to test, given the format, view dimensions and
 * number of samples.
 */
function textureMipCount(params)



{
  if (params.samples !== undefined && params.samples !== 1) {
    // https://www.w3.org/TR/webgpu/#texture-creation
    // If descriptor.sampleCount > 1: descriptor.mipLevelCount must be 1.
    return [1];
  }
  if (textureDimensionsForViewDimensions(params.dimensions) === '1d') {
    // https://www.w3.org/TR/webgpu/#dom-gputexturedimension-2d
    // Only "2d" textures may have mipmaps, be multisampled, use a compressed or depth/stencil
    // format, and be used as a render attachment.
    return [1];
  }
  return [1, kMaxMipsForTest];
}

/**
 * @returns a list of GPUTextureViewDescriptor.baseMipLevel to test, give the texture mipmap count.
 */
function baseMipLevel(params) {
  const out = [];
  for (let i = 0; i < params.textureMipCount; i++) {
    out.push(i);
  }
  return out;
}

/**
 * @returns the argument values for the textureDimensions() `level` parameter to test.
 * An `undefined` represents a call to textureDimensions() without the level argument.
 */
function textureDimensionsLevel(params)



{
  if (params.samples !== undefined && params.samples > 1) {
    return [undefined]; // textureDimensions() overload with `level` not available.
  }
  const out = [undefined];
  for (let i = 0; i < params.textureMipCount - params.baseMipLevel; i++) {
    out.push(i);
  }
  return out;
}

/** @returns the GPUTextureViewDimensions to test for the format and number of samples */
function viewDimensions(params)


{
  if (params.samples !== undefined && params.samples > 1) {
    // https://www.w3.org/TR/webgpu/#dom-gputexturedimension-2d
    // Only 2d textures can be multisampled
    return ['2d'];
  }

  return kAllViewDimensions.filter((dim) =>
  textureDimensionAndFormatCompatible(textureDimensionsForViewDimensions(dim), params.format)
  );
}

/** @returns the GPUTextureDimension for the GPUTextureViewDimension */
function textureDimensionsForViewDimensions(dim) {
  switch (dim) {
    case '1d':
      return '1d';
    case '2d':
    case '2d-array':
    case 'cube':
    case 'cube-array':
      return '2d';
    case '3d':
      return '3d';
  }
}

/** TestValues holds the texture size and expected return value of textureDimensions() */







/** @returns The TestValues to use for the given texture dimensions and format */
function testValues(params)




{
  // The minimum dimension length, given the number of mipmap levels that are being tested.
  const kMinLen = 1 << kMaxMipsForTest;
  const kNumCubeFaces = 6;

  const formatInfo = kTextureFormatInfo[params.format];
  const bw = formatInfo.blockWidth;
  const bh = formatInfo.blockHeight;
  let mip = params.baseMipLevel;
  if (params.textureDimensionsLevel !== undefined) {
    mip += params.textureDimensionsLevel;
  }

  // Magic constants to multiply the minimum texture dimensions with, to provide
  // different dimension values in the test. These could be parameterized, but
  // these are currently fixed to reduce the number of test parameterizations.
  const kMultipleA = 2;
  const kMultipleB = 3;
  const kMultipleC = 4;

  switch (params.dimensions) {
    case '1d':{
        const w = align(kMinLen, bw) * kMultipleA;
        return { size: [w], expected: [w >>> mip] };
      }
    case '2d':{
        const w = align(kMinLen, bw) * kMultipleA;
        const h = align(kMinLen, bh) * kMultipleB;
        return { size: [w, h], expected: [w >>> mip, h >>> mip] };
      }
    case '2d-array':{
        const w = align(kMinLen, bw) * kMultipleC;
        const h = align(kMinLen, bh) * kMultipleB;
        return { size: [w, h, 4], expected: [w >>> mip, h >>> mip] };
      }
    case '3d':{
        const w = align(kMinLen, bw) * kMultipleA;
        const h = align(kMinLen, bh) * kMultipleB;
        const d = kMinLen * kMultipleC;
        return {
          size: [w, h, d],
          expected: [w >>> mip, h >>> mip, d >>> mip]
        };
      }
    case 'cube':{
        const l = align(kMinLen, bw) * align(kMinLen, bh) * kMultipleB;
        return {
          size: [l, l, kNumCubeFaces],
          expected: [l >>> mip, l >>> mip]
        };
      }
    case 'cube-array':{
        const l = align(kMinLen, bw) * align(kMinLen, bh) * kMultipleC;
        return {
          size: [l, l, kNumCubeFaces * 3],
          expected: [l >>> mip, l >>> mip]
        };
      }
  }
}

/**
 * Builds a shader module with the texture view bound to the WGSL texture with the given WGSL type,
 * which calls textureDimensions(), assigning the result to an output buffer.
 * This shader is executed with a compute shader, and the output buffer is compared to
 * `values.expected`.
 */
function run(
t,
view,
textureType,
levelArg,
values)
{
  const outputType = values.expected.length > 1 ? `vec${values.expected.length}u` : 'u32';
  const wgsl = `
@group(0) @binding(0) var texture : ${textureType};
@group(0) @binding(1) var<storage, read_write> output : ${outputType};

@compute @workgroup_size(1)
fn main() {
output = ${
  levelArg !== undefined ?
  `textureDimensions(texture, ${levelArg})` :
  'textureDimensions(texture)'
  };
}
`;
  const module = t.device.createShaderModule({
    code: wgsl
  });
  const pipeline = t.device.createComputePipeline({
    compute: { module },
    layout: 'auto'
  });
  const outputBuffer = t.createBufferTracked({
    size: 32,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.STORAGE
  });
  const bindgroup = t.device.createBindGroup({
    layout: pipeline.getBindGroupLayout(0),
    entries: [
    { binding: 0, resource: view },
    { binding: 1, resource: { buffer: outputBuffer } }]

  });
  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginComputePass();
  pass.setPipeline(pipeline);
  pass.setBindGroup(0, bindgroup);
  pass.dispatchWorkgroups(1);
  pass.end();
  t.device.queue.submit([encoder.finish()]);

  t.expectGPUBufferValuesEqual(outputBuffer, new Uint32Array(values.expected));
}

/** @returns true if the GPUTextureViewDimension is valid for a storage texture */
function dimensionsValidForStorage(dimensions) {
  switch (dimensions) {
    case '1d':
    case '2d':
    case '2d-array':
    case '3d':
      return true;
    default:
      return false;
  }
}

g.test('sampled_and_multisampled').
specURL('https://www.w3.org/TR/WGSL/#texturedimensions').
desc(
  `
T: f32, i32, u32

fn textureDimensions(t: texture_1d<T>) -> u32
fn textureDimensions(t: texture_1d<T>, level: u32) -> u32
fn textureDimensions(t: texture_2d<T>) -> vec2<u32>
fn textureDimensions(t: texture_2d<T>, level: u32) -> vec2<u32>
fn textureDimensions(t: texture_2d_array<T>) -> vec2<u32>
fn textureDimensions(t: texture_2d_array<T>, level: u32) -> vec2<u32>
fn textureDimensions(t: texture_3d<T>) -> vec3<u32>
fn textureDimensions(t: texture_3d<T>, level: u32) -> vec3<u32>
fn textureDimensions(t: texture_cube<T>) -> vec2<u32>
fn textureDimensions(t: texture_cube<T>, level: u32) -> vec2<u32>
fn textureDimensions(t: texture_cube_array<T>) -> vec2<u32>
fn textureDimensions(t: texture_cube_array<T>, level: u32) -> vec2<u32>
fn textureDimensions(t: texture_multisampled_2d<T>)-> vec2<u32>

Parameters:
 * t: the sampled texture
 * level:
   - The mip level, with level 0 containing a full size version of the texture.
   - If omitted, the dimensions of level 0 are returned.
`
).
params((u) =>
u.
combine('format', kAllTextureFormats).
unless((p) => kTextureFormatInfo[p.format].color?.type === 'unfilterable-float').
expand('aspect', (u) => aspectsForFormat(u.format)).
expand('samples', (u) => samplesForFormat(u.format)).
beginSubcases().
expand('dimensions', viewDimensions).
expand('textureMipCount', textureMipCount).
expand('baseMipLevel', baseMipLevel).
expand('textureDimensionsLevel', textureDimensionsLevel)
).
beforeAllSubcases((t) => {
  const info = kTextureFormatInfo[t.params.format];
  t.skipIfTextureFormatNotSupported(t.params.format);
  t.selectDeviceOrSkipTestCase(info.feature);
}).
fn((t) => {
  t.skipIfTextureViewDimensionNotSupported(t.params.dimensions);
  const values = testValues(t.params);
  const texture = t.createTextureTracked({
    size: values.size,
    dimension: textureDimensionsForViewDimensions(t.params.dimensions),
    ...(t.isCompatibility && { textureBindingViewDimension: t.params.dimensions }),
    usage:
    t.params.samples === 1 ?
    GPUTextureUsage.TEXTURE_BINDING :
    GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.RENDER_ATTACHMENT,
    format: t.params.format,
    sampleCount: t.params.samples,
    mipLevelCount: t.params.textureMipCount
  });
  const textureView = texture.createView({
    dimension: t.params.dimensions,
    aspect: t.params.aspect,
    baseMipLevel: t.params.baseMipLevel
  });

  function wgslSampledTextureType() {
    const base = t.params.samples !== 1 ? 'texture_multisampled' : 'texture';
    const dimensions = t.params.dimensions.replace('-', '_');
    const sampleType = sampleTypeForFormatAndAspect(t.params.format, t.params.aspect);
    switch (sampleType) {
      case 'depth':
      case 'float':
        return `${base}_${dimensions}<f32>`;
      case 'uint':
        return `${base}_${dimensions}<u32>`;
      case 'sint':
        return `${base}_${dimensions}<i32>`;
      case 'unfilterable-float':
        throw new Error(`'${t.params.format}' does not support sampling`);
    }
  }

  run(t, textureView, wgslSampledTextureType(), t.params.textureDimensionsLevel, values);
});

g.test('depth').
specURL('https://www.w3.org/TR/WGSL/#texturedimensions').
desc(
  `
fn textureDimensions(t: texture_depth_2d) -> vec2<u32>
fn textureDimensions(t: texture_depth_2d, level: u32) -> vec2<u32>
fn textureDimensions(t: texture_depth_2d_array) -> vec2<u32>
fn textureDimensions(t: texture_depth_2d_array, level: u32) -> vec2<u32>
fn textureDimensions(t: texture_depth_cube) -> vec2<u32>
fn textureDimensions(t: texture_depth_cube, level: u32) -> vec2<u32>
fn textureDimensions(t: texture_depth_cube_array) -> vec2<u32>
fn textureDimensions(t: texture_depth_cube_array, level: u32) -> vec2<u32>
fn textureDimensions(t: texture_depth_multisampled_2d)-> vec2<u32>

Parameters:
 * t: the depth or multisampled texture
 * level:
   - The mip level, with level 0 containing a full size version of the texture.
   - If omitted, the dimensions of level 0 are returned.
`
).
params((u) =>
u.
combine('format', kAllTextureFormats).
filter((p) => !!kTextureFormatInfo[p.format].depth).
expand('aspect', (u) => aspectsForFormat(u.format)).
unless((u) => u.aspect === 'stencil-only').
expand('samples', (u) => samplesForFormat(u.format)).
beginSubcases().
expand('dimensions', viewDimensions).
expand('textureMipCount', textureMipCount).
expand('baseMipLevel', baseMipLevel).
expand('textureDimensionsLevel', textureDimensionsLevel)
).
beforeAllSubcases((t) => {
  const info = kTextureFormatInfo[t.params.format];
  t.skipIfTextureFormatNotSupported(t.params.format);
  t.selectDeviceOrSkipTestCase(info.feature);
}).
fn((t) => {
  t.skipIfTextureViewDimensionNotSupported(t.params.dimensions);
  const values = testValues(t.params);
  const texture = t.createTextureTracked({
    size: values.size,
    dimension: textureDimensionsForViewDimensions(t.params.dimensions),
    ...(t.isCompatibility && { textureBindingViewDimension: t.params.dimensions }),
    usage:
    t.params.samples === 1 ?
    GPUTextureUsage.TEXTURE_BINDING :
    GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.RENDER_ATTACHMENT,
    format: t.params.format,
    sampleCount: t.params.samples,
    mipLevelCount: t.params.textureMipCount
  });
  const textureView = texture.createView({
    dimension: t.params.dimensions,
    aspect: t.params.aspect,
    baseMipLevel: t.params.baseMipLevel
  });

  function wgslDepthTextureType() {
    const base = t.params.samples !== 1 ? 'texture_depth_multisampled' : 'texture_depth';
    const dimensions = t.params.dimensions.replace('-', '_');
    return `${base}_${dimensions}`;
  }

  run(t, textureView, wgslDepthTextureType(), t.params.textureDimensionsLevel, values);
});

g.test('storage').
specURL('https://www.w3.org/TR/WGSL/#texturedimensions').
desc(
  `
F: rgba8unorm
   rgba8snorm
   rgba8uint
   rgba8sint
   rgba16uint
   rgba16sint
   rgba16float
   r32uint
   r32sint
   r32float
   rg32uint
   rg32sint
   rg32float
   rgba32uint
   rgba32sint
   rgba32float
A: read, write, read_write

fn textureDimensions(t: texture_storage_1d<F,A>) -> u32
fn textureDimensions(t: texture_storage_2d<F,A>) -> vec2<u32>
fn textureDimensions(t: texture_storage_2d_array<F,A>) -> vec2<u32>
fn textureDimensions(t: texture_storage_3d<F,A>) -> vec3<u32>

Parameters:
 * t: the storage texture
`
).
params((u) =>
u.
combine('format', kColorTextureFormats).
filter((p) => kTextureFormatInfo[p.format].color?.storage === true).
expand('aspect', (u) => aspectsForFormat(u.format)).
beginSubcases().
expand('dimensions', (u) => viewDimensions(u).filter(dimensionsValidForStorage)).
expand('textureMipCount', textureMipCount).
expand('baseMipLevel', baseMipLevel)
).
beforeAllSubcases((t) => {
  const info = kTextureFormatInfo[t.params.format];
  t.skipIfTextureFormatNotSupported(t.params.format);
  t.skipIfTextureFormatNotUsableAsStorageTexture(t.params.format);
  t.selectDeviceOrSkipTestCase(info.feature);
}).
fn((t) => {
  const values = testValues(t.params);
  const texture = t.createTextureTracked({
    size: values.size,
    dimension: textureDimensionsForViewDimensions(t.params.dimensions),
    usage: GPUTextureUsage.STORAGE_BINDING,
    format: t.params.format,
    mipLevelCount: t.params.textureMipCount
  });
  const textureView = texture.createView({
    dimension: t.params.dimensions,
    aspect: t.params.aspect,
    mipLevelCount: 1,
    baseMipLevel: t.params.baseMipLevel
  });

  function wgslStorageTextureType() {
    const dimensions = t.params.dimensions.replace('-', '_');
    return `texture_storage_${dimensions}<${t.params.format}, write>`;
  }

  run(t, textureView, wgslStorageTextureType(), undefined, values);
});

g.test('external').
specURL('https://www.w3.org/TR/WGSL/#texturedimensions').
desc(
  `
fn textureDimensions(t: texture_external) -> vec2<u32>

Parameters:
 * t: the external texture
`
).
unimplemented();