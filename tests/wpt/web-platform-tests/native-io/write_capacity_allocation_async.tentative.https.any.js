// META: title=NativeIO API: Write respects the allocated capacities.
// META: global=window,worker

promise_test(async testCase => {
  const file = await storageFoundation.open('test_file');
  testCase.add_cleanup(async () => {
    await file.close();
    await storageFoundation.delete('test_file');
  });
  const writeBuffer = new Uint8Array(4);
  writeBuffer.set([64, 65, 66, 67]);
  await promise_rejects_dom(
    testCase, 'QuotaExceededError', file.write(writeBuffer, 0));
}, 'NativeIOFile.write() fails without any capacity request.');

promise_test(async testCase => {
  const file = await storageFoundation.open('test_file');

  const granted_capacity = await storageFoundation.requestCapacity(4);
  assert_greater_than_equal(granted_capacity, 2);
  testCase.add_cleanup(async () => {
    await file.close();
    await storageFoundation.delete('test_file');
    await storageFoundation.releaseCapacity(granted_capacity);
  });
  const writeBuffer = new Uint8Array(granted_capacity - 1);
  writeBuffer.set(Array(granted_capacity - 1).fill(64));

  const {writtenBytes} = await file.write(writeBuffer, 0);
  assert_equals(writtenBytes, granted_capacity - 1);
}, 'NativeIOFile.write() succeeds when given a buffer of length ' +
     'granted capacity - 1');

promise_test(async testCase => {
  const file = await storageFoundation.open('test_file');

  const granted_capacity = await storageFoundation.requestCapacity(4);
  assert_greater_than_equal(granted_capacity, 2);
  testCase.add_cleanup(async () => {
    await file.close();
    await storageFoundation.delete('test_file');
  });
  const writeBuffer = new Uint8Array(granted_capacity);
  writeBuffer.set(Array(granted_capacity).fill(64));

  const {writtenBytes} = await file.write(writeBuffer, 0);
  assert_equals(writtenBytes, granted_capacity);
}, 'NativeIOFile.write() succeeds when given the granted capacity');

promise_test(async testCase => {
  const file = await storageFoundation.open('test_file');

  const granted_capacity = await storageFoundation.requestCapacity(4);
  assert_greater_than_equal(granted_capacity, 2);
  testCase.add_cleanup(async () => {
    await file.close();
    await storageFoundation.delete('test_file');
    await storageFoundation.releaseCapacity(granted_capacity);
  });
  const writeBuffer = new Uint8Array(granted_capacity + 1);
  writeBuffer.set(Array(granted_capacity + 1).fill(64));

  await promise_rejects_dom(testCase,
    'QuotaExceededError', file.write(writeBuffer, 0));
}, 'NativeIOFile.write() fails when given the granted capacity + 1');
