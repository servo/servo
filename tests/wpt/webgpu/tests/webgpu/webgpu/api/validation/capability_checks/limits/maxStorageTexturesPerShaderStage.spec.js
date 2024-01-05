/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { range, reorder,

  kReorderOrderKeys } from
'../../../../../common/util/util.js';
import { GPUConst } from '../../../../constants.js';

import {
  kMaximumLimitBaseParams,
  makeLimitTestGroup,
  kBindGroupTests,
  getPerStageWGSLForBindingCombinationStorageTextures,
  getPipelineTypeForBindingCombination } from

'./limit_utils.js';

const limit = 'maxStorageTexturesPerShaderStage';
export const { g, description } = makeLimitTestGroup(limit);

function createBindGroupLayout(
device,
visibility,
order,
numBindings)
{
  return device.createBindGroupLayout({
    entries: reorder(
      order,
      range(numBindings, (i) => ({
        binding: i,
        visibility,
        storageTexture: { format: 'rgba8unorm' }
      }))
    )
  });
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
  combine('visibility', [
  GPUConst.ShaderStage.FRAGMENT,
  GPUConst.ShaderStage.COMPUTE,
  GPUConst.ShaderStage.FRAGMENT | GPUConst.ShaderStage.COMPUTE]
  ).
  combine('order', kReorderOrderKeys)
).
fn(async (t) => {
  const { limitTest, testValueName, visibility, order } = t.params;
  await t.testDeviceWithRequestedMaximumLimits(
    limitTest,
    testValueName,
    async ({ device, testValue, shouldError }) => {
      await t.expectValidationError(
        () => createBindGroupLayout(device, visibility, order, testValue),
        shouldError
      );
    }
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
  combine('visibility', [
  GPUConst.ShaderStage.FRAGMENT,
  GPUConst.ShaderStage.COMPUTE,
  GPUConst.ShaderStage.FRAGMENT | GPUConst.ShaderStage.COMPUTE]
  ).
  combine('order', kReorderOrderKeys)
).
fn(async (t) => {
  const { limitTest, testValueName, visibility, order } = t.params;
  await t.testDeviceWithRequestedMaximumLimits(
    limitTest,
    testValueName,
    async ({ device, testValue, shouldError }) => {
      const kNumGroups = 3;
      const bindGroupLayouts = range(kNumGroups, (i) => {
        const minInGroup = Math.floor(testValue / kNumGroups);
        const numInGroup = i ? minInGroup : testValue - minInGroup * (kNumGroups - 1);
        return createBindGroupLayout(device, visibility, order, numInGroup);
      });
      await t.expectValidationError(
        () => device.createPipelineLayout({ bindGroupLayouts }),
        shouldError
      );
    }
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
  combine('bindingCombination', ['fragment', 'compute']).
  combine('order', kReorderOrderKeys).
  combine('bindGroupTest', kBindGroupTests)
).
fn(async (t) => {
  const { limitTest, testValueName, async, bindingCombination, order, bindGroupTest } = t.params;
  const pipelineType = getPipelineTypeForBindingCombination(bindingCombination);

  await t.testDeviceWithRequestedMaximumLimits(
    limitTest,
    testValueName,
    async ({ device, testValue, actualLimit, shouldError }) => {
      if (bindingCombination === 'fragment') {
        return;
      }

      const code = getPerStageWGSLForBindingCombinationStorageTextures(
        bindingCombination,
        order,
        bindGroupTest,
        (i, j) => `var u${j}_${i}: texture_storage_2d<rgba8unorm, write>`,
        (i, j) => `textureStore(u${j}_${i}, vec2u(0), vec4f(1));`,
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
    }
  );
});