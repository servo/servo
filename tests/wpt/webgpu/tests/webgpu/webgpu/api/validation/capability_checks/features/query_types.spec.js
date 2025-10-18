/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests for capability checking for features enabling optional query types.
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { UniqueFeaturesOrLimitsGPUTest } from '../../../../gpu_test.js';

export const g = makeTestGroup(UniqueFeaturesOrLimitsGPUTest);

g.test('createQuerySet').
desc(
  `
  Tests that creating a query set throws a type error exception if the features don't contain
  'timestamp-query'.
    - createQuerySet
      - type {occlusion, timestamp}
      - x= timestamp query {enable, disable}
  `
).
params((u) =>
u.
combine('type', ['occlusion', 'timestamp']).
combine('featureContainsTimestampQuery', [false, true])
).
beforeAllSubcases((t) => {
  const { featureContainsTimestampQuery } = t.params;

  const requiredFeatures = [];
  if (featureContainsTimestampQuery) {
    requiredFeatures.push('timestamp-query');
  }

  t.selectDeviceOrSkipTestCase({ requiredFeatures });
}).
fn((t) => {
  const { type, featureContainsTimestampQuery } = t.params;

  const count = 1;
  const shouldException = type === 'timestamp' && !featureContainsTimestampQuery;

  t.shouldThrow(shouldException ? 'TypeError' : false, () => {
    t.createQuerySetTracked({ type, count });
  });
});

g.test('timestamp').
desc(
  `
  Tests that writing a timestamp throws a type error exception if the features don't contain
  'timestamp-query'.
  `
).
params((u) => u.combine('featureContainsTimestampQuery', [false, true])).
beforeAllSubcases((t) => {
  const { featureContainsTimestampQuery } = t.params;

  const requiredFeatures = [];
  if (featureContainsTimestampQuery) {
    requiredFeatures.push('timestamp-query');
  }

  t.selectDeviceOrSkipTestCase({ requiredFeatures });
}).
fn((t) => {
  const { featureContainsTimestampQuery } = t.params;

  const querySet = t.createQuerySetTracked({
    type: featureContainsTimestampQuery ? 'timestamp' : 'occlusion',
    count: 2
  });

  {
    const encoder = t.createEncoder('non-pass');
    encoder.encoder.
    beginComputePass({
      timestampWrites: { querySet, beginningOfPassWriteIndex: 0, endOfPassWriteIndex: 1 }
    }).
    end();
    t.expectValidationError(() => {
      encoder.finish();
    }, !featureContainsTimestampQuery);
  }

  {
    const encoder = t.createEncoder('non-pass');
    const view = t.
    createTextureTracked({
      size: [16, 16, 1],
      format: 'rgba8unorm',
      usage: GPUTextureUsage.RENDER_ATTACHMENT
    }).
    createView();
    encoder.encoder.
    beginRenderPass({
      colorAttachments: [{ view, loadOp: 'clear', storeOp: 'discard' }],
      timestampWrites: { querySet, beginningOfPassWriteIndex: 0, endOfPassWriteIndex: 1 }
    }).
    end();
    t.expectValidationError(() => {
      encoder.finish();
    }, !featureContainsTimestampQuery);
  }
});