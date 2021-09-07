// META: title=NativeIO API: Failure handling for capacity allocation.
// META: global=window,worker

'use strict';

promise_test(async testCase => {
  await promise_rejects_dom(
    testCase, 'QuotaExceededError', storageFoundation.releaseCapacity(10));
}, 'Attempting to release more capacity than available fails.');
