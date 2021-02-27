// META: title=NativeIO API: Write respects the allocated capacities.
// META: global=window,worker

promise_test(async testCase => {
  const file = await storageFoundation.open('test_file');
  testCase.add_cleanup(async () => {
    await file.close();
    await storageFoundation.delete('test_file');
  });
  const writeSharedArrayBuffer = new SharedArrayBuffer(4);
  const writtenBytes = new Uint8Array(writeSharedArrayBuffer);
  writtenBytes.set([64, 65, 66, 67]);
  await promise_rejects_dom(
    testCase, 'QuotaExceededError', file.write(writtenBytes, 0));
}, 'write() fails without any capacity request.');

promise_test(async testCase => {
  const file = await storageFoundation.open('test_file');

  const granted_capacity = await storageFoundation.requestCapacity(4);
  assert_greater_than_equal(granted_capacity, 2);
  testCase.add_cleanup(async () => {
    await file.close();
    await storageFoundation.delete('test_file');
    await storageFoundation.releaseCapacity(1);
  });
  const writeSharedArrayBuffer = new SharedArrayBuffer(granted_capacity - 1);
  const writtenBytes = new Uint8Array(writeSharedArrayBuffer);
  writtenBytes.set(Array(granted_capacity - 1).fill(64));

  file.write(writtenBytes, 0);
}, 'write() succeeds when given a buffer of length granted capacity - 1');

promise_test(async testCase => {
  const file = await storageFoundation.open('test_file');

  const granted_capacity = await storageFoundation.requestCapacity(4);
  assert_greater_than_equal(granted_capacity, 2);
  testCase.add_cleanup(async () => {
    await file.close();
    await storageFoundation.delete('test_file');
  });
  const writeSharedArrayBuffer = new SharedArrayBuffer(granted_capacity);
  const writtenBytes = new Uint8Array(writeSharedArrayBuffer);
  writtenBytes.set(Array(granted_capacity).fill(64));

  file.write(writtenBytes, 0);
}, 'write() succeeds when given the granted capacity');

promise_test(async testCase => {
  const file = await storageFoundation.open('test_file');

  const granted_capacity = await storageFoundation.requestCapacity(4);
  assert_greater_than_equal(granted_capacity, 2);
  testCase.add_cleanup(async () => {
    await file.close();
    await storageFoundation.delete('test_file');
    await storageFoundation.releaseCapacity(granted_capacity);
  });
  const writeSharedArrayBuffer = new SharedArrayBuffer(granted_capacity + 1);
  const writtenBytes = new Uint8Array(writeSharedArrayBuffer);
  writtenBytes.set(Array(granted_capacity + 1).fill(64));

  await promise_rejects_dom(testCase,
    'QuotaExceededError', file.write(writtenBytes, 0));
}, 'write() fails when given the granted capacity + 1');
