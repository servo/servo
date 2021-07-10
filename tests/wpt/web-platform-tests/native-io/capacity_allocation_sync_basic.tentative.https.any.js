// META: title=NativeIO API: Acquiring, displaying and releasing capacity.
// META: global=dedicatedworker

'use strict';

test(testCase => {
  const available_capacity = storageFoundation.getRemainingCapacitySync();
  assert_equals(available_capacity, 0);
}, 'The starting capacity of a NativeIOFileManager is 0');

test(testCase => {
  const requested_capacity = 4;
  const granted_capacity = storageFoundation.requestCapacitySync(requested_capacity);
  const available_capacity = storageFoundation.getRemainingCapacitySync();
  assert_equals(available_capacity, granted_capacity);
  testCase.add_cleanup(() => {
    storageFoundation.releaseCapacitySync(available_capacity);
  });
}, 'getRemainingCapacitySync() reports the capacity granted by ' +
    'requestCapacitySync()');

test(testCase => {
  const requested_capacity = 4;
  const granted_capacity = storageFoundation.requestCapacitySync(requested_capacity);
  storageFoundation.releaseCapacitySync(granted_capacity);
  const available_capacity = storageFoundation.getRemainingCapacitySync();
  assert_equals(available_capacity, 0);
}, 'getRemainingCapacitySync() reports zero after a releaseCapacitySync() ' +
    'matching the capacity granted by a requestCapacitySync().');

test(testCase => {
  const requested_capacity = 4;
  const granted_capacity = storageFoundation.requestCapacitySync(requested_capacity);
  assert_greater_than_equal(granted_capacity, requested_capacity);
  testCase.add_cleanup(() => {
    storageFoundation.releaseCapacitySync(granted_capacity);
  });
}, 'requestCapacitySync() grants the requested capacity for small requests');
