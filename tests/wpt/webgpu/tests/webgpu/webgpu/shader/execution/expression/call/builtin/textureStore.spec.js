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
import { unreachable, iterRange, range } from '../../../../../../common/util/util.js';
import {
  isTextureFormatPossiblyStorageReadWritable,
  kPossibleStorageTextureFormats } from
'../../../../../format_info.js';
import { AllFeaturesMaxLimitsGPUTest } from '../../../../../gpu_test.js';
import * as ttu from '../../../../../texture_test_utils.js';
import {
  kFloat32Format,
  kFloat16Format,
  numberToFloatBits,
  pack4x8unorm,
  pack4x8snorm,
  pack2x16unorm,
  pack2x16snorm } from
'../../../../../util/conversion.js';
import { align, clamp } from '../../../../../util/math.js';
import { getTextureDimensionFromView, virtualMipSize } from '../../../../../util/texture/base.js';

import { getTextureFormatTypeInfo } from './texture_utils.js';

const kDims = ['1d', '2d', '3d'];
const kViewDimensions = ['1d', '2d', '2d-array', '3d'];

export const g = makeTestGroup(AllFeaturesMaxLimitsGPUTest);

// We require a few values that are out of range for a given type
// so we can check clamping behavior.
function inputArray(format) {
  switch (format) {
    case 'r8snorm':
    case 'rg8snorm':
    case 'rgba8snorm':
    case 'r16snorm':
    case 'rg16snorm':
    case 'rgba16snorm':
      return [-1.1, 1.0, -0.6, -0.3, 0, 0.3, 0.6, 1.0, 1.1];
    case 'r8unorm':
    case 'rg8unorm':
    case 'rgba8unorm':
    case 'bgra8unorm':
    case 'r16unorm':
    case 'rg16unorm':
    case 'rgba16unorm':
      return [-0.1, 0, 0.2, 0.4, 0.6, 0.8, 1.0, 1.1];
    case 'r8uint':
    case 'rg8uint':
    case 'rgba8uint':
      return [0, 8, 16, 24, 32, 64, 100, 128, 200, 255, 256, 512];
    case 'rgba16uint':
      return [0, 8, 16, 24, 32, 64, 100, 128, 200, 255, 0xffff, 0x1ffff];
    case 'rgba32uint':
    case 'r32uint':
    case 'rg32uint':
      return [0, 8, 16, 24, 32, 64, 100, 128, 200, 255, 256, 512, 0xffffffff];
    case 'r8sint':
    case 'rg8sint':
    case 'rgba8sint':
      return [-128, -100, -64, -32, -16, -8, 0, 8, 16, 32, 64, 100, 127];
    case 'rgba16sint':
      return [-32768, -32769, -100, -64, -32, -16, -8, 0, 8, 16, 32, 64, 100, 127, 0x7fff, 0x8000];
    case 'r32sint':
    case 'rg32sint':
    case 'rgba32sint':
      return [-0x8000000, -32769, -100, -64, -32, -16, -8, 0, 8, 16, 32, 64, 100, 127, 0x7ffffff];
    case 'r16float':
    case 'rg16float':
    case 'rgba16float':
    case 'rgba32float':
    case 'r32float':
    case 'rg32float':
      // Stick with simple values to avoid rounding issues.
      return [-100, -50, -32, -16, -8, -1, 0, 1, 8, 16, 32, 50, 100];
    case 'r16uint': // [0, 65535]
    case 'rg16uint':
      return [0, 1000, 32768, 65535, 65536, 70000];
    case 'r16sint': // [-32768, 32767]
    case 'rg16sint':
      return [-32769, -32768, -1000, 0, 1000, 32767, 32768];

    case 'rgb10a2uint':
      return [0, 500, 1023, 1024, 3, 4];
    case 'rgb10a2unorm':
      return [-0.1, 0, 0.5, 1.0, 1.1];
    case 'rg11b10ufloat':
      return [1, 0.5, 0, 1];
    default:
      unreachable(`unhandled format ${format}`);
      break;
  }
  return [];
}

