/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Tests using a destroyed query set on a queue.
`;
import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { ValidationTest } from '../../validation_test.js';

export const g = makeTestGroup(ValidationTest);

g.test('beginOcclusionQuery')
  .desc(
    `
Tests that use a destroyed query set in occlusion query on render pass encoder.
- x= {destroyed, not destroyed (control case)}
  `
  )
  .paramsSubcasesOnly(u => u.combine('querySetState', ['valid', 'destroyed']))
  .fn(t => {
    const occlusionQuerySet = t.createQuerySetWithState(t.params.querySetState);

    const encoder = t.createEncoder('render pass', { occlusionQuerySet });
    encoder.encoder.beginOcclusionQuery(0);
    encoder.encoder.endOcclusionQuery();
    encoder.validateFinishAndSubmitGivenState(t.params.querySetState);
  });

g.test('writeTimestamp')
  .desc(
    `
Tests that use a destroyed query set in writeTimestamp on {non-pass, compute, render} encoder.
- x= {destroyed, not destroyed (control case)}
  `
  )
  .params(u => u.beginSubcases().combine('querySetState', ['valid', 'destroyed']))
  .beforeAllSubcases(t => t.selectDeviceOrSkipTestCase('timestamp-query'))
  .fn(t => {
    const querySet = t.createQuerySetWithState(t.params.querySetState, {
      type: 'timestamp',
      count: 2,
    });

    const encoder = t.createEncoder('non-pass');
    encoder.encoder.writeTimestamp(querySet, 0);
    encoder.validateFinishAndSubmitGivenState(t.params.querySetState);
  });

g.test('resolveQuerySet')
  .desc(
    `
Tests that use a destroyed query set in resolveQuerySet.
- x= {destroyed, not destroyed (control case)}
  `
  )
  .paramsSubcasesOnly(u => u.combine('querySetState', ['valid', 'destroyed']))
  .fn(t => {
    const querySet = t.createQuerySetWithState(t.params.querySetState);

    const buffer = t.device.createBuffer({ size: 8, usage: GPUBufferUsage.QUERY_RESOLVE });

    const encoder = t.createEncoder('non-pass');
    encoder.encoder.resolveQuerySet(querySet, 0, 1, buffer, 0);
    encoder.validateFinishAndSubmitGivenState(t.params.querySetState);
  });
