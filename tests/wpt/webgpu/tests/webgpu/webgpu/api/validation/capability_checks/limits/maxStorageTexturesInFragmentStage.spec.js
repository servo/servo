/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { range, reorder,
  kReorderOrderKeys,

  assert } from
'../../../../../common/util/util.js';
import { kStorageTextureAccessValues } from '../../../../capability_info.js';

import {
  kMaximumLimitBaseParams,
  makeLimitTestGroup,
  kBindGroupTests,
  getPipelineTypeForBindingCombination,
  getPerStageWGSLForBindingCombination,

  getStageVisibilityForBindingCombination,
  testMaxStorageXXXInYYYStageDeviceCreationWithDependentLimit } from
'./limit_utils.js';

const limit = 'maxStorageTexturesInFragmentStage';

const kExtraLimits = {
  maxBindingsPerBindGroup: 'adapterLimit',
  maxBindGroups: 'adapterLimit'
};

export const { g, description } = makeLimitTestGroup(limit, {
  // MAINTAINANCE_TODO: remove once this limit is required.
  limitOptional: true
});

function createBindGroupLayout(
device,
visibility,
access,
order,
numBindings)
{
  const bindGroupLayoutDescription = {
    entries: reorder(
      order,
      range(numBindings, (i) => ({
        binding: i,
        visibility,
        storageTexture: { format: 'r32float', access }
      }))
    )
  };
  return device.createBindGroupLayout(bindGroupLayoutDescription);
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
  combine('access', kStorageTextureAccessValues).
  combine('order', kReorderOrderKeys)
).
fn(async (t) => {
  const { limitTest, testValueName, order, access } = t.params;

  await t.testDeviceWithRequestedMaximumLimits(
    limitTest,
    testValueName,
    async ({ device, testValue, shouldError }) => {
      t.skipIf(
        t.adapter.limits.maxBindingsPerBindGroup < testValue,
        `maxBindingsPerBindGroup = ${t.adapter.limits.maxBindingsPerBindGroup} which is less than ${testValue}`
      );

      const visibility = GPUShaderStage.FRAGMENT;
      await t.expectValidationError(() => {
        createBindGroupLayout(device, visibility, access, order, testValue);
      }, shouldError);
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
  combine('access', kStorageTextureAccessValues).
  combine('order', kReorderOrderKeys)
).
fn(async (t) => {
  const { limitTest, testValueName, order, access } = t.params;

  await t.testDeviceWithRequestedMaximumLimits(
    limitTest,
    testValueName,
    async ({ device, testValue, shouldError, actualLimit }) => {
      const visibility = GPUShaderStage.FRAGMENT;

      t.skipIf(
        actualLimit === 0,
        `can not make a bindGroupLayout to test createPipelineLaoyout if the actaul limit is 0`
      );

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
  beginSubcases().
  combine('order', kReorderOrderKeys).
  combine('bindGroupTest', kBindGroupTests)
).
fn(async (t) => {
  const { limitTest, testValueName, async, order, bindGroupTest } = t.params;
  const bindingCombination = 'fragment';
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
      t.skipIfNotEnoughStorageBuffersInStage(visibility, testValue);

      const code = getPerStageWGSLForBindingCombination(
        bindingCombination,
        order,
        bindGroupTest,
        (i, j) => `var u${j}_${i}: texture_storage_2d<r32float,read>`,
        (i, j) => `_ = u${j}_${i};`,
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

testMaxStorageXXXInYYYStageDeviceCreationWithDependentLimit(
  g,
  limit,
  'maxStorageTexturesPerShaderStage'
);