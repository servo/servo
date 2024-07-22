/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { keysOf } from '../../../../../../common/util/data_tables.js';import { assert, range, unreachable } from '../../../../../../common/util/util.js';import {

  isCompressedFloatTextureFormat,
  isCompressedTextureFormat,
  isDepthOrStencilTextureFormat,
  kEncodableTextureFormats,
  kTextureFormatInfo } from
'../../../../../format_info.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { float32ToUint32 } from '../../../../../util/conversion.js';
import {
  align,
  clamp,
  dotProduct,
  hashU32,
  lcm,
  lerp,
  quantizeToF32 } from
'../../../../../util/math.js';
import {
  effectiveViewDimensionForDimension,
  physicalMipSizeFromTexture,
  reifyTextureDescriptor,
  virtualMipSize } from
'../../../../../util/texture/base.js';
import {
  kTexelRepresentationInfo,



  TexelComponent } from

'../../../../../util/texture/texel_data.js';
import { TexelView } from '../../../../../util/texture/texel_view.js';
import { createTextureFromTexelViews } from '../../../../../util/texture.js';
import { reifyExtent3D } from '../../../../../util/unions.js';



export const kSampleTypeInfo = {
  f32: {
    format: 'rgba8unorm'
  },
  i32: {
    format: 'rgba8sint'
  },
  u32: {
    format: 'rgba8uint'
  }
};

/**
 * Used for textureDimension, textureNumLevels, textureNumLayers
 */
export class WGSLTextureQueryTest extends GPUTest {
  executeAndExpectResult(code, view, expected) {
    const { device } = this;
    const module = device.createShaderModule({ code });
    const pipeline = device.createComputePipeline({
      layout: 'auto',
      compute: {
        module
      }
    });

    const resultBuffer = this.createBufferTracked({
      size: 16,
      usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC
    });

    const bindGroup = device.createBindGroup({
      layout: pipeline.getBindGroupLayout(0),
      entries: [
      { binding: 0, resource: view },
      { binding: 1, resource: { buffer: resultBuffer } }]

    });

    const encoder = device.createCommandEncoder();
    const pass = encoder.beginComputePass();
    pass.setPipeline(pipeline);
    pass.setBindGroup(0, bindGroup);
    pass.dispatchWorkgroups(1);
    pass.end();
    device.queue.submit([encoder.finish()]);

    const e = new Uint32Array(4);
    e.set(expected);
    this.expectGPUBufferValuesEqual(resultBuffer, e);
  }
}

function getLimitValue(v) {
  switch (v) {
    case Number.POSITIVE_INFINITY:
      return 1000;
    case Number.NEGATIVE_INFINITY:
      return -1000;
    default:
      return v;
  }
}

function getValueBetweenMinAndMaxTexelValueInclusive(
rep,
component,
normalized)
{
  assert(!!rep.numericRange);
  const perComponentRanges = rep.numericRange;
  const perComponentRange = perComponentRanges[component];
  const range = rep.numericRange;
  const { min, max } = perComponentRange ? perComponentRange : range;
  return lerp(getLimitValue(min), getLimitValue(max), normalized);
}

/**
 * We need the software rendering to do the same interpolation as the hardware
 * rendered so for -srgb formats we set the TexelView to an -srgb format as
 * TexelView handles this case. Note: It might be nice to add rgba32float-srgb
 * or something similar to TexelView.
 */
export function getTexelViewFormatForTextureFormat(format) {
  return format.endsWith('-srgb') ? 'rgba8unorm-srgb' : 'rgba32float';
}

const kTextureTypeInfo = {
  depth: {
    componentType: 'f32',
    resultType: 'vec4f',
    resultFormat: 'rgba32float'
  },
  float: {
    componentType: 'f32',
    resultType: 'vec4f',
    resultFormat: 'rgba32float'
  },
  'unfilterable-float': {
    componentType: 'f32',
    resultType: 'vec4f',
    resultFormat: 'rgba32float'
  },
  sint: {
    componentType: 'i32',
    resultType: 'vec4i',
    resultFormat: 'rgba32sint'
  },
  uint: {
    componentType: 'u32',
    resultType: 'vec4u',
    resultFormat: 'rgba32uint'
  }
};

function getTextureFormatTypeInfo(format) {
  const info = kTextureFormatInfo[format];
  const type = info.color?.type ?? info.depth?.type ?? info.stencil?.type;
  assert(!!type);
  return kTextureTypeInfo[type];
}

/**
 * given a texture type 'base', returns the base with the correct component for the given texture format.
 * eg: `getTextureType('texture_2d', someUnsignedIntTextureFormat)` -> `texture_2d<u32>`
 */
export function appendComponentTypeForFormatToTextureType(base, format) {
  return `${base}<${getTextureFormatTypeInfo(format).componentType}>`;
}

/**
 * Creates a TexelView filled with random values.
 */
export function createRandomTexelView(info)


{
  const rep = kTexelRepresentationInfo[info.format];
  const generator = (coords) => {
    const texel = {};
    for (const component of rep.componentOrder) {
      const rnd = hashU32(coords.x, coords.y, coords.z, component.charCodeAt(0));
      const normalized = clamp(rnd / 0xffffffff, { min: 0, max: 1 });
      texel[component] = getValueBetweenMinAndMaxTexelValueInclusive(rep, component, normalized);
    }
    return quantize(texel, rep);
  };
  return TexelView.fromTexelsAsColors(info.format, generator);
}

/**
 * Creates a mip chain of TexelViews filled with random values
 */
export function createRandomTexelViewMipmap(info)




{
  const mipLevelCount = info.mipLevelCount ?? 1;
  const dimension = info.dimension ?? '2d';
  return range(mipLevelCount, (i) =>
  createRandomTexelView({
    format: info.format,
    size: virtualMipSize(dimension, info.size, i)
  })
  );
}

// Because it's easy to deal with if these types are all array of number






const kTextureCallArgNames = [
'coords',
'mipLevel',
'arrayIndex',
'ddx',
'ddy',
'offset'];


















function toArray(coords) {
  if (coords instanceof Array) {
    return coords;
  }
  return [coords];
}

function quantize(texel, repl) {
  return repl.bitsToNumber(repl.unpackBits(new Uint8Array(repl.pack(repl.encode(texel)))));
}

function apply(a, b, op) {
  assert(a.length === b.length, `apply(${a}, ${b}): arrays must have same length`);
  return a.map((v, i) => op(v, b[i]));
}

/**
 * At the corner of a cubemap we need to sample just 3 texels, not 4.
 * The texels are in
 *
 *   0:  (u,v)
 *   1:  (u + 1, v)
 *   2:  (u, v + 1)
 *   3:  (u + 1, v + 1)
 *
 * We pass in the original 2d (converted from cubemap) texture coordinate.
 * If it's within half a pixel of the edge in both directions then it's
 * a corner so we return the index of the one texel that's not needed.
 * Otherwise we return -1.
 */
function getUnusedCubeCornerSampleIndex(textureSize, coords) {
  const u = coords[0] * textureSize;
  const v = coords[1] * textureSize;
  if (v < 0.5) {
    if (u < 0.5) {
      return 0;
    } else if (u >= textureSize - 0.5) {
      return 1;
    }
  } else if (v >= textureSize - 0.5) {
    if (u < 0.5) {
      return 2;
    } else if (u >= textureSize - 0.5) {
      return 3;
    }
  }
  return -1;
}

const add = (a, b) => apply(a, b, (x, y) => x + y);







/**
 * Converts the src texel representation to an RGBA representation.
 */
function convertPerTexelComponentToResultFormat(
src,
format)
{
  const rep = kTexelRepresentationInfo[format];
  const out = { R: 0, G: 0, B: 0, A: 1 };
  for (const component of rep.componentOrder) {
    switch (component) {
      case 'Stencil':
      case 'Depth':
        out.R = src[component];
        break;
      default:
        assert(out[component] !== undefined); // checks that component = R, G, B or A
        out[component] = src[component];
    }
  }
  return out;
}

/**
 * Convert RGBA result format to texel view format of src texture.
 * Effectively this converts something like { R: 0.1, G: 0, B: 0, A: 1 }
 * to { Depth: 0.1 }
 */
function convertResultFormatToTexelViewFormat(
src,
format)
{
  const rep = kTexelRepresentationInfo[format];
  const out = {};
  for (const component of rep.componentOrder) {
    out[component] = src[component] ?? src.R;
  }
  return out;
}

/**
 * Returns the expect value for a WGSL builtin texture function for a single
 * mip level
 */
