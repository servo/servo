/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests for capabilities added by float32-blendable flag.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';

import { UniqueFeaturesOrLimitsGPUTest } from '../../../gpu_test.js';
import * as vtu from '../validation_test_utils.js';

import { getDescriptorForCreateRenderPipelineValidationTest } from './common.js';

export const g = makeTestGroup(UniqueFeaturesOrLimitsGPUTest);

const kFloat32Formats = ['r32float', 'rg32float', 'rgba32float'];

g.test('create_render_pipeline').
desc(
  `
Tests that the float32-blendable feature is required to create a render
pipeline that uses blending with any float32-format attachment.
`
).
params((u) =>
u.
combine('isAsync', [false, true]).
combine('enabled', [true, false]).
beginSubcases().
combine('hasBlend', [true, false]).
combine('format', kFloat32Formats)
).
beforeAllSubcases((t) => {
  if (t.params.enabled) {
    t.selectDeviceOrSkipTestCase('float32-blendable');
  }
}).
fn((t) => {
  const { isAsync, enabled, hasBlend, format } = t.params;
  const descriptor = getDescriptorForCreateRenderPipelineValidationTest(t.device, {
    targets: [
    {
      format,
      blend: hasBlend ? { color: {}, alpha: {} } : undefined
    }]

  });

  vtu.doCreateRenderPipelineTest(t, isAsync, enabled || !hasBlend, descriptor);
});