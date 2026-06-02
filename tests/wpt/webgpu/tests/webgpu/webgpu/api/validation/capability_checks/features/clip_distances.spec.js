/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { range } from '../../../../../common/util/util.js';import { align } from '../../../../util/math.js';import { kMaximumLimitBaseParams, makeLimitTestGroup } from '../limits/limit_utils.js';

function getPipelineDescriptorWithClipDistances(
device,
interStageShaderVariables,
pointList,
clipDistances,
startLocation = 0)
{
  const vertexOutputVariables =
  interStageShaderVariables - (pointList ? 1 : 0) - align(clipDistances, 4) / 4;
  const maxVertexOutputVariables =
  device.limits.maxInterStageShaderVariables - (pointList ? 1 : 0) - align(clipDistances, 4) / 4;

  const varyings = `
      ${range(
    vertexOutputVariables,
    (i) => `@location(${i + startLocation}) v4_${i + startLocation}: vec4f,`
  ).join('\n')}
  `;

  const code = `
    // test value                        : ${interStageShaderVariables}
    // maxInterStageShaderVariables     : ${device.limits.maxInterStageShaderVariables}
    // num variables in vertex shader : ${vertexOutputVariables}${
  pointList ? ' + point-list' : ''
  }${
  clipDistances > 0 ?
  ` + ${align(clipDistances, 4) / 4} (clip_distances[${clipDistances}])` :
  ''
  }
    // maxInterStageVariables:           : ${maxVertexOutputVariables}
    // num used inter stage variables    : ${vertexOutputVariables}
    // vertex output start location      : ${startLocation}

    enable clip_distances;

    struct VSOut {
      @builtin(position) p: vec4f,
      ${varyings}
      ${
  clipDistances > 0 ?
  `@builtin(clip_distances) clipDistances: array<f32, ${clipDistances}>,` :
  ''
  }
    }
    struct FSIn {
      ${varyings}
    }
    struct FSOut {
      @location(0) color: vec4f,
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
      module
    },
    fragment: {
      module,
      targets: [
      {
        format: 'rgba8unorm'
      }]

    }
  };
  return pipelineDescriptor;
}

const limit = 'maxInterStageShaderVariables';
export const { g, description } = makeLimitTestGroup(limit);

g.test('createRenderPipeline,at_over').
desc(`Test using at and over ${limit} limit with clip_distances in createRenderPipeline(Async)`).
params(
  kMaximumLimitBaseParams.
  combine('async', [false, true]).
  combine('pointList', [false, true]).
  combine('clipDistances', [1, 2, 3, 4, 5, 6, 7, 8])
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('clip-distances');
}).
fn(async (t) => {
  const { limitTest, testValueName, async, pointList, clipDistances } = t.params;
  await t.testDeviceWithRequestedMaximumLimits(
    limitTest,
    testValueName,
    async ({ device, testValue, shouldError }) => {
      const pipelineDescriptor = getPipelineDescriptorWithClipDistances(
        device,
        testValue,
        pointList,
        clipDistances
      );

      await t.testCreateRenderPipeline(pipelineDescriptor, async, shouldError);
    },
    undefined,
    ['clip-distances']
  );
});

g.test('createRenderPipeline,max_vertex_output_location').
desc(`Test using clip_distances will limit the maximum value of vertex output location`).
params((u) =>
u.
combine('pointList', [false, true]).
combine('clipDistances', [1, 2, 3, 4, 5, 6, 7, 8]).
combine('startLocation', [0, 1, 2])
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('clip-distances');
}).
fn(async (t) => {
  const { pointList, clipDistances, startLocation } = t.params;

  const maxInterStageShaderVariables = t.adapter.limits.maxInterStageShaderVariables;
  const deviceInTest = await t.requestDeviceTracked(t.adapter, {
    requiredFeatures: ['clip-distances'],
    requiredLimits: {
      maxInterStageShaderVariables: t.adapter.limits.maxInterStageShaderVariables
    }
  });
  const pipelineDescriptor = getPipelineDescriptorWithClipDistances(
    deviceInTest,
    maxInterStageShaderVariables,
    pointList,
    clipDistances,
    startLocation
  );
  const vertexOutputVariables =
  maxInterStageShaderVariables - (pointList ? 1 : 0) - align(clipDistances, 4) / 4;
  const maxLocationInTest = startLocation + vertexOutputVariables - 1;
  const maxAllowedLocation = maxInterStageShaderVariables - 1 - align(clipDistances, 4) / 4;
  const shouldError = maxLocationInTest > maxAllowedLocation;

  deviceInTest.pushErrorScope('validation');
  deviceInTest.createRenderPipeline(pipelineDescriptor);
  const error = await deviceInTest.popErrorScope();
  t.expect(!!error === shouldError, `${error?.message || 'no error when one was expected'}`);

  deviceInTest.destroy();
});