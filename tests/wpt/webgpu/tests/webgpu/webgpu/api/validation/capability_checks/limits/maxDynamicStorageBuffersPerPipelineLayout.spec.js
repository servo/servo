/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { range } from '../../../../../common/util/util.js';import { kShaderStageCombinationsWithStage } from '../../../../capability_info.js';import { GPUConst } from '../../../../constants.js';

import { kMaximumLimitBaseParams, makeLimitTestGroup } from './limit_utils.js';

const kExtraLimits = {
  maxBindingsPerBindGroup: 'adapterLimit',
  maxBindGroups: 'adapterLimit',
  maxStorageBuffersPerShaderStage: 'adapterLimit',
  maxStorageBuffersInFragmentStage: 'adapterLimit',
  maxStorageBuffersInVertexStage: 'adapterLimit'
};

const limit = 'maxDynamicStorageBuffersPerPipelineLayout';
export const { g, description } = makeLimitTestGroup(limit);

g.test('createBindGroupLayout,at_over').
desc(`Test using createBindGroupLayout at and over ${limit} limit`).
params(
  kMaximumLimitBaseParams.
  combine('visibility', kShaderStageCombinationsWithStage).
  combine('type', ['storage', 'read-only-storage']).
  filter(
    ({ visibility, type }) =>
    (visibility & GPUConst.ShaderStage.VERTEX) === 0 || type !== 'storage'
  )
).
fn(async (t) => {
  const { limitTest, testValueName, visibility, type } = t.params;
  await t.testDeviceWithRequestedMaximumLimits(
    limitTest,
    testValueName,
    async ({ device, testValue, shouldError }) => {
      t.skipIfNotEnoughStorageBuffersInStage(visibility, testValue);
      shouldError ||= testValue > t.device.limits.maxStorageBuffersPerShaderStage;
      await t.expectValidationError(() => {
        device.createBindGroupLayout({
          entries: range(testValue, (i) => ({
            binding: i,
            visibility,
            buffer: {
              type,
              hasDynamicOffset: true
            }
          }))
        });
      }, shouldError);
    },
    kExtraLimits
  );
});

g.test('createPipelineLayout,at_over').
desc(`Test using at and over ${limit} limit in createPipelineLayout`).
params(
  kMaximumLimitBaseParams.combine('type', [
  'storage',
  'read-only-storage']
  )
).
fn(async (t) => {
  const { limitTest, testValueName, type } = t.params;

  await t.testDeviceWithRequestedMaximumLimits(
    limitTest,
    testValueName,
    async ({ device, testValue, shouldError, actualLimit }) => {
      // We need to make the largest binding groups we can that don't exceed maxDynamicStorageBuffersPerPipelineLayout
      // otherwise, createBindGroupLayout will fail.
      const maxComputeBindings = Math.min(
        device.limits.maxStorageBuffersPerShaderStage,
        actualLimit
      );
      const maxFragmentBindings = Math.min(
        device.limits.maxStorageBuffersInFragmentStage ?? maxComputeBindings,
        actualLimit
      );
      // read-write storage buffers are not allowed in vertex stages.
      const maxVertexBindings =
      type === 'storage' ?
      0 :
      Math.min(
        device.limits.maxStorageBuffersInVertexStage ?? maxComputeBindings,
        actualLimit
      );

      const totalBindings = maxComputeBindings + maxFragmentBindings + maxVertexBindings;
      t.skipIf(
        totalBindings < testValue,
        `total storage buffer bindings across stages (${totalBindings}) < testValue(${testValue})`
      );

      // These are ordered by their stage visibility bits
      const maxBindingsPerStage = [maxVertexBindings, maxFragmentBindings, maxComputeBindings];

      // Make 3 groups using the max bindings allowed for that stage up to testValue bindings
      let numBindingsAvailable = testValue;
      const bindGroupLayouts = maxBindingsPerStage.map((maxBindings, visibilityBit) => {
        const numInGroup = Math.min(numBindingsAvailable, maxBindings);
        numBindingsAvailable -= numInGroup;
        t.debug(`group(${visibilityBit}) numBindings: ${numInGroup}`);

        return device.createBindGroupLayout({
          entries: range(numInGroup, (i) => ({
            binding: i,
            visibility: 1 << visibilityBit,
            buffer: {
              type,
              hasDynamicOffset: true
            }
          }))
        });
      });

      await t.expectValidationError(
        () => device.createPipelineLayout({ bindGroupLayouts }),
        shouldError
      );
    },
    kExtraLimits
  );
});