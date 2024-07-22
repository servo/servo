/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
This test dedicatedly tests createRenderPipeline validation issues related to the shader modules.

Note: entry point matching tests are in ../shader_module/entry_point.spec.ts
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import {
  getFragmentShaderCodeWithOutput,
  kDefaultVertexShaderCode,
  kDefaultFragmentShaderCode } from
'../../../util/shader.js';

import { CreateRenderPipelineValidationTest } from './common.js';

export const g = makeTestGroup(CreateRenderPipelineValidationTest);

const values = [0, 1, 0, 1];

g.test('device_mismatch').
desc(
  'Tests createRenderPipeline(Async) cannot be called with a shader module created from another device'
).
paramsSubcasesOnly((u) =>
u.combine('isAsync', [true, false]).combineWithParams([
{ vertex_mismatched: false, fragment_mismatched: false, _success: true },
{ vertex_mismatched: true, fragment_mismatched: false, _success: false },
{ vertex_mismatched: false, fragment_mismatched: true, _success: false }]
)
).
beforeAllSubcases((t) => {
  t.selectMismatchedDeviceOrSkipTestCase(undefined);
}).
fn((t) => {
  const { isAsync, vertex_mismatched, fragment_mismatched, _success } = t.params;

  const code = `
      @vertex fn main() -> @builtin(position) vec4<f32> {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
      }
    `;

  const descriptor = {
    vertex: {
      module: vertex_mismatched ?
      t.mismatchedDevice.createShaderModule({ code }) :
      t.device.createShaderModule({ code }),
      entryPoint: 'main'
    },
    fragment: {
      module: fragment_mismatched ?
      t.mismatchedDevice.createShaderModule({
        code: getFragmentShaderCodeWithOutput([
        { values, plainType: 'f32', componentCount: 4 }]
        )
      }) :
      t.device.createShaderModule({
        code: getFragmentShaderCodeWithOutput([
        { values, plainType: 'f32', componentCount: 4 }]
        )
      }),
      entryPoint: 'main',
      targets: [{ format: 'rgba8unorm' }]
    },
    layout: t.getPipelineLayout()
  };

  t.doCreateRenderPipelineTest(isAsync, _success, descriptor);
});

g.test('invalid,vertex').
desc(`Tests shader module must be valid.`).
params((u) => u.combine('isAsync', [true, false]).combine('isVertexShaderValid', [true, false])).
fn((t) => {
  const { isAsync, isVertexShaderValid } = t.params;
  t.doCreateRenderPipelineTest(isAsync, isVertexShaderValid, {
    layout: 'auto',
    vertex: {
      module: isVertexShaderValid ?
      t.device.createShaderModule({
        code: kDefaultVertexShaderCode
      }) :
      t.createInvalidShaderModule(),
      entryPoint: 'main'
    },
    // Specify a color attachment so we have at least one render target.
    fragment: {
      targets: [{ format: 'rgba8unorm' }],
      module: t.device.createShaderModule({
        code: `@fragment fn main() -> @location(0) vec4f { return vec4f(0); }`
      })
    }
  });
});

g.test('invalid,fragment').
desc(`Tests shader module must be valid.`).
params((u) => u.combine('isAsync', [true, false]).combine('isFragmentShaderValid', [true, false])).
fn((t) => {
  const { isAsync, isFragmentShaderValid } = t.params;
  t.doCreateRenderPipelineTest(isAsync, isFragmentShaderValid, {
    layout: 'auto',
    vertex: {
      module: t.device.createShaderModule({
        code: kDefaultVertexShaderCode
      }),
      entryPoint: 'main'
    },
    fragment: {
      module: isFragmentShaderValid ?
      t.device.createShaderModule({
        code: kDefaultFragmentShaderCode
      }) :
      t.createInvalidShaderModule(),
      entryPoint: 'main',
      targets: [{ format: 'rgba8unorm' }]
    }
  });
});