export function softwareTextureReadMipLevel(
call,
texture,
sampler,
mipLevel)
{
  const { format } = texture.texels[mipLevel];
  const rep = kTexelRepresentationInfo[format];
  const textureSize = virtualMipSize(
    texture.descriptor.dimension || '2d',
    texture.descriptor.size,
    mipLevel
  );
  const addressMode = [
  sampler?.addressModeU ?? 'clamp-to-edge',
  sampler?.addressModeV ?? 'clamp-to-edge',
  sampler?.addressModeW ?? 'clamp-to-edge'];


  const load = (at) =>
  texture.texels[mipLevel].color({
    x: Math.floor(at[0]),
    y: Math.floor(at[1] ?? 0),
    z: Math.floor(at[2] ?? 0)
  });

  const isCube = texture.viewDescriptor.dimension === 'cube';

  switch (call.builtin) {
    case 'textureSample':{
        let coords = toArray(call.coords);

        if (isCube) {
          coords = convertCubeCoordToNormalized3DTextureCoord(coords);
        }

        // convert normalized to absolute texel coordinate
        // ┌───┬───┬───┬───┐
        // │ a │   │   │   │  norm: a = 1/8, b = 7/8
        // ├───┼───┼───┼───┤   abs: a = 0,   b = 3
        // │   │   │   │   │
        // ├───┼───┼───┼───┤
        // │   │   │   │   │
        // ├───┼───┼───┼───┤
        // │   │   │   │ b │
        // └───┴───┴───┴───┘
        let at = coords.map((v, i) => v * textureSize[i] - 0.5);

        // Apply offset in whole texel units
        // This means the offset is added at each mip level in texels. There's no
        // scaling for each level.
        if (call.offset !== undefined) {
          at = add(at, toArray(call.offset));
        }

        const samples = [];

        const filter = sampler?.minFilter ?? 'nearest';
        switch (filter) {
          case 'linear':{
              // 'p0' is the lower texel for 'at'
              const p0 = at.map((v) => Math.floor(v));
              // 'p1' is the higher texel for 'at'
              // If it's cube then don't advance Z.
              const p1 = p0.map((v, i) => v + (isCube ? i === 2 ? 0 : 1 : 1));

              // interpolation weights for p0 and p1
              const p1W = at.map((v, i) => v - p0[i]);
              const p0W = p1W.map((v) => 1 - v);

              switch (coords.length) {
                case 1:
                  samples.push({ at: p0, weight: p0W[0] });
                  samples.push({ at: p1, weight: p1W[0] });
                  break;
                case 2:{
                    samples.push({ at: p0, weight: p0W[0] * p0W[1] });
                    samples.push({ at: [p1[0], p0[1]], weight: p1W[0] * p0W[1] });
                    samples.push({ at: [p0[0], p1[1]], weight: p0W[0] * p1W[1] });
                    samples.push({ at: p1, weight: p1W[0] * p1W[1] });
                    break;
                  }
                case 3:{
                    // cube sampling, here in the software renderer, is the same
                    // as 2d sampling. We'll sample at most 4 texels. The weights are
                    // the same as if it was just one plane. If the points fall outside
                    // the slice they'll be wrapped by wrapFaceCoordToCubeFaceAtEdgeBoundaries
                    // below.
                    if (isCube) {
                      samples.push({ at: p0, weight: p0W[0] * p0W[1] });
                      samples.push({ at: [p1[0], p0[1], p0[2]], weight: p1W[0] * p0W[1] });
                      samples.push({ at: [p0[0], p1[1], p0[2]], weight: p0W[0] * p1W[1] });
                      samples.push({ at: p1, weight: p1W[0] * p1W[1] });
                      const ndx = getUnusedCubeCornerSampleIndex(textureSize[0], coords);
                      if (ndx >= 0) {
                        // # Issues with corners of cubemaps
                        //
                        // note: I tried multiple things here
                        //
                        // 1. distribute 1/3 of the weight of the removed sample to each of the remaining samples
                        // 2. distribute 1/2 of the weight of the removed sample to the 2 samples that are not the "main" sample.
                        // 3. normalize the weights of the remaining 3 samples.
                        //
                        // none of them matched the M1 in all cases. Checking the dEQP I found this comment
                        //
                        // > If any of samples is out of both edges, implementations can do pretty much anything according to spec.
                        // https://github.com/KhronosGroup/VK-GL-CTS/blob/d2d6aa65607383bb29c8398fe6562c6b08b4de57/framework/common/tcuTexCompareVerifier.cpp#L882
                        //
                        // If I understand this correctly it matches the OpenGL ES 3.1 spec it says
                        // it's implementation defined.
                        //
                        // > OpenGL ES 3.1 section 8.12.1 Seamless Cubemap Filtering
                        // >
                        // > -  If a texture sample location would lie in the texture
                        // >    border in both u and v (in one of the corners of the
                        // >    cube), there is no unique neighboring face from which to
                        // >    extract one texel. The recommended method to generate this
                        // >    texel is to average the values of the three available
                        // >    samples. However, implementations are free to construct
                        // >    this fourth texel in another way, so long as, when the
                        // >    three available samples have the same value, this texel
                        // >    also has that value.
                        //
                        // I'm not sure what "average the values of the three available samples"
                        // means. To me that would be (a+b+c)/3 or in other words, set all the
                        // weights to 0.33333 but that's not what the M1 is doing.
                        unreachable('corners of cubemaps are not testable');
                      }
                    } else {
                      const p = [p0, p1];
                      const w = [p0W, p1W];
                      for (let z = 0; z < 2; ++z) {
                        for (let y = 0; y < 2; ++y) {
                          for (let x = 0; x < 2; ++x) {
                            samples.push({
                              at: [p[x][0], p[y][1], p[z][2]],
                              weight: w[x][0] * w[y][1] * w[z][2]
                            });
                          }
                        }
                      }
                    }
                    break;
                  }
              }
              break;
            }
          case 'nearest':{
              const p = at.map((v) => Math.round(quantizeToF32(v)));
              samples.push({ at: p, weight: 1 });
              break;
            }
          default:
            unreachable();
        }

        const out = {};
        const ss = [];
        for (const sample of samples) {
          const c = isCube ?
          wrapFaceCoordToCubeFaceAtEdgeBoundaries(textureSize[0], sample.at) :
          applyAddressModesToCoords(addressMode, textureSize, sample.at);
          const v = load(c);
          ss.push(v);
          for (const component of rep.componentOrder) {
            out[component] = (out[component] ?? 0) + v[component] * sample.weight;
          }
        }

        return convertPerTexelComponentToResultFormat(out, format);
      }
    case 'textureLoad':{
        const c = applyAddressModesToCoords(addressMode, textureSize, call.coords);
        return convertPerTexelComponentToResultFormat(load(c), format);
      }
  }
}

/**
 * The software version of a texture builtin (eg: textureSample)
 * Note that this is not a complete implementation. Rather it's only
 * what's needed to generate the correct expected value for the tests.
 */
export function softwareTextureRead(
call,
texture,
sampler)
{
  assert(call.ddx !== undefined);
  assert(call.ddy !== undefined);
  const rep = kTexelRepresentationInfo[texture.texels[0].format];
  const texSize = reifyExtent3D(texture.descriptor.size);
  const textureSize = [texSize.width, texSize.height];

  // ddx and ddy are the values that would be passed to textureSampleGrad
  // If we're emulating textureSample then they're the computed derivatives
  // such that if we passed them to textureSampleGrad they'd produce the
  // same result.
  const ddx = typeof call.ddx === 'number' ? [call.ddx] : call.ddx;
  const ddy = typeof call.ddy === 'number' ? [call.ddy] : call.ddy;

  // Compute the mip level the same way textureSampleGrad does
  const scaledDdx = ddx.map((v, i) => v * textureSize[i]);
  const scaledDdy = ddy.map((v, i) => v * textureSize[i]);
  const dotDDX = dotProduct(scaledDdx, scaledDdx);
  const dotDDY = dotProduct(scaledDdy, scaledDdy);
  const deltaMax = Math.max(dotDDX, dotDDY);
  // MAINTENANCE_TODO: handle texture view baseMipLevel and mipLevelCount?
  const mipLevel = 0.5 * Math.log2(deltaMax);

  const mipLevelCount = texture.texels.length;
  const maxLevel = mipLevelCount - 1;

  switch (sampler.mipmapFilter) {
    case 'linear':{
        const clampedMipLevel = clamp(mipLevel, { min: 0, max: maxLevel });
        const baseMipLevel = Math.floor(clampedMipLevel);
        const nextMipLevel = Math.ceil(clampedMipLevel);
        const t0 = softwareTextureReadMipLevel(call, texture, sampler, baseMipLevel);
        const t1 = softwareTextureReadMipLevel(call, texture, sampler, nextMipLevel);
        const mix = mipLevel % 1;
        const values = [
        { v: t0, weight: 1 - mix },
        { v: t1, weight: mix }];

        const out = {};
        for (const { v, weight } of values) {
          for (const component of rep.componentOrder) {
            out[component] = (out[component] ?? 0) + v[component] * weight;
          }
        }
        return out;
      }
    default:{
        const baseMipLevel = Math.floor(
          clamp(mipLevel + 0.5, { min: 0, max: texture.texels.length - 1 })
        );
        return softwareTextureReadMipLevel(call, texture, sampler, baseMipLevel);
      }
  }
}








/**
 * out of bounds is defined as any of the following being true
 *
 * * coords is outside the range [0, textureDimensions(t, level))
 * * array_index is outside the range [0, textureNumLayers(t))
 * * level is outside the range [0, textureNumLevels(t))
 * * sample_index is outside the range [0, textureNumSamples(s))
 */
function isOutOfBoundsCall(texture, call) {
  assert(call.mipLevel !== undefined);
  assert(call.coords !== undefined);
  assert(call.offset === undefined);

  const desc = reifyTextureDescriptor(texture.descriptor);

  const { coords, mipLevel, arrayIndex, sampleIndex } = call;

  if (mipLevel < 0 || mipLevel >= desc.mipLevelCount) {
    return true;
  }

  const size = virtualMipSize(
    texture.descriptor.dimension || '2d',
    texture.descriptor.size,
    mipLevel
  );

  for (let i = 0; i < coords.length; ++i) {
    const v = coords[i];
    if (v < 0 || v >= size[i]) {
      return true;
    }
  }

  if (arrayIndex !== undefined) {
    const size = reifyExtent3D(desc.size);
    if (arrayIndex < 0 || arrayIndex >= size.depthOrArrayLayers) {
      return true;
    }
  }

  if (sampleIndex !== undefined) {
    if (sampleIndex < 0 || sampleIndex >= desc.sampleCount) {
      return true;
    }
  }

  return false;
}

/**
 * For a texture builtin with no sampler (eg textureLoad),
 * any out of bounds access is allowed to return one of:
 *
 * * the value of any texel in the texture
 * * 0,0,0,0 or 0,0,0,1 if not a depth texture
 * * 0 if a depth texture
 */
