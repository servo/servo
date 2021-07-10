// META: title=NativeIO API: SetLength respects the allocated capacities.
// META: global=dedicatedworker

test(testCase => {
  const file = storageFoundation.openSync('test_file');
  testCase.add_cleanup(() => {
    file.close();
    storageFoundation.deleteSync('test_file');
  });
  assert_throws_dom('QuotaExceededError', () => {file.setLength(4)});
}, 'setLength() fails without any capacity request.');

test(testCase => {
  const file = storageFoundation.openSync('test_file');

  const granted_capacity = storageFoundation.requestCapacitySync(4);
  assert_greater_than_equal(granted_capacity, 2);
  testCase.add_cleanup(() => {
    file.close();
    storageFoundation.deleteSync('test_file');
    storageFoundation.releaseCapacitySync(1);
  });

  file.setLength(granted_capacity - 1);
}, 'setLength() succeeds when given the granted capacity - 1');

test(testCase => {
  const file = storageFoundation.openSync('test_file');

  const granted_capacity = storageFoundation.requestCapacitySync(4);
  assert_greater_than_equal(granted_capacity, 1);
  testCase.add_cleanup(() => {
    file.close();
    storageFoundation.deleteSync('test_file');
  });

  file.setLength(granted_capacity);
}, 'setLength() succeeds when given the granted capacity');

test(testCase => {
  const file = storageFoundation.openSync('test_file');

  const granted_capacity = storageFoundation.requestCapacitySync(4);
  assert_greater_than_equal(granted_capacity, 0);
  testCase.add_cleanup(() => {
    file.close();
    storageFoundation.deleteSync('test_file');
    storageFoundation.releaseCapacitySync(granted_capacity);
  });

  assert_throws_dom('QuotaExceededError',
                    () => {file.setLength(granted_capacity + 1)});
}, 'setLength() fails when given the granted capacity + 1');
