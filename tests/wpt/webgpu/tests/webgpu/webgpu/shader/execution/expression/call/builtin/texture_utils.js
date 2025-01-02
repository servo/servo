/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { keysOf } from '../../../../../../common/util/data_tables.js';import { assert, range, unreachable } from '../../../../../../common/util/util.js';import { Float16Array } from '../../../../../../external/petamoriken/float16/float16.js';
import {

  isCompressedFloatTextureFormat,
  isCompressedTextureFormat,
  isDepthOrStencilTextureFormat,
  isDepthTextureFormat,
  isEncodableTextureFormat,
  isStencilTextureFormat,
  kEncodableTextureFormats,
  kTextureFormatInfo } from
'../../../../../format_info.js';
import { GPUTest } from '../../../../../gpu_test.js';
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
  physicalMipSize,
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


// These are needed because the list of parameters was too long when converted to a filename.
export const kShortShaderStageToShaderStage = {
  c: 'compute',
  f: 'fragment',
  v: 'vertex'
};
export const kShortShaderStages = keysOf(kShortShaderStageToShaderStage);


// These are needed because the list of parameters was too long when converted to a filename.
export const kShortAddressModeToAddressMode = {
  c: 'clamp-to-edge',
  r: 'repeat',
  m: 'mirror-repeat'
};

export const kShortAddressModes = keysOf(kShortAddressModeToAddressMode);

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

// MAINTENANCE_TODO: Stop excluding sliced compressed 3d formats.
export function isSupportedViewFormatCombo(
format,
viewDimension)
{
  return !(
  (isCompressedTextureFormat(format) || isDepthTextureFormat(format)) &&
  viewDimension === '3d');

}

/**
 * Return the texture type for a given view dimension
 */
export function getTextureTypeForTextureViewDimension(viewDimension) {
  switch (viewDimension) {
    case '1d':
      return 'texture_1d<f32>';
    case '2d':
      return 'texture_2d<f32>';
    case '2d-array':
      return 'texture_2d_array<f32>';
    case '3d':
      return 'texture_3d<f32>';
    case 'cube':
      return 'texture_cube<f32>';
    case 'cube-array':
      return 'texture_cube_array<f32>';
    default:
      unreachable();
  }
}

const is32Float = (format) =>
format === 'r32float' || format === 'rg32float' || format === 'rgba32float';

/**
 * Skips a subcase if the filter === 'linear' and the format is type
 * 'unfilterable-float' and we cannot enable filtering.
 */
export function skipIfNeedsFilteringAndIsUnfilterableOrSelectDevice(
t,
filter,
format)
{
  const features = new Set();
  features.add(kTextureFormatInfo[format].feature);

  if (filter === 'linear') {
    t.skipIf(isDepthTextureFormat(format), 'depth texture are unfilterable');

    const type = kTextureFormatInfo[format].color?.type;
    if (type === 'unfilterable-float') {
      assert(is32Float(format));
      features.add('float32-filterable');
    }
  }

  if (features.size > 0) {
    t.selectDeviceOrSkipTestCase(Array.from(features));
  }
}

/**
 * Skips a test if filter === 'linear' and the format is not filterable
 */
export function skipIfNeedsFilteringAndIsUnfilterable(
t,
filter,
format)
{
  if (filter === 'linear') {
    t.skipIf(isDepthTextureFormat(format), 'depth textures are unfilterable');
  }
}

/**
 * Returns if a texture format can be filled with random data.
 */
export function isFillable(format) {
  // We can't easily put random bytes into compressed textures if they are float formats
  // since we want the range to be +/- 1000 and not +/- infinity or NaN.
  return !isCompressedTextureFormat(format) || !format.endsWith('float');
}

/**
 * Returns if a texture format can potentially be filtered and can be filled with random data.
 */
export function isPotentiallyFilterableAndFillable(format) {
  const info = kTextureFormatInfo[format];
  const type = info.color?.type ?? info.depth?.type;
  const canPotentiallyFilter =
  type === 'float' || type === 'unfilterable-float' || type === 'depth';
  const result = canPotentiallyFilter && isFillable(format);
  return result;
}

/**
 * skips the test if the texture format is not supported or not available or not filterable.
 */
export function skipIfTextureFormatNotSupportedNotAvailableOrNotFilterable(
t,
format)
{
  t.skipIfTextureFormatNotSupported(format);
  const info = kTextureFormatInfo[format];
  if (info.color?.type === 'unfilterable-float') {
    t.selectDeviceOrSkipTestCase('float32-filterable');
  } else {
    t.selectDeviceForTextureFormatOrSkipTestCase(format);
  }
}

/**
 * Splits in array into multiple arrays where every Nth value goes to a different array
 */
function unzip(array, num) {
  const arrays = range(num, () => []);
  array.forEach((v, i) => {
    arrays[i % num].push(v);
  });
  return arrays;
}







function makeGraph(width, height) {
  const data = new Uint8Array(width * height);

  return {
    plot(norm, x, c) {
      const y = clamp(Math.floor(norm * height), { min: 0, max: height - 1 });
      const offset = (height - y - 1) * width + x;
      data[offset] = c;
    },
    plotValues(values, c) {
      let i = 0;
      for (const v of values) {
        this.plot(v, i, c);
        ++i;
      }
    },
    toString(conversion = ['.', 'e', 'A']) {
      const lines = [];
      for (let y = 0; y < height; ++y) {
        const offset = y * width;
        lines.push([...data.subarray(offset, offset + width)].map((v) => conversion[v]).join(''));
      }
      return lines.join('\n');
    }
  };
}

function* linear0to1OverN(n) {
  for (let i = 0; i <= n; ++i) {
    yield i / n;
  }
}

function graphWeights(height, weights) {
  const graph = makeGraph(weights.length, height);
  graph.plotValues(linear0to1OverN(weights.length - 1), 1);
  graph.plotValues(weights, 2);
  return graph.toString();
}

/**
 * Validates the weights go from 0 to 1 in increasing order.
 */
function validateWeights(stage, weights) {
  const showWeights = () => `
${weights.map((v, i) => `${i.toString().padStart(2)}: ${v}`).join('\n')}

e = expected
A = actual
${graphWeights(32, weights)}
`;

  // Validate the weights
  assert(
    weights[0] === 0,
    `stage: ${stage}, weight 0 expected 0 but was ${weights[0]}\n${showWeights()}`
  );
  assert(
    weights[kMipGradientSteps] === 1,
    `stage: ${stage}, top weight expected 1 but was ${weights[kMipGradientSteps]}\n${showWeights()}`
  );

  // Note: for 16 steps, these are the AMD weights
  //
  //                 standard
  // step  mipLevel    gpu        AMD
  // ----  --------  --------  ----------
  //  0:   0         0           0
  //  1:   0.0625    0.0625      0
  //  2:   0.125     0.125       0.03125
  //  3:   0.1875    0.1875      0.109375
  //  4:   0.25      0.25        0.1875
  //  5:   0.3125    0.3125      0.265625
  //  6:   0.375     0.375       0.34375
  //  7:   0.4375    0.4375      0.421875
  //  8:   0.5       0.5         0.5
  //  9:   0.5625    0.5625      0.578125
  // 10:   0.625     0.625       0.65625
  // 11:   0.6875    0.6875      0.734375
  // 12:   0.75      0.75        0.8125
  // 13:   0.8125    0.8125      0.890625
  // 14:   0.875     0.875       0.96875
  // 15:   0.9375    0.9375      1
  // 16:   1         1           1
  //
  // notice step 1 is 0 and step 15 is 1.
  // so we only check the 1 through 14.
  //
  // Note: these 2 changes are effectively here to catch Intel Mac
  // issues and require implementations to work around them.
  //
  // Ideally the weights should form a straight line
  //
  // +----------------+
  // |              **|
  // |            **  |
  // |          **    |
  // |        **      |
  // |      **        |
  // |    **          |
  // |  **            |
  // |**              |
  // +----------------+
  //
  // AMD Mac goes like this: Not great but we allow it
  //
  // +----------------+
  // |             ***|
  // |           **   |
  // |          *     |
  // |        **      |
  // |      **        |
  // |     *          |
  // |   **           |
  // |***             |
  // +----------------+
  //
  // Intel Mac goes like this: Unacceptable
  //
  // +----------------+
  // |         *******|
  // |         *      |
  // |        *       |
  // |        *       |
  // |       *        |
  // |       *        |
  // |      *         |
  // |*******         |
  // +----------------+
  //
  const dx = 1 / kMipGradientSteps;
  for (let i = 0; i < kMipGradientSteps; ++i) {
    const dy = weights[i + 1] - weights[i];
    // dy / dx because dy might be 0
    const slope = dy / dx;
    assert(
      slope >= 0,
      `stage: ${stage}, weight[${i}] was not <= weight[${i + 1}]\n${showWeights()}`
    );
    assert(
      slope <= 2,
      `stage: ${stage}, slope from weight[${i}] to weight[${i + 1}] is > 2.\n${showWeights()}`
    );
  }

  assert(
    new Set(weights).size >= (weights.length * 0.66 | 0),
    `stage: ${stage}, expected more unique weights\n${showWeights()}`
  );
}

/**
 * In an attempt to pass on more devices without lowering the tolerances
 * so low they are meaningless, we ask the hardware to tell us, for a given
 * gradient, level, what mix weights are being used.
 *
 * This is done by drawing instanced quads and using instance_index to
 * write out results into an array. We sample a 2x2 pixel texture with
 * 2 mip levels and set the 2nd mip level to white. This means the value
 * we get back represents the weight used to mix the 2 mip levels.
 *
 * Just as a record of some differences across GPUs
 *
 * level weights: mapping from the mip level
 * parameter of `textureSampleLevel` to
 * the mix weight used by the GPU
 *
 * +--------+--------+--------+--------+
 * |        |        | intel  | amd    |
 * |        |  m1    | gen-9  | rna-1  |
 * | level  |  mac   | mac    | mac    |
 * +--------+--------+--------+--------+
 * | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
 * | 0.0313 | 0.0314 | 0.0313 | 0.0000 |
 * | 0.0625 | 0.0625 | 0.0625 | 0.0000 |
 * | 0.0938 | 0.0939 | 0.0938 | 0.0000 |
 * | 0.1250 | 0.1250 | 0.1250 | 0.0313 |
 * | 0.1563 | 0.1564 | 0.1563 | 0.0703 |
 * | 0.1875 | 0.1875 | 0.1875 | 0.1094 |
 * | 0.2188 | 0.2189 | 0.2188 | 0.1484 |
 * | 0.2500 | 0.2500 | 0.2500 | 0.1875 |
 * | 0.2813 | 0.2814 | 0.2813 | 0.2266 |
 * | 0.3125 | 0.3125 | 0.3125 | 0.2656 |
 * | 0.3438 | 0.3439 | 0.3438 | 0.3047 |
 * | 0.3750 | 0.3750 | 0.3750 | 0.3438 |
 * | 0.4063 | 0.4064 | 0.4063 | 0.3828 |
 * | 0.4375 | 0.4375 | 0.4375 | 0.4219 |
 * | 0.4688 | 0.4689 | 0.4688 | 0.4609 |
 * | 0.5000 | 0.5000 | 0.5000 | 0.5000 |
 * | 0.5313 | 0.5314 | 0.5313 | 0.5391 |
 * | 0.5625 | 0.5625 | 0.5625 | 0.5781 |
 * | 0.5938 | 0.5939 | 0.5938 | 0.6172 |
 * | 0.6250 | 0.6250 | 0.6250 | 0.6563 |
 * | 0.6563 | 0.6564 | 0.6563 | 0.6953 |
 * | 0.6875 | 0.6875 | 0.6875 | 0.7344 |
 * | 0.7188 | 0.7189 | 0.7188 | 0.7734 |
 * | 0.7500 | 0.7500 | 0.7500 | 0.8125 |
 * | 0.7813 | 0.7814 | 0.7813 | 0.8516 |
 * | 0.8125 | 0.8125 | 0.8125 | 0.8906 |
 * | 0.8438 | 0.8439 | 0.8438 | 0.9297 |
 * | 0.8750 | 0.8750 | 0.8750 | 0.9688 |
 * | 0.9063 | 0.9064 | 0.9063 | 1.0000 |
 * | 0.9375 | 0.9375 | 0.9375 | 1.0000 |
 * | 0.9688 | 0.9689 | 0.9688 | 1.0000 |
 * | 1.0000 | 1.0000 | 1.0000 | 1.0000 |
 * +--------+--------+--------+--------+
 *
 * grad weights: mapping from ddx value
 * passed into `textureSampleGrad` to
 * the mix weight used by the GPU
 *
 * +--------+--------+--------+--------+
 * |        |        | intel  | amd    |
 * |        |  m1    | gen-9  | rna-1  |
 * |  ddx   |  mac   | mac    | mac    |
 * +--------+--------+--------+--------+
 * | 0.5000 | 0.0000 | 0.0000 | 0.0000 |
 * | 0.5109 | 0.0390 | 0.0430 | 0.0000 |
 * | 0.5221 | 0.0821 | 0.0859 | 0.0000 |
 * | 0.5336 | 0.1211 | 0.1289 | 0.0352 |
 * | 0.5453 | 0.1600 | 0.1719 | 0.0898 |
 * | 0.5572 | 0.2032 | 0.2109 | 0.1328 |
 * | 0.5694 | 0.2422 | 0.2461 | 0.1797 |
 * | 0.5819 | 0.2814 | 0.2852 | 0.2305 |
 * | 0.5946 | 0.3203 | 0.3203 | 0.2773 |
 * | 0.6076 | 0.3554 | 0.3594 | 0.3164 |
 * | 0.6209 | 0.3868 | 0.3906 | 0.3633 |
 * | 0.6345 | 0.4218 | 0.4258 | 0.4063 |
 * | 0.6484 | 0.4532 | 0.4609 | 0.4492 |
 * | 0.6626 | 0.4882 | 0.4922 | 0.4883 |
 * | 0.6771 | 0.5196 | 0.5234 | 0.5273 |
 * | 0.6920 | 0.5507 | 0.5547 | 0.5664 |
 * | 0.7071 | 0.5860 | 0.5859 | 0.6055 |
 * | 0.7226 | 0.6132 | 0.6133 | 0.6406 |
 * | 0.7384 | 0.6407 | 0.6445 | 0.6797 |
 * | 0.7546 | 0.6679 | 0.6719 | 0.7148 |
 * | 0.7711 | 0.6953 | 0.6992 | 0.7461 |
 * | 0.7880 | 0.7225 | 0.7266 | 0.7813 |
 * | 0.8052 | 0.7500 | 0.7539 | 0.8164 |
 * | 0.8229 | 0.7814 | 0.7813 | 0.8516 |
 * | 0.8409 | 0.8086 | 0.8086 | 0.8828 |
 * | 0.8593 | 0.8321 | 0.8320 | 0.9141 |
 * | 0.8781 | 0.8554 | 0.8594 | 0.9492 |
 * | 0.8974 | 0.8789 | 0.8828 | 0.9766 |
 * | 0.9170 | 0.9025 | 0.9063 | 1.0000 |
 * | 0.9371 | 0.9297 | 0.9297 | 1.0000 |
 * | 0.9576 | 0.9532 | 0.9531 | 1.0000 |
 * | 0.9786 | 0.9765 | 0.9766 | 1.0000 |
 * | 1.0000 | 1.0000 | 1.0000 | 1.0000 |
 * +--------+--------+--------+--------+
 */