function okBecauseOutOfBounds(
texture,
call,
gotRGBA,
maxFractionalDiff)
{
  if (!isOutOfBoundsCall(texture, call)) {
    return false;
  }

  if (texture.descriptor.format.includes('depth')) {
    if (gotRGBA.R === 0) {
      return true;
    }
  } else {
    if (
    gotRGBA.R === 0 &&
    gotRGBA.B === 0 &&
    gotRGBA.G === 0 && (
    gotRGBA.A === 0 || gotRGBA.A === 1))
    {
      return true;
    }
  }

  for (let mipLevel = 0; mipLevel < texture.texels.length; ++mipLevel) {
    const mipTexels = texture.texels[mipLevel];
    const size = virtualMipSize(
      texture.descriptor.dimension || '2d',
      texture.descriptor.size,
      mipLevel
    );
    for (let z = 0; z < size[2]; ++z) {
      for (let y = 0; y < size[1]; ++y) {
        for (let x = 0; x < size[0]; ++x) {
          const texel = mipTexels.color({ x, y, z });
          const rgba = convertPerTexelComponentToResultFormat(texel, mipTexels.format);
          if (texelsApproximatelyEqual(gotRGBA, rgba, mipTexels.format, maxFractionalDiff)) {
            return true;
          }
        }
      }
    }
  }

  return false;
}

const kRGBAComponents = [
TexelComponent.R,
TexelComponent.G,
TexelComponent.B,
TexelComponent.A];


const kRComponent = [TexelComponent.R];

function texelsApproximatelyEqual(
gotRGBA,
expectRGBA,
format,
maxFractionalDiff)
{
  const rep = kTexelRepresentationInfo[format];
  const got = convertResultFormatToTexelViewFormat(gotRGBA, format);
  const expect = convertResultFormatToTexelViewFormat(expectRGBA, format);
  const gULP = rep.bitsToULPFromZero(rep.numberToBits(got));
  const eULP = rep.bitsToULPFromZero(rep.numberToBits(expect));

  const rgbaComponentsToCheck = isDepthOrStencilTextureFormat(format) ?
  kRComponent :
  kRGBAComponents;

  for (const component of rgbaComponentsToCheck) {
    const g = gotRGBA[component];
    const e = expectRGBA[component];
    const absDiff = Math.abs(g - e);
    const ulpDiff = Math.abs(gULP[component] - eULP[component]);
    if (ulpDiff > 3 && absDiff > maxFractionalDiff) {
      return false;
    }
  }
  return true;
}

/**
 * Checks the result of each call matches the expected result.
 */
export async function checkCallResults(
t,
texture,
textureType,
sampler,
calls,
results)
{
  const errs = [];
  const rep = kTexelRepresentationInfo[texture.texels[0].format];
  const maxFractionalDiff =
  sampler?.minFilter === 'linear' ||
  sampler?.magFilter === 'linear' ||
  sampler?.mipmapFilter === 'linear' ?
  getMaxFractionalDiffForTextureFormat(texture.descriptor.format) :
  0;

  for (let callIdx = 0; callIdx < calls.length; callIdx++) {
    const call = calls[callIdx];
    const gotRGBA = results[callIdx];
    const expectRGBA = softwareTextureReadMipLevel(call, texture, sampler, 0);

    if (
    texelsApproximatelyEqual(gotRGBA, expectRGBA, texture.texels[0].format, maxFractionalDiff))
    {
      continue;
    }

    if (!sampler && okBecauseOutOfBounds(texture, call, gotRGBA, maxFractionalDiff)) {
      continue;
    }

    const got = convertResultFormatToTexelViewFormat(gotRGBA, texture.texels[0].format);
    const expect = convertResultFormatToTexelViewFormat(expectRGBA, texture.texels[0].format);
    const gULP = rep.bitsToULPFromZero(rep.numberToBits(got));
    const eULP = rep.bitsToULPFromZero(rep.numberToBits(expect));
    for (const component of rep.componentOrder) {
      const g = got[component];
      const e = expect[component];
      const absDiff = Math.abs(g - e);
      const ulpDiff = Math.abs(gULP[component] - eULP[component]);
      const relDiff = absDiff / Math.max(Math.abs(g), Math.abs(e));
      if (ulpDiff > 3 && absDiff > maxFractionalDiff) {
        const desc = describeTextureCall(call);
        errs.push(`component was not as expected:
      call: ${desc}  // #${callIdx}
 component: ${component}
       got: ${g}
  expected: ${e}
  abs diff: ${absDiff.toFixed(4)}
  rel diff: ${(relDiff * 100).toFixed(2)}%
  ulp diff: ${ulpDiff}
`);
        if (sampler) {
          const expectedSamplePoints = [
          'expected:',
          ...(await identifySamplePoints(texture, (texels) => {
            return Promise.resolve(
              softwareTextureReadMipLevel(
                call,
                {
                  texels: [texels],
                  descriptor: texture.descriptor,
                  viewDescriptor: texture.viewDescriptor
                },
                sampler,
                0
              )
            );
          }))];

          const gotSamplePoints = [
          'got:',
          ...(await identifySamplePoints(texture, async (texels) => {
            const gpuTexture = createTextureFromTexelViews(t, [texels], texture.descriptor);
            const result = (
            await doTextureCalls(t, gpuTexture, texture.viewDescriptor, textureType, sampler, [
            call]
            ))[
            0];
            gpuTexture.destroy();
            return result;
          }))];

          errs.push('  sample points:');
          errs.push(layoutTwoColumns(expectedSamplePoints, gotSamplePoints).join('\n'));
          errs.push('', '');
        }
      }
    }
  }

  return errs.length > 0 ? new Error(errs.join('\n')) : undefined;
}

/**
 * "Renders a quad" to a TexelView with the given parameters,
 * sampling from the given Texture.
 */
export function softwareRasterize(
texture,
sampler,
targetSize,
options)
{
  const [width, height] = targetSize;
  const { ddx = 1, ddy = 1, uvwStart = [0, 0] } = options;
  const format = 'rgba32float';

  const textureSize = reifyExtent3D(texture.descriptor.size);

  // MAINTENANCE_TODO: Consider passing these in as a similar computation
  // happens in putDataInTextureThenDrawAndCheckResultsComparedToSoftwareRasterizer.
  // The issue is there, the calculation is "what do we need to multiply the unitQuad
  // by to get the derivatives we want". The calculation here is "what coordinate
  // will we get for a given frag coordinate". It turns out to be the same calculation
  // but needs rephrasing them so they are more obviously the same would help
  // consolidate them into one calculation.
  const screenSpaceUMult = ddx * width / textureSize.width;
  const screenSpaceVMult = ddy * height / textureSize.height;

  const rep = kTexelRepresentationInfo[format];

  const expData = new Float32Array(width * height * 4);
  for (let y = 0; y < height; ++y) {
    const fragY = height - y - 1 + 0.5;
    for (let x = 0; x < width; ++x) {
      const fragX = x + 0.5;
      // This code calculates the same value that will be passed to
      // `textureSample` in the fragment shader for a given frag coord (see the
      // WGSL code which uses the same formula, but using interpolation). That
      // shader renders a clip space quad and includes a inter-stage "uv"
      // coordinates that start with a unit quad (0,0) to (1,1) and is
      // multiplied by ddx,ddy and as added in uStart and vStart
      //
      // uv = unitQuad * vec2(ddx, ddy) + vec2(vStart, uStart);
      //
      // softwareTextureRead<T> simulates a single call to `textureSample` so
      // here we're computing the `uv` value that will be passed for a
      // particular fragment coordinate. fragX / width, fragY / height provides
      // the unitQuad value.
      //
      // ddx and ddy in this case are the derivative values we want to test. We
      // pass those into the softwareTextureRead<T> as they would normally be
      // derived from the change in coord.
      const coords = [
      fragX / width * screenSpaceUMult + uvwStart[0],
      fragY / height * screenSpaceVMult + uvwStart[1]];

      const call = {
        builtin: 'textureSample',
        coordType: 'f',
        coords,
        ddx: [ddx / textureSize.width, 0],
        ddy: [0, ddy / textureSize.height],
        offset: options.offset
      };
      const sample = softwareTextureRead(call, texture, sampler);
      const rgba = { R: 0, G: 0, B: 0, A: 1, ...sample };
      const asRgba32Float = new Float32Array(rep.pack(rgba));
      expData.set(asRgba32Float, (y * width + x) * 4);
    }
  }

  return TexelView.fromTextureDataByReference(format, new Uint8Array(expData.buffer), {
    bytesPerRow: width * 4 * 4,
    rowsPerImage: height,
    subrectOrigin: [0, 0, 0],
    subrectSize: targetSize
  });
}

/**
 * Render textured quad to an rgba32float texture.
 */
export function drawTexture(
t,
texture,
samplerDesc,
options)
{
  const device = t.device;
  const { ddx = 1, ddy = 1, uvwStart = [0, 0, 0], offset } = options;

  const format = 'rgba32float';
  const renderTarget = t.createTextureTracked({
    format,
    size: [32, 32],
    usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT
  });

  // Compute the amount we need to multiply the unitQuad by get the
  // derivatives we want.
  const uMult = ddx * renderTarget.width / texture.width;
  const vMult = ddy * renderTarget.height / texture.height;

  const offsetWGSL = offset ? `, vec2i(${offset[0]},${offset[1]})` : '';

  const code = `
struct InOut {
  @builtin(position) pos: vec4f,
  @location(0) uv: vec2f,
};

@vertex fn vs(@builtin(vertex_index) vertex_index : u32) -> InOut {
  let positions = array(
    vec2f(-1,  1), vec2f( 1,  1),
    vec2f(-1, -1), vec2f( 1, -1),
  );
  let pos = positions[vertex_index];
  return InOut(
    vec4f(pos, 0, 1),
    (pos * 0.5 + 0.5) * vec2f(${uMult}, ${vMult}) + vec2f(${uvwStart[0]}, ${uvwStart[1]}),
  );
}

@group(0) @binding(0) var          T    : texture_2d<f32>;
@group(0) @binding(1) var          S    : sampler;

@fragment fn fs(v: InOut) -> @location(0) vec4f {
  return textureSample(T, S, v.uv${offsetWGSL});
}
`;

  const shaderModule = device.createShaderModule({ code });

  const pipeline = device.createRenderPipeline({
    layout: 'auto',
    vertex: { module: shaderModule },
    fragment: {
      module: shaderModule,
      targets: [{ format }]
    },
    primitive: { topology: 'triangle-strip' }
  });

  const sampler = device.createSampler(samplerDesc);

  const bindGroup = device.createBindGroup({
    layout: pipeline.getBindGroupLayout(0),
    entries: [
    { binding: 0, resource: texture.createView() },
    { binding: 1, resource: sampler }]

  });

  const encoder = device.createCommandEncoder();

  const renderPass = encoder.beginRenderPass({
    colorAttachments: [{ view: renderTarget.createView(), loadOp: 'clear', storeOp: 'store' }]
  });

  renderPass.setPipeline(pipeline);
  renderPass.setBindGroup(0, bindGroup);
  renderPass.draw(4);
  renderPass.end();
  device.queue.submit([encoder.finish()]);

  return renderTarget;
}

