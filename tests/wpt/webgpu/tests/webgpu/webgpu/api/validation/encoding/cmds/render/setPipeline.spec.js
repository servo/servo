/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Validation tests for setPipeline on render pass and render bundle.
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { AllFeaturesMaxLimitsGPUTest } from '../../../../../gpu_test.js';
import { kRenderEncodeTypes } from '../../../../../util/command_buffer_maker.js';
import * as vtu from '../../../validation_test_utils.js';

import { kRenderEncodeTypeParams } from './render.js';

export const g = makeTestGroup(AllFeaturesMaxLimitsGPUTest);

g.test('invalid_pipeline').
desc(
  `
Tests setPipeline should generate an error iff using an 'invalid' pipeline.
  `
).
paramsSubcasesOnly((u) =>
u.combine('encoderType', kRenderEncodeTypes).combine('state', ['valid', 'invalid'])
).
fn((t) => {
  const { encoderType, state } = t.params;
  const pipeline = vtu.createRenderPipelineWithState(t, state);

  const { encoder, validateFinish } = t.createEncoder(encoderType);
  encoder.setPipeline(pipeline);
  validateFinish(state !== 'invalid');
});

g.test('pipeline,device_mismatch').
desc('Tests setPipeline cannot be called with a render pipeline created from another device').
paramsSubcasesOnly(kRenderEncodeTypeParams.combine('mismatched', [true, false])).
beforeAllSubcases((t) => t.usesMismatchedDevice()).
fn((t) => {
  const { encoderType, mismatched } = t.params;
  const sourceDevice = mismatched ? t.mismatchedDevice : t.device;

  const pipeline = sourceDevice.createRenderPipeline({
    layout: 'auto',
    vertex: {
      module: sourceDevice.createShaderModule({
        code: `@vertex fn main() -> @builtin(position) vec4<f32> { return vec4<f32>(); }`
      }),
      entryPoint: 'main'
    },
    fragment: {
      module: sourceDevice.createShaderModule({
        code: '@fragment fn main() {}'
      }),
      entryPoint: 'main',
      targets: [{ format: 'rgba8unorm', writeMask: 0 }]
    },
    primitive: { topology: 'triangle-list' }
  });

  const { encoder, validateFinish } = t.createEncoder(encoderType);
  encoder.setPipeline(pipeline);
  validateFinish(!mismatched);
});