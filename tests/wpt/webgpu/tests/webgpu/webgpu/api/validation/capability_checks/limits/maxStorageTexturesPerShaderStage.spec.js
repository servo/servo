/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { getGPU } from '../../../../../common/util/navigator_gpu.js';import { range,
reorder,

kReorderOrderKeys,
assert } from
'../../../../../common/util/util.js';
import {
  kShaderStageCombinationsWithStage,
  kStorageTextureAccessValues,
  storageTextureBindingTypeInfo } from
'../../../../capability_info.js';
import { GPUConst } from '../../../../constants.js';

import {
  kMaximumLimitBaseParams,
  makeLimitTestGroup,
  kBindGroupTests,
  getPerStageWGSLForBindingCombinationStorageTextures,
  getPipelineTypeForBindingCombination,


  kBindingCombinations,
  getStageVisibilityForBindingCombination,

  addMaximumLimitUpToDependentLimit } from
'./limit_utils.js';

const kExtraLimits = {
  maxBindingsPerBindGroup: 'adapterLimit',
  maxBindGroups: 'adapterLimit'
};

const limit = 'maxStorageTexturesPerShaderStage';
export const { g, description } = makeLimitTestGroup(limit);

function createBindGroupLayout(
device,
visibility,
access,
order,
numBindings)
{
  return device.createBindGroupLayout({
    entries: reorder(
      order,
      range(numBindings, (i) => ({
        binding: i,
        visibility,
        storageTexture: { format: 'r32float', access }
      }))
    )
  });
}

function skipIfNotEnoughStorageTexturesInStage(
t,
visibility,
testValue)
{
  t.skipIf(
    t.isCompatibility &&
    // If we're using the fragment stage
    (visibility & GPUConst.ShaderStage.FRAGMENT) !== 0 &&
    // If perShaderStage and inFragment stage are equal we want to
    // allow the test to run as otherwise we can't test overMaximum and overLimit
    t.device.limits.maxStorageTexturesPerShaderStage >
    t.device.limits.maxStorageTexturesInFragmentStage &&
    // They aren't equal so if there aren't enough supported in the fragment then skip
    !(t.device.limits.maxStorageTexturesInFragmentStage >= testValue),
    `maxStorageTexturesInFragmentShader = ${t.device.limits.maxStorageTexturesInFragmentStage} which is less than ${testValue}`
  );

  t.skipIf(
    t.isCompatibility &&
    // If we're using the vertex stage
    (visibility & GPUConst.ShaderStage.VERTEX) !== 0 &&
    // If perShaderStage and inVertex stage are equal we want to
    // allow the test to run as otherwise we can't test overMaximum and overLimit
    t.device.limits.maxStorageTexturesPerShaderStage >
    t.device.limits.maxStorageTexturesInVertexStage &&
    // They aren't equal so if there aren't enough supported in the vertex then skip
    !(t.device.limits.maxStorageTexturesInVertexStage >= testValue),
    `maxStorageTexturesInVertexShader = ${t.device.limits.maxStorageTexturesInVertexStage} which is less than ${testValue}`
  );
}

function skipIfAccessNotSupported(t, access) {
  t.skipIf(
    (access === 'read-only' || access === 'read-write') &&
    !getGPU(t.rec).wgslLanguageFeatures.has('readonly_and_readwrite_storage_textures'),
    `access = ${access} but navigator.gpu.wsglLanguageFeatures does not contain 'readonly_and_readwrite_storage_textures'`
  );
}

function filterWriteAccessInVertexStage(
visibility,
access)
{
  return access === 'read-only' || (visibility & GPUConst.ShaderStage.VERTEX) === 0;
}

function addExtraRequiredLimits(
adapter,
limits,
limitTest)
{
  const newLimits = { ...limits };

  addMaximumLimitUpToDependentLimit(
    adapter,
    newLimits,
    'maxStorageTexturesInFragmentStage',
    limit,
    limitTest
  );
  addMaximumLimitUpToDependentLimit(
    adapter,
    newLimits,
    'maxStorageTexturesInVertexStage',
    limit,
    limitTest
  );

  return newLimits;
}
g.test('createBindGroupLayout,at_over').
desc(
  `
  Test using at and over ${limit} limit in createBindGroupLayout

  Note: We also test order to make sure the implementation isn't just looking
  at just the last entry.
  `
).
params(
  kMaximumLimitBaseParams.
  combine('visibility', kShaderStageCombinationsWithStage).
  combine('access', kStorageTextureAccessValues).
  filter((t) => filterWriteAccessInVertexStage(t.visibility, t.access)).
  combine('order', kReorderOrderKeys)
).
fn(async (t) => {
  const { limitTest, testValueName, visibility, access, order } = t.params;

  skipIfAccessNotSupported(t, access);

  await t.testDeviceWithRequestedMaximumLimits(
    limitTest,
    testValueName,
    async ({ device, testValue, shouldError }) => {
      t.skipIf(
        t.adapter.limits.maxBindingsPerBindGroup < testValue,
        `maxBindingsPerBindGroup = ${t.adapter.limits.maxBindingsPerBindGroup} which is less than ${testValue}`
      );
      skipIfNotEnoughStorageTexturesInStage(t, visibility, testValue);
      await t.expectValidationError(
        () => createBindGroupLayout(device, visibility, access, order, testValue),
        shouldError
      );
    },
    addExtraRequiredLimits(t.adapter, kExtraLimits, limitTest)
  );
});