function getMaxFractionalDiffForTextureFormat(format) {
  // Note: I'm not sure what we should do here. My assumption is, given texels
  // have random values, the difference between 2 texels can be very large. In
  // the current version, for a float texture they can be +/- 1000 difference.
  // Sampling is very GPU dependent. So if one pixel gets a random value of
  // -1000 and the neighboring pixel gets +1000 then any slight variation in how
  // sampling is applied will generate a large difference when interpolating
  // between -1000 and +1000.
  //
  // We could make some entry for every format but for now I just put the
  // tolerances here based on format texture suffix.
  //
  // It's possible the math in the software rasterizer is just bad but the
  // results certainly seem close.
  //
  // These tolerances started from the OpenGL ES dEQP tests.
  // Those tests always render to an rgba8unorm texture. The shaders do effectively
  //
  //   result = textureSample(...) * scale + bias
  //
  // to get the results in a 0.0 to 1.0 range. After reading the values back they
  // expand them to their original ranges with
  //
  //   value = (result - bias) / scale;
  //
  // Tolerances from dEQP
  // --------------------
  // 8unorm: 3.9 / 255
  // 8snorm: 7.9 / 128
  // 2unorm: 7.9 / 512
  // ufloat: 156.249
  //  float: 31.2498
  //
  // The numbers below have been set empirically to get the tests to pass on all
  // devices. The devices with the most divergence from the calculated expected
  // values are MacOS Intel and AMD.
  //
  // MAINTENANCE_TODO: Double check the software rendering math and lower these
  // tolerances if possible.

  if (format.includes('8unorm')) {
    return 7 / 255;
  } else if (format.includes('2unorm')) {
    return 9 / 512;
  } else if (format.includes('unorm')) {
    return 7 / 255;
  } else if (format.includes('8snorm')) {
    return 7.9 / 128;
  } else if (format.includes('snorm')) {
    return 7.9 / 128;
  } else if (format.endsWith('ufloat')) {
    return 156.249;
  } else if (format.endsWith('float')) {
    return 44;
  } else {
    // It's likely an integer format. In any case, zero tolerance is passable.
    return 0;
  }
}

export function checkTextureMatchesExpectedTexelView(
t,
format,
actualTexture,
expectedTexelView)
{
  const maxFractionalDiff = getMaxFractionalDiffForTextureFormat(format);
  t.expectTexelViewComparisonIsOkInTexture(
    { texture: actualTexture },
    expectedTexelView,
    [actualTexture.width, actualTexture.height],
    { maxFractionalDiff }
  );
}

/**
 * Puts data in a texture. Renders a quad to a rgba32float. Then "software renders"
 * to a TexelView the expected result and compares the rendered texture to the
 * expected TexelView.
 */
export async function putDataInTextureThenDrawAndCheckResultsComparedToSoftwareRasterizer(


t,
descriptor,
viewDescriptor,
samplerDesc,
options)
{
  const { texture, texels } = await createTextureWithRandomDataAndGetTexels(t, descriptor);

  const actualTexture = drawTexture(t, texture, samplerDesc, options);
  const expectedTexelView = softwareRasterize(
    { descriptor, texels, viewDescriptor },
    samplerDesc,
    [actualTexture.width, actualTexture.height],
    options
  );

  checkTextureMatchesExpectedTexelView(t, texture.format, actualTexture, expectedTexelView);
}

const sumOfCharCodesOfString = (s) =>
String(s).
split('').
reduce((sum, c) => sum + c.charCodeAt(0), 0);

/**
 * Makes a function that fills a block portion of a Uint8Array with random valid data
 * for an astc block.
 *
 * The astc format is fairly complicated. For now we do the simplest thing.
 * which is to set the block as a "void-extent" block (a solid color).
 * This makes our test have far less precision.
 *
 * MAINTENANCE_TODO: generate other types of astc blocks. One option would
 * be to randomly select from set of pre-made blocks.
 *
 * See Spec:
 * https://registry.khronos.org/OpenGL/extensions/KHR/KHR_texture_compression_astc_hdr.txt
 */
function makeAstcBlockFiller(format) {
  const info = kTextureFormatInfo[format];
  const bytesPerBlock = info.color.bytes;
  return (data, offset, hashBase) => {
    // set the block to be a void-extent block
    data.set(
      [
      0b1111_1100, // 0
      0b1111_1101, // 1
      0b1111_1111, // 2
      0b1111_1111, // 3
      0b1111_1111, // 4
      0b1111_1111, // 5
      0b1111_1111, // 6
      0b1111_1111 // 7
      ],
      offset
    );
    // fill the rest of the block with random data
    const end = offset + bytesPerBlock;
    for (let i = offset + 8; i < end; ++i) {
      data[i] = hashU32(hashBase, i);
    }
  };
}

/**
 * Makes a function that fills a block portion of a Uint8Array with random bytes.
 */
function makeRandomBytesBlockFiller(format) {
  const info = kTextureFormatInfo[format];
  const bytesPerBlock = info.color.bytes;
  return (data, offset, hashBase) => {
    const end = offset + bytesPerBlock;
    for (let i = offset; i < end; ++i) {
      data[i] = hashU32(hashBase, i);
    }
  };
}

function getBlockFiller(format) {
  if (format.startsWith('astc')) {
    return makeAstcBlockFiller(format);
  } else {
    return makeRandomBytesBlockFiller(format);
  }
}

/**
 * Fills a texture with random data.
 */
export function fillTextureWithRandomData(device, texture) {
  assert(!isCompressedFloatTextureFormat(texture.format));
  const info = kTextureFormatInfo[texture.format];
  const hashBase =
  sumOfCharCodesOfString(texture.format) +
  sumOfCharCodesOfString(texture.dimension) +
  texture.width +
  texture.height +
  texture.depthOrArrayLayers +
  texture.mipLevelCount;
  const bytesPerBlock = info.color.bytes;
  const fillBlock = getBlockFiller(texture.format);
  for (let mipLevel = 0; mipLevel < texture.mipLevelCount; ++mipLevel) {
    const size = physicalMipSizeFromTexture(texture, mipLevel);
    const blocksAcross = Math.ceil(size[0] / info.blockWidth);
    const blocksDown = Math.ceil(size[1] / info.blockHeight);
    const bytesPerRow = blocksAcross * bytesPerBlock;
    const bytesNeeded = bytesPerRow * blocksDown * size[2];
    const data = new Uint8Array(bytesNeeded);
    for (let offset = 0; offset < bytesNeeded; offset += bytesPerBlock) {
      fillBlock(data, offset, hashBase);
    }
    device.queue.writeTexture(
      { texture, mipLevel },
      data,
      { bytesPerRow, rowsPerImage: blocksDown },
      size
    );
  }
}

const s_readTextureToRGBA32DeviceToPipeline = new WeakMap(


);

// MAINTENANCE_TODO: remove cast once textureBindingViewDimension is added to IDL
function getEffectiveViewDimension(
t,
descriptor)
{
  const { textureBindingViewDimension } = descriptor;


  const size = reifyExtent3D(descriptor.size);
  return effectiveViewDimensionForDimension(
    textureBindingViewDimension,
    descriptor.dimension,
    size.depthOrArrayLayers
  );
}

