/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests for GPUDevice.onuncapturederror.
`;import { makeTestGroup } from '../../../common/framework/test_group.js';
import { kGeneratableErrorScopeFilters } from '../../capability_info.js';
import { ErrorTest } from '../../error_test.js';

export const g = makeTestGroup(ErrorTest);

g.test('iff_uncaptured').
desc(
  `{validation, out-of-memory} error should fire uncapturederror iff not captured by a scope.`
).
params((u) => u.combine('errorType', kGeneratableErrorScopeFilters)).
fn(async (t) => {
  const { errorType } = t.params;
  const uncapturedErrorEvent = await t.expectUncapturedError(() => {
    t.generateError(errorType);
  });
  t.expect(t.isInstanceOfError(errorType, uncapturedErrorEvent.error));
});

g.test('only_original_device_is_event_target').
desc(
  `Original GPUDevice objects are EventTargets and have onuncapturederror, but
deserialized GPUDevices do not.`
).
unimplemented();

g.test('uncapturederror_from_non_originating_thread').
desc(
  `Uncaptured errors on any thread should always propagate to the original GPUDevice object
(since deserialized ones don't have EventTarget/onuncapturederror).`
).
unimplemented();