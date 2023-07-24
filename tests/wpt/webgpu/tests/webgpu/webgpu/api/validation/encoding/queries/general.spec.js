/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
TODO: pipeline statistics queries are removed from core; consider moving tests to another suite.
TODO:
- Start a pipeline statistics query in all possible encoders:
    - queryIndex {in, out of} range for GPUQuerySet
    - GPUQuerySet {valid, invalid, device mismatched}
    - x ={render pass, compute pass} encoder
`;
import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { kQueryTypes } from '../../../../capability_info.js';
import { ValidationTest } from '../../validation_test.js';

import { createQuerySetWithType } from './common.js';

export const g = makeTestGroup(ValidationTest);

g.test('occlusion_query,query_type')
  .desc(
    `
Tests that set occlusion query set with all types in render pass descriptor:
- type {occlusion (control case), pipeline statistics, timestamp}
- {undefined} for occlusion query set in render pass descriptor
  `
  )
  .params(u => u.combine('type', [undefined, ...kQueryTypes]))
  .beforeAllSubcases(t => {
    const { type } = t.params;
    if (type) {
      t.selectDeviceForQueryTypeOrSkipTestCase(type);
    }
  })
  .fn(t => {
    const type = t.params.type;
    const querySet = type === undefined ? undefined : createQuerySetWithType(t, type, 1);

    const encoder = t.createEncoder('render pass', { occlusionQuerySet: querySet });
    encoder.encoder.beginOcclusionQuery(0);
    encoder.encoder.endOcclusionQuery();
    encoder.validateFinish(type === 'occlusion');
  });

g.test('occlusion_query,invalid_query_set')
  .desc(
    `
Tests that begin occlusion query with a invalid query set that failed during creation.
  `
  )
  .paramsSubcasesOnly(u => u.combine('querySetState', ['valid', 'invalid']))
  .fn(t => {
    const occlusionQuerySet = t.createQuerySetWithState(t.params.querySetState);

    const encoder = t.createEncoder('render pass', { occlusionQuerySet });
    encoder.encoder.beginOcclusionQuery(0);
    encoder.encoder.endOcclusionQuery();
    encoder.validateFinishAndSubmitGivenState(t.params.querySetState);
  });

g.test('occlusion_query,query_index')
  .desc(
    `
Tests that begin occlusion query with query index:
- queryIndex {in, out of} range for GPUQuerySet
  `
  )
  .paramsSubcasesOnly(u => u.combine('queryIndex', [0, 2]))
  .fn(t => {
    const occlusionQuerySet = createQuerySetWithType(t, 'occlusion', 2);

    const encoder = t.createEncoder('render pass', { occlusionQuerySet });
    encoder.encoder.beginOcclusionQuery(t.params.queryIndex);
    encoder.encoder.endOcclusionQuery();
    encoder.validateFinish(t.params.queryIndex < 2);
  });

g.test('timestamp_query,query_type_and_index')
  .desc(
    `
Tests that write timestamp to all types of query set on all possible encoders:
- type {occlusion, pipeline statistics, timestamp}
- queryIndex {in, out of} range for GPUQuerySet
- x= {non-pass} encoder
  `
  )
  .params(u =>
    u
      .combine('type', kQueryTypes)
      .beginSubcases()
      .expand('queryIndex', p => (p.type === 'timestamp' ? [0, 2] : [0]))
  )
  .beforeAllSubcases(t => {
    const { type } = t.params;

    // writeTimestamp is only available for devices that enable the 'timestamp-query' feature.
    const queryTypes = ['timestamp'];
    if (type !== 'timestamp') {
      queryTypes.push(type);
    }

    t.selectDeviceForQueryTypeOrSkipTestCase(queryTypes);
  })
  .fn(t => {
    const { type, queryIndex } = t.params;

    const count = 2;
    const querySet = createQuerySetWithType(t, type, count);

    const encoder = t.createEncoder('non-pass');
    encoder.encoder.writeTimestamp(querySet, queryIndex);
    encoder.validateFinish(type === 'timestamp' && queryIndex < count);
  });

g.test('timestamp_query,invalid_query_set')
  .desc(
    `
Tests that write timestamp to a invalid query set that failed during creation:
- x= {non-pass} encoder
  `
  )
  .paramsSubcasesOnly(u => u.combine('querySetState', ['valid', 'invalid']))
  .beforeAllSubcases(t => {
    t.selectDeviceForQueryTypeOrSkipTestCase('timestamp');
  })
  .fn(t => {
    const { querySetState } = t.params;

    const querySet = t.createQuerySetWithState(querySetState, {
      type: 'timestamp',
      count: 2,
    });

    const encoder = t.createEncoder('non-pass');
    encoder.encoder.writeTimestamp(querySet, 0);
    encoder.validateFinish(querySetState !== 'invalid');
  });

g.test('timestamp_query,device_mismatch')
  .desc('Tests writeTimestamp cannot be called with a query set created from another device')
  .paramsSubcasesOnly(u => u.combine('mismatched', [true, false]))
  .beforeAllSubcases(t => {
    t.selectDeviceForQueryTypeOrSkipTestCase('timestamp');
    t.selectMismatchedDeviceOrSkipTestCase('timestamp-query');
  })
  .fn(t => {
    const { mismatched } = t.params;
    const sourceDevice = mismatched ? t.mismatchedDevice : t.device;

    const querySet = sourceDevice.createQuerySet({
      type: 'timestamp',
      count: 2,
    });
    t.trackForCleanup(querySet);

    const encoder = t.createEncoder('non-pass');
    encoder.encoder.writeTimestamp(querySet, 0);
    encoder.validateFinish(!mismatched);
  });