export async function readTextureToTexelViews(
t,
texture,
descriptor,
format)
{
  const device = t.device;
  const viewDimensionToPipelineMap =
  s_readTextureToRGBA32DeviceToPipeline.get(device) ??
  new Map();
  s_readTextureToRGBA32DeviceToPipeline.set(device, viewDimensionToPipelineMap);

  const viewDimension = getEffectiveViewDimension(t, descriptor);
  let pipeline = viewDimensionToPipelineMap.get(viewDimension);
  if (!pipeline) {
    let textureWGSL;
    let loadWGSL;
    switch (viewDimension) {
      case '2d':
        textureWGSL = 'texture_2d<f32>';
        loadWGSL = 'textureLoad(tex, global_invocation_id.xy, mipLevel)';
        break;
      case 'cube-array': // cube-array doesn't exist in compat so we can just use 2d_array for this
      case '2d-array':
        textureWGSL = 'texture_2d_array<f32>';
        loadWGSL = `
          textureLoad(
              tex,
              global_invocation_id.xy,
              global_invocation_id.z,
              mipLevel)`;
        break;
      case '3d':
        textureWGSL = 'texture_3d<f32>';
        loadWGSL = 'textureLoad(tex, global_invocation_id.xyz, mipLevel)';
        break;
      case 'cube':
        textureWGSL = 'texture_cube<f32>';
        loadWGSL = `
          textureLoadCubeAs2DArray(tex, global_invocation_id.xy, global_invocation_id.z, mipLevel);
        `;
        break;
      default:
        unreachable(`unsupported view: ${viewDimension}`);
    }
    const module = device.createShaderModule({
      code: `
        const faceMat = array(
          mat3x3f( 0,  0,  -2,  0, -2,   0,  1,  1,   1),   // pos-x
          mat3x3f( 0,  0,   2,  0, -2,   0, -1,  1,  -1),   // neg-x
          mat3x3f( 2,  0,   0,  0,  0,   2, -1,  1,  -1),   // pos-y
          mat3x3f( 2,  0,   0,  0,  0,  -2, -1, -1,   1),   // neg-y
          mat3x3f( 2,  0,   0,  0, -2,   0, -1,  1,   1),   // pos-z
          mat3x3f(-2,  0,   0,  0, -2,   0,  1,  1,  -1));  // neg-z

        // needed for compat mode.
        fn textureLoadCubeAs2DArray(tex: texture_cube<f32>, coord: vec2u, layer: u32, mipLevel: u32) -> vec4f {
          // convert texel coord normalized coord
          let size = textureDimensions(tex, mipLevel);
          let uv = (vec2f(coord) + 0.5) / vec2f(size.xy);

          // convert uv + layer into cube coord
          let cubeCoord = faceMat[layer] * vec3f(uv, 1.0);

          return textureSampleLevel(tex, smp, cubeCoord, f32(mipLevel));
        }

        @group(0) @binding(0) var<uniform> mipLevel: u32;
        @group(0) @binding(1) var tex: ${textureWGSL};
        @group(0) @binding(2) var smp: sampler;
        @group(0) @binding(3) var<storage, read_write> data: array<vec4f>;

        @compute @workgroup_size(1) fn cs(
          @builtin(global_invocation_id) global_invocation_id : vec3<u32>) {
          _ = smp;
          let size = textureDimensions(tex, mipLevel);
          let ndx = global_invocation_id.z * size.x * size.y +
                    global_invocation_id.y * size.x +
                    global_invocation_id.x;
          data[ndx] = ${loadWGSL};
        }
      `
    });
    pipeline = device.createComputePipeline({ layout: 'auto', compute: { module } });
    viewDimensionToPipelineMap.set(viewDimension, pipeline);
  }

  const encoder = device.createCommandEncoder();

  const readBuffers = [];
  for (let mipLevel = 0; mipLevel < texture.mipLevelCount; ++mipLevel) {
    const size = virtualMipSize(texture.dimension, texture, mipLevel);

    const uniformValues = new Uint32Array([mipLevel, 0, 0, 0]); // min size is 16 bytes
    const uniformBuffer = t.createBufferTracked({
      size: uniformValues.byteLength,
      usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST
    });
    device.queue.writeBuffer(uniformBuffer, 0, uniformValues);

    const storageBuffer = t.createBufferTracked({
      size: size[0] * size[1] * size[2] * 4 * 4, // rgba32float
      usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC
    });

    const readBuffer = t.createBufferTracked({
      size: storageBuffer.size,
      usage: GPUBufferUsage.MAP_READ | GPUBufferUsage.COPY_DST
    });
    readBuffers.push({ size, readBuffer });

    const sampler = device.createSampler();

    const bindGroup = device.createBindGroup({
      layout: pipeline.getBindGroupLayout(0),
      entries: [
      { binding: 0, resource: { buffer: uniformBuffer } },
      { binding: 1, resource: texture.createView({ dimension: viewDimension }) },
      { binding: 2, resource: sampler },
      { binding: 3, resource: { buffer: storageBuffer } }]

    });

    const pass = encoder.beginComputePass();
    pass.setPipeline(pipeline);
    pass.setBindGroup(0, bindGroup);
    pass.dispatchWorkgroups(...size);
    pass.end();
    encoder.copyBufferToBuffer(storageBuffer, 0, readBuffer, 0, readBuffer.size);
  }

  device.queue.submit([encoder.finish()]);

  const texelViews = [];

  for (const { readBuffer, size } of readBuffers) {
    await readBuffer.mapAsync(GPUMapMode.READ);

    // need a copy of the data since unmapping will nullify the typedarray view.
    const data = new Float32Array(readBuffer.getMappedRange()).slice();
    readBuffer.unmap();

    texelViews.push(
      TexelView.fromTexelsAsColors(format, (coord) => {
        const offset = (coord.z * size[0] * size[1] + coord.y * size[0] + coord.x) * 4;
        return {
          R: data[offset + 0],
          G: data[offset + 1],
          B: data[offset + 2],
          A: data[offset + 3]
        };
      })
    );
  }

  return texelViews;
}

/**
 * Fills a texture with random data and returns that data as
 * an array of TexelView.
 *
 * For compressed textures the texture is filled with random bytes
 * and then read back from the GPU by sampling so the GPU decompressed
 * the texture.
 *
 * For uncompressed textures the TexelViews are generated and then
 * copied to the texture.
 */
export async function createTextureWithRandomDataAndGetTexels(
t,
descriptor)
{
  if (isCompressedTextureFormat(descriptor.format)) {
    const texture = t.createTextureTracked(descriptor);

    fillTextureWithRandomData(t.device, texture);
    const texels = await readTextureToTexelViews(
      t,
      texture,
      descriptor,
      getTexelViewFormatForTextureFormat(texture.format)
    );
    return { texture, texels };
  } else {
    const texels = createRandomTexelViewMipmap(descriptor);
    const texture = createTextureFromTexelViews(t, texels, descriptor);
    return { texture, texels };
  }
}

const kFaceNames = ['+x', '-x', '+y', '-y', '+z', '-z'];

/**
 * Generates a text art grid showing which texels were sampled
 * followed by a list of the samples and the weights used for each
 * component.
 *
 * It works by making an index for every pixel in the texture. Then,
 * for each index it generates texture data using TexelView.fromTexelsAsColor
 * with a single [1, 1, 1, 1] texel at the texel for the current index.
 *
 * In then calls 'run' which renders a single `call`. `run` uses either
 * the software renderer or WebGPU. The result ends up being the weights
 * used when sampling that pixel. 0 = that texel was not sampled. > 0 =
 * it was sampled.
 *
 * This lets you see if the weights from the software renderer match the
 * weights from WebGPU.
 *
 * Example:
 *
 *     0   1   2   3   4   5   6   7
 *   ┌───┬───┬───┬───┬───┬───┬───┬───┐
 * 0 │   │   │   │   │   │   │   │   │
 *   ├───┼───┼───┼───┼───┼───┼───┼───┤
 * 1 │   │   │   │   │   │   │   │ a │
 *   ├───┼───┼───┼───┼───┼───┼───┼───┤
 * 2 │   │   │   │   │   │   │   │ b │
 *   ├───┼───┼───┼───┼───┼───┼───┼───┤
 * 3 │   │   │   │   │   │   │   │   │
 *   ├───┼───┼───┼───┼───┼───┼───┼───┤
 * 4 │   │   │   │   │   │   │   │   │
 *   ├───┼───┼───┼───┼───┼───┼───┼───┤
 * 5 │   │   │   │   │   │   │   │   │
 *   ├───┼───┼───┼───┼───┼───┼───┼───┤
 * 6 │   │   │   │   │   │   │   │   │
 *   ├───┼───┼───┼───┼───┼───┼───┼───┤
 * 7 │   │   │   │   │   │   │   │   │
 *   └───┴───┴───┴───┴───┴───┴───┴───┘
 * a: at: [7, 1], weights: [R: 0.75000]
 * b: at: [7, 2], weights: [R: 0.25000]
 */