g.test('texel_formats').
desc(
  `
    Test storage of texel formats

    - test values make it through.
    - test out of range values get clamped.
    - test 1d, 2d, 2d-array, 3d.
    - test all storage formats.
  `
).
params((u) =>
u.
combine('format', kPossibleStorageTextureFormats).
combine('viewDimension', kViewDimensions)
// Note: We can't use writable storage textures in a vertex stage.
.combine('stage', ['compute', 'fragment']).
combine('access', ['write', 'read_write']).
unless(
  (t) => t.access === 'read_write' && !isTextureFormatPossiblyStorageReadWritable(t.format)
).
combine('mipLevel', [0, 1, 2]).
unless((t) => t.viewDimension === '1d' && t.mipLevel !== 0)
).
fn((t) => {
  const { format, stage, access, viewDimension, mipLevel } = t.params;
  t.skipIfTextureFormatNotSupported(format);
  t.skipIfTextureFormatNotUsableWithStorageAccessMode(access, format);

  const { componentType } = getTextureFormatTypeInfo(format);
  const values = inputArray(format);

  t.skipIf(
    t.isCompatibility &&
    stage === 'fragment' &&
    t.device.limits.maxStorageTexturesInFragmentStage < 1,
    'device does not support storage textures in fragment shaders'
  );

  const suffix = format.endsWith('sint') ? 'i' : format.endsWith('uint') ? 'u' : 'f';
  const swizzleWGSL = viewDimension === '1d' ? 'x' : viewDimension === '3d' ? 'xyz' : 'xy';
  const layerWGSL = viewDimension === '2d-array' ? ', gid.z' : '';
  const wgsl = `
const range = array(${values.map((v) => `${v}${suffix}`).join(',')});

@group(0) @binding(0)
var tex : texture_storage_${viewDimension.replace('-', '_')}<${format}, ${access}>;

fn setValue(gid: vec3u) {
  let ndx = gid.x + gid.y + gid.z;
  let vecVal = vec4(
    range[(ndx + 0) % ${values.length}],
    range[(ndx + 1) % ${values.length}],
    range[(ndx + 2) % ${values.length}],
    range[(ndx + 3) % ${values.length}],
  );
  var val = vec4<${componentType}>(vecVal);
  let coord = gid.${swizzleWGSL};
  textureStore(tex, coord${layerWGSL}, val);
}

@compute @workgroup_size(${values.length})
fn cs(@builtin(global_invocation_id) gid : vec3u) {
  setValue(gid);
}

struct VOut {
  @builtin(position) pos: vec4f,
  @location(0) @interpolate(flat, either) z: u32,
}
@vertex fn vs(
  @builtin(vertex_index) vNdx: u32,
  @builtin(instance_index) iNdx: u32,
) -> VOut {
  let pos = array(vec2f(-1, 3), vec2f(3, -1), vec2f(-1, -1));
  return VOut(vec4f(pos[vNdx], 0, 1), iNdx);
}

@fragment fn fs(v: VOut) -> @location(0) vec4f {
  setValue(vec3u(u32(v.pos.x), u32(v.pos.y), v.z));
  return vec4f(0);
}
`;

  // choose a size so the mipLevel we will write to is the size we want to test
  const mipMult = 2 ** mipLevel;
  const size = values.length * mipMult;
  const mipLevel0Size = [
  size,
  viewDimension === '1d' ? 1 : size,
  viewDimension === '2d-array' ? values.length : viewDimension === '3d' ? size : 1];

  const testMipLevelSize = [
  values.length,
  viewDimension === '1d' ? 1 : values.length,
  viewDimension === '2d-array' || viewDimension === '3d' ? values.length : 1];

  const dimension = getTextureDimensionFromView(viewDimension);
  const texture = t.createTextureTracked({
    format: format,
    size: mipLevel0Size,
    mipLevelCount: viewDimension === '1d' ? 1 : 3,
    dimension,
    usage: GPUTextureUsage.STORAGE_BINDING | GPUTextureUsage.COPY_SRC
  });

  const module = t.device.createShaderModule({
    code: wgsl
  });

  const pipeline =
  stage === 'compute' ?
  t.device.createComputePipeline({
    layout: 'auto',
    compute: { module }
  }) :
  t.device.createRenderPipeline({
    layout: 'auto',
    vertex: { module },
    fragment: { module, targets: [{ format: 'rgba8unorm' }] }
  });

  const bg = t.device.createBindGroup({
    layout: pipeline.getBindGroupLayout(0),
    entries: [
    {
      binding: 0,
      resource: texture.createView({
        format: format,
        dimension: viewDimension,
        baseMipLevel: mipLevel,
        mipLevelCount: 1
      })
    }]

  });

  const encoder = t.device.createCommandEncoder();
  switch (stage) {
    case 'compute':{
        const pass = encoder.beginComputePass();
        pass.setPipeline(pipeline);
        pass.setBindGroup(0, bg);
        pass.dispatchWorkgroups(...testMipLevelSize);
        pass.end();
        break;
      }
    case 'fragment':{
        const renderTarget = t.createTextureTracked({
          size: testMipLevelSize.slice(0, 2),
          format: 'rgba8unorm',
          usage: GPUTextureUsage.RENDER_ATTACHMENT
        });
        const pass = encoder.beginRenderPass({
          colorAttachments: [
          {
            view: renderTarget.createView(),
            loadOp: 'clear',
            storeOp: 'store'
          }]

        });
        pass.setPipeline(pipeline);
        pass.setBindGroup(0, bg);
        pass.draw(3, testMipLevelSize[2]);
        pass.end();
        break;
      }
  }
  t.queue.submit([encoder.finish()]);

  let bytesPerTexel = 4;
  switch (format) {
    case 'r8unorm':
    case 'r8uint':
    case 'r8snorm':
    case 'r8sint':
      bytesPerTexel = 1;
      break;
    case 'r16unorm':
    case 'r16uint':
    case 'r16snorm':
    case 'r16sint':
    case 'r16float':
    case 'rg8unorm':
    case 'rg8uint':
    case 'rg8snorm':
    case 'rg8sint':
      bytesPerTexel = 2;
      break;
    case 'rg16unorm':
    case 'rg16uint':
    case 'rg16snorm':
    case 'rg16sint':
    case 'rg16float':
    case 'rg11b10ufloat':
    case 'rgb10a2uint':
    case 'rgb10a2unorm':
      bytesPerTexel = 4;
      break;
    case 'rgba16unorm':
    case 'rgba16snorm':
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

  const buffer = ttu.copyWholeTextureToNewBufferSimple(t, texture, mipLevel);
  const u32sPerTexel = bytesPerTexel / 4;
  const u8sPerTexel = bytesPerTexel;
  const bytesPerRow = align(testMipLevelSize[0] * bytesPerTexel, 256);
  const texelsPerRow = bytesPerRow / bytesPerTexel;
  const texelsPerSlice = texelsPerRow * testMipLevelSize[1];
  const getValue = (i) => values[i % values.length];
  const clampedPack4x8unorm = (...v) => {
    const c = v.map((v) => clamp(v, { min: 0, max: 1 }));
    return pack4x8unorm(c[0], c[1], c[2], c[3]);
  };
  const clampedPack4x8snorm = (...v) => {
    const c = v.map((v) => clamp(v, { min: -1, max: 1 }));
    return pack4x8snorm(c[0], c[1], c[2], c[3]);
  };
  if (format.startsWith('r8') || format.startsWith('rg8')) {
    const expected = new Uint8Array([
    // iterate over each u8
    ...iterRange(buffer.size, (i) => {
      const texelId = i / u8sPerTexel | 0;
      const z = texelId / texelsPerSlice | 0;
      const y = (texelId / texelsPerRow | 0) % testMipLevelSize[1];
      const x = texelId % texelsPerRow;
      // buffer is padded to 256 per row so when x is out of range just return 0
      if (x >= testMipLevelSize[0]) {
        return 0;
      }
      const id = x + y + z;
      const unit = i % u8sPerTexel;
      switch (format) {
        case 'r8snorm':{
            const vals = getValue(id);
            const c = clamp(vals, { min: -1, max: 1 });
            return Math.floor(0.5 + 127 * c);
          }
        case 'r8unorm':{
            const vals = getValue(id);
            const c = clamp(vals, { min: 0, max: 1 });
            return Math.floor(0.5 + 255 * c);
          }
        case 'r8uint':{
            const val = clamp(getValue(id), { min: 0, max: 255 });
            return val & 0xff;
          }
        case 'r8sint':{
            const val = clamp(getValue(id), { min: -0x80, max: 0x7f });
            return val & 0xff;
          }
        case 'rg8snorm':{
            const vals = getValue(id + unit);
            const c = clamp(vals, { min: -1, max: 1 });
            return Math.floor(0.5 + 127 * c);
          }
        case 'rg8unorm':{
            const vals = getValue(id + unit);
            const c = clamp(vals, { min: 0, max: 1 });
            return Math.floor(0.5 + 255 * c);
          }
        case 'rg8uint':{
            const val = clamp(getValue(id + unit), { min: 0, max: 255 });
            return val & 0xff;
          }
        case 'rg8sint':{
            const val = clamp(getValue(id + unit), { min: -0x80, max: 0x7f });
            return val & 0xff;
          }
        default:
          unreachable(`unhandled format ${format}`);
          break;
      }
    })]
    );
    t.expectGPUBufferValuesEqual(buffer, expected);
  } else if (format.startsWith('r16')) {
    const expected = new Uint16Array([
    // iterate over each u16
    ...iterRange(buffer.size / 2, (i) => {
      const texelId = i;
      const z = texelId / texelsPerSlice | 0;
      const y = (texelId / texelsPerRow | 0) % testMipLevelSize[1];
      const x = texelId % texelsPerRow;
      // buffer is padded to 256 per row so when x is out of range just return 0
      if (x >= testMipLevelSize[0]) {
        return 0;
      }
      const id = x + y + z;
      switch (format) {
        case 'r16sint':{
            const vals = clamp(getValue(id), { min: -0x8000, max: 0x7fff });
            return vals & 0xffff;
          }
        case 'r16uint':{
            const vals = clamp(getValue(id), { min: 0, max: 0xffff });
            return vals & 0xffff;
          }
        case 'r16snorm':{
            const vals = getValue(id);
            const c = clamp(vals, { min: -1, max: 1 });
            return Math.floor(0.5 + 32767 * c);
          }
        case 'r16unorm':{
            const vals = getValue(id);
            const c = clamp(vals, { min: 0, max: 1 });
            return Math.floor(0.5 + 65535 * c);
          }
        case 'r16float':{
            const vals = numberToFloatBits(getValue(id), kFloat16Format);
            return vals & 0xffff;
          }
        default:
          unreachable(`unhandled format ${format}`);
          break;
      }
    })]
    );
    t.expectGPUBufferValuesEqual(buffer, expected);
  } else {
    const expected = new Uint32Array([
    // iterate over each u32
    ...iterRange(buffer.size / 4, (i) => {
      const texelId = i / u32sPerTexel | 0;
      const z = texelId / texelsPerSlice | 0;
      const y = (texelId / texelsPerRow | 0) % testMipLevelSize[1];
      const x = texelId % texelsPerRow;
      // buffer is padded to 256 per row so when x is out of range just return 0
      if (x >= testMipLevelSize[0]) {
        return 0;
      }
      const id = x + y + z;
      const unit = i % u32sPerTexel;
      switch (format) {
        case 'rgba8unorm':{
            const vals = range(4, (i) => getValue(id + i));
            return clampedPack4x8unorm(vals[0], vals[1], vals[2], vals[3]);
          }
        case 'bgra8unorm':{
            const vals = range(4, (i) => getValue(id + i));
            return clampedPack4x8unorm(vals[2], vals[1], vals[0], vals[3]);
          }
        case 'rgba8snorm':{
            const vals = range(4, (i) => getValue(id + i));
            return clampedPack4x8snorm(vals[0], vals[1], vals[2], vals[3]);
          }
        case 'r32uint':
          return clamp(getValue(id), { min: 0, max: 0xffffffff });
        case 'r32sint':
          return clamp(getValue(id), { min: -0x80000000, max: 0x7fffffff });
        case 'rg32uint':
        case 'rgba32uint':
          return clamp(getValue(id + unit), { min: 0, max: 0xffffffff });
        case 'rg32sint':
        case 'rgba32sint':
          return clamp(getValue(id + unit), { min: -0x80000000, max: 0x7fffffff });
        case 'rgba8uint':{
            const vals = range(4, (i) => clamp(getValue(id + i), { min: 0, max: 255 }));
            return (
              (vals[3] & 0xff) << 24 |
              (vals[2] & 0xff) << 16 |
              (vals[1] & 0xff) << 8 |
              vals[0] & 0xff);

          }
        case 'rgba8sint':{
            const vals = range(4, (i) => clamp(getValue(id + i), { min: -0x80, max: 0x7f }));
            return (
              (vals[3] & 0xff) << 24 |
              (vals[2] & 0xff) << 16 |
              (vals[1] & 0xff) << 8 |
              vals[0] & 0xff);

          }
        case 'rgba16uint':{
            const vals = range(2, (i) =>
            clamp(getValue(id + unit * 2 + i), { min: 0, max: 0xffff })
            );
            return (vals[1] & 0xffff) << 16 | vals[0] & 0xffff;
          }
        case 'rgba16sint':{
            const vals = range(2, (i) =>
            clamp(getValue(id + unit * 2 + i), { min: -0x8000, max: 0x7fff })
            );
            return (vals[1] & 0xffff) << 16 | vals[0] & 0xffff;
          }
        case 'r32float':
        case 'rg32float':
        case 'rgba32float':{
            return numberToFloatBits(getValue(id + unit), kFloat32Format);
          }
        case 'rgba16float':{
            const vals = range(2, (i) =>
            numberToFloatBits(getValue(id + unit * 2 + i), kFloat16Format)
            );
            return (vals[1] & 0xffff) << 16 | vals[0] & 0xffff;
          }

        case 'rg16uint':{
            const vals = range(2, (i) => clamp(getValue(id + i), { min: 0, max: 0xffff }));
            return (vals[1] & 0xffff) << 16 | vals[0] & 0xffff;
          }
        case 'rg16sint':{
            const vals = range(2, (i) => clamp(getValue(id + i), { min: -0x8000, max: 0x7fff }));
            return (vals[1] & 0xffff) << 16 | vals[0] & 0xffff;
          }
        case 'rg16unorm':{
            const vals = range(2, (i) => getValue(id + i));
            return pack2x16unorm(vals[0], vals[1]);
          }
        case 'rg16snorm':{
            const vals = range(2, (i) => getValue(id + i));
            return pack2x16snorm(vals[0], vals[1]);
          }
        case 'rg16float':{
            const vals = range(2, (i) => numberToFloatBits(getValue(id + i), kFloat16Format));
            return (vals[1] & 0xffff) << 16 | vals[0] & 0xffff;
          }
        case 'rgba16unorm':{
            const vals = range(2, (i) => clamp(getValue(id + unit * 2 + i), { min: 0, max: 1 }));
            return pack2x16unorm(vals[0], vals[1]);
          }
        case 'rgba16snorm':{
            const vals = range(2, (i) => clamp(getValue(id + unit * 2 + i), { min: -1, max: 1 }));
            return pack2x16snorm(vals[0], vals[1]);
          }
        case 'rgb10a2uint':{
            const r = Math.max(Math.min(getValue(id), 1023), 0);
            const g = Math.max(Math.min(getValue(id + 1), 1023), 0);
            const b = Math.max(Math.min(getValue(id + 2), 1023), 0);
            const a = Math.max(Math.min(getValue(id + 3), 3), 0);
            return a << 30 | b << 20 | g << 10 | r;
          }
        case 'rgb10a2unorm':{
            const r = Math.round(Math.max(Math.min(getValue(id), 1), 0) * 1023);
            const g = Math.round(Math.max(Math.min(getValue(id + 1), 1), 0) * 1023);
            const b = Math.round(Math.max(Math.min(getValue(id + 2), 1), 0) * 1023);
            const a = Math.round(Math.max(Math.min(getValue(id + 3), 1), 0) * 3);
            return a << 30 | b << 20 | g << 10 | r;
          }
        case 'rg11b10ufloat':{
            const float11 = { zero: 0, one: 0x3c0, half: 0x380 }; // 11 bits: 1, 0, 0.5
            const float10 = { zero: 0, one: 0x1e0, half: 0x1c0 }; // 10 bits: 1, 0, 0.5
            const mapValue = (
            val,
            { zero, one, half }) =>
            val === 0 ? zero : val === 1 ? one : half;
            const r = mapValue(getValue(id), float11);
            const g = mapValue(getValue(id + 1), float11);
            const b = mapValue(getValue(id + 2), float10);
            return b << 22 | g << 11 | r;
          }
        default:
          unreachable(`unhandled format ${format}`);
          break;
      }
    })]
    );
    t.expectGPUBufferValuesEqual(buffer, expected);
  }
});

g.test('bgra8unorm_swizzle').
desc('Test bgra8unorm swizzling').
fn((t) => {
  t.skipIfDeviceDoesNotHaveFeature('bgra8unorm-storage');
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

  const buffer = ttu.copyWholeTextureToNewBufferSimple(t, texture, 0);
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
    const buffer = ttu.copyWholeTextureToNewBufferSimple(t, texture, m);
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
beforeAllSubcases((t) => {
  if (t.isCompatibility) {
    t.skipIf(
      t.params.baseLevel !== 0,
      'view base array layer must equal 0 in compatibility mode'
    );
    t.skipIf(
      t.params.arrayLevels !== kArrayLevels,
      'view array layers must equal texture array layers in compatibility mode'
    );
  }
}).
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

  const buffer = ttu.copyWholeTextureToNewBufferSimple(t, texture, 0);
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