/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { range } from '../../../../../common/util/util.js';import { kShaderStageCombinationsWithStage } from '../../../../capability_info.js';
import { kMaximumLimitBaseParams, makeLimitTestGroup } from './limit_utils.js';

const kExtraLimits = {
  maxBindingsPerBindGroup: 'adapterLimit',
  maxBindGroups: 'adapterLimit',
  maxUniformBuffersPerShaderStage: 'adapterLimit'
};

const limit = 'maxDynamicUniformBuffersPerPipelineLayout';
export const { g, description } = makeLimitTestGroup(limit);

g.test('createBindGroupLayout,at_over').
desc(`Test using createBindGroupLayout at and over ${limit} limit`).
params(kMaximumLimitBaseParams.combine('visibility', kShaderStageCombinationsWithStage)).
fn(async (t) => {
  const { limitTest, testValueName, visibility } = t.params;
  await t.testDeviceWithRequestedMaximumLimits(
    limitTest,
    testValueName,
    async ({ device, testValue, shouldError }) => {
      await t.expectValidationError(() => {
        device.createBindGroupLayout({
          entries: range(testValue, (i) => ({
            binding: i,
            visibility,
            buffer: {
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
params(kMaximumLimitBaseParams).
fn(async (t) => {
  const { limitTest, testValueName } = t.params;

  await t.testDeviceWithRequestedMaximumLimits(
    limitTest,
    testValueName,
    async ({ device, testValue, shouldError, actualLimit }) => {
      // We need to make the largest binding groups we can that don't exceed maxDynamicUniformBuffersPerPipelineLayout
      // otherwise, createBindGroupLayout will fail.
      const maxUniformBindings = Math.min(
        device.limits.maxUniformBuffersPerShaderStage,
        actualLimit
      );

      const totalBindings = maxUniformBindings * 3;
      t.skipIf(
        totalBindings < testValue,
        `total uniform buffer bindings across stages (${totalBindings}) < testValue(${testValue})`
      );

      // These are ordered by their stage visibility bits
      const maxBindingsPerStage = [maxUniformBindings, maxUniformBindings, maxUniformBindings];

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