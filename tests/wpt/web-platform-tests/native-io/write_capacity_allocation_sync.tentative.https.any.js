// META: title=NativeIO API: Write respects the allocated capacities.
// META: global=dedicatedworker

test(testCase => {
  const file =  storageFoundation.openSync('test_file');
  testCase.add_cleanup(() => {
    file.close();
    storageFoundation.deleteSync('test_file');
  });
  const writeBuffer = Uint8Array.from([64, 65, 66, 67]);
  assert_throws_dom('QuotaExceededError', () => {file.write(writeBuffer, 0)});
}, 'NativeIOFileSync.write() fails without any capacity request.');

test(testCase => {
  const file =  storageFoundation.openSync('test_file');

  const granted_capacity =  storageFoundation.requestCapacitySync(4);
  assert_greater_than_equal(granted_capacity, 2);

  testCase.add_cleanup(() => {
    file.close();
    storageFoundation.deleteSync('test_file');
    storageFoundation.releaseCapacitySync(granted_capacity);
  });

  const writeBuffer = new Uint8Array(granted_capacity - 1).fill(64);
  file.write(writeBuffer, 0);
}, 'NativeIOFileSync.write() succeeds when given a buffer of length ' +
     'granted capacity - 1');

test(testCase => {
  const file =  storageFoundation.openSync('test_file');

  const granted_capacity =  storageFoundation.requestCapacitySync(4);
  assert_greater_than_equal(granted_capacity, 2);
  testCase.add_cleanup(() => {
    file.close();
    storageFoundation.deleteSync('test_file');
    storageFoundation.releaseCapacitySync(granted_capacity);
  });
  const writeBuffer = new Uint8Array(granted_capacity).fill(64);

  file.write(writeBuffer, 0);
}, 'NativeIOFileSync.write() succeeds when given the granted capacity');

test(testCase => {
  const file =  storageFoundation.openSync('test_file');

  const granted_capacity =  storageFoundation.requestCapacitySync(4);
  assert_greater_than_equal(granted_capacity, 2);
  testCase.add_cleanup(() => {
    file.close();
    storageFoundation.deleteSync('test_file');
    storageFoundation.releaseCapacitySync(granted_capacity);
  });
  const writeBuffer = new Uint8Array(granted_capacity + 1).fill(64);

  assert_throws_dom('QuotaExceededError', () => {file.write(writeBuffer, 0)});
}, 'NativeIOFileSync.write() fails when given the granted capacity + 1');
