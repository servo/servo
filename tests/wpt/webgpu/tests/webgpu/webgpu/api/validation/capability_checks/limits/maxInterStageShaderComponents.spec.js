/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { range } from '../../../../../common/util/util.js';import { kMaximumLimitBaseParams, makeLimitTestGroup } from './limit_utils.js';

function getPipelineDescriptor(
device,
testValue,
pointList,
frontFacing,
sampleIndex,
sampleMaskIn,
sampleMaskOut)
{
  const success = testValue <= device.limits.maxInterStageShaderComponents;

  const maxVertexOutputComponents =
  device.limits.maxInterStageShaderComponents - (pointList ? 1 : 0);
  const maxFragmentInputComponents =
  device.limits.maxInterStageShaderComponents - (
  frontFacing ? 1 : 0) - (
  sampleIndex ? 1 : 0) - (
  sampleMaskIn ? 1 : 0);
  const maxOutputComponents = Math.min(maxVertexOutputComponents, maxFragmentInputComponents);
  const maxInterStageVariables = Math.floor(maxOutputComponents / 4);
  const maxUserDefinedVertexComponents = Math.floor(maxVertexOutputComponents / 4) * 4;
  const maxUserDefinedFragmentComponents = Math.floor(maxFragmentInputComponents / 4) * 4;

  const numInterStageVariables = success ? maxInterStageVariables : maxInterStageVariables + 1;
  const numUserDefinedComponents = numInterStageVariables * 4;

  const varyings = `
      ${range(numInterStageVariables, (i) => `@location(${i}) v4_${i}: vec4f,`).join('\n')}
  `;

  const code = `
    // test value                        : ${testValue}
    // maxInterStageShaderComponents     : ${device.limits.maxInterStageShaderComponents}
    // num components in vertex shader   : ${numUserDefinedComponents}${
  pointList ? ' + point-list' : ''
  }
    // num components in fragment shader : ${numUserDefinedComponents}${
  frontFacing ? ' + front-facing' : ''
  }${sampleIndex ? ' + sample_index' : ''}${sampleMaskIn ? ' + sample_mask' : ''}
    // maxUserDefinedVertexShaderOutputComponents   : ${maxUserDefinedVertexComponents}
    // maxUserDefinedFragmentShaderInputComponents  : ${maxUserDefinedFragmentComponents}
    // maxInterStageVariables:           : ${maxInterStageVariables}
    // num used inter stage variables    : ${numInterStageVariables}

    struct VSOut {
      @builtin(position) p: vec4f,
      ${varyings}
    }
    struct FSIn {
      ${frontFacing ? '@builtin(front_facing) frontFacing: bool,' : ''}
      ${sampleIndex ? '@builtin(sample_index) sampleIndex: u32,' : ''}
      ${sampleMaskIn ? '@builtin(sample_mask) sampleMask: u32,' : ''}
      ${varyings}
    }
    struct FSOut {
      @location(0) color: vec4f,
      ${sampleMaskOut ? '@builtin(sample_mask) sampleMask: u32,' : ''}
    }
    @vertex fn vs() -> VSOut {
      var o: VSOut;
      o.p = vec4f(0);
      return o;
    }
    @fragment fn fs(i: FSIn) -> FSOut {
      var o: FSOut;
      o.color = vec4f(0);
      return o;
    }
  `;
  const module = device.createShaderModule({ code });
  const pipelineDescriptor = {
    layout: 'auto',
    primitive: {
      topology: pointList ? 'point-list' : 'triangle-list'
    },
    vertex: {
      module,
      entryPoint: 'vs'
    },
    fragment: {
      module,
      entryPoint: 'fs',
      targets: [
      {
        format: 'rgba8unorm'
      }]

    }
  };
  return { pipelineDescriptor, code };
}

const limit = 'maxInterStageShaderComponents';
export const { g, description } = makeLimitTestGroup(limit);

g.test('createRenderPipeline,at_over').
desc(`Test using at and over ${limit} limit in createRenderPipeline(Async)`).
params(
  kMaximumLimitBaseParams.
  combine('async', [false, true]).
  combine('pointList', [false, true]).
  combine('frontFacing', [false, true]).
  combine('sampleIndex', [false, true]).
  combine('sampleMaskIn', [false, true]).
  combine('sampleMaskOut', [false, true])
).
beforeAllSubcases((t) => {
  if (t.isCompatibility) {
    t.skipIf(
      t.params.sampleMaskIn || t.params.sampleMaskOut,
      'sample_mask not supported in compatibility mode'
    );
    t.skipIf(t.params.sampleIndex, 'sample_index not supported in compatibility mode');
  }
}).
fn(async (t) => {
  const {
    limitTest,
    testValueName,
    async,
    pointList,
    frontFacing,
    sampleIndex,
    sampleMaskIn,
    sampleMaskOut
  } = t.params;
  // Request the largest value of maxInterStageShaderVariables to allow the test using as many
  // inter-stage shader components as possible without being limited by
  // maxInterStageShaderVariables.
  const extraLimits = { maxInterStageShaderVariables: 'adapterLimit' };
  await t.testDeviceWithRequestedMaximumLimits(
    limitTest,
    testValueName,
    async ({ device, testValue, shouldError }) => {
      const { pipelineDescriptor, code } = getPipelineDescriptor(
        device,
        testValue,
        pointList,
        frontFacing,
        sampleIndex,
        sampleMaskIn,
        sampleMaskOut
      );

      await t.testCreateRenderPipeline(pipelineDescriptor, async, shouldError, code);
    },
    extraLimits
  );
});