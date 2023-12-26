/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { range } from '../../../../../common/util/util.js';import {
  kCreatePipelineTypes,
  kEncoderTypes,
  kMaximumLimitBaseParams,
  makeLimitTestGroup } from
'./limit_utils.js';

const limit = 'maxBindGroups';
export const { g, description } = makeLimitTestGroup(limit);

g.test('createPipelineLayout,at_over').
desc(`Test using createPipelineLayout at and over ${limit} limit`).
params(kMaximumLimitBaseParams).
fn(async (t) => {
  const { limitTest, testValueName } = t.params;
  await t.testDeviceWithRequestedMaximumLimits(
    limitTest,
    testValueName,
    async ({ device, testValue, shouldError }) => {
      const bindGroupLayouts = range(testValue, (_i) =>
      device.createBindGroupLayout({
        entries: [
        {
          binding: 0,
          visibility: GPUShaderStage.VERTEX,
          buffer: {}
        }]

      })
      );

      await t.expectValidationError(() => {
        device.createPipelineLayout({ bindGroupLayouts });
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

      const code = t.getGroupIndexWGSLForPipelineType(createPipelineType, lastIndex);
      const module = device.createShaderModule({ code });

      await t.testCreatePipeline(createPipelineType, async, module, shouldError);
    }
  );
});

g.test('setBindGroup,at_over').
desc(`Test using setBindGroup at and over ${limit} limit`).
params(kMaximumLimitBaseParams.combine('encoderType', kEncoderTypes)).
fn(async (t) => {
  const { limitTest, testValueName, encoderType } = t.params;
  await t.testDeviceWithRequestedMaximumLimits(
    limitTest,
    testValueName,
    async ({ testValue, actualLimit, shouldError }) => {
      const lastIndex = testValue - 1;
      await t.testGPUBindingCommandsMixin(
        encoderType,
        ({ mixin, bindGroup }) => {
          mixin.setBindGroup(lastIndex, bindGroup);
        },
        shouldError,
        `shouldError: ${shouldError}, actualLimit: ${actualLimit}, testValue: ${lastIndex}`
      );
    }
  );
});

g.test('validate,maxBindGroupsPlusVertexBuffers').
desc(`Test that ${limit} <= maxBindGroupsPlusVertexBuffers`).
fn((t) => {
  const { adapter, defaultLimit, adapterLimit } = t;
  t.expect(defaultLimit <= t.getDefaultLimit('maxBindGroupsPlusVertexBuffers'));
  t.expect(adapterLimit <= adapter.limits.maxBindGroupsPlusVertexBuffers);
});