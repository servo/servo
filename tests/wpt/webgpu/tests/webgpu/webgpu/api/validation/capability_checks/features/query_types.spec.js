/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Tests for capability checking for features enabling optional query types.
`;
import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { ValidationTest } from '../../validation_test.js';

export const g = makeTestGroup(ValidationTest);

g.test('createQuerySet')
  .desc(
    `
  Tests that creating a query set throws a type error exception if the features don't contain
  'timestamp-query'.
    - createQuerySet
      - type {occlusion, timestamp}
      - x= {pipeline statistics, timestamp} query {enable, disable}
  `
  )
  .params(u =>
    u
      .combine('type', ['occlusion', 'timestamp'])
      .combine('featureContainsTimestampQuery', [false, true])
  )
  .beforeAllSubcases(t => {
    const { featureContainsTimestampQuery } = t.params;

    const requiredFeatures = [];
    if (featureContainsTimestampQuery) {
      requiredFeatures.push('timestamp-query');
    }

    t.selectDeviceOrSkipTestCase({ requiredFeatures });
  })
  .fn(t => {
    const { type, featureContainsTimestampQuery } = t.params;

    const count = 1;
    const shouldException = type === 'timestamp' && !featureContainsTimestampQuery;

    t.shouldThrow(shouldException ? 'TypeError' : false, () => {
      t.device.createQuerySet({ type, count });
    });
  });

g.test('writeTimestamp')
  .desc(
    `
  Tests that writing a timestamp throws a type error exception if the features don't contain
  'timestamp-query'.
  `
  )
  .params(u => u.combine('featureContainsTimestampQuery', [false, true]))
  .beforeAllSubcases(t => {
    const { featureContainsTimestampQuery } = t.params;

    const requiredFeatures = [];
    if (featureContainsTimestampQuery) {
      requiredFeatures.push('timestamp-query');
    }

    t.selectDeviceOrSkipTestCase({ requiredFeatures });
  })
  .fn(t => {
    const { featureContainsTimestampQuery } = t.params;

    const querySet = t.device.createQuerySet({
      type: featureContainsTimestampQuery ? 'timestamp' : 'occlusion',
      count: 1,
    });
    const encoder = t.createEncoder('non-pass');

    t.shouldThrow(featureContainsTimestampQuery ? false : 'TypeError', () => {
      encoder.encoder.writeTimestamp(querySet, 0);
    });
  });