async function queryMipGradientValuesForDevice(t, stage) {
  const { device } = t;
  const kNumWeightTypes = 2;
  const module = device.createShaderModule({
    code: `
      @group(0) @binding(0) var tex: texture_2d<f32>;
      @group(0) @binding(1) var smp: sampler;
      @group(0) @binding(2) var<storage, read_write> result: array<f32>;

      struct VSOutput {
        @builtin(position) pos: vec4f,
        @location(0) @interpolate(flat, either) ndx: u32,
        @location(1) @interpolate(flat, either) result: vec4f,
      };

      fn getMixLevels(wNdx: u32) -> vec4f {
        let mipLevel = f32(wNdx) / ${kMipGradientSteps};
        let size = textureDimensions(tex);
        let g = mix(1.0, 2.0, mipLevel) / f32(size.x);
        let ddx = vec2f(g, 0);
        return vec4f(
          textureSampleLevel(tex, smp, vec2f(0.5), mipLevel).r,
          textureSampleGrad(tex, smp, vec2f(0.5), ddx, vec2f(0)).r,
          0,
          0);
      }

      fn recordMixLevels(wNdx: u32, r: vec4f) {
        let ndx = wNdx * ${kNumWeightTypes};
        for (var i: u32 = 0; i < ${kNumWeightTypes}; i++) {
          result[ndx + i] = r[i];
        }
      }

      fn getPosition(vNdx: u32) -> vec4f {
        let pos = array(
          vec2f(-1,  3),
          vec2f( 3, -1),
          vec2f(-1, -1),
        );
        let p = pos[vNdx];
        return vec4f(p, 0, 1);
      }

      @vertex fn vs(@builtin(vertex_index) vNdx: u32, @builtin(instance_index) iNdx: u32) -> VSOutput {
        return VSOutput(getPosition(vNdx), iNdx, vec4f(0));
      }

      @fragment fn fsRecord(v: VSOutput) -> @location(0) vec4f {
        recordMixLevels(v.ndx, getMixLevels(v.ndx));
        return vec4f(0);
      }

      @compute @workgroup_size(1) fn csRecord(@builtin(global_invocation_id) id: vec3u) {
        recordMixLevels(id.x, getMixLevels(id.x));
      }

      @vertex fn vsRecord(@builtin(vertex_index) vNdx: u32, @builtin(instance_index) iNdx: u32) -> VSOutput {
        return VSOutput(getPosition(vNdx), iNdx, getMixLevels(iNdx));
      }

      @fragment fn fsSaveVs(v: VSOutput) -> @location(0) vec4f {
        recordMixLevels(v.ndx, v.result);
        return vec4f(0);
      }
    `
  });

  const texture = t.createTextureTracked({
    size: [2, 2, 1],
    format: 'r8unorm',
    usage: GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.COPY_DST,
    mipLevelCount: 2
  });

  device.queue.writeTexture(
    { texture, mipLevel: 1 },
    new Uint8Array([255]),
    { bytesPerRow: 1 },
    [1, 1]
  );

  const sampler = device.createSampler({
    minFilter: 'linear',
    magFilter: 'linear',
    mipmapFilter: 'linear'
  });

  const target = t.createTextureTracked({
    size: [1, 1],
    format: 'rgba8unorm',
    usage: GPUTextureUsage.RENDER_ATTACHMENT
  });

  const storageBuffer = t.createBufferTracked({
    size: 4 * (kMipGradientSteps + 1) * kNumWeightTypes,
    usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC
  });

  const resultBuffer = t.createBufferTracked({
    size: storageBuffer.size,
    usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ
  });

  const createBindGroup = (pipeline) =>
  device.createBindGroup({
    layout: pipeline.getBindGroupLayout(0),
    entries: [
    { binding: 0, resource: texture.createView() },
    { binding: 1, resource: sampler },
    { binding: 2, resource: { buffer: storageBuffer } }]

  });

  const encoder = device.createCommandEncoder();
  switch (stage) {
    case 'compute':{
        const pipeline = device.createComputePipeline({
          layout: 'auto',
          compute: { module }
        });
        const pass = encoder.beginComputePass();
        pass.setPipeline(pipeline);
        pass.setBindGroup(0, createBindGroup(pipeline));
        pass.dispatchWorkgroups(kMipGradientSteps + 1);
        pass.end();
        break;
      }
    case 'fragment':{
        const pipeline = device.createRenderPipeline({
          layout: 'auto',
          vertex: { module, entryPoint: 'vs' },
          fragment: { module, entryPoint: 'fsRecord', targets: [{ format: 'rgba8unorm' }] }
        });
        const pass = encoder.beginRenderPass({
          colorAttachments: [
          {
            view: target.createView(),
            loadOp: 'clear',
            storeOp: 'store'
          }]

        });
        pass.setPipeline(pipeline);
        pass.setBindGroup(0, createBindGroup(pipeline));
        pass.draw(3, kMipGradientSteps + 1);
        pass.end();
        break;
      }
    case 'vertex':{
        const pipeline = device.createRenderPipeline({
          layout: 'auto',
          vertex: { module, entryPoint: 'vsRecord' },
          fragment: { module, entryPoint: 'fsSaveVs', targets: [{ format: 'rgba8unorm' }] }
        });
        const pass = encoder.beginRenderPass({
          colorAttachments: [
          {
            view: target.createView(),
            loadOp: 'clear',
            storeOp: 'store'
          }]

        });
        pass.setPipeline(pipeline);
        pass.setBindGroup(0, createBindGroup(pipeline));
        pass.draw(3, kMipGradientSteps + 1);
        pass.end();
        break;
      }
  }
  encoder.copyBufferToBuffer(storageBuffer, 0, resultBuffer, 0, resultBuffer.size);
  device.queue.submit([encoder.finish()]);

  await resultBuffer.mapAsync(GPUMapMode.READ);
  const result = Array.from(new Float32Array(resultBuffer.getMappedRange()));
  resultBuffer.unmap();
  resultBuffer.destroy();

  const [sampleLevelWeights, gradWeights] = unzip(result, kNumWeightTypes);

  validateWeights(stage, sampleLevelWeights);
  validateWeights(stage, gradWeights);

  texture.destroy();
  storageBuffer.destroy();

  return {
    sampleLevelWeights,
    softwareMixToGPUMixGradWeights: generateSoftwareMixToGPUMixGradWeights(gradWeights, texture)
  };
}

// Given an array of ascending values and a value v, finds
// which indices in the array v is between. Returns the lower
// index and the mix weight between the 2 indices for v.
//
// In other words, if values = [10, 20, 30, 40, 50]
//
//    getIndexAndWeight(values, 38)  -> [2, 0.8]
//
// Example:
//
//    values = [10, 20, 30, 40, 50]
//    v = 38
//    [ndx, weight] = getIndexAndWeight(values, v);
//    v2 = lerp(values[ndx], values[ndx + 1], weight);
//    assert(v === v2)
function getIndexAndWeight(values, v) {
  assert(v >= values[0] && v <= values[values.length - 1]);
  let lo = 0;
  let hi = values.length - 1;
  for (;;) {
    const i = lo + (hi - lo) / 2 | 0;
    const w0 = values[i];
    const w1 = values[i + 1];
    if (lo === hi || v >= w0 && v <= w1) {
      const weight = (v - w0) / (w1 - w0);
      return [i, weight];
    }
    if (v < w0) {
      hi = i;
    } else {
      lo = i + 1;
    }
  }
}

/**
 * Given a fractional number between 0 and values.length returns the value between
 * 2 values. Effectively lerp(values[ndx], values[ndx + 1], weight)
 */
function bilinearFilter(values, ndx, weight) {
  const v0 = values[ndx];
  const v1 = values[ndx + 1] ?? 0;
  assert(ndx < values.length - 1 || ndx === values.length - 1 && weight === 0);
  return lerp(v0, v1, weight);
}

/**
 * Generates an array of values that maps between the software renderer's gradient computed
 * mip level and the GPUs gradient computed mip level for mip level 0 to 1.
 */
function generateSoftwareMixToGPUMixGradWeights(gpuWeights, texture) {
  const numSteps = gpuWeights.length - 1;
  const size = [texture.width, texture.height, texture.depthOrArrayLayers];
  const softwareWeights = range(numSteps + 1, (i) => {
    // u goes from 0 to 1
    const u = i / numSteps;
    const g = lerp(1, 2, u) / texture.width;
    const mipLevel = computeMipLevelFromGradients([g], [0], size);
    assert(mipLevel >= 0 && mipLevel <= 1);
    return mipLevel;
  });
  const softwareMixToGPUMixMap = range(numSteps + 1, (i) => {
    const mix = i / numSteps;
    const [ndx, weight] = getIndexAndWeight(softwareWeights, mix);
    return bilinearFilter(gpuWeights, ndx, weight);
  });
  return softwareMixToGPUMixMap;
}

function mapSoftwareMipLevelToGPUMipLevel(t, stage, mipLevel) {
  const baseLevel = Math.floor(mipLevel);
  const softwareMix = mipLevel - baseLevel;
  const gpuMix = getMixWeightByTypeForMipLevel(
    t,
    stage,
    'softwareMixToGPUMixGradWeights',
    softwareMix
  );
  return baseLevel + gpuMix;
}

const euclideanModulo = (n, m) => (n % m + m) % m;

/**
 * Gets the mip gradient values for the current device.
 * The issue is, different GPUs have different ways of mixing between mip levels.
 * For most GPUs it's linear but for AMD GPUs on Mac in particular, it's something
 * else (which AFAICT is against all the specs).
 *
 * We seemingly have 3 options:
 *
 * 1. Increase the tolerances of tests so they pass on AMD.
 * 2. Mark AMD as failing
 * 3. Try to figure out how the GPU converts mip levels into weights
 *
 * We're doing 3.
 *
 * There's an assumption that the gradient will be the same for all formats
 * and usages.
 *
 * Note: The code below has 2 maps. One device->Promise, the other device->weights
 * device->weights is meant to be used synchronously by other code so we don't
 * want to leave initMipGradientValuesForDevice until the weights have been read.
 * But, multiple subcases will run because this function is async. So, subcase 1
 * runs, hits this init code, this code waits for the weights. Then, subcase 2
 * runs and hits this init code. The weights will not be in the device->weights map
 * yet which is why we have the device->Promise map. This is so subcase 2 waits
 * for subcase 1's "query the weights" step. Otherwise, all subcases would do the
 * "get the weights" step separately.
 */
const kMipGradientSteps = 64;
const s_deviceToMipGradientValuesPromise = new WeakMap(


);
const s_deviceToMipGradientValues = new WeakMap();

async function initMipGradientValuesForDevice(t, stage) {
  const { device } = t;
  // Get the per stage promises (or make them)
  const stageWeightsP =
  s_deviceToMipGradientValuesPromise.get(device) ??
  {};
  s_deviceToMipGradientValuesPromise.set(device, stageWeightsP);

  let weightsP = stageWeightsP[stage];
  if (!weightsP) {
    // There was no promise for this weight so request it
    // and add a then clause so the first thing that will happen
    // when the promise resolves is that we'll record the weights for
    // that stage.
    weightsP = queryMipGradientValuesForDevice(t, stage);
    weightsP.
    then((weights) => {
      const stageWeights =
      s_deviceToMipGradientValues.get(device) ?? {};
      s_deviceToMipGradientValues.set(device, stageWeights);
      stageWeights[stage] = weights;
    }).
    catch((e) => {
      throw e;
    });
    stageWeightsP[stage] = weightsP;
  }
  return await weightsP;
}

function getMixWeightByTypeForMipLevel(
t,
stage,
weightType,
mipLevel)
{
  if (weightType === 'identity') {
    return euclideanModulo(mipLevel, 1);
  }
  // linear interpolate between weights
  const weights = s_deviceToMipGradientValues.get(t.device)[stage][weightType];
  assert(
    !!weights,
    'you must use WGSLTextureSampleTest or call initializeDeviceMipWeights before calling this function'
  );
  const steps = weights.length - 1;
  const w = euclideanModulo(mipLevel, 1) * steps;
  const lowerNdx = Math.floor(w);
  const upperNdx = Math.ceil(w);
  const mix = w % 1;
  return lerp(weights[lowerNdx], weights[upperNdx], mix);
}

function getWeightForMipLevel(
t,
stage,
weightType,
mipLevelCount,
mipLevel)
{
  if (mipLevel < 0 || mipLevel >= mipLevelCount) {
    return 1;
  }
  return getMixWeightByTypeForMipLevel(t, stage, weightType, mipLevel);
}

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

/**
 * Used for textureSampleXXX
 */
export class WGSLTextureSampleTest extends GPUTest {
  async init() {
    await super.init();
  }
}

/**
 * Used to specify a range from [0, num)
 * The type is used to determine if values should be integers and if they can be negative.
 */





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

function getMinAndMaxTexelValueForComponent(
rep,
component)
{
  assert(!!rep.numericRange);
  const perComponentRanges = rep.numericRange;
  const perComponentRange = perComponentRanges[component];
  const range = rep.numericRange;
  const { min, max } = perComponentRange ? perComponentRange : range;
  return { min: getLimitValue(min), max: getLimitValue(max) };
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
  return base.includes('depth') ?
  base :
  `${base}<${getTextureFormatTypeInfo(format).componentType}>`;
}





/**
 * Make a generator for texels for depth comparison tests.
 */
export function makeRandomDepthComparisonTexelGenerator(
info,



comparison)
{
  const rep = kTexelRepresentationInfo[info.format];
  const size = reifyExtent3D(info.size);

  const comparisonIsEqualOrNotEqual = comparison === 'equal' || comparison === 'not-equal';

  // for equal and not-equal we just want to test 0, 0.6, and 1
  // for everything else we want 0 to 1
  // Note: 0.6 is chosen because we'll never choose 0.6 as our depth reference
  // value. (see generateTextureBuiltinInputsImpl and generateSamplePointsCube)
  // The problem with comparing equal is other than 0.0 and 1.0, no other
  // values are guaranteed to be equal.
  const fixedValues = [0, 0.6, 1, 1];
  const format = comparisonIsEqualOrNotEqual ?
  (norm) => fixedValues[norm * (fixedValues.length - 1) | 0] :
  (norm) => norm;

  return (coords) => {
    const texel = {};
    for (const component of rep.componentOrder) {
      const rnd = hashU32(
        coords.x,
        coords.y,
        coords.z,
        coords.sampleIndex ?? 0,
        component.charCodeAt(0),
        size.width,
        size.height,
        size.depthOrArrayLayers
      );
      const normalized = clamp(rnd / 0xffffffff, { min: 0, max: 1 });
      texel[component] = format(normalized);
    }
    return quantize(texel, rep);
  };
}

function createRandomTexelViewViaColors(
info,




options)
{
  const rep = kTexelRepresentationInfo[info.format];
  const size = reifyExtent3D(info.size);
  const minMax = Object.fromEntries(
    rep.componentOrder.map((component) => [
    component,
    getMinAndMaxTexelValueForComponent(rep, component)]
    )
  );
  const generator = (coords) => {
    const texel = {};
    for (const component of rep.componentOrder) {
      const rnd = hashU32(
        coords.x,
        coords.y,
        coords.z,
        coords.sampleIndex ?? 0,
        component.charCodeAt(0),
        info.mipLevel,
        size.width,
        size.height,
        size.depthOrArrayLayers
      );
      const normalized = clamp(rnd / 0xffffffff, { min: 0, max: 1 });
      const { min, max } = minMax[component];
      texel[component] = lerp(min, max, normalized);
    }
    return quantize(texel, rep);
  };
  return TexelView.fromTexelsAsColors(
    info.format,
    options?.generator ?? generator
  );
}

function createRandomTexelViewViaBytes(info)




