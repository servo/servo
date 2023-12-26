/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { kCreatePipelineTypes, kMaximumLimitBaseParams,
  makeLimitTestGroup } from
'./limit_utils.js';

const limit = 'maxBindingsPerBindGroup';
export const { g, description } = makeLimitTestGroup(limit);

g.test('createBindGroupLayout,at_over').
desc(`Test using createBindGroupLayout at and over ${limit} limit`).
params(kMaximumLimitBaseParams).
fn(async (t) => {
  const { limitTest, testValueName } = t.params;
  await t.testDeviceWithRequestedMaximumLimits(
    limitTest,
    testValueName,
    async ({ device, testValue, shouldError }) => {
      await t.expectValidationError(() => {
        device.createBindGroupLayout({
          entries: [
          {
            binding: testValue - 1,
            visibility: GPUShaderStage.VERTEX,
            buffer: {}
          }]

        });
      }, shouldError);
    }
  );
});

g.test('createPipeline,at_over').
desc(
  `Test using createRenderPipeline(Async) and createComputePipeline(Async) at and over ${limit} limit`
).
params(
  kMaximumLimitBaseParams.
  combine('createPipelineType', kCreatePipelineTypes).
  combine('async', [false, true])
).
fn(async (t) => {
  const { limitTest, testValueName, createPipelineType, async } = t.params;
  await t.testDeviceWithRequestedMaximumLimits(
    limitTest,
    testValueName,
    async ({ device, testValue, shouldError }) => {
      const lastIndex = testValue - 1;

      const code = t.getBindingIndexWGSLForPipelineType(createPipelineType, lastIndex);
      const module = device.createShaderModule({ code });

      await t.testCreatePipeline(createPipelineType, async, module, shouldError, code);
    }
  );
});

g.test('validate').
desc(`Test ${limit} matches the spec limits`).
fn((t) => {
  const { adapter, adapterLimit } = t;
  const maxBindingsPerShaderStage =
  adapter.limits.maxSampledTexturesPerShaderStage +
  adapter.limits.maxSamplersPerShaderStage +
  adapter.limits.maxStorageBuffersPerShaderStage +
  adapter.limits.maxStorageTexturesPerShaderStage +
  adapter.limits.maxUniformBuffersPerShaderStage;
  const maxShaderStagesPerPipeline = 2;
  const minMaxBindingsPerBindGroup = maxBindingsPerShaderStage * maxShaderStagesPerPipeline;
  t.expect(
    adapterLimit >= minMaxBindingsPerBindGroup,
    `maxBindingsPerBindGroup(${adapterLimit}) >= maxBindingsPerShaderStage(${maxBindingsPerShaderStage}) * maxShaderStagesPerPipeline(${maxShaderStagesPerPipeline} = (${minMaxBindingsPerBindGroup}))`
  );
});