g.test('createPipelineLayout,at_over').
desc(
  `
  Test using at and over ${limit} limit in createPipelineLayout

  Note: We also test order to make sure the implementation isn't just looking
  at just the last entry.
  `
).
params(
  kMaximumLimitBaseParams.
  combine('visibility', kShaderStageCombinationsWithStage).
  combine('access', kStorageTextureAccessValues).
  filter((t) => filterWriteAccessInVertexStage(t.visibility, t.access)).
  combine('order', kReorderOrderKeys)
).
fn(async (t) => {
  const { limitTest, testValueName, visibility, access, order } = t.params;

  skipIfAccessNotSupported(t, access);

  await t.testDeviceWithRequestedMaximumLimits(
    limitTest,
    testValueName,
    async ({ device, testValue, shouldError, actualLimit }) => {
      skipIfNotEnoughStorageTexturesInStage(t, visibility, testValue);

      const maxBindingsPerBindGroup = Math.min(
        t.device.limits.maxBindingsPerBindGroup,
        actualLimit
      );
      const kNumGroups = Math.ceil(testValue / maxBindingsPerBindGroup);

      // Not sure what to do in this case but best we get notified if it happens.
      assert(kNumGroups <= t.device.limits.maxBindGroups);

      const bindGroupLayouts = range(kNumGroups, (i) => {
        const numInGroup = Math.min(
          testValue - i * maxBindingsPerBindGroup,
          maxBindingsPerBindGroup
        );
        return createBindGroupLayout(device, visibility, access, order, numInGroup);
      });

      await t.expectValidationError(
        () => device.createPipelineLayout({ bindGroupLayouts }),
        shouldError
      );
    },
    addExtraRequiredLimits(t.adapter, kExtraLimits, limitTest)
  );
});

g.test('createPipeline,at_over').
desc(
  `
  Test using createRenderPipeline(Async) and createComputePipeline(Async) at and over ${limit} limit

  Note: We also test order to make sure the implementation isn't just looking
  at just the last entry.
  `
).
params(
  kMaximumLimitBaseParams.
  combine('async', [false, true]).
  combine('bindingCombination', kBindingCombinations).
  combine('access', kStorageTextureAccessValues).
  filter((t) =>
  filterWriteAccessInVertexStage(
    getStageVisibilityForBindingCombination(t.bindingCombination),
    t.access
  )
  ).
  beginSubcases().
  combine('order', kReorderOrderKeys).
  combine('bindGroupTest', kBindGroupTests)
).
fn(async (t) => {
  const { limitTest, testValueName, async, bindingCombination, access, order, bindGroupTest } =
  t.params;

  skipIfAccessNotSupported(t, access);

  const pipelineType = getPipelineTypeForBindingCombination(bindingCombination);

  await t.testDeviceWithRequestedMaximumLimits(
    limitTest,
    testValueName,
    async ({ device, testValue, actualLimit, shouldError }) => {
      t.skipIf(
        bindGroupTest === 'sameGroup' && testValue > device.limits.maxBindingsPerBindGroup,
        `can not test ${testValue} bindings in same group because maxBindingsPerBindGroup = ${device.limits.maxBindingsPerBindGroup}`
      );

      const visibility = getStageVisibilityForBindingCombination(bindingCombination);
      skipIfNotEnoughStorageTexturesInStage(t, visibility, testValue);

      const { wgslAccess } = storageTextureBindingTypeInfo({ access });

      const code = getPerStageWGSLForBindingCombinationStorageTextures(
        bindingCombination,
        order,
        bindGroupTest,
        (i, j) => `var u${j}_${i}: texture_storage_2d<r32float, ${wgslAccess}>`,
        (i, j) =>
        access === 'write-only' ?
        `textureStore(u${j}_${i}, vec2u(0), vec4f(0));` :
        `_ = textureLoad(u${j}_${i}, vec2u(0));`,
        device.limits.maxBindGroups,
        testValue
      );
      const module = device.createShaderModule({ code });

      await t.testCreatePipeline(
        pipelineType,
        async,
        module,
        shouldError,
        `actualLimit: ${actualLimit}, testValue: ${testValue}\n:${code}`
      );
    },
    addExtraRequiredLimits(t.adapter, kExtraLimits, limitTest)
  );
});