async function identifySamplePoints(
texture,
run)
{
  const info = texture.descriptor;
  const isCube = texture.viewDescriptor.dimension === 'cube';
  const textureSize = reifyExtent3D(info.size);
  const numTexels = textureSize.width * textureSize.height * textureSize.height;
  const texelsPerRow = textureSize.width;
  const texelsPerSlice = textureSize.width * textureSize.height;
  // This isn't perfect. We already know there was an error. We're just
  // generating info so it seems okay it's not perfect. This format will
  // be used to generate weights by drawing with a texture of this format
  // with a specific pixel set to [1, 1, 1, 1]. As such, if the result
  // is > 0 then that pixel was sampled and the results are the weights.
  //
  // Ideally, this texture with a single pixel set to [1, 1, 1, 1] would
  // be the same format we were originally testing, the one we already
  // detected an error for. This way, whatever subtle issues there are
  // from that format will affect the weight values we're computing. But,
  // if that format is not encodable, for example if it's a compressed
  // texture format, then we have no way to build a texture so we use
  // rgba8unorm instead.
  const format =
  kEncodableTextureFormats.includes(info.format) ?
  info.format :
  'rgba8unorm';

  const rep = kTexelRepresentationInfo[format];

  // Identify all the texels that are sampled, and their weights.
  const sampledTexelWeights = new Map();
  const unclassifiedStack = [new Set(range(numTexels, (v) => v))];
  while (unclassifiedStack.length > 0) {
    // Pop the an unclassified texels stack
    const unclassified = unclassifiedStack.pop();

    // Split unclassified texels evenly into two new sets
    const setA = new Set();
    const setB = new Set();
    [...unclassified.keys()].forEach((t, i) => ((i & 1) === 0 ? setA : setB).add(t));

    // Push setB to the unclassified texels stack
    if (setB.size > 0) {
      unclassifiedStack.push(setB);
    }

    // See if any of the texels in setA were sampled.
    const results = await run(
      TexelView.fromTexelsAsColors(
        format,
        (coords) => {
          const isCandidate = setA.has(
            coords.x + coords.y * texelsPerRow + coords.z * texelsPerSlice
          );
          const texel = {};
          for (const component of rep.componentOrder) {
            texel[component] = isCandidate ? 1 : 0;
          }
          return texel;
        }
      )
    );
    if (rep.componentOrder.some((c) => results[c] !== 0)) {
      // One or more texels of setA were sampled.
      if (setA.size === 1) {
        // We identified a specific texel was sampled.
        // As there was only one texel in the set, results holds the sampling weights.
        setA.forEach((texel) => sampledTexelWeights.set(texel, results));
      } else {
        // More than one texel in the set. Needs splitting.
        unclassifiedStack.push(setA);
      }
    }
  }

  // ┌───┬───┬───┬───┐
  // │ a │   │   │   │
  // ├───┼───┼───┼───┤
  // │   │   │   │   │
  // ├───┼───┼───┼───┤
  // │   │   │   │   │
  // ├───┼───┼───┼───┤
  // │   │   │   │ b │
  // └───┴───┴───┴───┘
  const letter = (idx) => String.fromCharCode(97 + idx); // 97: 'a'
  const orderedTexelIndices = [];
  const lines = [];
  for (let z = 0; z < textureSize.depthOrArrayLayers; ++z) {
    lines.push(`slice: ${z}${isCube ? ` (${kFaceNames[z]})` : ''}`);
    {
      let line = '  ';
      for (let x = 0; x < textureSize.width; x++) {
        line += `  ${x.toString().padEnd(2)}`;
      }
      lines.push(line);
    }
    {
      let line = '  ┌';
      for (let x = 0; x < textureSize.width; x++) {
        line += x === textureSize.width - 1 ? '───┐' : '───┬';
      }
      lines.push(line);
    }
    for (let y = 0; y < textureSize.height; y++) {
      {
        let line = `${y.toString().padEnd(2)}│`;
        for (let x = 0; x < textureSize.width; x++) {
          const texelIdx = x + y * texelsPerRow + z * texelsPerSlice;
          const weight = sampledTexelWeights.get(texelIdx);
          if (weight !== undefined) {
            line += ` ${letter(orderedTexelIndices.length)} │`;
            orderedTexelIndices.push(texelIdx);
          } else {
            line += '   │';
          }
        }
        lines.push(line);
      }
      if (y < textureSize.height - 1) {
        let line = '  ├';
        for (let x = 0; x < textureSize.width; x++) {
          line += x === textureSize.width - 1 ? '───┤' : '───┼';
        }
        lines.push(line);
      }
    }
    {
      let line = '  └';
      for (let x = 0; x < textureSize.width; x++) {
        line += x === textureSize.width - 1 ? '───┘' : '───┴';
      }
      lines.push(line);
    }
  }

  const pad2 = (n) => n.toString().padStart(2);
  orderedTexelIndices.forEach((texelIdx, i) => {
    const weights = sampledTexelWeights.get(texelIdx);
    const z = Math.floor(texelIdx / texelsPerSlice);
    const y = Math.floor(texelIdx % texelsPerSlice / texelsPerRow);
    const x = texelIdx % texelsPerRow;
    const w = rep.componentOrder.map((c) => `${c}: ${weights[c]?.toFixed(5)}`).join(', ');
    lines.push(`${letter(i)}: at: [${pad2(x)}, ${pad2(y)}, ${pad2(z)}], weights: [${w}]`);
  });
  return lines;
}

function layoutTwoColumns(columnA, columnB) {
  const widthA = Math.max(...columnA.map((l) => l.length));
  const lines = Math.max(columnA.length, columnB.length);
  const out = new Array(lines);
  for (let line = 0; line < lines; line++) {
    const a = columnA[line] ?? '';
    const b = columnB[line] ?? '';
    out[line] = `${a}${' '.repeat(widthA - a.length)} | ${b}`;
  }
  return out;
}

function getDepthOrArrayLayersForViewDimension(viewDimension) {
  switch (viewDimension) {
    case undefined:
    case '2d':
      return 1;
    case '3d':
      return 8;
    case 'cube':
      return 6;
    default:
      unreachable();
  }
}

/**
 * Choose a texture size based on the given parameters.
 * The size will be in a multiple of blocks. If it's a cube
 * map the size will so be square.
 */
export function chooseTextureSize({
  minSize,
  minBlocks,
  format,
  viewDimension





}) {
  const { blockWidth, blockHeight } = kTextureFormatInfo[format];
  const width = align(Math.max(minSize, blockWidth * minBlocks), blockWidth);
  const height = align(Math.max(minSize, blockHeight * minBlocks), blockHeight);
  if (viewDimension === 'cube') {
    const size = lcm(width, height);
    return [size, size, 6];
  }
  const depthOrArrayLayers = getDepthOrArrayLayersForViewDimension(viewDimension);
  return [width, height, depthOrArrayLayers];
}

export const kSamplePointMethods = ['texel-centre', 'spiral'];


export const kCubeSamplePointMethods = ['cube-edges', 'texel-centre', 'spiral'];


/**
 * Generates an array of coordinates at which to sample a texture.
 */
function generateSamplePointsImpl(
makeValue,
n,
nearest,
args)














{
  const { method, textureWidth, textureHeight, textureDepthOrArrayLayers = 1 } = args;
  const out = [];
  switch (method) {
    case 'texel-centre':{
        for (let i = 0; i < n; i++) {
          const r = hashU32(i);
          const x = Math.floor(lerp(0, textureWidth - 1, (r & 0xff) / 0xff)) + 0.5;
          const y = Math.floor(lerp(0, textureHeight - 1, (r >> 8 & 0xff) / 0xff)) + 0.5;
          const z =
          Math.floor(lerp(0, textureDepthOrArrayLayers - 1, (r >> 16 & 0xff) / 0xff)) + 0.5;
          out.push(makeValue(x / textureWidth, y / textureHeight, z / textureDepthOrArrayLayers));
        }
        break;
      }
    case 'spiral':{
        const { radius = 1.5, loops = 2 } = args;
        for (let i = 0; i < n; i++) {
          const f = i / (Math.max(n, 2) - 1);
          const r = radius * f;
          const a = loops * 2 * Math.PI * f;
          out.push(makeValue(0.5 + r * Math.cos(a), 0.5 + r * Math.sin(a), 0));
        }
        break;
      }
  }

  // Samplers across devices use different methods to interpolate.
  // Quantizing the texture coordinates seems to hit coords that produce
  // comparable results to our computed results.
  // Note: This value works with 8x8 textures. Other sizes have not been tested.
  // Values that worked for reference:
  // Win 11, NVidia 2070 Super: 16
  // Linux, AMD Radeon Pro WX 3200: 256
  // MacOS, M1 Mac: 256
  const kSubdivisionsPerTexel = 4;
  const q = [
  textureWidth * kSubdivisionsPerTexel,
  textureHeight * kSubdivisionsPerTexel,
  textureDepthOrArrayLayers * kSubdivisionsPerTexel];

  return out.map(
    (c) =>
    c.map((v, i) => {
      // Quantize to kSubdivisionsPerPixel
      const v1 = Math.floor(v * q[i]);
      // If it's nearest and we're on the edge of a texel then move us off the edge
      // since the edge could choose one texel or another in nearest mode
      const v2 = nearest && v1 % kSubdivisionsPerTexel === 0 ? v1 + 1 : v1;
      // Convert back to texture coords
      return v2 / q[i];
    })
  );
}

// Removes the first element from an array of types




export function generateSamplePoints1D(...args) {
  return generateSamplePointsImpl((x) => [x], ...args);
}

export function generateSamplePoints2D(...args) {
  return generateSamplePointsImpl((x, y) => [x, y], ...args);
}

export function generateSamplePoints3D(...args) {
  return generateSamplePointsImpl((x, y, z) => [x, y, z], ...args);
}








const kFaceUVMatrices =
[
[0, 0, -2, 0, -2, 0, 1, 1, 1], // pos-x
[0, 0, 2, 0, -2, 0, -1, 1, -1], // neg-x
[2, 0, 0, 0, 0, 2, -1, 1, -1], // pos-y
[2, 0, 0, 0, 0, -2, -1, -1, 1], // neg-y
[2, 0, 0, 0, -2, 0, -1, 1, 1], // pos-z
[-2, 0, 0, 0, -2, 0, 1, 1, -1] // neg-z
];

/** multiply a vec3 by mat3 */
function transformMat3(v, m) {
  const x = v[0];
  const y = v[1];
  const z = v[2];

  return [
  x * m[0] + y * m[3] + z * m[6],
  x * m[1] + y * m[4] + z * m[7],
  x * m[2] + y * m[5] + z * m[8]];

}

/** normalize a vec3 */
function normalize(v) {
  const length = Math.sqrt(v[0] * v[0] + v[1] * v[1] + v[2] * v[2]);
  assert(length > 0);
  return v.map((v) => v / length);
}

/**
 * Converts a cube map coordinate to a uv coordinate (0 to 1) and layer (0.5/6.0 to 5.5/6.0).
 * Also returns the length of the original coordinate.
 */
function convertCubeCoordToNormalized3DTextureCoord(v) {
  let uvw;
  let layer;
  // normalize the coord.
  // MAINTENANCE_TODO: handle(0, 0, 0)
  const r = normalize(v);
  const absR = r.map((v) => Math.abs(v));
  if (absR[0] > absR[1] && absR[0] > absR[2]) {
    // x major
    const negX = r[0] < 0.0 ? 1 : 0;
    uvw = [negX ? r[2] : -r[2], -r[1], absR[0]];
    layer = negX;
  } else if (absR[1] > absR[2]) {
    // y major
    const negY = r[1] < 0.0 ? 1 : 0;
    uvw = [r[0], negY ? -r[2] : r[2], absR[1]];
    layer = 2 + negY;
  } else {
    // z major
    const negZ = r[2] < 0.0 ? 1 : 0;
    uvw = [negZ ? -r[0] : r[0], -r[1], absR[2]];
    layer = 4 + negZ;
  }
  return [(uvw[0] / uvw[2] + 1) * 0.5, (uvw[1] / uvw[2] + 1) * 0.5, (layer + 0.5) / 6];
}

/**
 * Convert a 3d texcoord into a cube map coordinate.
 */
function convertNormalized3DTexCoordToCubeCoord(uvLayer) {
  const [u, v, faceLayer] = uvLayer;
  return normalize(transformMat3([u, v, 1], kFaceUVMatrices[Math.min(5, faceLayer * 6) | 0]));
}

