/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { kRenderEncoderTypes, kMaximumLimitBaseParams, makeLimitTestGroup } from './limit_utils.js';function getPipelineDescriptor(device, testValue) {
  const module = device.createShaderModule({
    code: `
      @vertex fn vs(@location(0) p: vec4f) -> @builtin(position) vec4f {
        return p;
      }`
  });
  const buffers = new Array(testValue);
  buffers[testValue - 1] = {
    arrayStride: 16,
    attributes: [{ shaderLocation: 0, offset: 0, format: 'float32' }]
  };

  return {
    layout: 'auto',
    vertex: {
      module,
      buffers
    }
  };
}

const limit = 'maxVertexBuffers';
export const { g, description } = makeLimitTestGroup(limit);

g.test('createRenderPipeline,at_over').
desc(`Test using at and over ${limit} limit in createRenderPipeline(Async)`).
params(kMaximumLimitBaseParams.combine('async', [false, true])).
fn(async (t) => {
  const { limitTest, testValueName, async } = t.params;
  await t.testDeviceWithRequestedMaximumLimits(
    limitTest,
    testValueName,
    async ({ device, testValue, shouldError, actualLimit }) => {
      const pipelineDescriptor = getPipelineDescriptor(device, testValue);
      const lastIndex = testValue - 1;

      await t.testCreateRenderPipeline(
        pipelineDescriptor,
        async,
        shouldError,
        `lastIndex: ${lastIndex}, actualLimit: ${actualLimit}, shouldError: ${shouldError}`
      );
    }
  );
});

g.test('setVertexBuffer,at_over').
desc(`Test using at and over ${limit} limit in setVertexBuffer`).
params(kMaximumLimitBaseParams.combine('encoderType', kRenderEncoderTypes)).
fn(async (t) => {
  const { limitTest, testValueName, encoderType } = t.params;
  await t.testDeviceWithRequestedMaximumLimits(
    limitTest,
    testValueName,
    async ({ testValue, shouldError, actualLimit }) => {
      const lastIndex = testValue - 1;

      const buffer = t.createBufferTracked({
        size: 16,
        usage: GPUBufferUsage.VERTEX
      });

      await t.testGPURenderAndBindingCommandsMixin(
        encoderType,
        ({ passEncoder }) => {
          passEncoder.setVertexBuffer(lastIndex, buffer);
        },
        shouldError,
        `lastIndex: ${lastIndex}, actualLimit: ${actualLimit}, shouldError: ${shouldError}`
      );

      buffer.destroy();
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