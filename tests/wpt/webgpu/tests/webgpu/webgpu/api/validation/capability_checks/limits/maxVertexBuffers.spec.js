/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { range } from '../../../../../common/util/util.js';import { kRenderEncoderTypes, kMaximumLimitBaseParams, makeLimitTestGroup } from './limit_utils.js';

const kPipelineTypes = ['withoutLocations', 'withLocations'];


function getPipelineDescriptor(
device,
pipelineType,
testValue)
{
  const code =
  pipelineType === 'withLocations' ?
  `
        struct VSInput {
          ${range(testValue, (i) => `@location(${i}) p${i}: f32,`).join('\n')}
        }
        @vertex fn vs(v: VSInput) -> @builtin(position) vec4f {
          let x = ${range(testValue, (i) => `v.p${i}`).join(' + ')};
          return vec4f(x, 0, 0, 1);
        }
        ` :
  `
        @vertex fn vs() -> @builtin(position) vec4f {
          return vec4f(0);
        }
        `;
  const module = device.createShaderModule({ code });
  return {
    layout: 'auto',
    vertex: {
      module,
      entryPoint: 'vs',
      buffers: range(testValue, (i) => ({
        arrayStride: 32,
        attributes: [{ shaderLocation: i, offset: 0, format: 'float32' }]
      }))
    }
  };
}

const limit = 'maxVertexBuffers';
export const { g, description } = makeLimitTestGroup(limit);

g.test('createRenderPipeline,at_over').
desc(`Test using at and over ${limit} limit in createRenderPipeline(Async)`).
params(
  kMaximumLimitBaseParams.combine('async', [false, true]).combine('pipelineType', kPipelineTypes)
).
fn(async (t) => {
  const { limitTest, testValueName, async, pipelineType } = t.params;
  await t.testDeviceWithRequestedMaximumLimits(
    limitTest,
    testValueName,
    async ({ device, testValue, shouldError }) => {
      const pipelineDescriptor = getPipelineDescriptor(device, pipelineType, testValue);

      await t.testCreateRenderPipeline(pipelineDescriptor, async, shouldError);
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
    async ({ device, testValue, shouldError, actualLimit }) => {
      const lastIndex = testValue - 1;

      const buffer = device.createBuffer({
        size: 16,
        usage: GPUBufferUsage.VERTEX
      });

      await t.testGPURenderCommandsMixin(
        encoderType,
        ({ mixin }) => {
          mixin.setVertexBuffer(lastIndex, buffer);
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