/**
 * We have a face texture in texels coord where U/V choose a texel and W chooses the face.
 * If U/V are outside the size of the texture then, when normalized and converted
 * to a cube map coordinate, they'll end up pointing to a different face.
 *
 * addressMode is effectively ignored for cube
 *
 *             +-----------+
 *             |0->u       |
 *             |↓          |
 *             |v   +y     |
 *             |    (2)    |
 *             |           |
 * +-----------+-----------+-----------+-----------+
 * |0->u       |0->u       |0->u       |0->u       |
 * |↓          |↓          |↓          |↓          |
 * |v   -x     |v   +z     |v   +x     |v   -z     |
 * |    (1)    |    (4)    |    (0)    |    (5)    |
 * |           |           |           |           |
 * +-----------+-----------+-----------+-----------+
 *             |0->u       |
 *             |↓          |
 *             |v   -y     |
 *             |    (3)    |
 *             |           |
 *             +-----------+
 */
const kFaceConversions = {
  u: (textureSize, faceCoord) => faceCoord[0],
  v: (textureSize, faceCoord) => faceCoord[1],
  'u+t': (textureSize, faceCoord) => faceCoord[0] + textureSize,
  'u-t': (textureSize, faceCoord) => faceCoord[0] - textureSize,
  'v+t': (textureSize, faceCoord) => faceCoord[1] + textureSize,
  'v-t': (textureSize, faceCoord) => faceCoord[1] - textureSize,
  't-v': (textureSize, faceCoord) => textureSize - faceCoord[1],
  '1+u': (textureSize, faceCoord) => 1 + faceCoord[0],
  '1+v': (textureSize, faceCoord) => 1 + faceCoord[1],
  '-v-1': (textureSize, faceCoord) => -faceCoord[1] - 1,
  't-u-1': (textureSize, faceCoord) => textureSize - faceCoord[0] - 1,
  't-v-1': (textureSize, faceCoord) => textureSize - faceCoord[1] - 1,
  '2t-u-1': (textureSize, faceCoord) => textureSize * 2 - faceCoord[0] - 1,
  '2t-v-1': (textureSize, faceCoord) => textureSize * 2 - faceCoord[1] - 1
};
const kFaceConversionEnums = keysOf(kFaceConversions);


// For Each face
//   face to go if u < 0
//   face to go if u >= textureSize
//   face to go if v < 0
//   face to go if v >= textureSize
const kFaceToFaceRemap = [
// 0
[
/* -u */{ to: 4, u: 'u+t', v: 'v' },
/* +u */{ to: 5, u: 'u-t', v: 'v' },
/* -v */{ to: 2, u: 'v+t', v: 't-u-1' },
/* +v */{ to: 3, u: '2t-v-1', v: 'u' }],

// 1
[
/* -u */{ to: 5, u: 'u+t', v: 'v' },
/* +u */{ to: 4, u: 'u-t', v: 'v' },
/* -v */{ to: 2, u: '-v-1', v: 'u' }, // -1->0, -2->1  -3->2
/* +v */{ to: 3, u: 't-v', v: 't-u-1' }],

// 2
[
/* -u */{ to: 1, u: 'v', v: '1+u' },
/* +u */{ to: 0, u: 't-v-1', v: 'u-t' },
/* -v */{ to: 5, u: 't-u-1', v: 't-v-1' },
/* +v */{ to: 4, u: 'u', v: 'v-t' }],

// 3
[
/* -u */{ to: 1, u: 't-v-1', v: 'u+t' },
/* +u */{ to: 0, u: 'v', v: '2t-u-1' },
/* -v */{ to: 4, u: 'u', v: 'v+t' },
/* +v */{ to: 5, u: 't-u-1', v: '2t-v-1' }],

// 4
[
/* -u */{ to: 1, u: 'u+t', v: 'v' },
/* +u */{ to: 0, u: 'u-t', v: 'v' },
/* -v */{ to: 2, u: 'u', v: 'v+t' },
/* +v */{ to: 3, u: 'u', v: 'v-t' }],

// 5
[
/* -u */{ to: 0, u: 'u+t', v: 'v' },
/* +u */{ to: 1, u: 'u-t', v: 'v' },
/* -v */{ to: 2, u: 't-u-1', v: '1+v' },
/* +v */{ to: 3, u: 't-u-1', v: '2t-v-1' }]];



function getFaceWrapIndex(textureSize, faceCoord) {
  if (faceCoord[0] < 0) {
    return 0;
  }
  if (faceCoord[0] >= textureSize) {
    return 1;
  }
  if (faceCoord[1] < 0) {
    return 2;
  }
  if (faceCoord[1] >= textureSize) {
    return 3;
  }
  return -1;
}

function applyFaceWrap(textureSize, faceCoord) {
  const ndx = getFaceWrapIndex(textureSize, faceCoord);
  if (ndx < 0) {
    return faceCoord;
  }
  const { to, u, v } = kFaceToFaceRemap[faceCoord[2]][ndx];
  return [
  kFaceConversions[u](textureSize, faceCoord),
  kFaceConversions[v](textureSize, faceCoord),
  to];

}

function wrapFaceCoordToCubeFaceAtEdgeBoundaries(textureSize, faceCoord) {
  // If we're off both edges we need to wrap twice, once for each edge.
  faceCoord = applyFaceWrap(textureSize, faceCoord);
  faceCoord = applyFaceWrap(textureSize, faceCoord);
  return faceCoord;
}

function applyAddressModesToCoords(
addressMode,
textureSize,
coord)
{
  return coord.map((v, i) => {
    switch (addressMode[i]) {
      case 'clamp-to-edge':
        return clamp(v, { min: 0, max: textureSize[i] - 1 });
      case 'mirror-repeat':{
          const n = Math.floor(v / textureSize[i]);
          v = v - n * textureSize[i];
          return (n & 1) !== 0 ? textureSize[i] - v - 1 : v;
        }
      case 'repeat':
        return v - Math.floor(v / textureSize[i]) * textureSize[i];
      default:
        unreachable();
    }
  });
}

/**
 * Generates an array of coordinates at which to sample a texture for a cubemap
 */
export function generateSamplePointsCube(
n,
nearest,
args)

















{
  const { method, textureWidth } = args;
  const out = [];
  switch (method) {
    case 'texel-centre':{
        for (let i = 0; i < n; i++) {
          const r = hashU32(i);
          const u = (Math.floor(lerp(0, textureWidth - 1, (r & 0xff) / 0xff)) + 0.5) / textureWidth;
          const v =
          (Math.floor(lerp(0, textureWidth - 1, (r >> 8 & 0xff) / 0xff)) + 0.5) / textureWidth;
          const face = Math.floor(lerp(0, 6, (r >> 16 & 0xff) / 0x100));
          out.push(convertNormalized3DTexCoordToCubeCoord([u, v, face]));
        }
        break;
      }
    case 'spiral':{
        const { radius = 1.5, loops = 2 } = args;
        for (let i = 0; i < n; i++) {
          const f = (i + 1) / (Math.max(n, 2) - 1);
          const r = radius * f;
          const theta = loops * 2 * Math.PI * f;
          const phi = loops * 1.3 * Math.PI * f;
          const sinTheta = Math.sin(theta);
          const cosTheta = Math.cos(theta);
          const sinPhi = Math.sin(phi);
          const cosPhi = Math.cos(phi);
          const ux = cosTheta * sinPhi;
          const uy = cosPhi;
          const uz = sinTheta * sinPhi;
          out.push([ux * r, uy * r, uz * r]);
        }
        break;
      }
    case 'cube-edges':{

        out.push(
          // between edges
          [-1.01, -1.02, 0],
          [1.01, -1.02, 0],
          [-1.01, 1.02, 0],
          [1.01, 1.02, 0],

          [-1.01, 0, -1.02],
          [1.01, 0, -1.02],
          [-1.01, 0, 1.02],
          [1.01, 0, 1.02],

          [-1.01, -1.02, 0],
          [1.01, -1.02, 0],
          [-1.01, 1.02, 0],
          [1.01, 1.02, 0]

          // corners (see comment "Issues with corners of cubemaps")
          // for why these are commented out.
          // [-1.01, -1.02, -1.03],
          // [ 1.01, -1.02, -1.03],
          // [-1.01,  1.02, -1.03],
          // [ 1.01,  1.02, -1.03],
          // [-1.01, -1.02,  1.03],
          // [ 1.01, -1.02,  1.03],
          // [-1.01,  1.02,  1.03],
          // [ 1.01,  1.02,  1.03],
        );
        break;
      }
  }

  // Samplers across devices use different methods to interpolate.
  // Quantizing the texture coordinates seems to hit coords that produce
  // comparable results to our computed results.
  // Note: This value works with 8x8 textures. Other sizes have not been tested.
  // Values that worked for reference:
  // Win 11, NVidia 2070 Super: 16
  // Linux, AMD Radeon Pro WX 3200: 256
  // MacOS, M1 Mac: 256
  const kSubdivisionsPerTexel = 4;
  const q = [
  textureWidth * kSubdivisionsPerTexel,
  textureWidth * kSubdivisionsPerTexel,
  6 * kSubdivisionsPerTexel];

  return out.map((c) => {
    const uvw = convertCubeCoordToNormalized3DTextureCoord(c);

    // If this is a corner, move to in so it's not
    // (see comment "Issues with corners of cubemaps")
    const ndx = getUnusedCubeCornerSampleIndex(textureWidth, uvw);
    if (ndx >= 0) {
      const halfTexel = 0.5 / textureWidth;
      uvw[0] = clamp(uvw[0], { min: halfTexel, max: 1 - halfTexel });
    }

    const quantizedUVW = uvw.map((v, i) => {
      // Quantize to kSubdivisionsPerPixel
      const v1 = Math.floor(v * q[i]);
      // If it's nearest and we're on the edge of a texel then move us off the edge
      // since the edge could choose one texel or another in nearest mode
      const v2 = nearest && v1 % kSubdivisionsPerTexel === 0 ? v1 + 1 : v1;
      // Convert back to texture coords
      return v2 / q[i];
    });
    return convertNormalized3DTexCoordToCubeCoord(quantizedUVW);
  });
}

