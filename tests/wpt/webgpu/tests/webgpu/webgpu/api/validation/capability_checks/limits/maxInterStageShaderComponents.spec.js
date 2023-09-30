/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ import { assert, range } from '../../../../../common/util/util.js';
import { kMaximumLimitBaseParams, makeLimitTestGroup } from './limit_utils.js';

function getTypeForNumComponents(numComponents) {
  return numComponents > 1 ? `vec${numComponents}f` : 'f32';
}

function getPipelineDescriptor(
  device,
  testValue,
  pointList,
  frontFacing,
  sampleIndex,
  sampleMaskIn,
  sampleMaskOut
) {
  const maxVertexShaderOutputComponents = testValue - (pointList ? 1 : 0);
  const maxFragmentShaderInputComponents =
    testValue - (frontFacing ? 1 : 0) - (sampleIndex ? 1 : 0) - (sampleMaskIn ? 1 : 0);

  const maxInterStageVariables = device.limits.maxInterStageShaderVariables;
  const numComponents = Math.min(maxVertexShaderOutputComponents, maxFragmentShaderInputComponents);
  assert(Math.ceil(numComponents / 4) <= maxInterStageVariables);

  const num4ComponentVaryings = Math.floor(numComponents / 4);
  const lastVaryingNumComponents = numComponents % 4;

  const varyings = `
      ${range(num4ComponentVaryings, i => `@location(${i}) v4_${i}: vec4f,`).join('\n')}
      ${
        lastVaryingNumComponents > 0
          ? `@location(${num4ComponentVaryings}) vx: ${getTypeForNumComponents(
              lastVaryingNumComponents
            )},`
          : ``
      }
  `;

  const code = `
    // test value                        : ${testValue}
    // maxInterStageShaderComponents     : ${device.limits.maxInterStageShaderComponents}
    // num components in vertex shader   : ${numComponents}${pointList ? ' + point-list' : ''}
    // num components in fragment shader : ${numComponents}${frontFacing ? ' + front-facing' : ''}${
    sampleIndex ? ' + sample_index' : ''
  }${sampleMaskIn ? ' + sample_mask' : ''}
    // maxVertexShaderOutputComponents   : ${maxVertexShaderOutputComponents}
    // maxFragmentShaderInputComponents  : ${maxFragmentShaderInputComponents}
    // maxInterStageVariables:           : ${maxInterStageVariables}
    // num used inter stage variables    : ${Math.ceil(numComponents / 4)}

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
      topology: pointList ? 'point-list' : 'triangle-list',
    },
    vertex: {
      module,
      entryPoint: 'vs',
    },
    fragment: {
      module,
      entryPoint: 'fs',
      targets: [
        {
          format: 'rgba8unorm',
        },
      ],
    },
  };
  return { pipelineDescriptor, code };
}

const limit = 'maxInterStageShaderComponents';
export const { g, description } = makeLimitTestGroup(limit);

g.test('createRenderPipeline,at_over')
  .desc(`Test using at and over ${limit} limit in createRenderPipeline(Async)`)
  .params(
    kMaximumLimitBaseParams
      .combine('async', [false, true])
      .combine('pointList', [false, true])
      .combine('frontFacing', [false, true])
      .combine('sampleIndex', [false, true])
      .combine('sampleMaskIn', [false, true])
      .combine('sampleMaskOut', [false, true])
  )
  .beforeAllSubcases(t => {
    if (t.isCompatibility && (t.params.sampleMaskIn || t.params.sampleMaskOut)) {
      t.skip('sample_mask not supported in compatibility mode');
    }
  })
  .fn(async t => {
    const {
      limitTest,
      testValueName,
      async,
      pointList,
      frontFacing,
      sampleIndex,
      sampleMaskIn,
      sampleMaskOut,
    } = t.params;
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
      }
    );
  });
