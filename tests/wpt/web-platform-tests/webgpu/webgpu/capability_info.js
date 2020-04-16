/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

import * as C from '../common/constants.js'; // Textures

export const kTextureFormatInfo =
/* prettier-ignore */
{
  // Try to keep these manually-formatted in a readable grid.
  // (Note: this list should always match the one in the spec.)
  // 8-bit formats
  'r8unorm': {
    renderable: true,
    color: true
  },
  'r8snorm': {
    renderable: false,
    color: true
  },
  'r8uint': {
    renderable: true,
    color: true
  },
  'r8sint': {
    renderable: true,
    color: true
  },
  // 16-bit formats
  'r16uint': {
    renderable: true,
    color: true
  },
  'r16sint': {
    renderable: true,
    color: true
  },
  'r16float': {
    renderable: true,
    color: true
  },
  'rg8unorm': {
    renderable: true,
    color: true
  },
  'rg8snorm': {
    renderable: false,
    color: true
  },
  'rg8uint': {
    renderable: true,
    color: true
  },
  'rg8sint': {
    renderable: true,
    color: true
  },
  // 32-bit formats
  'r32uint': {
    renderable: true,
    color: true
  },
  'r32sint': {
    renderable: true,
    color: true
  },
  'r32float': {
    renderable: true,
    color: true
  },
  'rg16uint': {
    renderable: true,
    color: true
  },
  'rg16sint': {
    renderable: true,
    color: true
  },
  'rg16float': {
    renderable: true,
    color: true
  },
  'rgba8unorm': {
    renderable: true,
    color: true
  },
  'rgba8unorm-srgb': {
    renderable: true,
    color: true
  },
  'rgba8snorm': {
    renderable: false,
    color: true
  },
  'rgba8uint': {
    renderable: true,
    color: true
  },
  'rgba8sint': {
    renderable: true,
    color: true
  },
  'bgra8unorm': {
    renderable: true,
    color: true
  },
  'bgra8unorm-srgb': {
    renderable: true,
    color: true
  },
  // Packed 32-bit formats
  'rgb10a2unorm': {
    renderable: true,
    color: true
  },
  'rg11b10float': {
    renderable: false,
    color: true
  },
  // 64-bit formats
  'rg32uint': {
    renderable: true,
    color: true
  },
  'rg32sint': {
    renderable: true,
    color: true
  },
  'rg32float': {
    renderable: true,
    color: true
  },
  'rgba16uint': {
    renderable: true,
    color: true
  },
  'rgba16sint': {
    renderable: true,
    color: true
  },
  'rgba16float': {
    renderable: true,
    color: true
  },
  // 128-bit formats
  'rgba32uint': {
    renderable: true,
    color: true
  },
  'rgba32sint': {
    renderable: true,
    color: true
  },
  'rgba32float': {
    renderable: true,
    color: true
  },
  // Depth/stencil formats
  'depth32float': {
    renderable: true,
    color: false
  },
  'depth24plus': {
    renderable: true,
    color: false
  },
  'depth24plus-stencil8': {
    renderable: true,
    color: false
  }
};
export const kTextureFormats = Object.keys(kTextureFormatInfo); // Bindings

export const kMaxBindingsPerBindGroup = 16;
export const kPerStageBindingLimits =
/* prettier-ignore */
{
  'uniform-buffer': 12,
  'storage-buffer': 4,
  'sampler': 16,
  'sampled-texture': 16,
  'storage-texture': 4
};
const kStagesAll = C.ShaderStage.Vertex | C.ShaderStage.Fragment | C.ShaderStage.Compute;
const kStagesCompute = C.ShaderStage.Compute;
export const kBindingTypeInfo =
/* prettier-ignore */
{
  'uniform-buffer': {
    type: 'buffer',
    validStages: kStagesAll,
    perStageLimitType: 'uniform-buffer',
    maxDynamicCount: 8
  },
  'storage-buffer': {
    type: 'buffer',
    validStages: kStagesCompute,
    perStageLimitType: 'storage-buffer',
    maxDynamicCount: 4
  },
  'readonly-storage-buffer': {
    type: 'buffer',
    validStages: kStagesAll,
    perStageLimitType: 'storage-buffer',
    maxDynamicCount: 4
  },
  'sampler': {
    type: 'sampler',
    validStages: kStagesAll,
    perStageLimitType: 'sampler',
    maxDynamicCount: 0
  },
  'comparison-sampler': {
    type: 'sampler',
    validStages: kStagesAll,
    perStageLimitType: 'sampler',
    maxDynamicCount: 0
  },
  'sampled-texture': {
    type: 'texture',
    validStages: kStagesAll,
    perStageLimitType: 'sampled-texture',
    maxDynamicCount: 0
  },
  'writeonly-storage-texture': {
    type: 'texture',
    validStages: kStagesCompute,
    perStageLimitType: 'storage-texture',
    maxDynamicCount: 0
  },
  'readonly-storage-texture': {
    type: 'texture',
    validStages: kStagesAll,
    perStageLimitType: 'storage-texture',
    maxDynamicCount: 0
  }
};
export const kBindingTypes = Object.keys(kBindingTypeInfo);
export const kShaderStages = [C.ShaderStage.Vertex, C.ShaderStage.Fragment, C.ShaderStage.Compute];
export const kShaderStageCombinations = [0, 1, 2, 3, 4, 5, 6, 7];
//# sourceMappingURL=capability_info.js.map