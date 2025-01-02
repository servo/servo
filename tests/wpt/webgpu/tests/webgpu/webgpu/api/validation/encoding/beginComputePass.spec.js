/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests for validation in beginComputePass and GPUComputePassDescriptor as its optional descriptor.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { kQueryTypes } from '../../../capability_info.js';
import { ValidationTest } from '../validation_test.js';

class F extends ValidationTest {
  tryComputePass(success, descriptor) {
    const encoder = this.device.createCommandEncoder();
    const computePass = encoder.beginComputePass(descriptor);
    computePass.end();

    this.expectValidationError(() => {
      encoder.finish();
    }, !success);
  }
}

export const g = makeTestGroup(F);

g.test('timestampWrites,query_set_type').
desc(
  `
  Test that all entries of the timestampWrites must have type 'timestamp'. If all query types are
  not 'timestamp' in GPUComputePassDescriptor, a validation error should be generated.
  `
).
params((u) =>
u //
.combine('queryType', kQueryTypes)
).
beforeAllSubcases((t) => {
  t.selectDeviceForQueryTypeOrSkipTestCase(['timestamp', t.params.queryType]);
}).
fn((t) => {
  const { queryType } = t.params;

  const isValid = queryType === 'timestamp';

  const timestampWrites = {
    querySet: t.createQuerySetTracked({ type: queryType, count: 2 }),
    beginningOfPassWriteIndex: 0,
    endOfPassWriteIndex: 1
  };

  const descriptor = {
    timestampWrites
  };

  t.tryComputePass(isValid, descriptor);
});

g.test('timestampWrites,invalid_query_set').
desc(`Tests that timestampWrite that has an invalid query set generates a validation error.`).
params((u) => u.combine('querySetState', ['valid', 'invalid'])).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase(['timestamp-query']);
}).
fn((t) => {
  const { querySetState } = t.params;

  const querySet = t.createQuerySetWithState(querySetState, {
    type: 'timestamp',
    count: 1
  });

  const timestampWrites = {
    querySet,
    beginningOfPassWriteIndex: 0
  };

  const descriptor = {
    timestampWrites
  };

  t.tryComputePass(querySetState === 'valid', descriptor);
});

g.test('timestampWrites,query_index').
desc(
  `Test that querySet.count should be greater than timestampWrite.queryIndex, and that the
         query indexes are unique.`
).
paramsSubcasesOnly((u) =>
u //
.combine('beginningOfPassWriteIndex', [undefined, 0, 1, 2, 3]).
combine('endOfPassWriteIndex', [undefined, 0, 1, 2, 3])
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase(['timestamp-query']);
}).
fn((t) => {
  const { beginningOfPassWriteIndex, endOfPassWriteIndex } = t.params;

  const querySetCount = 2;

  const timestampWrites = {
    querySet: t.createQuerySetTracked({ type: 'timestamp', count: querySetCount }),
    beginningOfPassWriteIndex,
    endOfPassWriteIndex
  };

  const isValid =
  beginningOfPassWriteIndex !== endOfPassWriteIndex && (
  beginningOfPassWriteIndex === undefined || beginningOfPassWriteIndex < querySetCount) && (
  endOfPassWriteIndex === undefined || endOfPassWriteIndex < querySetCount);

  const descriptor = {
    timestampWrites
  };

  t.tryComputePass(isValid, descriptor);
});

g.test('timestamp_query_set,device_mismatch').
desc(
  `
  Tests beginComputePass cannot be called with a timestamp query set created from another device.
  `
).
paramsSubcasesOnly((u) => u.combine('mismatched', [true, false])).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase(['timestamp-query']);
  t.selectMismatchedDeviceOrSkipTestCase('timestamp-query');
}).
fn((t) => {
  const { mismatched } = t.params;
  const sourceDevice = mismatched ? t.mismatchedDevice : t.device;

  const timestampQuerySet = t.trackForCleanup(
    sourceDevice.createQuerySet({
      type: 'timestamp',
      count: 1
    })
  );

  const timestampWrites = {
    querySet: timestampQuerySet,
    beginningOfPassWriteIndex: 0
  };

  const descriptor = {
    timestampWrites
  };

  t.tryComputePass(!mismatched, descriptor);
});