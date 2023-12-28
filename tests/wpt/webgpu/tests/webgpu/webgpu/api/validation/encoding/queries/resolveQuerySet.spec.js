/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Validation tests for resolveQuerySet.
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';import { GPUConst } from '../../../../constants.js';
import { kResourceStates } from '../../../../gpu_test.js';
import { ValidationTest } from '../../validation_test.js';

export const g = makeTestGroup(ValidationTest);

export const kQueryCount = 2;

g.test('queryset_and_destination_buffer_state').
desc(
  `
Tests that resolve query set must be with valid query set and destination buffer.
- {invalid, destroyed} GPUQuerySet results in validation error.
- {invalid, destroyed} destination buffer results in validation error.
  `
).
params((u) =>
u //
.combine('querySetState', kResourceStates).
combine('destinationState', kResourceStates)
).
fn((t) => {
  const { querySetState, destinationState } = t.params;

  const shouldBeValid = querySetState !== 'invalid' && destinationState !== 'invalid';
  const shouldSubmitSuccess = querySetState === 'valid' && destinationState === 'valid';

  const querySet = t.createQuerySetWithState(querySetState);

  const destination = t.createBufferWithState(destinationState, {
    size: kQueryCount * 8,
    usage: GPUBufferUsage.QUERY_RESOLVE
  });

  const encoder = t.createEncoder('non-pass');
  encoder.encoder.resolveQuerySet(querySet, 0, 1, destination, 0);
  encoder.validateFinishAndSubmit(shouldBeValid, shouldSubmitSuccess);
});

g.test('first_query_and_query_count').
desc(
  `
Tests that resolve query set with invalid firstQuery and queryCount:
- firstQuery and/or queryCount out of range
  `
).
paramsSubcasesOnly([
{ firstQuery: 0, queryCount: kQueryCount }, // control case
{ firstQuery: 0, queryCount: kQueryCount + 1 },
{ firstQuery: 1, queryCount: kQueryCount },
{ firstQuery: kQueryCount, queryCount: 1 }]
).
fn((t) => {
  const { firstQuery, queryCount } = t.params;

  const querySet = t.device.createQuerySet({ type: 'occlusion', count: kQueryCount });
  const destination = t.device.createBuffer({
    size: kQueryCount * 8,
    usage: GPUBufferUsage.QUERY_RESOLVE
  });

  const encoder = t.createEncoder('non-pass');
  encoder.encoder.resolveQuerySet(querySet, firstQuery, queryCount, destination, 0);
  encoder.validateFinish(firstQuery + queryCount <= kQueryCount);
});

g.test('destination_buffer_usage').
desc(
  `
Tests that resolve query set with invalid destinationBuffer:
- Buffer usage {with, without} QUERY_RESOLVE
  `
).
paramsSubcasesOnly((u) =>
u //
.combine('bufferUsage', [
GPUConst.BufferUsage.STORAGE,
GPUConst.BufferUsage.QUERY_RESOLVE // control case
])
).
fn((t) => {
  const querySet = t.device.createQuerySet({ type: 'occlusion', count: kQueryCount });
  const destination = t.device.createBuffer({
    size: kQueryCount * 8,
    usage: t.params.bufferUsage
  });

  const encoder = t.createEncoder('non-pass');
  encoder.encoder.resolveQuerySet(querySet, 0, kQueryCount, destination, 0);
  encoder.validateFinish(t.params.bufferUsage === GPUConst.BufferUsage.QUERY_RESOLVE);
});

g.test('destination_offset_alignment').
desc(
  `
Tests that resolve query set with invalid destinationOffset:
- destinationOffset is not a multiple of 256
  `
).
paramsSubcasesOnly((u) => u.combine('destinationOffset', [0, 128, 256, 384])).
fn((t) => {
  const { destinationOffset } = t.params;
  const querySet = t.device.createQuerySet({ type: 'occlusion', count: kQueryCount });
  const destination = t.device.createBuffer({
    size: 512,
    usage: GPUBufferUsage.QUERY_RESOLVE
  });

  const encoder = t.createEncoder('non-pass');
  encoder.encoder.resolveQuerySet(querySet, 0, kQueryCount, destination, destinationOffset);
  encoder.validateFinish(destinationOffset % 256 === 0);
});

g.test('resolve_buffer_oob').
desc(
  `
Tests that resolve query set with the size oob:
- The size of destinationBuffer - destinationOffset < queryCount * 8
  `
).
paramsSubcasesOnly((u) =>
u.combineWithParams([
{ queryCount: 2, bufferSize: 16, destinationOffset: 0, _success: true },
{ queryCount: 3, bufferSize: 16, destinationOffset: 0, _success: false },
{ queryCount: 2, bufferSize: 16, destinationOffset: 256, _success: false },
{ queryCount: 2, bufferSize: 272, destinationOffset: 256, _success: true },
{ queryCount: 2, bufferSize: 264, destinationOffset: 256, _success: false }]
)
).
fn((t) => {
  const { queryCount, bufferSize, destinationOffset, _success } = t.params;
  const querySet = t.device.createQuerySet({ type: 'occlusion', count: queryCount });
  const destination = t.device.createBuffer({
    size: bufferSize,
    usage: GPUBufferUsage.QUERY_RESOLVE
  });

  const encoder = t.createEncoder('non-pass');
  encoder.encoder.resolveQuerySet(querySet, 0, queryCount, destination, destinationOffset);
  encoder.validateFinish(_success);
});

g.test('query_set_buffer,device_mismatch').
desc(
  'Tests resolveQuerySet cannot be called with a query set or destination buffer created from another device'
).
paramsSubcasesOnly([
{ querySetMismatched: false, bufferMismatched: false }, // control case
{ querySetMismatched: true, bufferMismatched: false },
{ querySetMismatched: false, bufferMismatched: true }]
).
beforeAllSubcases((t) => {
  t.selectMismatchedDeviceOrSkipTestCase(undefined);
}).
fn((t) => {
  const { querySetMismatched, bufferMismatched } = t.params;

  const kQueryCount = 1;

  const querySetDevice = querySetMismatched ? t.mismatchedDevice : t.device;
  const querySet = querySetDevice.createQuerySet({
    type: 'occlusion',
    count: kQueryCount
  });
  t.trackForCleanup(querySet);

  const bufferDevice = bufferMismatched ? t.mismatchedDevice : t.device;
  const buffer = bufferDevice.createBuffer({
    size: kQueryCount * 8,
    usage: GPUBufferUsage.QUERY_RESOLVE
  });
  t.trackForCleanup(buffer);

  const encoder = t.createEncoder('non-pass');
  encoder.encoder.resolveQuerySet(querySet, 0, kQueryCount, buffer, 0);
  encoder.validateFinish(!(querySetMismatched || bufferMismatched));
});