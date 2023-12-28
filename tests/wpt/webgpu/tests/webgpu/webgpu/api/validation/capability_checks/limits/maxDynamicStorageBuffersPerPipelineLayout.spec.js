/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { range } from '../../../../../common/util/util.js';import { GPUConst } from '../../../../constants.js';
import { kMaximumLimitBaseParams, makeLimitTestGroup } from './limit_utils.js';

const limit = 'maxDynamicStorageBuffersPerPipelineLayout';
export const { g, description } = makeLimitTestGroup(limit);

g.test('createBindGroupLayout,at_over').
desc(`Test using createBindGroupLayout at and over ${limit} limit`).
params(
  kMaximumLimitBaseParams.combine('visibility', [
  GPUConst.ShaderStage.FRAGMENT,
  GPUConst.ShaderStage.COMPUTE,
  GPUConst.ShaderStage.COMPUTE | GPUConst.ShaderStage.FRAGMENT]
  )
).
fn(async (t) => {
  const { limitTest, testValueName, visibility } = t.params;
  await t.testDeviceWithRequestedMaximumLimits(
    limitTest,
    testValueName,
    async ({ device, testValue, shouldError }) => {
      shouldError ||= testValue > t.device.limits.maxStorageBuffersPerShaderStage;
      await t.expectValidationError(() => {
        device.createBindGroupLayout({
          entries: range(testValue, (i) => ({
            binding: i,
            visibility,
            buffer: {
              type: 'storage',
              hasDynamicOffset: true
            }
          }))
        });
      }, shouldError);
    }
  );
});