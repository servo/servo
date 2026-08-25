/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Operational tests for the 'texture-component-swizzle' feature.

Test that:
* when the feature is on, swizzling is applied correctly.

Note: for texture_depth_xxx we only get f32
What happens in the GPU (at least in Metal)

  1. we start with [depthOrCompareResult, depthOrCompareResult, depthOrCompareResult, 1]
  2. we then swizzle
  3. we then read the RED channel

Gather will do this 4 times and give us the result of step 3 in each channel.

The WebGPU spec says we should be starting with [depthOrCompare, 0, 0, 1] and the
implementation should deal with this.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { assert, range, unreachable } from '../../../../common/util/util.js';
import {
  isSintOrUintFormat,
  isDepthOrStencilTextureFormat,
  kAllTextureFormats,
  isDepthTextureFormat,
  getBlockInfoForTextureFormat,
  isStencilTextureFormat,
  isDepthStencilTextureFormat,
  isTextureFormatPossiblyMultisampled,
  isTextureFormatUsableAsRenderAttachment } from
'../../../format_info.js';
import { AllFeaturesMaxLimitsGPUTest } from '../../../gpu_test.js';
import {
  applyCompareToTexel,
  chooseTextureSize,
  convertPerTexelComponentToResultFormat,
  createTextureWithRandomDataAndGetTexelsForEachAspect,
  getTextureFormatTypeInfo,
  isBuiltinComparison,
  isBuiltinGather,
  isFillable,
  swizzleTexel } from

'../../../shader/execution/expression/call/builtin/texture_utils.js';
import * as ttu from '../../../texture_test_utils.js';
import { TexelComponent } from '../../../util/texture/texel_data.js';
import { TexelView } from '../../../util/texture/texel_view.js';
import {
  kSwizzleTests } from

'../../validation/capability_checks/features/texture_component_swizzle_utils.js';











function isSingleChannelInput(input) {
  return input === 'texture_depth_2d' || input === 'texture_depth_multisampled_2d';
}

function isMultisampledInput(input) {
  return (
    input === 'texture_multisampled_2d<f32>' ||
    input === 'texture_multisampled_2d<u32>' ||
    input === 'texture_multisampled_2d<i32>' ||
    input === 'texture_depth_multisampled_2d');

}

function getSwizzleByOffsetFromSwizzle(swizzle, offset) {
  return kSwizzleTests[(kSwizzleTests.indexOf(swizzle) + offset) % kSwizzleTests.length];
}

const kTextureBuiltinFunctions = [
'textureGather',
'textureGatherCompare',
'textureLoad',
'textureSample',
'textureSampleBias',
'textureSampleCompare',
'textureSampleCompareLevel',
'textureSampleGrad',
'textureSampleLevel'
// 'textureSampleBaseClampToEdge',  // external textures don't have a view to swizzle.
];

function canBuiltinTakeTextureDepth(func) {
  return (
    func === 'textureGather' ||
    func === 'textureGatherCompare' ||
    func === 'textureLoad' ||
    func === 'textureSample' ||
    func === 'textureSampleCompare' ||
    func === 'textureSampleCompareLevel' ||
    func === 'textureSampleLevel');

}

function canUseBuiltinFuncWithFormat(
func,
format,
aspect)
{
  const effectiveFormat = aspect === 'stencil-only' ? 'stencil8' : format;
  if (isSintOrUintFormat(effectiveFormat)) {
    return func === 'textureGather' || func === 'textureLoad';
  } else if (!isDepthTextureFormat(effectiveFormat)) {
    return (
      func !== 'textureGatherCompare' &&
      func !== 'textureSampleCompare' &&
      func !== 'textureSampleCompareLevel');

  } else {
    return true;
  }
}

function channelIndexToTexelComponent(channel) {
  switch (channel) {
    case 0:
      return TexelComponent.R;
    case 1:
      return TexelComponent.G;
    case 2:
      return TexelComponent.B;
    case 3:
      return TexelComponent.A;
    default:
      throw new Error(`Invalid channel index: ${channel}`);
  }
}