{
  const { format } = info;
  const formatInfo = kTextureFormatInfo[format];
  const rep = kTexelRepresentationInfo[info.format];
  assert(!!rep);
  const bytesPerBlock = formatInfo.color?.bytes ?? formatInfo.stencil?.bytes;
  assert(bytesPerBlock > 0);
  const size = physicalMipSize(reifyExtent3D(info.size), info.format, '2d', 0);
  const blocksAcross = Math.ceil(size.width / formatInfo.blockWidth);
  const blocksDown = Math.ceil(size.height / formatInfo.blockHeight);
  const bytesPerRow = blocksAcross * bytesPerBlock * info.sampleCount;
  const bytesNeeded = bytesPerRow * blocksDown * size.depthOrArrayLayers;
  const data = new Uint8Array(bytesNeeded);

  const hashBase =
  sumOfCharCodesOfString(info.format) +
  size.width +
  size.height +
  size.depthOrArrayLayers +
  info.mipLevel +
  info.sampleCount;

  if (info.format.includes('32float') || info.format.includes('16float')) {
    const { min, max } = getMinAndMaxTexelValueForComponent(rep, TexelComponent.R);
    const asFloat = info.format.includes('32float') ?
    new Float32Array(data.buffer) :
    new Float16Array(data.buffer);
    for (let i = 0; i < asFloat.length; ++i) {
      asFloat[i] = lerp(min, max, hashU32(hashBase + i) / 0xffff_ffff);
    }
  } else if (bytesNeeded % 4 === 0) {
    const asU32 = new Uint32Array(data.buffer);
    for (let i = 0; i < asU32.length; ++i) {
      asU32[i] = hashU32(hashBase + i);
    }
  } else {
    for (let i = 0; i < bytesNeeded; ++i) {
      data[i] = hashU32(hashBase + i);
    }
  }

  return TexelView.fromTextureDataByReference(info.format, data, {
    bytesPerRow,
    rowsPerImage: size.height,
    subrectOrigin: [0, 0, 0],
    subrectSize: size
  });
}

/**
 * Creates a TexelView filled with random values.
 */
function createRandomTexelView(
info,





options)
{
  assert(!isCompressedTextureFormat(info.format));
  const formatInfo = kTextureFormatInfo[info.format];
  const type = formatInfo.color?.type ?? formatInfo.depth?.type ?? formatInfo.stencil?.type;
  const canFillWithRandomTypedData =
  !options &&
  isEncodableTextureFormat(info.format) && (
  info.format.includes('norm') && type !== 'depth' ||
  info.format.includes('16float') ||
  info.format.includes('32float') && type !== 'depth' ||
  type === 'sint' ||
  type === 'uint');

  return canFillWithRandomTypedData ?
  createRandomTexelViewViaBytes(info) :
  createRandomTexelViewViaColors(info, options);
}

/**
 * Creates a mip chain of TexelViews filled with random values
 */
function createRandomTexelViewMipmap(
info,






options)
{
  const mipLevelCount = info.mipLevelCount ?? 1;
  const dimension = info.dimension ?? '2d';
  return range(mipLevelCount, (i) =>
  createRandomTexelView(
    {
      format: info.format,
      size: virtualMipSize(dimension, info.size, i),
      mipLevel: i,
      sampleCount: info.sampleCount ?? 1
    },
    options
  )
  );
}

// Because it's easy to deal with if these types are all array of number






const kTextureCallArgNames = [
'component',
'coords',
'derivativeMult', // NOTE: derivativeMult not an argument but is used with coords for implicit derivatives.
'arrayIndex',
'bias',
'sampleIndex',
'mipLevel',
'ddx',
'ddy',
'depthRef',
'offset'];





































const isBuiltinComparison = (builtin) =>
builtin === 'textureGatherCompare' ||
builtin === 'textureSampleCompare' ||
builtin === 'textureSampleCompareLevel';
const isBuiltinGather = (builtin) =>
builtin === 'textureGather' || builtin === 'textureGatherCompare';
const builtinNeedsSampler = (builtin) =>
builtin.startsWith('textureSample') || builtin.startsWith('textureGather');
const builtinNeedsDerivatives = (builtin) =>
builtin === 'textureSample' ||
builtin === 'textureSampleBias' ||
builtin === 'textureSampleCompare';

const isCubeViewDimension = (viewDescriptor) =>
viewDescriptor?.dimension === 'cube' || viewDescriptor?.dimension === 'cube-array';

const s_u32 = new Uint32Array(1);
const s_f32 = new Float32Array(s_u32.buffer);
const s_i32 = new Int32Array(s_u32.buffer);

const kBitCastFunctions = {
  f: (v) => {
    s_f32[0] = v;
    return s_u32[0];
  },
  i: (v) => {
    s_i32[0] = v;
    assert(s_i32[0] === v, 'check we are not casting non-int or out-of-range value');
    return s_u32[0];
  },
  u: (v) => {
    s_u32[0] = v;
    assert(s_u32[0] === v, 'check we are not casting non-uint or out-of-range value');
    return s_u32[0];
  }
};

function getCallArgType(
call,
argName)
{
  switch (argName) {
    case 'coords':
    case 'derivativeMult':
      return call.coordType;
    case 'component':
      assert(call.componentType !== undefined);
      return call.componentType;
    case 'mipLevel':
      assert(call.levelType !== undefined);
      return call.levelType;
    case 'arrayIndex':
      assert(call.arrayIndexType !== undefined);
      return call.arrayIndexType;
    case 'sampleIndex':
      assert(call.sampleIndexType !== undefined);
      return call.sampleIndexType;
    case 'bias':
    case 'depthRef':
    case 'ddx':
    case 'ddy':
      return 'f';
    default:
      unreachable();
  }
}

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
 * Convert RGBA result format to texel view format.
 * Example, converts
 *   { R: 0.1, G: 0, B: 0, A: 1 } to { Depth: 0.1 }
 *   { R: 0.1 } to { R: 0.1, G: 0, B: 0, A: 1 }
 */