function wgslTypeFor(data, type) {
  if (Array.isArray(data)) {
    switch (data.length) {
      case 1:
        return `${type}32`;
      case 2:
        return `vec2${type}`;
      case 3:
        return `vec3${type}`;
      default:
        unreachable();
    }
  }
  return `${type}32`;
}

function wgslExpr(data) {
  if (Array.isArray(data)) {
    switch (data.length) {
      case 1:
        return data[0].toString();
      case 2:
        return `vec2(${data.map((v) => v.toString()).join(', ')})`;
      case 3:
        return `vec3(${data.map((v) => v.toString()).join(', ')})`;
      default:
        unreachable();
    }
  }
  return data.toString();
}

function wgslExprFor(data, type) {
  if (Array.isArray(data)) {
    switch (data.length) {
      case 1:
        return `${type}(${data[0].toString()})`;
      case 2:
        return `vec2${type}(${data.map((v) => v.toString()).join(', ')})`;
      case 3:
        return `vec3${type}(${data.map((v) => v.toString()).join(', ')})`;
      default:
        unreachable();
    }
  }
  return `${type}32(${data.toString()})`;
}

function binKey(call) {
  const keys = [];
  for (const name of kTextureCallArgNames) {
    const value = call[name];
    if (value !== undefined) {
      if (name === 'offset') {
        // offset must be a constant expression
        keys.push(`${name}: ${wgslExpr(value)}`);
      } else {
        keys.push(`${name}: ${wgslTypeFor(value, call.coordType)}`);
      }
    }
  }
  return `${call.builtin}(${keys.join(', ')})`;
}

function buildBinnedCalls(calls) {
  const args = ['T']; // All texture builtins take the texture as the first argument
  const fields = [];
  const data = [];

  const prototype = calls[0];
  if (prototype.builtin.startsWith('textureSample')) {
    // textureSample*() builtins take a sampler as the second argument
    args.push('S');
  }

  for (const name of kTextureCallArgNames) {
    const value = prototype[name];
    if (value !== undefined) {
      if (name === 'offset') {
        args.push(`/* offset */ ${wgslExpr(value)}`);
      } else {
        const type = name === 'mipLevel' ? prototype.levelType : prototype.coordType;
        args.push(`args.${name}`);
        fields.push(`@align(16) ${name} : ${wgslTypeFor(value, type)}`);
      }
    }
  }

  for (const call of calls) {
    for (const name of kTextureCallArgNames) {
      const value = call[name];
      assert(
        prototype[name] === undefined === (value === undefined),
        'texture calls are not binned correctly'
      );
      if (value !== undefined && name !== 'offset') {
        const bitcastToU32 = (value) => {
          if (calls[0].coordType === 'f') {
            return float32ToUint32(value);
          }
          return value;
        };
        if (value instanceof Array) {
          for (const c of value) {
            data.push(bitcastToU32(c));
          }
        } else {
          data.push(bitcastToU32(value));
        }
        // All fields are aligned to 16 bytes.
        while ((data.length & 3) !== 0) {
          data.push(0);
        }
      }
    }
  }

  const expr = `${prototype.builtin}(${args.join(', ')})`;

  return { expr, fields, data };
}

function binCalls(calls) {
  const map = new Map(); // key to bin index
  const bins = [];
  calls.forEach((call, callIdx) => {
    const key = binKey(call);
    const binIdx = map.get(key);
    if (binIdx === undefined) {
      map.set(key, bins.length);
      bins.push([callIdx]);
    } else {
      bins[binIdx].push(callIdx);
    }
  });
  return bins;
}

export function describeTextureCall(call) {
  const args = ['texture: T'];
  if (call.builtin.startsWith('textureSample')) {
    args.push('sampler: S');
  }
  for (const name of kTextureCallArgNames) {
    const value = call[name];
    if (value !== undefined) {
      if (name === 'coords') {
        args.push(`${name}: ${wgslExprFor(value, call.coordType)}`);
      } else if (name === 'mipLevel') {
        args.push(`${name}: ${wgslExprFor(value, call.levelType)}`);
      } else {
        args.push(`${name}: ${wgslExpr(value)}`);
      }
    }
  }
  return `${call.builtin}(${args.join(', ')})`;
}

const s_deviceToPipelines = new WeakMap();

/**
 * Given a list of "calls", each one of which has a texture coordinate,
 * generates a fragment shader that uses the fragment position as an index
 * (position.y * 256 + position.x) That index is then used to look up a
 * coordinate from a storage buffer which is used to call the WGSL texture
 * function to read/sample the texture, and then write to an rgba32float
 * texture.  We then read the rgba32float texture for the per "call" results.
 *
 * Calls are "binned" by call parameters. Each bin has its own structure and
 * field in the storage buffer. This allows the calls to be non-homogenous and
 * each have their own data type for coordinates.
 */
export async function doTextureCalls(
t,
gpuTexture,
viewDescriptor,
textureType,
sampler,
calls)
{
  let structs = '';
  let body = '';
  let dataFields = '';
  const data = [];
  let callCount = 0;
  const binned = binCalls(calls);
  binned.forEach((binCalls, binIdx) => {
    const b = buildBinnedCalls(binCalls.map((callIdx) => calls[callIdx]));
    structs += `struct Args${binIdx} {
  ${b.fields.join(',  \n')}
}
`;
    dataFields += `  args${binIdx} : array<Args${binIdx}, ${binCalls.length}>,
`;
    body += `
  {
    let is_active = (frag_idx >= ${callCount}) & (frag_idx < ${callCount + binCalls.length});
    let args = data.args${binIdx}[frag_idx - ${callCount}];
    let call = ${b.expr};
    result = select(result, call, is_active);
  }
`;
    callCount += binCalls.length;
    data.push(...b.data);
  });

  const dataBuffer = t.createBufferTracked({
    size: data.length * 4,
    usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.STORAGE
  });
  t.device.queue.writeBuffer(dataBuffer, 0, new Uint32Array(data));

  const { resultType, resultFormat } = getTextureFormatTypeInfo(gpuTexture.format);

  const rtWidth = 256;
  const renderTarget = t.createTextureTracked({
    format: resultFormat,
    size: { width: rtWidth, height: Math.ceil(calls.length / rtWidth) },
    usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT
  });

  const code = `
${structs}

struct Data {
${dataFields}
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index : u32) -> @builtin(position) vec4f {
  let positions = array(
    vec4f(-1,  1, 0, 1), vec4f( 1,  1, 0, 1),
    vec4f(-1, -1, 0, 1), vec4f( 1, -1, 0, 1),
  );
  return positions[vertex_index];
}

@group(0) @binding(0) var          T    : ${textureType};
${sampler ? '@group(0) @binding(1) var          S    : sampler' : ''};
@group(0) @binding(2) var<storage> data : Data;

@fragment
fn fs_main(@builtin(position) frag_pos : vec4f) -> @location(0) ${resultType} {
  let frag_idx = u32(frag_pos.x) + u32(frag_pos.y) * ${renderTarget.width};
  var result : ${resultType};
${body}
  return result;
}
`;

  const pipelines = s_deviceToPipelines.get(t.device) ?? new Map();
  s_deviceToPipelines.set(t.device, pipelines);

  let pipeline = pipelines.get(code);
  if (!pipeline) {
    const shaderModule = t.device.createShaderModule({ code });

    pipeline = t.device.createRenderPipeline({
      layout: 'auto',
      vertex: { module: shaderModule },
      fragment: {
        module: shaderModule,
        targets: [{ format: renderTarget.format }]
      },
      primitive: { topology: 'triangle-strip' }
    });

    pipelines.set(code, pipeline);
  }

  const gpuSampler = sampler ? t.device.createSampler(sampler) : undefined;

  const bindGroup = t.device.createBindGroup({
    layout: pipeline.getBindGroupLayout(0),
    entries: [
    { binding: 0, resource: gpuTexture.createView(viewDescriptor) },
    ...(sampler ? [{ binding: 1, resource: gpuSampler }] : []),
    { binding: 2, resource: { buffer: dataBuffer } }]

  });

  const bytesPerRow = align(16 * renderTarget.width, 256);
  const resultBuffer = t.createBufferTracked({
    size: renderTarget.height * bytesPerRow,
    usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ
  });
  const encoder = t.device.createCommandEncoder();

  const renderPass = encoder.beginRenderPass({
    colorAttachments: [
    {
      view: renderTarget.createView(),
      loadOp: 'clear',
      storeOp: 'store'
    }]

  });

  renderPass.setPipeline(pipeline);
  renderPass.setBindGroup(0, bindGroup);
  renderPass.draw(4);
  renderPass.end();
  encoder.copyTextureToBuffer(
    { texture: renderTarget },
    { buffer: resultBuffer, bytesPerRow },
    { width: renderTarget.width, height: renderTarget.height }
  );
  t.device.queue.submit([encoder.finish()]);

  await resultBuffer.mapAsync(GPUMapMode.READ);

  const view = TexelView.fromTextureDataByReference(
    renderTarget.format,
    new Uint8Array(resultBuffer.getMappedRange()),
    {
      bytesPerRow,
      rowsPerImage: renderTarget.height,
      subrectOrigin: [0, 0, 0],
      subrectSize: [renderTarget.width, renderTarget.height]
    }
  );

  let outIdx = 0;
  const out = new Array(calls.length);
  for (const bin of binned) {
    for (const callIdx of bin) {
      const x = outIdx % rtWidth;
      const y = Math.floor(outIdx / rtWidth);
      out[callIdx] = view.color({ x, y, z: 0 });
      outIdx++;
    }
  }

  renderTarget.destroy();
  resultBuffer.destroy();

  return out;
}