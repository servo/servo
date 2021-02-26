// META: title=NativeIO API: Acquiring, displaying and releasing capacity.
// META: global=window,worker

'use strict';

promise_test(async testCase => {
  const available_capacity = await storageFoundation.getRemainingCapacity();
  assert_equals(available_capacity, 0);
}, 'The starting capacity of a NativeIOFileManager is 0');

promise_test(async testCase => {
  const requested_capacity = 4;
  const granted_capacity = await storageFoundation.requestCapacity(requested_capacity);
  const available_capacity = await storageFoundation.getRemainingCapacity();
  assert_equals(available_capacity, granted_capacity);
  testCase.add_cleanup(async () => {
    await storageFoundation.releaseCapacity(available_capacity);
  });
}, 'getRemainingCapacity() reports the capacity granted by requestCapacity()');

promise_test(async testCase => {
  const requested_capacity = 4;
  const granted_capacity = await storageFoundation.requestCapacity(requested_capacity);
  await storageFoundation.releaseCapacity(granted_capacity);
  const available_capacity = await storageFoundation.getRemainingCapacity();
  assert_equals(available_capacity, 0);
}, 'getRemainingCapacity() reports zero after a releaseCapacity() matching ' +
   'the capacity granted by a requestCapacity().');

promise_test(async testCase => {
  const requested_capacity = 4;
  const granted_capacity = await storageFoundation.requestCapacity(requested_capacity);
  assert_greater_than_equal(granted_capacity, requested_capacity);
  testCase.add_cleanup(async () => {
    await storageFoundation.releaseCapacity(granted_capacity);
  });
}, 'requestCapacity() grants the requested capacity for small requests');
