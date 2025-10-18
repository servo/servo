/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Validation for encoding queries.
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { kQueryTypes } from '../../../../capability_info.js';
import { AllFeaturesMaxLimitsGPUTest } from '../../../../gpu_test.js';
import * as vtu from '../../validation_test_utils.js';

import { createQuerySetWithType } from './common.js';

export const g = makeTestGroup(AllFeaturesMaxLimitsGPUTest);

g.test('occlusion_query,query_type').
desc(
  `
Tests that set occlusion query set with all types in render pass descriptor:
- type {occlusion (control case), timestamp}
- {undefined} for occlusion query set in render pass descriptor
  `
).
params((u) => u.combine('type', [undefined, ...kQueryTypes])).
fn((t) => {
  const type = t.params.type;
  if (type) {
    t.skipIfDeviceDoesNotSupportQueryType(type);
  }
  const querySet = type === undefined ? undefined : createQuerySetWithType(t, type, 1);

  const encoder = t.createEncoder('render pass', { occlusionQuerySet: querySet });
  encoder.encoder.beginOcclusionQuery(0);
  encoder.encoder.endOcclusionQuery();
  encoder.validateFinish(type === 'occlusion');
});

g.test('occlusion_query,invalid_query_set').
desc(
  `
Tests that begin occlusion query with a invalid query set that failed during creation.
  `
).
paramsSubcasesOnly((u) => u.combine('querySetState', ['valid', 'invalid'])).
fn((t) => {
  const occlusionQuerySet = vtu.createQuerySetWithState(t, t.params.querySetState);

  const encoder = t.createEncoder('render pass', { occlusionQuerySet });
  encoder.encoder.beginOcclusionQuery(0);
  encoder.encoder.endOcclusionQuery();
  encoder.validateFinishAndSubmitGivenState(t.params.querySetState);
});

g.test('occlusion_query,query_index').
desc(
  `
Tests that begin occlusion query with query index:
- queryIndex {in, out of} range for GPUQuerySet
  `
).
paramsSubcasesOnly((u) => u.combine('queryIndex', [0, 2])).
fn((t) => {
  const occlusionQuerySet = createQuerySetWithType(t, 'occlusion', 2);

  const encoder = t.createEncoder('render pass', { occlusionQuerySet });
  encoder.encoder.beginOcclusionQuery(t.params.queryIndex);
  encoder.encoder.endOcclusionQuery();
  encoder.validateFinish(t.params.queryIndex < 2);
});