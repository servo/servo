/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests using a destroyed query set on a queue.
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { AllFeaturesMaxLimitsGPUTest } from '../../../../gpu_test.js';
import * as vtu from '../../validation_test_utils.js';

export const g = makeTestGroup(AllFeaturesMaxLimitsGPUTest);

g.test('beginOcclusionQuery').
desc(
  `
Tests that use a destroyed query set in occlusion query on render pass encoder.
- x= {destroyed, not destroyed (control case)}
  `
).
paramsSubcasesOnly((u) => u.combine('querySetState', ['valid', 'destroyed'])).
fn((t) => {
  const occlusionQuerySet = vtu.createQuerySetWithState(t, t.params.querySetState);

  const encoder = t.createEncoder('render pass', { occlusionQuerySet });
  encoder.encoder.beginOcclusionQuery(0);
  encoder.encoder.endOcclusionQuery();
  encoder.validateFinishAndSubmitGivenState(t.params.querySetState);
});

g.test('timestamps').
desc(
  `
Tests that use a destroyed query set in timestamp query on {non-pass, compute, render} encoder.
- x= {destroyed, not destroyed (control case)}
  `
).
params((u) => u.beginSubcases().combine('querySetState', ['valid', 'destroyed'])).
fn((t) => {
  t.skipIfDeviceDoesNotSupportQueryType('timestamp');
  const querySet = vtu.createQuerySetWithState(t, t.params.querySetState, {
    type: 'timestamp',
    count: 2
  });

  {
    const encoder = t.createEncoder('non-pass');
    encoder.encoder.
    beginComputePass({
      timestampWrites: { querySet, beginningOfPassWriteIndex: 0 }
    }).
    end();
    encoder.validateFinishAndSubmitGivenState(t.params.querySetState);
  }

  {
    const texture = t.createTextureTracked({
      size: [1, 1, 1],
      format: 'rgba8unorm',
      usage: GPUTextureUsage.RENDER_ATTACHMENT
    });
    const encoder = t.createEncoder('non-pass');
    encoder.encoder.
    beginRenderPass({
      colorAttachments: [
      {
        view: texture.createView(),
        loadOp: 'load',
        storeOp: 'store'
      }],

      timestampWrites: { querySet, beginningOfPassWriteIndex: 0 }
    }).
    end();
    encoder.validateFinishAndSubmitGivenState(t.params.querySetState);
  }
});

g.test('resolveQuerySet').
desc(
  `
Tests that use a destroyed query set in resolveQuerySet.
- x= {destroyed, not destroyed (control case)}
  `
).
paramsSubcasesOnly((u) => u.combine('querySetState', ['valid', 'destroyed'])).
fn((t) => {
  const querySet = vtu.createQuerySetWithState(t, t.params.querySetState);

  const buffer = t.createBufferTracked({ size: 8, usage: GPUBufferUsage.QUERY_RESOLVE });

  const encoder = t.createEncoder('non-pass');
  encoder.encoder.resolveQuerySet(querySet, 0, 1, buffer, 0);
  encoder.validateFinishAndSubmitGivenState(t.params.querySetState);
});