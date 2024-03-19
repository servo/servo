/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { range, reorder,

  kReorderOrderKeys,
  assert } from
'../../../../../common/util/util.js';
import { GPUConst } from '../../../../constants.js';

import {
  kMaximumLimitBaseParams,
  makeLimitTestGroup,
  kBindGroupTests,
  getPerStageWGSLForBindingCombinationStorageTextures,
  getPipelineTypeForBindingCombination } from


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
      t.skipIf(
        t.adapter.limits.maxBindingsPerBindGroup < testValue,
        `maxBindingsPerBindGroup = ${t.adapter.limits.maxBindingsPerBindGroup} which is less than ${testValue}`
      );

      await t.expectValidationError(
        () => createBindGroupLayout(device, visibility, order, testValue),
        shouldError
      );
    },
    kExtraLimits
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
    async ({ device, testValue, shouldError, actualLimit }) => {
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
        return createBindGroupLayout(device, visibility, order, numInGroup);
      });

      await t.expectValidationError(
        () => device.createPipelineLayout({ bindGroupLayouts }),
        shouldError
      );
    },
    kExtraLimits
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
      t.skipIf(
        bindGroupTest === 'sameGroup' && testValue > device.limits.maxBindingsPerBindGroup,
        `can not test ${testValue} bindings in same group because maxBindingsPerBindGroup = ${device.limits.maxBindingsPerBindGroup}`
      );

      if (bindingCombination === 'fragment') {
        return;
      }

      const code = getPerStageWGSLForBindingCombinationStorageTextures(
        bindingCombination,
        order,
        bindGroupTest,
        (i, j) => `var u${j}_${i}: texture_storage_2d<rgba8unorm, write>`,
        (i, j) => `textureStore(u${j}_${i}, vec2u(0), vec4f(1));`,
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
    kExtraLimits
  );
});