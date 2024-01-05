/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
misc createRenderPipeline and createRenderPipelineAsync validation tests.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { kDefaultVertexShaderCode, kDefaultFragmentShaderCode } from '../../../util/shader.js';

import { CreateRenderPipelineValidationTest } from './common.js';

export const g = makeTestGroup(CreateRenderPipelineValidationTest);

g.test('basic').
desc(`Test basic usage of createRenderPipeline.`).
params((u) => u.combine('isAsync', [false, true])).
fn((t) => {
  const { isAsync } = t.params;
  const descriptor = t.getDescriptor();

  t.doCreateRenderPipelineTest(isAsync, true, descriptor);
});

g.test('vertex_state_only').
desc(
  `Tests creating vertex-state-only render pipeline. A vertex-only render pipeline has no fragment
state (and thus has no color state), and can be created with or without depth stencil state.`
).
params((u) =>
u.
combine('isAsync', [false, true]).
beginSubcases().
combine('depthStencilFormat', [
'depth24plus',
'depth24plus-stencil8',
'depth32float',
'']
).
combine('hasColor', [false, true])
).
fn((t) => {
  const { isAsync, depthStencilFormat, hasColor } = t.params;

  let depthStencilState;
  if (depthStencilFormat === '') {
    depthStencilState = undefined;
  } else {
    depthStencilState = {
      format: depthStencilFormat,
      depthWriteEnabled: false,
      depthCompare: 'always'
    };
  }

  // Having targets or not should have no effect in result, since it will not appear in the
  // descriptor in vertex-only render pipeline
  const descriptor = t.getDescriptor({
    noFragment: true,
    depthStencil: depthStencilState,
    targets: hasColor ? [{ format: 'rgba8unorm' }] : []
  });

  t.doCreateRenderPipelineTest(isAsync, true, descriptor);
});

g.test('pipeline_layout,device_mismatch').
desc(
  'Tests createRenderPipeline(Async) cannot be called with a pipeline layout created from another device'
).
paramsSubcasesOnly((u) => u.combine('isAsync', [true, false]).combine('mismatched', [true, false])).
beforeAllSubcases((t) => {
  t.selectMismatchedDeviceOrSkipTestCase(undefined);
}).
fn((t) => {
  const { isAsync, mismatched } = t.params;

  const sourceDevice = mismatched ? t.mismatchedDevice : t.device;

  const layout = sourceDevice.createPipelineLayout({ bindGroupLayouts: [] });

  const format = 'rgba8unorm';
  const descriptor = {
    layout,
    vertex: {
      module: t.device.createShaderModule({
        code: kDefaultVertexShaderCode
      }),
      entryPoint: 'main'
    },
    fragment: {
      module: t.device.createShaderModule({
        code: kDefaultFragmentShaderCode
      }),
      entryPoint: 'main',
      targets: [{ format }]
    }
  };

  t.doCreateRenderPipelineTest(isAsync, !mismatched, descriptor);
});