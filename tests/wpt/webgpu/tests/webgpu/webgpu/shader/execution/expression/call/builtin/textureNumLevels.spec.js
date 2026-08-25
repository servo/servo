/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution tests for the 'textureNumLevels' builtin function

Returns the number of mip levels of a texture.
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { AllFeaturesMaxLimitsGPUTest } from '../../../../../gpu_test.js';
import { getTextureDimensionFromView } from '../../../../../util/texture/base.js';
import { kShaderStages } from '../../../../validation/decl/util.js';

import { executeTextureQueryAndExpectResult, kSampleTypeInfo } from './texture_utils.js';

function getLevelSettingsAndExpected(viewType, mipLevelCount) {
  return viewType === 'partial' ?
  {
    baseMipLevel: 1,
    mipLevelCount: 2,
    expected: [2]
  } :
  {
    baseMipLevel: 0,
    mipLevelCount,
    expected: [mipLevelCount]
  };
}

const kTextureTypeToViewDimension = {
  texture_1d: '1d',
  texture_2d: '2d',
  texture_2d_array: '2d-array',
  texture_3d: '3d',
  texture_cube: 'cube',
  texture_cube_array: 'cube-array',
  texture_depth_2d: '2d',
  texture_depth_2d_array: '2d-array',
  texture_depth_cube: 'cube',
  texture_depth_cube_array: 'cube-array'
};

export const g = makeTestGroup(AllFeaturesMaxLimitsGPUTest);

g.test('sampled').
specURL('https://www.w3.org/TR/WGSL/#texturenumlevels').
desc(
  `
T, a sampled type.

fn textureNumLevels(t: texture_1d<T>) -> u32
fn textureNumLevels(t: texture_2d<T>) -> u32
fn textureNumLevels(t: texture_2d_array<T>) -> u32
fn textureNumLevels(t: texture_3d<T>) -> u32
fn textureNumLevels(t: texture_cube<T>) -> u32
fn textureNumLevels(t: texture_cube_array<T>) -> u32

Parameters
 * t The sampled array texture.
`
).
params((u) =>
u.
combine('texture_type', [
'texture_1d',
'texture_2d',
'texture_2d_array',
'texture_3d',
'texture_cube',
'texture_cube_array']
).
beginSubcases().
combine('stage', kShaderStages).
combine('sampled_type', ['f32', 'i32', 'u32']).
combine('view_type', ['full', 'partial'])
// 1d textures can't have mipLevelCount > 0
.filter((t) => t.texture_type !== 'texture_1d' || t.view_type !== 'partial')
).
fn((t) => {
  const { stage, texture_type, sampled_type, view_type } = t.params;
  const { format } = kSampleTypeInfo[sampled_type];
  t.skipIfTextureViewDimensionNotSupported(kTextureTypeToViewDimension[t.params.texture_type]);

  const viewDimension = kTextureTypeToViewDimension[texture_type];
  const dimension = getTextureDimensionFromView(viewDimension);
  const isCube = texture_type.includes('cube');
  const width = 64;
  const height = dimension === '1d' ? 1 : width;
  const depthOrArrayLayers = isCube ? 6 : 1;
  const mipCount = dimension === '1d' ? 1 : 4;
  const texture = t.createTextureTracked({
    format,
    dimension,
    ...(t.isCompatibility && { textureBindingViewDimension: viewDimension }),
    usage: GPUTextureUsage.TEXTURE_BINDING,
    size: {
      width,
      height,
      depthOrArrayLayers
    },
    mipLevelCount: mipCount
  });

  const code = `
@group(0) @binding(0) var t: ${texture_type}<${sampled_type}>;
@group(0) @binding(1) var<storage, read_write> result: u32;
fn getValue() -> u32 {
  return textureNumLevels(t);
}
    `;

  const { baseMipLevel, mipLevelCount, expected } = getLevelSettingsAndExpected(
    view_type,
    mipCount
  );
  const viewDescription = {
    dimension: viewDimension,
    baseMipLevel,
    mipLevelCount
  };

  executeTextureQueryAndExpectResult(t, stage, code, texture, viewDescription, expected);
});

g.test('depth').
specURL('https://www.w3.org/TR/WGSL/#texturenumlevels').
desc(
  `
fn textureNumLevels(t: texture_depth_2d) -> u32
fn textureNumLevels(t: texture_depth_2d_array) -> u32
fn textureNumLevels(t: texture_depth_cube) -> u32
fn textureNumLevels(t: texture_depth_cube_array) -> u32

Parameters
 * t The depth array texture.
`
).
params((u) =>
u.
combine('texture_type', [
'texture_depth_2d',
'texture_depth_2d_array',
'texture_depth_cube',
'texture_depth_cube_array']
).
combine('view_type', ['full', 'partial']).
beginSubcases().
combine('stage', kShaderStages)
).
fn((t) => {
  const { stage, texture_type, view_type } = t.params;
  t.skipIfTextureViewDimensionNotSupported(kTextureTypeToViewDimension[t.params.texture_type]);

  const viewDimension = kTextureTypeToViewDimension[texture_type];
  const dimension = getTextureDimensionFromView(viewDimension);
  const isCube = texture_type.includes('cube');
  const width = 64;
  const height = dimension === '1d' ? 1 : width;
  const depthOrArrayLayers = isCube ? 6 : 1;
  const mipCount = dimension === '1d' ? 1 : 4;
  const texture = t.createTextureTracked({
    format: 'depth32float',
    dimension,
    ...(t.isCompatibility && { textureBindingViewDimension: viewDimension }),
    usage: GPUTextureUsage.TEXTURE_BINDING,
    size: {
      width,
      height,
      depthOrArrayLayers
    },
    mipLevelCount: mipCount
  });

  const code = `
@group(0) @binding(0) var t: ${texture_type};
@group(0) @binding(1) var<storage, read_write> result: u32;
fn getValue() -> u32 {
  return textureNumLevels(t);
}
    `;

  const { baseMipLevel, mipLevelCount, expected } = getLevelSettingsAndExpected(
    view_type,
    mipCount
  );
  const viewDescription = {
    dimension: viewDimension,
    baseMipLevel,
    mipLevelCount
  };

  executeTextureQueryAndExpectResult(t, stage, code, texture, viewDescription, expected);
});