function convertToTexelViewFormat(src, format) {
  const componentOrder = isDepthTextureFormat(format) ?
  [TexelComponent.Depth] :
  isStencilTextureFormat(format) ?
  [TexelComponent.Stencil] :
  [TexelComponent.R, TexelComponent.G, TexelComponent.B, TexelComponent.A];
  const out = {};
  for (const component of componentOrder) {
    let v = src[component];
    if (v === undefined) {
      if (component === 'Depth' || component === 'Stencil') {
        v = src.R;
      } else if (component === 'G' || component === 'B') {
        v = 0;
      } else {
        v = 1;
      }
    }
    out[component] = v;
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

function zeroValuePerTexelComponent(components) {
  const out = {};
  for (const component of components) {
    out[component] = 0;
  }
  return out;
}

const kSamplerFns = {
  never: (ref, v) => false,
  less: (ref, v) => ref < v,
  equal: (ref, v) => ref === v,
  'less-equal': (ref, v) => ref <= v,
  greater: (ref, v) => ref > v,
  'not-equal': (ref, v) => ref !== v,
  'greater-equal': (ref, v) => ref >= v,
  always: (ref, v) => true
};

function applyCompare(
call,
sampler,
components,
src)
{
  if (isBuiltinComparison(call.builtin)) {
    assert(sampler !== undefined);
    assert(call.depthRef !== undefined);
    const out = {};
    const compareFn = kSamplerFns[sampler.compare];
    for (const component of components) {
      out[component] = compareFn(call.depthRef, src[component]) ? 1 : 0;
    }
    return out;
  } else {
    return src;
  }
}

/**
 * Returns the expect value for a WGSL builtin texture function for a single
 * mip level
 */
function softwareTextureReadMipLevel(
call,
texture,
sampler,
mipLevel)
{
  assert(mipLevel % 1 === 0);
  const { format } = texture.texels[0];
  const rep = kTexelRepresentationInfo[format];
  const textureSize = virtualMipSize(
    texture.descriptor.dimension || '2d',
    texture.descriptor.size,
    mipLevel
  );
  const addressMode =
  call.builtin === 'textureSampleBaseClampToEdge' ?
  ['clamp-to-edge', 'clamp-to-edge', 'clamp-to-edge'] :
  [
  sampler?.addressModeU ?? 'clamp-to-edge',
  sampler?.addressModeV ?? 'clamp-to-edge',
  sampler?.addressModeW ?? 'clamp-to-edge'];


  const isCube = isCubeViewDimension(texture.viewDescriptor);
  const arrayIndexMult = isCube ? 6 : 1;
  const numLayers = textureSize[2] / arrayIndexMult;
  assert(numLayers % 1 === 0);
  const textureSizeForCube = [textureSize[0], textureSize[1], 6];

  const load = (at) => {
    const zFromArrayIndex =
    call.arrayIndex !== undefined ?
    clamp(call.arrayIndex, { min: 0, max: numLayers - 1 }) * arrayIndexMult :
    0;
    return texture.texels[mipLevel].color({
      x: Math.floor(at[0]),
      y: Math.floor(at[1] ?? 0),
      z: Math.floor(at[2] ?? 0) + zFromArrayIndex,
      sampleIndex: call.sampleIndex
    });
  };

  switch (call.builtin) {
    case 'textureGather':
    case 'textureGatherCompare':
    case 'textureSample':
    case 'textureSampleBias':
    case 'textureSampleBaseClampToEdge':
    case 'textureSampleCompare':
    case 'textureSampleCompareLevel':
    case 'textureSampleGrad':
    case 'textureSampleLevel':{
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
        let at = coords.map((v, i) => v * (isCube ? textureSizeForCube : textureSize)[i] - 0.5);

        // Apply offset in whole texel units
        // This means the offset is added at each mip level in texels. There's no
        // scaling for each level.
        if (call.offset !== undefined) {
          at = add(at, toArray(call.offset));
        }

        const samples = [];

        const filter = isBuiltinGather(call.builtin) ? 'linear' : sampler?.minFilter ?? 'nearest';
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
                    // Note: These are ordered to match textureGather
                    samples.push({ at: [p0[0], p1[1]], weight: p0W[0] * p1W[1] });
                    samples.push({ at: p1, weight: p1W[0] * p1W[1] });
                    samples.push({ at: [p1[0], p0[1]], weight: p1W[0] * p0W[1] });
                    samples.push({ at: p0, weight: p0W[0] * p0W[1] });
                    break;
                  }
                case 3:{
                    // cube sampling, here in the software renderer, is the same
                    // as 2d sampling. We'll sample at most 4 texels. The weights are
                    // the same as if it was just one plane. If the points fall outside
                    // the slice they'll be wrapped by wrapFaceCoordToCubeFaceAtEdgeBoundaries
                    // below.
                    if (isCube) {
                      // Note: These are ordered to match textureGather
                      samples.push({ at: [p0[0], p1[1], p0[2]], weight: p0W[0] * p1W[1] });
                      samples.push({ at: p1, weight: p1W[0] * p1W[1] });
                      samples.push({ at: [p1[0], p0[1], p0[2]], weight: p1W[0] * p0W[1] });
                      samples.push({ at: p0, weight: p0W[0] * p0W[1] });
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
                        //
                        // We could check that, given the 3 texels at the corner, if all 3 texels
                        // are the same value then the result must be the same value. Otherwise,
                        // the result must be between the 3 values. For now, the code that
                        // chooses test coordinates avoids corners. This has the restriction
                        // that the smallest mip level be at least 4x4 so there are some non
                        // corners to choose from.
                        unreachable(
                          `corners of cubemaps are not testable:\n   ${describeTextureCall(call)}`
                        );
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

        if (isBuiltinGather(call.builtin)) {
          const componentNdx = call.component ?? 0;
          assert(componentNdx >= 0 && componentNdx < 4);
          assert(samples.length === 4);
          const component = kRGBAComponents[componentNdx];
          const out = {};
          samples.forEach((sample, i) => {
            const c = isCube ?
            wrapFaceCoordToCubeFaceAtEdgeBoundaries(textureSize[0], sample.at) :
            applyAddressModesToCoords(addressMode, textureSize, sample.at);
            const v = load(c);
            const postV = applyCompare(call, sampler, rep.componentOrder, v);
            const rgba = convertPerTexelComponentToResultFormat(postV, format);
            out[kRGBAComponents[i]] = rgba[component];
          });
          return out;
        }

        const out = {};
        for (const sample of samples) {
          const c = isCube ?
          wrapFaceCoordToCubeFaceAtEdgeBoundaries(textureSize[0], sample.at) :
          applyAddressModesToCoords(addressMode, textureSize, sample.at);
          const v = load(c);
          const postV = applyCompare(call, sampler, rep.componentOrder, v);
          for (const component of rep.componentOrder) {
            out[component] = (out[component] ?? 0) + postV[component] * sample.weight;
          }
        }

        return convertPerTexelComponentToResultFormat(out, format);
      }
    case 'textureLoad':{
        const out = isOutOfBoundsCall(texture, call) ?
        zeroValuePerTexelComponent(rep.componentOrder) :
        load(call.coords);
        return convertPerTexelComponentToResultFormat(out, format);
      }
    default:
      unreachable();
  }
}

/**
 * Reads a texture, optionally sampling between 2 mipLevels
 */
function softwareTextureReadLevel(
t,
stage,
call,
texture,
sampler,
mipLevel)
{
  const mipLevelCount = texture.texels.length;
  const maxLevel = mipLevelCount - 1;

  if (!sampler) {
    return softwareTextureReadMipLevel(call, texture, sampler, mipLevel);
  }

  const effectiveMipmapFilter = isBuiltinGather(call.builtin) ? 'nearest' : sampler.mipmapFilter;
  switch (effectiveMipmapFilter) {
    case 'linear':{
        const clampedMipLevel = clamp(mipLevel, { min: 0, max: maxLevel });
        const baseMipLevel = Math.floor(clampedMipLevel);
        const nextMipLevel = Math.ceil(clampedMipLevel);
        const t0 = softwareTextureReadMipLevel(call, texture, sampler, baseMipLevel);
        const t1 = softwareTextureReadMipLevel(call, texture, sampler, nextMipLevel);
        const weightType = call.builtin === 'textureSampleLevel' ? 'sampleLevelWeights' : 'identity';
        const mix = getWeightForMipLevel(t, stage, weightType, mipLevelCount, clampedMipLevel);
        assert(mix >= 0 && mix <= 1);
        const values = [
        { v: t0, weight: 1 - mix },
        { v: t1, weight: mix }];

        const out = {};
        for (const { v, weight } of values) {
          for (const component of kRGBAComponents) {
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

function computeMipLevelFromGradients(
ddx,
ddy,
size)
{
  const texSize = reifyExtent3D(size);
  const textureSize = [texSize.width, texSize.height, texSize.depthOrArrayLayers];

  // Compute the mip level the same way textureSampleGrad does according to the spec.
  const scaledDdx = ddx.map((v, i) => v * textureSize[i]);
  const scaledDdy = ddy.map((v, i) => v * textureSize[i]);
  const dotDDX = dotProduct(scaledDdx, scaledDdx);
  const dotDDY = dotProduct(scaledDdy, scaledDdy);
  const deltaMax = Math.max(dotDDX, dotDDY);
  const mipLevel = 0.5 * Math.log2(deltaMax);
  return mipLevel;
}

function computeMipLevelFromGradientsForCall(
call,
size)
{
  assert(!!call.ddx);
  assert(!!call.ddy);
  // ddx and ddy are the values that would be passed to textureSampleGrad
  // If we're emulating textureSample then they're the computed derivatives
  // such that if we passed them to textureSampleGrad they'd produce the
  // same result.
  const ddx = typeof call.ddx === 'number' ? [call.ddx] : call.ddx;
  const ddy = typeof call.ddy === 'number' ? [call.ddy] : call.ddy;

  return computeMipLevelFromGradients(ddx, ddy, size);
}

/**
 * The software version of textureSampleGrad except with optional level.
 */
function softwareTextureReadGrad(
t,
stage,
call,
texture,
sampler)
{
  const bias = call.bias === undefined ? 0 : clamp(call.bias, { min: -16.0, max: 15.99 });
  if (call.ddx) {
    const mipLevel = computeMipLevelFromGradientsForCall(call, texture.descriptor.size);
    const mipLevelCount = texture.descriptor.mipLevelCount ?? 1;
    const clampedMipLevel = clamp(mipLevel + bias, { min: 0, max: mipLevelCount - 1 });
    const weightMipLevel = mapSoftwareMipLevelToGPUMipLevel(t, stage, clampedMipLevel);
    return softwareTextureReadLevel(t, stage, call, texture, sampler, weightMipLevel);
  } else {
    return softwareTextureReadLevel(t, stage, call, texture, sampler, (call.mipLevel ?? 0) + bias);
  }
}

/**
 * This must match the code in doTextureCalls for derivativeBase
 *
 * Note: normal implicit derivatives are computed like this
 *
 * fn textureSample(T, S, coord) -> vec4f {
 *    return textureSampleGrad(T, S, dpdx(coord), dpdy(coord));
 * }
 *
 * dpdx and dpdy are effectively computed by,
 * getting the values of coord for 2x2 adjacent texels.
 *
 *   p0 = coord value at x, y
 *   p1 = coord value at x + 1, y
 *   p2 = coord value at x, y + 1
 *   p3 = coord value at x + 1, y + 1
 *
 * dpdx is the average delta in x and dpdy is the average delta in y
 *
 *   dpdx = (p1 - p0 + p3 - p2) / 2   // average of horizontal change
 *   dpdy = (p2 - p0 + p3 - p1) / 2   // average of vertical change
 *
 * derivativeBase is
 *
 *       '1d'    '2d'     '3d'
 *   p0 = [0]   [0, 0]  [0, 0, 0]
 *   p1 = [1]   [1, 0]  [1, 0, 0]
 *   p2 = [0]   [0, 1]  [0, 1, 0]
 *   p3 = [1]   [1, 1]  [1, 1, 0]
 *
 * But, these values are normalized texels coords so if the src texture
 * is 8x8 these would be * 0.125
 *
 * Note: to test other derivatives we add in a multiplier but,
 * this base gives us something to add that starts at 0,0 at the call
 * but who's derivatives we can easily set. We need the default
 * derivativeBase to be 1 otherwise it's 0 which makes the computed mip level
 * be -Infinity which means bias in `textureSampleBias` has no meaning.
 */
function derivativeBaseForCall(texture, isDDX) {
  const texSize = reifyExtent3D(texture.descriptor.size);
  const textureSize = [texSize.width, texSize.height, texSize.depthOrArrayLayers];
  if (isCubeViewDimension(texture.viewDescriptor)) {
    return isDDX ? [1 / textureSize[0], 0, 1] : [0, 1 / textureSize[1], 1];
  } else if (texture.descriptor.dimension === '3d') {
    return isDDX ? [1 / textureSize[0], 0, 0] : [0, 1 / textureSize[1], 0];
  } else if (texture.descriptor.dimension === '1d') {
    return [1 / textureSize[0]];
  } else {
    return isDDX ? [1 / textureSize[0], 0] : [0, 1 / textureSize[1]];
  }
}

/**
 * Multiplies derivativeBase by derivativeMult or 1
 */
function derivativeForCall(
texture,
call,
isDDX)
{
  const dd = derivativeBaseForCall(texture, isDDX);
  return dd.map((v, i) => v * (call.derivativeMult?.[i] ?? 1));
}

function softwareTextureRead(
t,
stage,
call,
texture,
sampler)
{
  // add the implicit derivatives that we use from WGSL in doTextureCalls
  if (builtinNeedsDerivatives(call.builtin) && !call.ddx) {
    const newCall = {
      ...call,
      ddx: call.ddx ?? derivativeForCall(texture, call, true),
      ddy: call.ddy ?? derivativeForCall(texture, call, false)
    };
    call = newCall;
  }
  return softwareTextureReadGrad(t, stage, call, texture, sampler);
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
  assert(call.coords !== undefined);

  const desc = reifyTextureDescriptor(texture.descriptor);
  const { coords, mipLevel, arrayIndex, sampleIndex } = call;

  if (mipLevel !== undefined && (mipLevel < 0 || mipLevel >= desc.mipLevelCount)) {
    return true;
  }

  const size = virtualMipSize(
    texture.descriptor.dimension || '2d',
    texture.descriptor.size,
    mipLevel ?? 0
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

function isValidOutOfBoundsValue(
texture,
gotRGBA,
maxFractionalDiff)
{
  // For a texture builtin with no sampler (eg textureLoad),
  // any out of bounds access is allowed to return one of:
  //
  // * the value of any texel in the texture
  // * 0,0,0,0 or 0,0,0,1 if not a depth texture
  // * 0 if a depth texture
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

  // Can be any texel value
  for (let mipLevel = 0; mipLevel < texture.texels.length; ++mipLevel) {
    const mipTexels = texture.texels[mipLevel];
    const size = virtualMipSize(
      texture.descriptor.dimension || '2d',
      texture.descriptor.size,
      mipLevel
    );
    const sampleCount = texture.descriptor.sampleCount ?? 1;
    for (let z = 0; z < size[2]; ++z) {
      for (let y = 0; y < size[1]; ++y) {
        for (let x = 0; x < size[0]; ++x) {
          for (let sampleIndex = 0; sampleIndex < sampleCount; ++sampleIndex) {
            const texel = mipTexels.color({ x, y, z, sampleIndex });
            const rgba = convertPerTexelComponentToResultFormat(texel, mipTexels.format);
            if (texelsApproximatelyEqual(gotRGBA, rgba, mipTexels.format, maxFractionalDiff)) {
              return true;
            }
          }
        }
      }
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

  return isValidOutOfBoundsValue(texture, gotRGBA, maxFractionalDiff);
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
  const gULP = convertPerTexelComponentToResultFormat(
    rep.bitsToULPFromZero(rep.numberToBits(got)),
    format
  );
  const eULP = convertPerTexelComponentToResultFormat(
    rep.bitsToULPFromZero(rep.numberToBits(expect)),
    format
  );

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

// If it's `textureGather` then we need to convert all values to one component.
// In other words, imagine the format is rg11b10ufloat. If it was
// `textureSample` we'd have `r11, g11, b10, a=1` but for `textureGather`
//
// component = 0 => `r11, r11, r11, r11`
// component = 1 => `g11, g11, g11, g11`
// component = 2 => `b10, b10, b10, b10`
//
// etc..., each from a different texel
//
// The Texel utils don't handle this. So if `component = 2` we take each value,
// copy it to the `B` component, run it through the texel utils so it returns
// the correct ULP for a 10bit float (not an 11 bit float). Then copy it back to
// the channel it came from.
function getULPFromZeroForComponents(
rgba,
format,
builtin,
componentNdx)
{
  const rep = kTexelRepresentationInfo[format];
  if (isBuiltinGather(builtin)) {
    const out = {};
    const component = kRGBAComponents[componentNdx ?? 0];
    const temp = { R: 0, G: 0, B: 0, A: 1 };
    for (const comp of kRGBAComponents) {
      temp[component] = rgba[comp];
      const texel = convertResultFormatToTexelViewFormat(temp, format);
      const ulp = convertPerTexelComponentToResultFormat(
        rep.bitsToULPFromZero(rep.numberToBits(texel)),
        format
      );
      out[comp] = ulp[component];
    }
    return out;
  } else {
    const texel = convertResultFormatToTexelViewFormat(rgba, format);
    return convertPerTexelComponentToResultFormat(
      rep.bitsToULPFromZero(rep.numberToBits(texel)),
      format
    );
  }
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
results,
shortShaderStage,
gpuTexture)
{
  const stage = kShortShaderStageToShaderStage[shortShaderStage];
  await initMipGradientValuesForDevice(t, stage);

  let haveComparisonCheckInfo = false;
  let checkInfo = {
    runner: results.runner,
    calls,
    sampler
  };
  // These are only read if the tests fail. They are used to get the values from the
  // GPU texture for displaying in diagnostics.
  let gpuTexels;
  const errs = [];
  const format = texture.texels[0].format;
  const size = reifyExtent3D(texture.descriptor.size);
  const maxFractionalDiff =
  sampler?.minFilter === 'linear' ||
  sampler?.magFilter === 'linear' ||
  sampler?.mipmapFilter === 'linear' ?
  getMaxFractionalDiffForTextureFormat(texture.descriptor.format) :
  0;

  for (let callIdx = 0; callIdx < calls.length; callIdx++) {
    const call = calls[callIdx];
    const gotRGBA = results.results[callIdx];
    const expectRGBA = softwareTextureRead(t, stage, call, texture, sampler);

    // The spec says depth and stencil have implementation defined values for G, B, and A
    // so if this is `textureGather` and component > 0 then there's nothing to check.
    if (
    isDepthOrStencilTextureFormat(format) &&
    isBuiltinGather(call.builtin) &&
    call.component > 0)
    {
      continue;
    }

    if (texelsApproximatelyEqual(gotRGBA, expectRGBA, format, maxFractionalDiff)) {
      continue;
    }

    if (!sampler && okBecauseOutOfBounds(texture, call, gotRGBA, maxFractionalDiff)) {
      continue;
    }

    const gULP = getULPFromZeroForComponents(gotRGBA, format, call.builtin, call.component);
    const eULP = getULPFromZeroForComponents(expectRGBA, format, call.builtin, call.component);

    // from the spec: https://gpuweb.github.io/gpuweb/#reading-depth-stencil
    // depth and stencil values are D, ?, ?, ?
    const rgbaComponentsToCheck =
    isBuiltinGather(call.builtin) || !isDepthOrStencilTextureFormat(format) ?
    kRGBAComponents :
    kRComponent;

    let bad = false;
    const diffs = rgbaComponentsToCheck.map((component) => {
      const g = gotRGBA[component];
      const e = expectRGBA[component];
      const absDiff = Math.abs(g - e);
      const ulpDiff = Math.abs(gULP[component] - eULP[component]);
      assert(!Number.isNaN(ulpDiff));
      const maxAbs = Math.max(Math.abs(g), Math.abs(e));
      const relDiff = maxAbs > 0 ? absDiff / maxAbs : 0;
      if (ulpDiff > 3 && absDiff > maxFractionalDiff) {
        bad = true;
      }
      return { absDiff, relDiff, ulpDiff };
    });

    const isFloatType = (format) => {
      const info = kTextureFormatInfo[format];
      return info.color?.type === 'float' || info.depth?.type === 'depth';
    };
    const fix5 = (n) => isFloatType(format) ? n.toFixed(5) : n.toString();
    const fix5v = (arr) => arr.map((v) => fix5(v)).join(', ');
    const rgbaToArray = (p) =>
    rgbaComponentsToCheck.map((component) => p[component]);

    if (bad) {
      const desc = describeTextureCall(call);
      errs.push(`result was not as expected:
      size: [${size.width}, ${size.height}, ${size.depthOrArrayLayers}]
  mipCount: ${texture.descriptor.mipLevelCount ?? 1}
      call: ${desc}  // #${callIdx}`);
      if (isCubeViewDimension(texture.viewDescriptor)) {
        const coord = convertCubeCoordToNormalized3DTextureCoord(call.coords);
        const faceNdx = Math.floor(coord[2] * 6);
        errs.push(`          : as 3D texture coord: (${coord[0]}, ${coord[1]}, ${coord[2]})`);
        for (let mipLevel = 0; mipLevel < (texture.descriptor.mipLevelCount ?? 1); ++mipLevel) {
          const mipSize = virtualMipSize(
            texture.descriptor.dimension ?? '2d',
            texture.descriptor.size,
            mipLevel
          );
          const t = coord.slice(0, 2).map((v, i) => (v * mipSize[i]).toFixed(3));
          errs.push(
            `          : as texel coord mip level[${mipLevel}]: (${t[0]}, ${t[1]}), face: ${faceNdx}(${kFaceNames[faceNdx]})`
          );
        }
      } else {
        for (let mipLevel = 0; mipLevel < (texture.descriptor.mipLevelCount ?? 1); ++mipLevel) {
          const mipSize = virtualMipSize(
            texture.descriptor.dimension ?? '2d',
            texture.descriptor.size,
            mipLevel
          );
          const t = call.coords.map((v, i) => (v * mipSize[i]).toFixed(3));
          errs.push(`          : as texel coord @ mip level[${mipLevel}]: (${t.join(', ')})`);
        }
      }
      if (builtinNeedsDerivatives(call.builtin)) {
        const ddx = derivativeForCall(texture, call, true);
        const ddy = derivativeForCall(texture, call, false);
        const mipLevel = computeMipLevelFromGradients(ddx, ddy, size);
        const biasStr = call.bias === undefined ? '' : ' (without bias)';
        errs.push(`implicit derivative based mip level: ${fix5(mipLevel)}${biasStr}`);
        if (call.bias) {
          const clampedBias = clamp(call.bias ?? 0, { min: -16.0, max: 15.99 });
          errs.push(`\
                       clamped bias: ${fix5(clampedBias)}
                mip level with bias: ${fix5(mipLevel + clampedBias)}`);
        }
      } else if (call.ddx) {
        const mipLevel = computeMipLevelFromGradientsForCall(call, size);
        errs.push(`gradient based mip level: ${mipLevel}`);
      }
      errs.push(`\
       got: ${fix5v(rgbaToArray(gotRGBA))}
  expected: ${fix5v(rgbaToArray(expectRGBA))}
  max diff: ${maxFractionalDiff}
 abs diffs: ${fix5v(diffs.map(({ absDiff }) => absDiff))}
 rel diffs: ${diffs.map(({ relDiff }) => `${(relDiff * 100).toFixed(2)}%`).join(', ')}
 ulp diffs: ${diffs.map(({ ulpDiff }) => ulpDiff).join(', ')}
`);

      if (sampler) {
        if (t.rec.debugging) {
          // For compares, we can't use the builtin (textureXXXCompareXXX) because it only
          // returns 0 or 1 or the average of 0 and 1 for multiple samples. And, for example,
          // if the comparison is `always` then every sample returns 1. So we need to use the
          // corresponding sample function to get the actual values from the textures
          //
          // textureSampleCompare -> textureSample
          // textureSampleCompareLevel -> textureSampleLevel
          // textureGatherCompare -> textureGather
          if (isBuiltinComparison(call.builtin)) {
            if (!haveComparisonCheckInfo) {
              // Convert the comparison calls to their corresponding non-comparison call
              const debugCalls = calls.map((call) => {
                const debugCall = { ...call };
                debugCall.depthRef = undefined;
                switch (call.builtin) {
                  case 'textureGatherCompare':
                    debugCall.builtin = 'textureGather';
                    break;
                  case 'textureSampleCompare':
                    debugCall.builtin = 'textureSample';
                    break;
                  case 'textureSampleCompareLevel':
                    debugCall.builtin = 'textureSampleLevel';
                    debugCall.levelType = 'f';
                    debugCall.mipLevel = 0;
                    break;
                  default:
                    unreachable();
                }
                return debugCall;
              });

              // Convert the comparison sampler to a non-comparison sampler
              const debugSampler = { ...sampler };
              delete debugSampler.compare;

              // Make a runner for these changed calls.
              const debugRunner = createTextureCallsRunner(
                t,
                {
                  format,
                  dimension: texture.descriptor.dimension ?? '2d',
                  sampleCount: texture.descriptor.sampleCount ?? 1,
                  depthOrArrayLayers: size.depthOrArrayLayers
                },
                texture.viewDescriptor,
                textureType,
                debugSampler,
                debugCalls,
                stage
              );
              checkInfo = {
                runner: debugRunner,
                sampler: debugSampler,
                calls: debugCalls
              };
              haveComparisonCheckInfo = true;
            }
          }

          if (!gpuTexels && gpuTexture) {
            // Read the texture back if we haven't yet. We'll use this
            // to get values for each sample point.
            gpuTexels = await readTextureToTexelViews(
              t,
              gpuTexture,
              texture.descriptor,
              getTexelViewFormatForTextureFormat(gpuTexture.format)
            );
          }

          const callForSamplePoints = checkInfo.calls[callIdx];

          const expectedSamplePoints = [
          'expected:',
          ...(await identifySamplePoints(
            texture,
            sampler,
            callForSamplePoints,
            call,
            texture.texels,
            (texels) => {
              return Promise.resolve(
                softwareTextureRead(
                  t,
                  stage,
                  callForSamplePoints,
                  {
                    texels,
                    descriptor: texture.descriptor,
                    viewDescriptor: texture.viewDescriptor
                  },
                  checkInfo.sampler
                )
              );
            }
          ))];

          const gotSamplePoints = [
          'got:',
          ...(await identifySamplePoints(
            texture,
            sampler,
            callForSamplePoints,
            call,
            gpuTexels,
            async (texels) => {
              const gpuTexture = createTextureFromTexelViewsLocal(t, texels, texture.descriptor);
              const result = (await checkInfo.runner.run(gpuTexture))[callIdx];
              gpuTexture.destroy();
              return result;
            }
          ))];

          errs.push('  sample points:');
          errs.push(layoutTwoColumns(expectedSamplePoints, gotSamplePoints).join('\n'));
          errs.push('', '');
        }

        // this is not an else because it's common to comment out the previous `if` for running on a CQ.
        if (!t.rec.debugging) {
          errs.push('### turn on debugging to see sample points ###');
        }
      } // if (sampler)

      // Don't report the other errors. There 50 sample points per subcase and
      // 50-100 subcases so the log would get enormous if all 50 fail. One
      // report per subcase is enough.
      break;
    } // if (bad)
  } // for cellNdx

  results.runner.destroy();
  checkInfo.runner.destroy();

  return errs.length > 0 ? new Error(errs.join('\n')) : undefined;
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

  if (format.includes('depth')) {
    return 3 / 100;
  } else if (format.includes('8unorm')) {
    return 7 / 255;
  } else if (format.includes('2unorm')) {
    return 13 / 512;
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
function fillTextureWithRandomData(device, texture) {
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
  const id = `${viewDimension}:${texture.sampleCount}`;
  let pipeline = viewDimensionToPipelineMap.get(id);
  if (!pipeline) {
    let textureWGSL;
    let loadWGSL;
    let dimensionWGSL = 'textureDimensions(tex, uni.mipLevel)';
    switch (viewDimension) {
      case '2d':
        if (texture.sampleCount > 1) {
          textureWGSL = 'texture_multisampled_2d<f32>';
          loadWGSL = 'textureLoad(tex, coord.xy, sampleIndex)';
          dimensionWGSL = 'textureDimensions(tex)';
        } else {
          textureWGSL = 'texture_2d<f32>';
          loadWGSL = 'textureLoad(tex, coord.xy, mipLevel)';
        }
        break;
      case 'cube-array': // cube-array doesn't exist in compat so we can just use 2d_array for this
      case '2d-array':
        textureWGSL = 'texture_2d_array<f32>';
        loadWGSL = `
          textureLoad(
              tex,
              coord.xy,
              coord.z,
              mipLevel)`;
        break;
      case '3d':
        textureWGSL = 'texture_3d<f32>';
        loadWGSL = 'textureLoad(tex, coord.xyz, mipLevel)';
        break;
      case 'cube':
        textureWGSL = 'texture_cube<f32>';
        loadWGSL = `
          textureLoadCubeAs2DArray(tex, coord.xy, coord.z, mipLevel);
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

        struct Uniforms {
          mipLevel: u32,
          sampleCount: u32,
        };

        @group(0) @binding(0) var<uniform> uni: Uniforms;
        @group(0) @binding(1) var tex: ${textureWGSL};
        @group(0) @binding(2) var smp: sampler;
        @group(0) @binding(3) var<storage, read_write> data: array<vec4f>;

        @compute @workgroup_size(1) fn cs(
          @builtin(global_invocation_id) global_invocation_id : vec3<u32>) {
          _ = smp;
          let size = ${dimensionWGSL};
          let ndx = global_invocation_id.z * size.x * size.y * uni.sampleCount +
                    global_invocation_id.y * size.x * uni.sampleCount +
                    global_invocation_id.x;
          let coord = vec3u(global_invocation_id.x / uni.sampleCount, global_invocation_id.yz);
          let sampleIndex = global_invocation_id.x % uni.sampleCount;
          let mipLevel = uni.mipLevel;
          data[ndx] = ${loadWGSL};
        }
      `
    });
    pipeline = device.createComputePipeline({ layout: 'auto', compute: { module } });
    viewDimensionToPipelineMap.set(id, pipeline);
  }

  const encoder = device.createCommandEncoder();

  const readBuffers = [];
  for (let mipLevel = 0; mipLevel < texture.mipLevelCount; ++mipLevel) {
    const size = virtualMipSize(texture.dimension, texture, mipLevel);

    const uniformValues = new Uint32Array([mipLevel, texture.sampleCount, 0, 0]); // min size is 16 bytes
    const uniformBuffer = t.createBufferTracked({
      size: uniformValues.byteLength,
      usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST
    });
    device.queue.writeBuffer(uniformBuffer, 0, uniformValues);

    const storageBuffer = t.createBufferTracked({
      size: size[0] * size[1] * size[2] * 4 * 4 * texture.sampleCount, // rgba32float
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
    pass.dispatchWorkgroups(size[0] * texture.sampleCount, size[1], size[2]);
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

    const { sampleCount } = texture;
    texelViews.push(
      TexelView.fromTexelsAsColors(format, (coord) => {
        const offset =
        ((coord.z * size[0] * size[1] + coord.y * size[0] + coord.x) * sampleCount + (
        coord.sampleIndex ?? 0)) *
        4;
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

function createTextureFromTexelViewsLocal(
t,
texelViews,
desc)
{
  const modifiedDescriptor = { ...desc };
  // If it's a depth or stencil texture we need to render to it to fill it with data.
  if (isDepthOrStencilTextureFormat(texelViews[0].format)) {
    modifiedDescriptor.usage = desc.usage | GPUTextureUsage.RENDER_ATTACHMENT;
  }
  return createTextureFromTexelViews(t, texelViews, modifiedDescriptor);
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
descriptor,
options)
{
  if (isCompressedTextureFormat(descriptor.format)) {
    assert(!options, 'options not supported for compressed textures');
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
    const texels = createRandomTexelViewMipmap(descriptor, options);
    const texture = createTextureFromTexelViewsLocal(t, texels, descriptor);
    return { texture, texels };
  }
}

function valueIfAllComponentsAreEqual(
c,
componentOrder)
{
  const s = new Set(componentOrder.map((component) => c[component]));
  return s.size === 1 ? s.values().next().value : undefined;
}

/**
 * Creates a VideoFrame with random data and a TexelView with the same data.
 */
export function createVideoFrameWithRandomDataAndGetTexels(textureSize) {
  const size = reifyExtent3D(textureSize);
  assert(size.depthOrArrayLayers === 1);

  // Fill ImageData with random values.
  const imageData = new ImageData(size.width, size.height);
  const data = imageData.data;
  const asU32 = new Uint32Array(data.buffer);
  for (let i = 0; i < asU32.length; ++i) {
    asU32[i] = hashU32(i);
  }

  // Put the ImageData into a canvas and make a VideoFrame
  const canvas = new OffscreenCanvas(size.width, size.height);
  const ctx = canvas.getContext('2d');
  ctx.putImageData(imageData, 0, 0);
  const videoFrame = new VideoFrame(canvas, { timestamp: 0 });

  // Premultiply the ImageData
  for (let i = 0; i < data.length; i += 4) {
    const alpha = data[i + 3] / 255;
    data[i + 0] = data[i + 0] * alpha;
    data[i + 1] = data[i + 1] * alpha;
    data[i + 2] = data[i + 2] * alpha;
  }

  // Create a TexelView from the premultiplied ImageData
  const texels = [
  TexelView.fromTextureDataByReference('rgba8unorm', data, {
    bytesPerRow: size.width * 4,
    rowsPerImage: size.height,
    subrectOrigin: [0, 0, 0],
    subrectSize: size
  })];


  return { videoFrame, texels };
}

const kFaceNames = ['+x', '-x', '+y', '-y', '+z', '-z'];

/**
 * Generates a text art grid showing which texels were sampled
 * followed by a list of the samples and the weights used for each
 * component.
 *
 * It works by making a set of indices for every texel in the texture.
 * It splits the set into 2. It picks one set and generates texture data
 * using TexelView.fromTexelsAsColor with [1, 1, 1, 1] texels for members
 * of the current set.
 *
 * In then calls 'run' which renders a single `call`. `run` uses either
 * the software renderer or WebGPU. It then checks the results. If the
 * result is zero, all texels in the current had no influence when sampling
 * and can be discarded.
 *
 * If the result is > 0 then, if the set has more than one member, the
 * set is split and added to the list to sets to test. If the set only
 * had one member then the result is the weight used when sampling that texel.
 *
 * This lets you see if the weights from the software renderer match the
 * weights from WebGPU.
 *
 * Example:
 *
 *     0   1   2   3   4   5   6   7
 *   +---+---+---+---+---+---+---+---+
 * 0 |   |   |   |   |   |   |   |   |
 *   +---+---+---+---+---+---+---+---+
 * 1 |   |   |   |   |   |   |   | a |
 *   +---+---+---+---+---+---+---+---+
 * 2 |   |   |   |   |   |   |   | b |
 *   +---+---+---+---+---+---+---+---+
 * 3 |   |   |   |   |   |   |   |   |
 *   +---+---+---+---+---+---+---+---+
 * 4 |   |   |   |   |   |   |   |   |
 *   +---+---+---+---+---+---+---+---+
 * 5 |   |   |   |   |   |   |   |   |
 *   +---+---+---+---+---+---+---+---+
 * 6 |   |   |   |   |   |   |   |   |
 *   +---+---+---+---+---+---+---+---+
 * 7 |   |   |   |   |   |   |   |   |
 *   +---+---+---+---+---+---+---+---+
 * a: at: [7, 1], weights: [R: 0.75000]
 * b: at: [7, 2], weights: [R: 0.25000]
 */
async function identifySamplePoints(
texture,
sampler,
callForSamples,
originalCall,
texels,
run)
{
  const info = texture.descriptor;
  const isCube = isCubeViewDimension(texture.viewDescriptor);
  const mipLevelCount = texture.descriptor.mipLevelCount ?? 1;
  const mipLevelSize = range(mipLevelCount, (mipLevel) =>
  virtualMipSize(texture.descriptor.dimension ?? '2d', texture.descriptor.size, mipLevel)
  );
  const numTexelsPerLevel = mipLevelSize.map((size) => size.reduce((s, v) => s * v));
  const numTexelsOfPrecedingLevels = (() => {
    let total = 0;
    return numTexelsPerLevel.map((v) => {
      const num = total;
      total += v;
      return num;
    });
  })();
  const numTexels = numTexelsPerLevel.reduce((sum, v) => sum + v);

  const getMipLevelFromTexelId = (texelId) => {
    for (let mipLevel = mipLevelCount - 1; mipLevel > 0; --mipLevel) {
      if (texelId - numTexelsOfPrecedingLevels[mipLevel] >= 0) {
        return mipLevel;
      }
    }
    return 0;
  };

  const getTexelCoordFromTexelId = (texelId) => {
    const mipLevel = getMipLevelFromTexelId(texelId);
    const size = mipLevelSize[mipLevel];
    const texelsPerSlice = size[0] * size[1];
    const id = texelId - numTexelsOfPrecedingLevels[mipLevel];
    const layer = Math.floor(id / texelsPerSlice);
    const xyId = id - layer * texelsPerSlice;
    const y = xyId / size[0] | 0;
    const x = xyId % size[0];
    return { x, y, z: layer, mipLevel, xyId };
  };

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

  const components = isBuiltinGather(callForSamples.builtin) ? kRGBAComponents : rep.componentOrder;
  const convertResultAsAppropriate = isBuiltinGather(callForSamples.builtin) ?
  (v) => v :
  convertResultFormatToTexelViewFormat;

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

    // See if any of the texels in setA were sampled.0
    const results = convertResultAsAppropriate(
      await run(
        range(mipLevelCount, (mipLevel) =>
        TexelView.fromTexelsAsColors(
          format,
          (coords) => {
            const size = mipLevelSize[mipLevel];
            const texelsPerSlice = size[0] * size[1];
            const texelsPerRow = size[0];
            const texelId =
            numTexelsOfPrecedingLevels[mipLevel] +
            coords.x +
            coords.y * texelsPerRow +
            coords.z * texelsPerSlice;
            const isCandidate = setA.has(texelId);
            const texel = {};
            for (const component of rep.componentOrder) {
              texel[component] = isCandidate ? 1 : 0;
            }
            return texel;
          }
        )
        )
      ),
      format
    );
    if (components.some((c) => results[c] !== 0)) {
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

  // separate the sampledTexelWeights by mipLevel, then by layer, within a layer the texelId only includes x and y
  const levels = [];
  for (const [texelId, weight] of sampledTexelWeights.entries()) {
    const { xyId, z, mipLevel } = getTexelCoordFromTexelId(texelId);
    const level = levels[mipLevel] ?? [];
    levels[mipLevel] = level;
    const layerEntries = level[z] ?? new Map();
    level[z] = layerEntries;
    layerEntries.set(xyId, weight);
  }

  // +---+---+---+---+
  // | a |   |   |   |
  // +---+---+---+---+
  // |   |   |   |   |
  // +---+---+---+---+
  // |   |   |   |   |
  // +---+---+---+---+
  // |   |   |   | b |
  // +---+---+---+---+
  const lines = [];
  const letter = (idx) => String.fromCodePoint(idx < 30 ? 97 + idx : idx + 9600 - 30); // 97: 'a'
  let idCount = 0;

  for (let mipLevel = 0; mipLevel < mipLevelCount; ++mipLevel) {
    const level = levels[mipLevel];
    if (!level) {
      continue;
    }

    const [width, height, depthOrArrayLayers] = mipLevelSize[mipLevel];
    const texelsPerRow = width;

    for (let layer = 0; layer < depthOrArrayLayers; ++layer) {
      const layerEntries = level[layer];

      const orderedTexelIndices = [];
      lines.push('');
      const unSampled = layerEntries ? '' : 'un-sampled';
      if (isCube) {
        const face = kFaceNames[layer % 6];
        lines.push(`layer: ${layer}, cube-layer: ${layer / 6 | 0} (${face}) ${unSampled}`);
      } else {
        lines.push(`layer: ${layer} ${unSampled}`);
      }

      if (!layerEntries) {
        continue;
      }

      {
        let line = '  ';
        for (let x = 0; x < width; x++) {
          line += `  ${x.toString().padEnd(2)}`;
        }
        lines.push(line);
      }
      {
        let line = '  +';
        for (let x = 0; x < width; x++) {
          line += x === width - 1 ? '---+' : '---+';
        }
        lines.push(line);
      }
      for (let y = 0; y < height; y++) {
        {
          let line = `${y.toString().padEnd(2)}|`;
          for (let x = 0; x < width; x++) {
            const texelIdx = x + y * texelsPerRow;
            const weight = layerEntries.get(texelIdx);
            if (weight !== undefined) {
              line += ` ${letter(idCount + orderedTexelIndices.length)} |`;
              orderedTexelIndices.push(texelIdx);
            } else {
              line += '   |';
            }
          }
          lines.push(line);
        }
        if (y < height - 1) {
          let line = '  +';
          for (let x = 0; x < width; x++) {
            line += x === width - 1 ? '---+' : '---+';
          }
          lines.push(line);
        }
      }
      {
        let line = '  +';
        for (let x = 0; x < width; x++) {
          line += x === width - 1 ? '---+' : '---+';
        }
        lines.push(line);
      }

      const pad2 = (n) => n.toString().padStart(2);
      const fix5 = (n) => n.toFixed(5);
      const formatTexel = (texel) =>
      texel ?
      Object.entries(texel).
      map(([k, v]) => `${k}: ${fix5(v)}`).
      join(', ') :
      '*texel values unavailable*';

      const colorLines = [];
      const compareLines = [];
      let levelWeight = 0;
      orderedTexelIndices.forEach((texelIdx, i) => {
        const weights = layerEntries.get(texelIdx);
        const y = Math.floor(texelIdx / texelsPerRow);
        const x = texelIdx % texelsPerRow;
        const singleWeight = valueIfAllComponentsAreEqual(weights, components);
        levelWeight += singleWeight;
        const w =
        singleWeight !== undefined ?
        `weight: ${fix5(singleWeight)}` :
        `weights: [${components.map((c) => `${c}: ${fix5(weights[c])}`).join(', ')}]`;
        const coord = `${pad2(x)}, ${pad2(y)}, ${pad2(layer)}`;
        const texel =
        texels &&
        convertToTexelViewFormat(
          texels[mipLevel].color({ x, y, z: layer }),
          texture.descriptor.format
        );

        const texelStr = formatTexel(texel);
        const id = letter(idCount + i);
        lines.push(`${id}: mip(${mipLevel}) at: [${coord}], ${w}`);
        colorLines.push(`${id}: value: ${texelStr}`);
        if (isBuiltinComparison(originalCall.builtin)) {
          assert(!!texel);
          const compareTexel = applyCompare(originalCall, sampler, [TexelComponent.Depth], texel);
          compareLines.push(
            `${id}: compare(${sampler.compare}) result with depthRef(${fix5(
              originalCall.depthRef
            )}): ${fix5(compareTexel.Depth)}`
          );
        }
      });
      lines.push(...colorLines);
      lines.push(...compareLines);
      if (!isNaN(levelWeight)) {
        lines.push(`level weight: ${fix5(levelWeight)}`);
      }
      idCount += orderedTexelIndices.length;
    }
  }

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

/**
 * Returns the number of layers ot test for a given view dimension
 */
export function getDepthOrArrayLayersForViewDimension(viewDimension) {
  switch (viewDimension) {
    case '1d':
      return 1;
    case undefined:
    case '2d':
      return 1;
    case '2d-array':
      return 4;
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
  const height =
  viewDimension === '1d' ? 1 : align(Math.max(minSize, blockHeight * minBlocks), blockHeight);
  if (viewDimension === 'cube' || viewDimension === 'cube-array') {
    const blockLCM = lcm(blockWidth, blockHeight);
    const largest = Math.max(width, height);
    const size = align(largest, blockLCM);
    return [size, size, viewDimension === 'cube-array' ? 24 : 6];
  }
  const depthOrArrayLayers = getDepthOrArrayLayersForViewDimension(viewDimension);
  return [width, height, depthOrArrayLayers];
}

export const kSamplePointMethods = ['texel-centre', 'spiral'];


export const kCubeSamplePointMethods = ['cube-edges', 'texel-centre', 'spiral'];


















/**
 * Generates an array of coordinates at which to sample a texture.
 */
function generateTextureBuiltinInputsImpl(
makeValue,
n,
args)




















{
  const { method, descriptor } = args;
  const dimension = descriptor.dimension ?? '2d';
  const mipLevelCount = descriptor.mipLevelCount ?? 1;
  const size = virtualMipSize(dimension, descriptor.size, 0);
  const coords = [];
  switch (method) {
    case 'texel-centre':{
        for (let i = 0; i < n; i++) {
          const r = hashU32(i);
          const x = Math.floor(lerp(0, size[0] - 1, (r & 0xff) / 0xff)) + 0.5;
          const y = Math.floor(lerp(0, size[1] - 1, (r >> 8 & 0xff) / 0xff)) + 0.5;
          const z = Math.floor(lerp(0, size[2] - 1, (r >> 16 & 0xff) / 0xff)) + 0.5;
          coords.push(makeValue(x / size[0], y / size[1], z / size[2]));
        }
        break;
      }
    case 'spiral':{
        const { radius = 1.5, loops = 2 } = args;
        for (let i = 0; i < n; i++) {
          const f = i / (Math.max(n, 2) - 1);
          const r = radius * f;
          const a = loops * 2 * Math.PI * f;
          coords.push(makeValue(0.5 + r * Math.cos(a), 0.5 + r * Math.sin(a), 0));
        }
        break;
      }
  }

  const _hashInputs = args.hashInputs.map((v) =>
  typeof v === 'string' ? sumOfCharCodesOfString(v) : typeof v === 'boolean' ? v ? 1 : 0 : v
  );

  // returns a number between [0 and N)
  const makeRandValue = ({ num, type }, ...hashInputs) => {
    const range = num;
    const number = hashU32(..._hashInputs, ...hashInputs) / 0x1_0000_0000 * range;
    return type === 'f32' ? number : Math.floor(number);
  };

  // for signed and float values returns [-1 to num]
  // for unsigned values returns [0 to num]
  const makeRangeValue = ({ num, type }, ...hashInputs) => {
    const range = num + (type === 'u32' ? 1 : 2);
    const number =
    hashU32(..._hashInputs, ...hashInputs) / 0x1_0000_0000 * range - (type === 'u32' ? 0 : 1);
    return type === 'f32' ? number : Math.floor(number);
  };

  // Generates the same values per coord instead of using all the extra `_hashInputs`.
  const makeIntHashValueRepeatable = (min, max, ...hashInputs) => {
    const range = max - min;
    return min + Math.floor(hashU32(...hashInputs) / 0x1_0000_0000 * range);
  };

  // Samplers across devices use different methods to interpolate.
  // Quantizing the texture coordinates seems to hit coords that produce
  // comparable results to our computed results.
  // Note: This value works with 8x8 textures. Other sizes have not been tested.
  // Values that worked for reference:
  // Win 11, NVidia 2070 Super: 16
  // Linux, AMD Radeon Pro WX 3200: 256
  // MacOS, M1 Mac: 256
  const kSubdivisionsPerTexel = 4;

  // When filtering is nearest then we want to avoid edges of texels
  //
  //             U
  //             |
  //     +---+---+---+---+---+---+---+---+
  //     |   | A | B |   |   |   |   |   |
  //     +---+---+---+---+---+---+---+---+
  //
  // Above, coordinate U could sample either A or B
  //
  //               U
  //               |
  //     +---+---+---+---+---+---+---+---+
  //     |   | A | B | C |   |   |   |   |
  //     +---+---+---+---+---+---+---+---+
  //
  // For textureGather we want to avoid texel centers
  // as for coordinate U could either gather A,B or B,C.

  const avoidEdgeCase =
  !args.sampler || args.sampler.minFilter === 'nearest' || isBuiltinGather(args.textureBuiltin);
  const edgeRemainder = isBuiltinGather(args.textureBuiltin) ? kSubdivisionsPerTexel / 2 : 0;

  // textureGather issues for 2d/3d textures
  //
  // If addressModeU is repeat, then on an 8x1 texture, u = 0.01 or u = 0.99
  // would gather these texels
  //
  //     +---+---+---+---+---+---+---+---+
  //     | * |   |   |   |   |   |   | * |
  //     +---+---+---+---+---+---+---+---+
  //
  // If addressModeU is clamp-to-edge or mirror-repeat,
  // then on an 8x1 texture, u = 0.01 would gather this texel
  //
  //     +---+---+---+---+---+---+---+---+
  //     | * |   |   |   |   |   |   |   |
  //     +---+---+---+---+---+---+---+---+
  //
  // and 0.99 would gather this texel
  //
  //     +---+---+---+---+---+---+---+---+
  //     |   |   |   |   |   |   |   | * |
  //     +---+---+---+---+---+---+---+---+
  //
  // This means we have to if addressMode is not `repeat`, we
  // need to avoid the edge of the texture.
  //
  // Note: we don't have these specific issues with cube maps
  // as they ignore addressMode
  const euclideanModulo = (n, m) => (n % m + m) % m;
  const addressMode =
  args.textureBuiltin === 'textureSampleBaseClampToEdge' ?
  ['clamp-to-edge', 'clamp-to-edge', 'clamp-to-edge'] :
  [
  args.sampler?.addressModeU ?? 'clamp-to-edge',
  args.sampler?.addressModeV ?? 'clamp-to-edge',
  args.sampler?.addressModeW ?? 'clamp-to-edge'];

  const avoidTextureEdge = (axis, textureDimensionUnits, v) => {
    assert(isBuiltinGather(args.textureBuiltin));
    if (addressMode[axis] === 'repeat') {
      return v;
    }
    const inside = euclideanModulo(v, textureDimensionUnits);
    const outside = v - inside;
    return outside + clamp(inside, { min: 1, max: textureDimensionUnits - 1 });
  };

  const numComponents = isDepthOrStencilTextureFormat(descriptor.format) ? 1 : 4;
  return coords.map((c, i) => {
    const mipLevel = args.mipLevel ?
    quantizeMipLevel(makeRangeValue(args.mipLevel, i), args.sampler?.mipmapFilter ?? 'nearest') :
    0;
    const clampedMipLevel = clamp(mipLevel, { min: 0, max: mipLevelCount - 1 });
    const mipSize = virtualMipSize(dimension, size, clampedMipLevel);
    const q = mipSize.map((v) => v * kSubdivisionsPerTexel);

    const coords = c.map((v, i) => {
      // Quantize to kSubdivisionsPerPixel
      const v1 = Math.floor(v * q[i]);
      // If it's nearest or textureGather and we're on the edge of a texel then move us off the edge
      // since the edge could choose one texel or another.
      const isTexelEdgeCase = Math.abs(v1 % kSubdivisionsPerTexel) === edgeRemainder;
      const v2 = isTexelEdgeCase && avoidEdgeCase ? v1 + 1 : v1;
      const v3 = isBuiltinGather(args.textureBuiltin) ? avoidTextureEdge(i, q[i], v2) : v2;
      // Convert back to texture coords
      return v3 / q[i];
    });

    const makeGradient = (hashInput) => {
      return coords.map((_, i) => {
        // a value between -4 and 4 integer then add +/- 0.25
        // We want to be able to choose levels but we want to avoid the area where the
        // gpu might choose 2 different levels than the software renderer.
        const intPart = makeRangeValue({ num: 8, type: 'u32' }, i, hashInput) - 4;
        const fractPart = makeRangeValue({ num: 0, type: 'f32' }, i, hashInput + 1) * 0.25;
        assert(fractPart >= -0.25 && fractPart <= 0.25);
        return intPart + fractPart;
      });
    };

    // choose a derivative value that will select a mipLevel.
    const makeDerivativeMult = (coords, mipLevel) => {
      // Make an identity vec (all 1s).
      const mult = new Array(coords.length).fill(0);
      // choose one axis to set
      const ndx = makeRangeValue({ num: coords.length - 1, type: 'u32' }, i, 8);
      assert(ndx < coords.length);
      mult[ndx] = Math.pow(2, mipLevel);
      return mult;
    };

    // Choose a mip level. If mipmapFilter is 'nearest' then avoid centers of levels
    // else avoid edges.
    const chooseMipLevel = () => {
      const innerLevelR = makeRandValue({ num: 9, type: 'u32' }, i, 11);
      const innerLevel =
      args?.sampler?.mipmapFilter === 'linear' ?
      innerLevelR + 1 :
      innerLevelR < 5 ?
      innerLevelR :
      innerLevelR + 1;
      const outerLevel = makeRangeValue({ num: mipLevelCount - 1, type: 'i32' }, i, 11);
      return outerLevel + innerLevel / 10;
    };

    // for textureSample, choose a derivative value that will select a mipLevel near
    // the range of mip levels.
    const makeDerivativeMultForTextureSample = (coords) => {
      const mipLevel = chooseMipLevel();
      return makeDerivativeMult(coords, mipLevel);
    };

    // for textureSampleBias we choose a mipLevel we want to sample, then a bias between -17 and 17.
    // and then a derivative that, given the chosen bias will arrive at the chosen mipLevel.
    // The GPU is supposed to clamp between -16.0 and 15.99.
    const makeBiasAndDerivativeMult = (coords) => {
      const mipLevel = chooseMipLevel();
      const bias = makeRangeValue({ num: 34, type: 'f32' }, i, 9) - 17;
      const clampedBias = clamp(bias, { min: -16, max: 15.99 });
      const derivativeBasedMipLevel = mipLevel - clampedBias;
      const derivativeMult = makeDerivativeMult(coords, derivativeBasedMipLevel);
      return [bias, derivativeMult];
    };

    // If bias is set this is textureSampleBias. If bias is not set but derivatives
    // is then this is one of the other functions that needs implicit derivatives.
    const [bias, derivativeMult] = args.bias ?
    makeBiasAndDerivativeMult(coords) :
    args.derivatives ?
    [undefined, makeDerivativeMultForTextureSample(coords)] :
    [];

    return {
      coords,
      derivativeMult,
      mipLevel,
      sampleIndex: args.sampleIndex ? makeRangeValue(args.sampleIndex, i, 1) : undefined,
      arrayIndex: args.arrayIndex ? makeRangeValue(args.arrayIndex, i, 2) : undefined,
      // use 0.0, 0.5, or 1.0 for depthRef. We can't test for equality except for values 0 and 1
      // The texture will be filled with random values unless our comparison is 'equal' or 'not-equal'
      // in which case the texture will be filled with only 0, 0.6, 1. Choosing 0.0, 0.5, 1.0 here
      // means we can test 'equal' and 'not-equal'. For other comparisons, the fact that the texture's
      // contents is random seems enough to test all the comparison modes.
      depthRef: args.depthRef ? makeRandValue({ num: 3, type: 'u32' }, i, 5) / 2 : undefined,
      ddx: args.grad ? makeGradient(7) : undefined,
      ddy: args.grad ? makeGradient(8) : undefined,
      bias,
      offset: args.offset ?
      coords.map((_, j) => makeIntHashValueRepeatable(-8, 8, i, 3 + j)) :
      undefined,
      component: args.component ? makeIntHashValueRepeatable(0, numComponents, i, 4) : undefined
    };
  });
}

/**
 * When mipmapFilter === 'nearest' we need to stay away from 0.5
 * because the GPU could decide to choose one mip or the other.
 *
 * Some example transition values, the value at which the GPU chooses
 * mip level 1 over mip level 0:
 *
 * M1 Mac: 0.515381
 * Intel Mac: 0.49999
 * AMD Mac: 0.5
 */
const kMipEpsilon = 0.02;
function quantizeMipLevel(mipLevel, mipmapFilter) {
  if (mipmapFilter === 'linear') {
    return mipLevel;
  }
  const intMip = Math.floor(mipLevel);
  const fractionalMip = mipLevel - intMip;
  if (fractionalMip < 0.5 - kMipEpsilon || fractionalMip > 0.5 + kMipEpsilon) {
    return mipLevel;
  } else {
    return intMip + 0.5 + (fractionalMip < 0.5 ? -kMipEpsilon : +kMipEpsilon);
  }
}

// Removes the first element from an array of types






export function generateTextureBuiltinInputs1D(...args) {
  return generateTextureBuiltinInputsImpl((x) => [x], ...args);
}

export function generateTextureBuiltinInputs2D(...args) {
  return generateTextureBuiltinInputsImpl((x, y) => [x, y], ...args);
}

export function generateTextureBuiltinInputs3D(...args) {
  return generateTextureBuiltinInputsImpl(
    (x, y, z) => [x, y, z],
    ...args
  );
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
 * Wrap a texel based face coord across cube faces
 *
 * We have a face texture in texels coord where U/V choose a texel and W chooses the face.
 * If U/V are outside the size of the texture then, when normalized and converted
 * to a cube map coordinate, they'll end up pointing to a different face.
 *
 * addressMode is effectively ignored for cube
 *
 * By converting from a texel based coord to a normalized coord and then to a cube map coord,
 * if the texel was outside of the face, the cube map coord will end up pointing to a different
 * face. We then convert back cube coord -> normalized face coord -> texel based coord
 */
function wrapFaceCoordToCubeFaceAtEdgeBoundaries(textureSize, faceCoord) {
  // convert texel based face coord to normalized 2d-array coord
  const nc0 = [
  (faceCoord[0] + 0.5) / textureSize,
  (faceCoord[1] + 0.5) / textureSize,
  (faceCoord[2] + 0.5) / 6];

  const cc = convertNormalized3DTexCoordToCubeCoord(nc0);
  const nc1 = convertCubeCoordToNormalized3DTextureCoord(cc);
  // convert normalized 2d-array coord back texel based face coord
  const fc = [
  Math.floor(nc1[0] * textureSize),
  Math.floor(nc1[1] * textureSize),
  Math.floor(nc1[2] * 6)];


  return fc;
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
args)






















{
  const { method, descriptor } = args;
  const mipLevelCount = descriptor.mipLevelCount ?? 1;
  const size = virtualMipSize('2d', descriptor.size, 0);
  const textureWidth = size[0];
  const coords = [];
  switch (method) {
    case 'texel-centre':{
        for (let i = 0; i < n; i++) {
          const r = hashU32(i);
          const u = (Math.floor(lerp(0, textureWidth - 1, (r & 0xff) / 0xff)) + 0.5) / textureWidth;
          const v =
          (Math.floor(lerp(0, textureWidth - 1, (r >> 8 & 0xff) / 0xff)) + 0.5) / textureWidth;
          const face = Math.floor(lerp(0, 6, (r >> 16 & 0xff) / 0x100));
          coords.push(convertNormalized3DTexCoordToCubeCoord([u, v, face]));
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
          coords.push([ux * r, uy * r, uz * r]);
        }
        break;
      }
    case 'cube-edges':{

        coords.push(
          // between edges
          // +x
          [1, -1.01, 0], // wrap -y
          [1, +1.01, 0], // wrap +y
          [1, 0, -1.01], // wrap -z
          [1, 0, +1.01], // wrap +z
          // -x
          [-1, -1.01, 0], // wrap -y
          [-1, +1.01, 0], // wrap +y
          [-1, 0, -1.01], // wrap -z
          [-1, 0, +1.01], // wrap +z

          // +y
          [-1.01, 1, 0], // wrap -x
          [+1.01, 1, 0], // wrap +x
          [0, 1, -1.01], // wrap -z
          [0, 1, +1.01], // wrap +z
          // -y
          [-1.01, -1, 0], // wrap -x
          [+1.01, -1, 0], // wrap +x
          [0, -1, -1.01], // wrap -z
          [0, -1, +1.01], // wrap +z

          // +z
          [-1.01, 0, 1], // wrap -x
          [+1.01, 0, 1], // wrap +x
          [0, -1.01, 1], // wrap -y
          [0, +1.01, 1], // wrap +y
          // -z
          [-1.01, 0, -1], // wrap -x
          [+1.01, 0, -1], // wrap +x
          [0, -1.01, -1], // wrap -y
          [0, +1.01, -1] // wrap +y

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

  const _hashInputs = args.hashInputs.map((v) =>
  typeof v === 'string' ? sumOfCharCodesOfString(v) : typeof v === 'boolean' ? v ? 1 : 0 : v
  );

  // returns a number between [0 and N)
  const makeRandValue = ({ num, type }, ...hashInputs) => {
    const range = num;
    const number = hashU32(..._hashInputs, ...hashInputs) / 0x1_0000_0000 * range;
    return type === 'f32' ? number : Math.floor(number);
  };

  // for signed and float values returns [-1 to num]
  // for unsigned values returns [0 to num]
  const makeRangeValue = ({ num, type }, ...hashInputs) => {
    const range = num + (type === 'u32' ? 1 : 2);
    const number =
    hashU32(..._hashInputs, ...hashInputs) / 0x1_0000_0000 * range - (type === 'u32' ? 0 : 1);
    return type === 'f32' ? number : Math.floor(number);
  };

  const makeIntHashValue = (min, max, ...hashInputs) => {
    const range = max - min;
    return min + Math.floor(hashU32(..._hashInputs, ...hashInputs) / 0x1_0000_0000 * range);
  };

  // Samplers across devices use different methods to interpolate.
  // Quantizing the texture coordinates seems to hit coords that produce
  // comparable results to our computed results.
  // Note: This value works with 8x8 textures. Other sizes have not been tested.
  // Values that worked for reference:
  // Win 11, NVidia 2070 Super: 16
  // Linux, AMD Radeon Pro WX 3200: 256
  // MacOS, M1 Mac: 256
  //
  // Note: When doing `textureGather...` we can't use texel centers
  // because which 4 pixels will be gathered jumps if we're slightly under
  // or slightly over the center
  //
  // Similarly, if we're using 'nearest' filtering then we don't want texel
  // edges for the same reason.
  //
  // Also note that for textureGather. The way it works for cube maps is to
  // first convert from cube map coordinate to a 2D texture coordinate and
  // a face. Then, choose 4 texels just like normal 2D texture coordinates.
  // If one of the 4 texels is outside the current face, wrap it to the correct
  // face.
  //
  // An issue this brings up though. Imagine a 2D texture with addressMode = 'repeat'
  //
  //       2d texture   (same texture repeated to show 'repeat')
  //     ┌───┬───┬───┐     ┌───┬───┬───┐
  //     │   │   │   │     │   │   │   │
  //     ├───┼───┼───┤     ├───┼───┼───┤
  //     │   │   │  a│     │c  │   │   │
  //     ├───┼───┼───┤     ├───┼───┼───┤
  //     │   │   │  b│     │d  │   │   │
  //     └───┴───┴───┘     └───┴───┴───┘
  //
  // Assume the texture coordinate is at the bottom right corner of a.
  // Then textureGather will grab c, d, b, a (no idea why that order).
  // but think of it as top-right, bottom-right, bottom-left, top-left.
  // Similarly, if the texture coordinate is at the top left of d it
  // will select the same 4 texels.
  //
  // But, in the case of a cubemap, each face is in different direction
  // relative to the face next to it.
  //
  //             +-----------+
  //             |0->u       |
  //             |↓          |
  //             |v   +y     |
  //             |    (2)    |
  //             |           |
  // +-----------+-----------+-----------+-----------+
  // |0->u       |0->u       |0->u       |0->u       |
  // |↓          |↓          |↓          |↓          |
  // |v   -x     |v   +z     |v   +x     |v   -z     |
  // |    (1)    |    (4)    |    (0)    |    (5)    |
  // |           |           |           |           |
  // +-----------+-----------+-----------+-----------+
  //             |0->u       |
  //             |↓          |
  //             |v   -y     |
  //             |    (3)    |
  //             |           |
  //             +-----------+
  //
  // As an example, imagine going from the +y to the +x face.
  // See diagram above, the right edge of the +y face wraps
  // to the top edge of the +x face.
  //
  //                             +---+---+
  //                             |  a|c  |
  //     ┌───┬───┬───┐           ┌───┬───┬───┐
  //     │   │   │   │           │  b│d  │   │
  //     ├───┼───┼───┤---+       ├───┼───┼───┤
  //     │   │   │  a│ c |       │   │   │   │
  //     ├───┼───┼───┤---+       ├───┼───┼───┤
  //     │   │   │  b│ d |       │   │   │   │
  //     └───┴───┴───┘---+       └───┴───┴───┘
  //        +y face                 +x face
  //
  // If the texture coordinate is in the bottom right corner of a,
  // the rectangle of texels we read are a,b,c,d and, if we the
  // texture coordinate is in the top left corner of d we also
  // read a,b,c,d according to the 2 diagrams above.
  //
  // But, notice that when reading from the POV of +y vs +x,
  // which actual a,b,c,d texels are different.
  //
  // From the POV of face +x: a,b are in face +x and c,d are in face +y
  // From the POV of face +y: a,c are in face +x and b,d are in face +y
  //
  // This is all the long way of saying that if we're on the edge of a cube
  // face we could get drastically different results because the orientation
  // of the rectangle of the 4 texels we use, rotates. So, we need to avoid
  // any values too close to the edge just in case our math is different than
  // the GPU's.
  //
  const kSubdivisionsPerTexel = 4;
  const avoidEdgeCase =
  !args.sampler || args.sampler.minFilter === 'nearest' || isBuiltinGather(args.textureBuiltin);
  const edgeRemainder = isBuiltinGather(args.textureBuiltin) ? kSubdivisionsPerTexel / 2 : 0;
  return coords.map((c, i) => {
    const mipLevel = args.mipLevel ?
    quantizeMipLevel(makeRangeValue(args.mipLevel, i), args.sampler?.mipmapFilter ?? 'nearest') :
    0;
    const clampedMipLevel = clamp(mipLevel, { min: 0, max: mipLevelCount - 1 });
    const mipSize = virtualMipSize('2d', size, Math.ceil(clampedMipLevel));
    const q = [
    mipSize[0] * kSubdivisionsPerTexel,
    mipSize[0] * kSubdivisionsPerTexel,
    6 * kSubdivisionsPerTexel];


    const uvw = convertCubeCoordToNormalized3DTextureCoord(c);

    // If this is a corner, move to in so it's not
    // (see comment "Issues with corners of cubemaps")
    const ndx = getUnusedCubeCornerSampleIndex(mipSize[0], uvw);
    if (ndx >= 0) {
      const halfTexel = 0.5 / mipSize[0];
      uvw[0] = clamp(uvw[0], { min: halfTexel, max: 1 - halfTexel });
    }

    const quantizedUVW = uvw.map((v, i) => {
      // Quantize to kSubdivisionsPerPixel
      const v1 = Math.floor(v * q[i]);
      // If it's nearest or textureGather and we're on the edge of a texel then move us off the edge
      // since the edge could choose one texel or another.
      const isEdgeCase = Math.abs(v1 % kSubdivisionsPerTexel) === edgeRemainder;
      const v2 = isEdgeCase && avoidEdgeCase ? v1 + 1 : v1;
      // Convert back to texture coords slightly off
      return (v2 + 1 / 16) / q[i];
    });

    const quantize = (v, units) => Math.floor(v * units) * units;

    const makeGradient = (hashInput) => {
      return coords.map((_, i) =>
      // a value between -4 and 4, quantized to 1/3rd.
      quantize(makeRangeValue({ num: 8, type: 'f32' }, i, hashInput) - 4, 1 / 3)
      );
    };

    const coords = convertNormalized3DTexCoordToCubeCoord(quantizedUVW);

    // choose a derivative value that will select a mipLevel.
    const makeDerivativeMult = (coords, mipLevel) => {
      // Make an identity vec (all 1s).
      const mult = new Array(coords.length).fill(0);
      // choose one axis to set
      const ndx = makeRangeValue({ num: coords.length - 1, type: 'u32' }, i, 8);
      assert(ndx < coords.length);
      mult[ndx] = Math.pow(2, mipLevel);
      return mult;
    };

    // Choose a mip level. If mipmapFilter is 'nearest' then avoid centers of levels
    // else avoid edges.
    const chooseMipLevel = () => {
      const innerLevelR = makeRandValue({ num: 9, type: 'u32' }, i, 11);
      const innerLevel =
      args?.sampler?.mipmapFilter === 'linear' ?
      innerLevelR + 1 :
      innerLevelR < 4 ?
      innerLevelR :
      innerLevelR + 1;
      const outerLevel = makeRangeValue({ num: mipLevelCount - 1, type: 'i32' }, i, 11);
      return outerLevel + innerLevel / 10;
    };

    // for textureSample, choose a derivative value that will select a mipLevel near
    // the range of mip levels.
    const makeDerivativeMultForTextureSample = (coords) => {
      const mipLevel = chooseMipLevel();
      return makeDerivativeMult(coords, mipLevel);
    };

    // for textureSampleBias we choose a mipLevel we want to sample, then a bias between -17 and 17.
    // and then a derivative that, given the chosen bias will arrive at the chosen mipLevel.
    // The GPU is supposed to clamp between -16.0 and 15.99.
    const makeBiasAndDerivativeMult = (coords) => {
      const mipLevel = chooseMipLevel();
      const bias = makeRangeValue({ num: 34, type: 'f32' }, i, 9) - 17;
      const clampedBias = clamp(bias, { min: -16, max: 15.99 });
      const derivativeBasedMipLevel = mipLevel - clampedBias;
      const derivativeMult = makeDerivativeMult(coords, derivativeBasedMipLevel);
      return [bias, derivativeMult];
    };

    // If bias is set this is textureSampleBias. If bias is not set but derivatives
    // is then this is one of the other functions that needs implicit derivatives.
    const [bias, derivativeMult] = args.bias ?
    makeBiasAndDerivativeMult(coords) :
    args.derivatives ?
    [undefined, makeDerivativeMultForTextureSample(coords)] :
    [];

    return {
      coords,
      derivativeMult,
      ddx: args.grad ? makeGradient(7) : undefined,
      ddy: args.grad ? makeGradient(8) : undefined,
      mipLevel,
      arrayIndex: args.arrayIndex ? makeRangeValue(args.arrayIndex, i, 2) : undefined,
      bias,
      // use 0.0, 0.5, or 1.0 for depthRef. We can't test for equality except for values 0 and 1
      // The texture will be filled with random values unless our comparison is 'equal' or 'not-equal'
      // in which case the texture will be filled with only 0, 0.6, 1. Choosing 0.0, 0.5, 1.0 here
      // means we can test 'equal' and 'not-equal'. For other comparisons, the fact that the texture's
      // contents is random seems enough to test all the comparison modes.
      depthRef: args.depthRef ? makeRandValue({ num: 3, type: 'u32' }, i, 5) / 2 : undefined,
      component: args.component ? makeIntHashValue(0, 4, i, 4) : undefined
    };
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

function wgslExpr(
data)
{
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
      if (name === 'offset' || name === 'component') {
        // offset and component must be constant expressions
        keys.push(`${name}: ${wgslExpr(value)}`);
      } else {
        keys.push(`${name}: ${wgslTypeFor(value, call.coordType)}`);
      }
    }
  }
  return `${call.builtin}(${keys.join(', ')})`;
}

function buildBinnedCalls(calls) {
  const args = [];
  const fields = [];
  const data = [];
  const prototype = calls[0];

  if (isBuiltinGather(prototype.builtin) && prototype['componentType']) {
    args.push(`/* component */ ${wgslExpr(prototype['component'])}`);
  }

  // All texture builtins take a Texture
  args.push('T');

  if (builtinNeedsSampler(prototype.builtin)) {
    // textureSample*() builtins take a sampler as the second argument
    args.push('S');
  }

  for (const name of kTextureCallArgNames) {
    const value = prototype[name];
    if (value !== undefined) {
      if (name === 'offset') {
        args.push(`/* offset */ ${wgslExpr(value)}`);
      } else if (name === 'component') {

        // was handled above
      } else {const type =
        name === 'mipLevel' ?
        prototype.levelType :
        name === 'arrayIndex' ?
        prototype.arrayIndexType :
        name === 'sampleIndex' ?
        prototype.sampleIndexType :
        name === 'bias' || name === 'depthRef' || name === 'ddx' || name === 'ddy' ?
        'f' :
        prototype.coordType;
        if (name !== 'derivativeMult') {
          args.push(
            `args.${name}${
            name === 'coords' && builtinNeedsDerivatives(prototype.builtin) ?
            ' + derivativeBase * args.derivativeMult' :
            ''
            }`
          );
        }
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
      if (value !== undefined && name !== 'offset' && name !== 'component') {
        const type = getCallArgType(call, name);
        const bitcastToU32 = kBitCastFunctions[type];
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

function describeTextureCall(call) {
  const args = [];
  if (isBuiltinGather(call.builtin) && call.componentType) {
    args.push(`component: ${wgslExprFor(call.component, call.componentType)}`);
  }
  args.push('texture: T');
  if (builtinNeedsSampler(call.builtin)) {
    args.push('sampler: S');
  }
  for (const name of kTextureCallArgNames) {
    const value = call[name];
    if (value !== undefined && name !== 'component') {
      if (name === 'coords') {
        const derivativeWGSL = builtinNeedsDerivatives(call.builtin) ?
        ` + derivativeBase * derivativeMult(${
        call.derivativeMult ? wgslExprFor(call.derivativeMult, call.coordType) : '1'
        })` :
        '';
        args.push(`${name}: ${wgslExprFor(value, call.coordType)}${derivativeWGSL}`);
      } else if (name === 'derivativeMult') {

        // skip this - it's covered in 'coords'
      } else if (name === 'ddx' || name === 'ddy') {args.push(`${name}: ${wgslExprFor(value, call.coordType)}`);
      } else if (name === 'mipLevel') {
        args.push(`${name}: ${wgslExprFor(value, call.levelType)}`);
      } else if (name === 'arrayIndex') {
        args.push(`${name}: ${wgslExprFor(value, call.arrayIndexType)}`);
      } else if (name === 'bias') {
        args.push(`${name}: ${wgslExprFor(value, 'f')}`);
      } else if (name === 'sampleIndex') {
        args.push(`${name}: ${wgslExprFor(value, call.sampleIndexType)}`);
      } else if (name === 'depthRef') {
        args.push(`${name}: ${wgslExprFor(value, 'f')}`);
      } else {
        args.push(`${name}: ${wgslExpr(value)}`);
      }
    }
  }
  return `${call.builtin}(${args.join(', ')})`;
}

const s_deviceToPipelines = new WeakMap(


);

/**
 * Given a list of "calls", each one of which has a texture coordinate,
 * generates a fragment shader that uses the instance_index as an index. That
 * index is then used to look up a coordinate from a storage buffer which is
 * used to call the WGSL texture function to read/sample the texture, and then
 * write to a storage buffer. We then read the storage buffer for the per "call"
 * results.
 *
 * We use a 1x1 target and use instance drawing, once instance per call.
 * This allows use to more easily adjust derivatives per call.
 *
 * An issue we ran into before this "one draw call per instance" change;
 * Before we had a single draw call and wrote the result of one call per
 * pixel rendered.
 *
 * Imagine we have code like this:
 *
 * ```
 * @group(0) @binding(0) var T: texture_2d<f32>;
 * @group(0) @binding(1) var S: sampler;
 * @group(0) @binding(2) var<storage> coords: array<vec4f>;
 * @fragment fn fs(@builtin(position) pos: vec4f) -> vec4f {
 *   let ndx = u32(pos.x) * u32(pos.y) * targetWidth;
 *   return textureSample(T, S, coords[ndx].xy);
 * }
 * ```
 *
 * T points to 8x8 pixel texture with 3 mip levels
 * S is 'nearest'
 * coords: is a storage buffer, 16 bytes long [0,0,0,0], one vec4f.
 * our render target is 1x1 pixels
 *
 * Looking above it appears `ndx` will only ever be 0 but that's
 * not what happens. Instead, the GPU will run the fragment shader for
 * a 2x2 area. It does this to compute derivatives by running the code
 * above and looking at what values it gets passed as coords to
 * textureSample. When it does this it ends up with
 *
 * ndx = 0 for invocation 0
 * ndx = 1 for invocation 1
 * ndx = 0 + 1 * targetWidth for invocation 2
 * ndx = 1 + 1 * targetWidth for invocation 3
 *
 * In 3 of those cases `ndx` is out of bounds with respect to `coords`.
 * Out of bounds access is indeterminate. That means the derivatives are
 * indeterminate so what lod it tries to read is indeterminate.
 *
 * By using instance_index for ndx we avoid this issue. ndx is the same
 * on all 4 executions.
 *
 * Calls are "binned" by call parameters. Each bin has its own structure and
 * field in the storage buffer. This allows the calls to be non-homogenous and
 * each have their own data type for coordinates.
 *
 * Note: this function returns:
 *
 * 'results': an array of results, one for each call.
 *
 * 'run': a function that accepts a texture and runs the same class pipeline with
 *        that texture as input, returning an array of results. This can be used by
 *        identifySamplePoints to query the mix-weights used. We do this so we're
 *        using the same shader that generated the original results when querying
 *        the weights.
 *
 * 'destroy': a function that cleans up the buffers used by `run`.
 */
function createTextureCallsRunner(
t,
{
  format,
  dimension,
  sampleCount,
  depthOrArrayLayers





},
viewDescriptor,
textureType,
sampler,
calls,
stage)
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
  ${b.fields.join(',\n  ')}
}
`;
    dataFields += `  args${binIdx} : array<Args${binIdx}, ${binCalls.length}>,
`;
    body += `
  {
    let is_active = (idx >= ${callCount}) & (idx < ${callCount + binCalls.length});
    let args = data.args${binIdx}[idx - ${callCount}];
    let call = ${b.expr};
    result = select(result, call, is_active);
  }
`;
    callCount += binCalls.length;
    data.push(...b.data);
  });

  const dataBuffer = t.createBufferTracked({
    size: data.length * 4,
    usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.UNIFORM
  });
  t.device.queue.writeBuffer(dataBuffer, 0, new Uint32Array(data));

  const builtin = calls[0].builtin;
  const isCompare = isBuiltinComparison(builtin);

  const { resultType, resultFormat, componentType } = isBuiltinGather(builtin) ?
  getTextureFormatTypeInfo(format) :
  textureType === 'texture_external' ?
  { resultType: 'vec4f', resultFormat: 'rgba32float', componentType: 'f32' } :
  textureType.includes('depth') ?
  { resultType: 'f32', resultFormat: 'rgba32float', componentType: 'f32' } :
  getTextureFormatTypeInfo(format);
  const returnType = `vec4<${componentType}>`;

  const samplerType = isCompare ? 'sampler_comparison' : 'sampler';

  const renderTarget = t.createTextureTracked({
    format: resultFormat,
    size: [calls.length, 1],
    usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT
  });

  // derivativeBase is a number that starts at (0, 0, 0) and advances by 1 in x, y
  // for each fragment shader iteration in texel space. It is then converted to normalized
  // texture space by dividing by the textureDimensions.
  // Since it's moving by 1 texel unit we can multiply it to get any specific lod value we want.
  // Because it starts at (0, 0, 0) it will not affect our texture coordinate.
  const derivativeBaseWGSL = `
  let derivativeBase = ${
  isCubeViewDimension(viewDescriptor) ?
  '(v.pos.xyx - 0.5 - vec3f(f32(v.ndx), 0, f32(v.ndx))) / vec3f(vec2f(textureDimensions(T)), 1.0)' :
  dimension === '1d' ?
  'f32(v.pos.x - 0.5 - f32(v.ndx)) / f32(textureDimensions(T))' :
  dimension === '3d' ?
  'vec3f(v.pos.xy - 0.5 - vec2f(f32(v.ndx), 0), 0) / vec3f(textureDimensions(T))' :
  '(v.pos.xy - 0.5 - vec2f(f32(v.ndx), 0)) / vec2f(textureDimensions(T))'
  };`;
  const derivativeType =
  isCubeViewDimension(viewDescriptor) || dimension === '3d' ?
  'vec3f' :
  dimension === '1d' ?
  'f32' :
  'vec2f';

  const stageWGSL =
  stage === 'vertex' ?
  `
// --------------------------- vertex stage shaders --------------------------------
@vertex fn vsVertex(
    @builtin(vertex_index) vertex_index : u32,
    @builtin(instance_index) instance_index : u32) -> VOut {
  let positions = array(vec2f(-1, 3), vec2f(3, -1), vec2f(-1, -1));
  return VOut(vec4f(positions[vertex_index], 0, 1),
              instance_index,
              getResult(instance_index, ${derivativeType}(0)));
}

@fragment fn fsVertex(v: VOut) -> @location(0) ${returnType} {
  return v.result;
}
` :
  stage === 'fragment' ?
  `
// --------------------------- fragment stage shaders --------------------------------
@vertex fn vsFragment(
    @builtin(vertex_index) vertex_index : u32,
    @builtin(instance_index) instance_index : u32) -> VOut {
  let positions = array(vec2f(-1, 3), vec2f(3, -1), vec2f(-1, -1));
  return VOut(vec4f(positions[vertex_index], 0, 1), instance_index, ${returnType}(0));
}

@fragment fn fsFragment(v: VOut) -> @location(0) ${returnType} {
  ${derivativeBaseWGSL}
  return getResult(v.ndx, derivativeBase);
}
` :
  `
// --------------------------- compute stage shaders --------------------------------
@group(1) @binding(0) var<storage, read_write> results: array<${returnType}>;

@compute @workgroup_size(1) fn csCompute(@builtin(global_invocation_id) id: vec3u) {
  results[id.x] = getResult(id.x, ${derivativeType}(0));
}
`;

  const code = `
${structs}

struct Data {
${dataFields}
}

struct VOut {
  @builtin(position) pos: vec4f,
  @location(0) @interpolate(flat, either) ndx: u32,
  @location(1) @interpolate(flat, either) result: ${returnType},
};

@group(0) @binding(0) var          T    : ${textureType};
${sampler ? `@group(0) @binding(1) var          S    : ${samplerType}` : ''};
@group(0) @binding(2) var<uniform> data : Data;

fn getResult(idx: u32, derivativeBase: ${derivativeType}) -> ${returnType} {
  var result : ${resultType};
${body}
  return ${returnType}(result);
}

${stageWGSL}
`;

  const pipelines =
  s_deviceToPipelines.get(t.device) ?? new Map();
  s_deviceToPipelines.set(t.device, pipelines);

  // unfilterable-float textures can only be used with manually created bindGroupLayouts
  // since the default 'auto' layout requires filterable textures/samplers.
  // So, if we don't need filtering, don't request a filtering sampler. If we require
  // filtering then check if the format is 32float format and if float32-filterable
  // is enabled.
  const info = kTextureFormatInfo[format ?? 'rgba8unorm'];
  const isFiltering =
  !!sampler && (
  sampler.minFilter === 'linear' ||
  sampler.magFilter === 'linear' ||
  sampler.mipmapFilter === 'linear');
  let sampleType = textureType.startsWith('texture_depth') ?
  'depth' :
  isDepthTextureFormat(format) ?
  'unfilterable-float' :
  isStencilTextureFormat(format) ?
  'uint' :
  info.color?.type ?? 'float';
  if (isFiltering && sampleType === 'unfilterable-float') {
    assert(is32Float(format));
    assert(t.device.features.has('float32-filterable'));
    sampleType = 'float';
  }
  if (sampleCount > 1 && sampleType === 'float') {
    sampleType = 'unfilterable-float';
  }

  const visibility =
  stage === 'compute' ?
  GPUShaderStage.COMPUTE :
  stage === 'fragment' ?
  GPUShaderStage.FRAGMENT :
  GPUShaderStage.VERTEX;

  const entries = [
  {
    binding: 2,
    visibility,
    buffer: {
      type: 'uniform'
    }
  }];


  const viewDimension = effectiveViewDimensionForDimension(
    viewDescriptor.dimension,
    dimension,
    depthOrArrayLayers
  );

  if (textureType.includes('storage')) {
    entries.push({
      binding: 0,
      visibility,
      storageTexture: {
        access: 'read-only',
        viewDimension,
        format
      }
    });
  } else if (textureType === 'texture_external') {
    entries.push({
      binding: 0,
      visibility,
      externalTexture: {}
    });
  } else {
    entries.push({
      binding: 0,
      visibility,
      texture: {
        sampleType,
        viewDimension,
        multisampled: sampleCount > 1
      }
    });
  }

  if (sampler) {
    entries.push({
      binding: 1,
      visibility,
      sampler: {
        type: isCompare ? 'comparison' : isFiltering ? 'filtering' : 'non-filtering'
      }
    });
  }

  const id = `${resultType}:${stage}:${JSON.stringify(entries)}:${code}`;
  let pipeline = pipelines.get(id);
  if (!pipeline) {
    const module = t.device.createShaderModule({ code });
    const bindGroupLayout0 = t.device.createBindGroupLayout({ entries });
    const bindGroupLayouts = [bindGroupLayout0];

    if (stage === 'compute') {
      const bindGroupLayout1 = t.device.createBindGroupLayout({
        entries: [
        {
          binding: 0,
          visibility: GPUShaderStage.FRAGMENT | GPUShaderStage.COMPUTE,
          buffer: {
            type: 'storage'
          }
        }]

      });
      bindGroupLayouts.push(bindGroupLayout1);
    }

    const layout = t.device.createPipelineLayout({
      bindGroupLayouts
    });

    switch (stage) {
      case 'compute':
        pipeline = t.device.createComputePipeline({
          layout,
          compute: { module }
        });
        break;
      case 'fragment':
      case 'vertex':
        pipeline = t.device.createRenderPipeline({
          layout,
          vertex: { module },
          fragment: {
            module,
            targets: [{ format: renderTarget.format }]
          }
        });
        break;
    }
    pipelines.set(id, pipeline);
  }

  const gpuSampler = sampler ? t.device.createSampler(sampler) : undefined;

  const run = async (gpuTexture) => {
    const resultBuffer = t.createBufferTracked({
      size: align(calls.length * 16, 256),
      usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ
    });

    const bindGroup0 = t.device.createBindGroup({
      layout: pipeline.getBindGroupLayout(0),
      entries: [
      {
        binding: 0,
        resource:
        gpuTexture instanceof GPUExternalTexture ?
        gpuTexture :
        gpuTexture.createView(viewDescriptor)
      },
      ...(sampler ? [{ binding: 1, resource: gpuSampler }] : []),
      { binding: 2, resource: { buffer: dataBuffer } }]

    });

    let storageBuffer;
    const encoder = t.device.createCommandEncoder();

    if (stage === 'compute') {
      storageBuffer = t.createBufferTracked({
        size: resultBuffer.size,
        usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC
      });

      const bindGroup1 = t.device.createBindGroup({
        layout: pipeline.getBindGroupLayout(1),
        entries: [{ binding: 0, resource: { buffer: storageBuffer } }]
      });

      const pass = encoder.beginComputePass();
      pass.setPipeline(pipeline);
      pass.setBindGroup(0, bindGroup0);
      pass.setBindGroup(1, bindGroup1);
      pass.dispatchWorkgroups(calls.length);
      pass.end();
      encoder.copyBufferToBuffer(storageBuffer, 0, resultBuffer, 0, storageBuffer.size);
    } else {
      const pass = encoder.beginRenderPass({
        colorAttachments: [
        {
          view: renderTarget.createView(),
          loadOp: 'clear',
          storeOp: 'store'
        }]

      });

      pass.setPipeline(pipeline);
      pass.setBindGroup(0, bindGroup0);
      for (let i = 0; i < calls.length; ++i) {
        pass.setViewport(i, 0, 1, 1, 0, 1);
        pass.draw(3, 1, 0, i);
      }
      pass.end();
      encoder.copyTextureToBuffer(
        { texture: renderTarget },
        {
          buffer: resultBuffer,
          bytesPerRow: resultBuffer.size
        },
        [renderTarget.width, 1]
      );
    }
    t.device.queue.submit([encoder.finish()]);

    await resultBuffer.mapAsync(GPUMapMode.READ);

    const view = TexelView.fromTextureDataByReference(
      resultFormat,
      new Uint8Array(resultBuffer.getMappedRange()),
      {
        bytesPerRow: calls.length * 16,
        rowsPerImage: 1,
        subrectOrigin: [0, 0, 0],
        subrectSize: [calls.length, 1]
      }
    );

    let outIdx = 0;
    const out = new Array(calls.length);
    for (const bin of binned) {
      for (const callIdx of bin) {
        const x = outIdx;
        out[callIdx] = view.color({ x, y: 0, z: 0 });
        outIdx++;
      }
    }

    storageBuffer?.destroy();
    resultBuffer.destroy();

    return out;
  };

  return {
    run,
    destroy() {
      dataBuffer.destroy();
      renderTarget.destroy();
    }
  };
}

export async function doTextureCalls(
t,
gpuTexture,
viewDescriptor,
textureType,
sampler,
calls,
shortShaderStage)
{
  const stage = kShortShaderStageToShaderStage[shortShaderStage];
  const runner = createTextureCallsRunner(
    t,
    gpuTexture instanceof GPUExternalTexture ?
    { format: 'rgba8unorm', dimension: '2d', depthOrArrayLayers: 1, sampleCount: 1 } :
    gpuTexture,
    viewDescriptor,
    textureType,
    sampler,
    calls,
    stage
  );
  const results = await runner.run(gpuTexture);

  return {
    runner,
    results
  };
}