function getColorByChannelIndex(texel, channel) {
  const component = channelIndexToTexelComponent(channel);
  const v = texel[component];
  assert(v !== undefined);
  return v;
}

function gather(
srcColors,
channel)
{
  //   texel gather offsets
  // r [0, 1] 2
  // g [1, 1] 3
  // b [1, 0] 1
  // a [0, 0] 0
  return {
    R: getColorByChannelIndex(srcColors[2], channel),
    G: getColorByChannelIndex(srcColors[3], channel),
    B: getColorByChannelIndex(srcColors[1], channel),
    A: getColorByChannelIndex(srcColors[0], channel)
  };
}

const kGatherComponentOrder = ['B', 'A', 'R', 'G'];


const s_deviceToPipelines = new WeakMap();

export const g = makeTestGroup(AllFeaturesMaxLimitsGPUTest);

g.test('read_swizzle').
desc(
  `
  Test reading textures with swizzles.
  * Test that multiple swizzles of the same texture work.
  * Test that multiple swizzles of the same fails in compat if the swizzles are different.
  `
).
params(
  (u) =>
  u.
  combine('format', kAllTextureFormats).
  filter((t) => isFillable(t.format)).
  combine('func', kTextureBuiltinFunctions).
  beginSubcases().
  expand('compare', function* (t) {
    if (isBuiltinComparison(t.func)) {
      yield 'less';
      yield 'greater';
    } else {
      yield 'always';
    }
  }).
  expand('aspect', function* (t) {
    if (isDepthOrStencilTextureFormat(t.format)) {
      if (isDepthTextureFormat(t.format)) {
        yield 'depth-only';
      }
      if (isStencilTextureFormat(t.format)) {
        yield 'stencil-only';
      }
    } else {
      yield 'all';
    }
  }).
  filter((t) => canUseBuiltinFuncWithFormat(t.func, t.format, t.aspect)).
  expand('input', function* (t) {
    if (!isBuiltinComparison(t.func)) {
      const { componentType } = getTextureFormatTypeInfo(t.format, t.aspect);
      switch (componentType) {
        case 'f32':
          yield `texture_2d<f32>`;
          break;
        case 'u32':
          yield `texture_2d<u32>`;
          break;
        case 'i32':
          yield `texture_2d<i32>`;
          break;
        default:
          unreachable();
      }
    }
    if (
    isDepthTextureFormat(t.format) &&
    canBuiltinTakeTextureDepth(t.func) &&
    t.aspect === 'depth-only')
    {
      yield `texture_depth_2d`;
    }
    if (t.func === 'textureLoad' && isTextureFormatPossiblyMultisampled(t.format)) {
      const { componentType } = getTextureFormatTypeInfo(t.format, t.aspect);
      switch (componentType) {
        case 'f32':
          yield `texture_multisampled_2d<f32>`;
          break;
        case 'u32':
          yield `texture_multisampled_2d<u32>`;
          break;
        case 'i32':
          yield `texture_multisampled_2d<i32>`;
          break;
        default:
          unreachable();
      }
      if (
      isDepthTextureFormat(t.format) &&
      canBuiltinTakeTextureDepth(t.func) &&
      t.aspect === 'depth-only')
      {
        yield `texture_depth_multisampled_2d`;
      }
    }
  }).
  expand('channel', function* (t) {
    if (t.func === 'textureGather' && !isSingleChannelInput(t.input)) {
      yield 0;
      yield 1;
      yield 2;
      yield 3;
    } else {
      yield 0;
    }
  }).
  combine('swizzle', kSwizzleTests).
  combine('otherSwizzleIndexOffset', [0, 1, 5]) // used to choose a different 2nd swizzle. 0 = same swizzle as 1st
).
fn(async (t) => {
  t.skipIfDeviceDoesNotHaveFeature('texture-component-swizzle');
  const { format, func, channel, compare, input, aspect, swizzle, otherSwizzleIndexOffset } =
  t.params;
  t.skipIfTextureFormatNotSupported(format);
  if (func === 'textureLoad') {
    t.skipIfTextureLoadNotSupportedForTextureType(input);
  }
  if (isMultisampledInput(input)) {
    t.skipIfTextureFormatNotMultisampled(format);
  }
  const otherSwizzle = getSwizzleByOffsetFromSwizzle(swizzle, otherSwizzleIndexOffset);
  t.debug(() => `swizzle: ${swizzle}, otherSwizzle: ${otherSwizzle}`);

  if (t.isCompatibility) {
    t.skipIf(
      swizzle !== otherSwizzle,
      `swizzles must be equivalent in compatibility mode: ${swizzle} != ${otherSwizzle}`
    );
    t.skipIf(
      !isBuiltinComparison(func) && input === 'texture_depth_2d',
      'can not use depth textures with non-comparison samplers in compatibility mode'
    );
  }

  const depthRef = 0.5;
  const size = chooseTextureSize({ minSize: 2, minBlocks: 2, format });
  const { blockWidth, blockHeight } = getBlockInfoForTextureFormat(format);
  // Choose a texture coordinate that will cross a block boundary for gather.
  // This is because we only create solid color blocks for some formats so we
  // won't get a different color per channel unless we sample across blocks.
  const tx = blockWidth - 0.4;
  const ty = blockHeight - 0.4;
  const descriptor = {
    label: 'swizzle test texture',
    format,
    size,
    usage:
    GPUTextureUsage.COPY_DST |
    GPUTextureUsage.TEXTURE_BINDING | (
    isTextureFormatUsableAsRenderAttachment(t.device.features, format) ?
    GPUTextureUsage.RENDER_ATTACHMENT :
    0),
    sampleCount: isMultisampledInput(input) ? 4 : 1
  };
  const { texels: srcTexelViews, texture } =
  await createTextureWithRandomDataAndGetTexelsForEachAspect(t, descriptor);
  const aspectNdx = isDepthStencilTextureFormat(format) && aspect === 'stencil-only' ? 1 : 0;
  const srcTexelView = srcTexelViews[aspectNdx];

  const samples = [];
  const sampledColors = range(4, (i) => {
    const x = (tx | 0) + i % 2;
    const y = (ty | 0) + (i / 2 | 0);

    const sample = srcTexelView[0].color({ x, y, z: 0 });
    samples.push(sample);
    return convertPerTexelComponentToResultFormat(sample, format, aspect);
  });
  t.debug(
    () => `samples:
${sampledColors.map((c, i) => `${i % 2}, ${i / 2 | 0}, ${JSON.stringify(c)}`).join('\n')}`
  );

  const components = [TexelComponent.R, TexelComponent.G, TexelComponent.B, TexelComponent.A];
  const readColors = sampledColors.map((sampledColor) =>
  isBuiltinComparison(func) ?
  applyCompareToTexel(components, sampledColor, compare, depthRef) :
  sampledColor
  );

  const {
    resultType,
    sampleType: srcSampleType,
    resultFormat: expFormat
  } = getTextureFormatTypeInfo(format, aspect);

  const testData = [swizzle, otherSwizzle].map((swizzle) => {
    const swizzledColors = readColors.map((readColor) => swizzleTexel(readColor, swizzle));
    const expColor = isBuiltinGather(func) ?
    gather(swizzledColors, channel) :
    isSingleChannelInput(input) ?
    {
      R: swizzledColors[0].R,
      G: swizzledColors[0].R,
      B: swizzledColors[0].R,
      A: swizzledColors[0].R
    } :
    swizzledColors[0];

    const expTexelView = TexelView.fromTexelsAsColors(expFormat, (_coords) => expColor);
    const textureView = texture.createView({
      label: `swizzle texture view(${swizzle})`,
      swizzle,
      aspect,
      usage: GPUTextureUsage.TEXTURE_BINDING
    });

    // BA  in a 2x2 texel area this is
    // RG  the order of gather.
    t.debug(
      () => `\
  swizzle: ${swizzle}, channel: ${channel}, ${
      compare === 'always' ? '' : `compare: ${depthRef} is ${compare} than Texel`
      }
  readColors:
${readColors.
      map((c, i) => `${i % 2}, ${i / 2 | 0}, ${JSON.stringify(c)} ${kGatherComponentOrder[i]}`).
      join('\n')}
  swizzledColors:
${swizzledColors.
      map((c, i) => `${i % 2}, ${i / 2 | 0}, ${JSON.stringify(c)} ${kGatherComponentOrder[i]}`).
      join('\n')}
  `
    );
    return { swizzle, expColor, expFormat, expTexelView, textureView };
  });

  t.debug(
    () => `expColors:
${testData.
    map(({ expColor }, i) => `${i % 2}, ${i / 2 | 0}, ${JSON.stringify(expColor)}`).
    join('\n')}`
  );

  const loadFn = ((func) => {
    switch (func) {
      case 'textureGather':
        return (v) =>
        isSingleChannelInput(input) ?
        `textureGather(tex${v}, smp, uni.texCoord)` :
        `textureGather(${channel}, tex${v}, smp, uni.texCoord)`;
      case 'textureGatherCompare':
        return (v) => `textureGatherCompare(tex${v}, smp, uni.texCoord, ${depthRef})`;
      case 'textureLoad':
        return (v) =>
        `textureLoad(tex${v}, vec2u(uni.texCoord * vec2f(textureDimensions(tex${v}))), 0)`;
      case 'textureSample':
        return (v) => `textureSample(tex${v}, smp, uni.texCoord)`;
      case 'textureSampleBias':
        return (v) => `textureSampleBias(tex${v}, smp, uni.texCoord, 0)`;
      case 'textureSampleCompare':
        return (v) => `textureSampleCompare(tex${v}, smp, uni.texCoord, ${depthRef})`;
      case 'textureSampleCompareLevel':
        return (v) =>
        `textureSampleCompareLevel(tex${v}, smp, uni.texCoord, ${depthRef})`;
      case 'textureSampleGrad':
        return (v) => `textureSampleGrad(tex${v}, smp, uni.texCoord, vec2f(0), vec2f(0))`;
      case 'textureSampleLevel':
        return (v) => `textureSampleLevel(tex${v}, smp, uni.texCoord, 0)`;
      default:
        throw new Error(`Unsupported texture builtin function: ${func}`);
    }
  })(func);
  const loadWGSL = (v) => `${resultType}(${loadFn(v)})`;

  const samplerWGSL = isBuiltinComparison(func) ? 'sampler_comparison' : 'sampler';
  const code = `
      struct Uniforms {
        texCoord: vec2f,
      };

      // These are intentionally in different bindGroups to test in compat that different swizzles
      // of the same texture are not allowed.
      @group(0) @binding(0) var tex0: ${input};
      @group(1) @binding(0) var tex1: ${input};
      @group(0) @binding(1) var smp: ${samplerWGSL};
      @group(0) @binding(2) var<uniform> uni: Uniforms;
      @group(0) @binding(3) var result: texture_storage_2d<${expFormat}, write>;

      @vertex fn vsFSResults() -> @builtin(position) vec4f {
        return vec4f(0, 0, 0, 1);
      }

      @fragment fn fsFSResults() -> @location(0) vec4f {
        let c0 = ${loadWGSL(0)};
        let c1 = ${loadWGSL(1)};
        textureStore(result, vec2u(0, 0), c0);
        textureStore(result, vec2u(1, 0), c1);
        return vec4f(0, 0, 0, 1);
      }
    `;

  const sampleType = isSingleChannelInput(input) ?
  'depth' :
  srcSampleType === 'depth' ?
  isBuiltinComparison(func) ?
  'depth' :
  'unfilterable-float' :
  srcSampleType === 'float' && isMultisampledInput(input) ?
  'unfilterable-float' :
  srcSampleType;
  const samplerType = isBuiltinComparison(func) ? 'comparison' : 'non-filtering';

  const pipelineId = `${sampleType}:${samplerType}${code}`;
  const cache = s_deviceToPipelines.get(t.device) ?? new Map();
  s_deviceToPipelines.set(t.device, cache);
  let pipeline = cache.get(pipelineId);
  if (!pipeline) {
    const module = t.device.createShaderModule({ code });

    const bgl0 = t.device.createBindGroupLayout({
      entries: [
      {
        binding: 0,
        visibility: GPUShaderStage.COMPUTE | GPUShaderStage.FRAGMENT | GPUShaderStage.VERTEX,
        texture: {
          sampleType,
          multisampled: isMultisampledInput(input)
        }
      },
      {
        binding: 1,
        visibility: GPUShaderStage.COMPUTE | GPUShaderStage.FRAGMENT | GPUShaderStage.VERTEX,
        sampler: {
          type: samplerType
        }
      },
      {
        binding: 2,
        visibility: GPUShaderStage.COMPUTE | GPUShaderStage.FRAGMENT | GPUShaderStage.VERTEX,
        buffer: {}
      },
      {
        binding: 3,
        visibility: GPUShaderStage.COMPUTE | GPUShaderStage.FRAGMENT,
        storageTexture: {
          format: expFormat
        }
      }]

    });

    const bgl1 = t.device.createBindGroupLayout({
      entries: [
      {
        binding: 0,
        visibility: GPUShaderStage.COMPUTE | GPUShaderStage.FRAGMENT | GPUShaderStage.VERTEX,
        texture: {
          sampleType,
          multisampled: isMultisampledInput(input)
        }
      }]

    });

    const layout = t.device.createPipelineLayout({
      bindGroupLayouts: [bgl0, bgl1]
    });

    pipeline = t.device.createRenderPipeline({
      layout,
      vertex: { module },
      fragment: { module, targets: [{ format: 'rgba8unorm' }] },
      primitive: { topology: 'point-list' }
    });
    cache.set(pipelineId, pipeline);
  }

  const outputTexture = t.createTextureTracked({
    format: expFormat,
    size: [2],
    usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.STORAGE_BINDING
  });

  const sampler = t.device.createSampler(
    isBuiltinComparison(func) ?
    {
      compare
    } :
    {}
  );

  const uniformBuffer = t.createBufferTracked({
    size: (2 + 2) * 4, // vec2f + padding
    usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST
  });
  const uniformValues = new ArrayBuffer(uniformBuffer.size);
  const asF32 = new Float32Array(uniformValues);
  asF32.set([tx / texture.width, ty / texture.height]);
  t.debug(
    () =>
    `texcoords: ${[...asF32]}  tx = ${tx}, ty = ${ty}, size: ${texture.width}, ${
    texture.height
    }`
  );
  t.device.queue.writeBuffer(uniformBuffer, 0, new Uint32Array(uniformValues));

  const bindGroup0 = t.device.createBindGroup({
    layout: pipeline.getBindGroupLayout(0),
    entries: [
    { binding: 0, resource: testData[0].textureView },
    { binding: 1, resource: sampler },
    { binding: 2, resource: uniformBuffer },
    { binding: 3, resource: outputTexture }]

  });

  const bindGroup1 = t.device.createBindGroup({
    layout: pipeline.getBindGroupLayout(1),
    entries: [{ binding: 0, resource: testData[1].textureView }]
  });

  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginRenderPass({
    colorAttachments: [
    {
      view: t.createTextureTracked({
        format: 'rgba8unorm',
        size: [1],
        usage: GPUTextureUsage.RENDER_ATTACHMENT
      }),
      loadOp: 'clear',
      storeOp: 'store'
    }]

  });
  pass.setPipeline(pipeline);
  pass.setBindGroup(0, bindGroup0);
  pass.setBindGroup(1, bindGroup1);
  pass.draw(1);
  pass.end();

  if (t.isCompatibility && testData[0].swizzle !== testData[1].swizzle) {
    // Swizzles can not be different in compatibility mode
    t.expectValidationError(() => {
      t.device.queue.submit([encoder.finish()]);
    });
  } else {
    t.device.queue.submit([encoder.finish()]);

    testData.forEach(({ swizzle, expTexelView }, i) => {
      t.debug(() => `${i}: ${swizzle}`);

      ttu.expectTexelViewComparisonIsOkInTexture(
        t,
        { texture: outputTexture, origin: [i, 0, 0] },
        expTexelView,
        [1, 1, 1],
        { maxFractionalDiff: 0.01 }
      );
    });
  }
});