// META: title=NativeIO API: Failure handling for capacity allocation.
// META: global=dedicatedworker

'use strict';

test(testCase => {
  assert_throws_dom(
    'QuotaExceededError',
    () => storageFoundation.releaseCapacitySync(10));
}, 'Attempting to release more capacity than available fails.');
