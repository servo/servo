// META: title=NativeIO API: SetLength respects the allocated capacities.
// META: global=window,worker

promise_test(async testCase => {
  const file = await storageFoundation.open('test_file');
  testCase.add_cleanup(async () => {
    await file.close();
    await storageFoundation.delete('test_file');
  });
  await promise_rejects_dom(testCase, 'QuotaExceededError', file.setLength(4));
}, 'setLength() fails without any capacity request.');

promise_test(async testCase => {
  const file = await storageFoundation.open('test_file');

  const granted_capacity = await storageFoundation.requestCapacity(4);
  assert_greater_than_equal(granted_capacity, 2);
  testCase.add_cleanup(async () => {
    await file.close();
    await storageFoundation.delete('test_file');
    await storageFoundation.releaseCapacity(1);
  });

  file.setLength(granted_capacity - 1);
}, 'setLength() succeeds when given the granted capacity - 1');

promise_test(async testCase => {
  const file = await storageFoundation.open('test_file');

  const granted_capacity = await storageFoundation.requestCapacity(4);
  assert_greater_than_equal(granted_capacity, 1);
  testCase.add_cleanup(async () => {
    await file.close();
    await storageFoundation.delete('test_file');
  });

  file.setLength(granted_capacity);
}, 'setLength() succeeds when given the granted capacity');

promise_test(async testCase => {
  const file = await storageFoundation.open('test_file');

  const granted_capacity = await storageFoundation.requestCapacity(4);
  assert_greater_than_equal(granted_capacity, 0);
  testCase.add_cleanup(async () => {
    await file.close();
    await storageFoundation.delete('test_file');
    await storageFoundation.releaseCapacity(granted_capacity);
  });

  await promise_rejects_dom(
    testCase, 'QuotaExceededError', file.setLength(granted_capacity + 1));
}, 'setLength() fails when given the granted capacity + 1');
