/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { kMaximumLimitBaseParams, makeLimitTestGroup } from './limit_utils.js';function getPipelineDescriptor(device, lastIndex) {
  const code = `
  @vertex fn vs(@location(${lastIndex}) v: vec4f) -> @builtin(position) vec4f {
    return v;
  }
  `;
  const module = device.createShaderModule({ code });
  return {
    layout: 'auto',
    vertex: {
      module,
      entryPoint: 'vs',
      buffers: [
      {
        arrayStride: 32,
        attributes: [{ shaderLocation: lastIndex, offset: 0, format: 'float32x4' }]
      }]

    },
    depthStencil: { format: 'depth32float', depthWriteEnabled: true, depthCompare: 'always' }
  };
}

const limit = 'maxVertexAttributes';
export const { g, description } = makeLimitTestGroup(limit);

g.test('createRenderPipeline,at_over').
desc(`Test using createRenderPipeline(Async) at and over ${limit} limit`).
params(kMaximumLimitBaseParams.combine('async', [false, true])).
fn(async (t) => {
  const { limitTest, testValueName, async } = t.params;
  await t.testDeviceWithRequestedMaximumLimits(
    limitTest,
    testValueName,
    async ({ device, testValue, shouldError }) => {
      const lastIndex = testValue - 1;
      const pipelineDescriptor = getPipelineDescriptor(device, lastIndex);

      await t.testCreateRenderPipeline(pipelineDescriptor, async, shouldError);
    }
  );
});