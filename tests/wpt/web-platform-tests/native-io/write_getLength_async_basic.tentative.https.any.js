// META: title=Synchronous NativeIO API: getLength reports written bytes.
// META: global=window,worker
// META: script=resources/support.js

'use strict';

promise_test(async testCase => {
  await reserveAndCleanupCapacity(testCase);

  const file = await createFile(testCase, 'test_file', [64, 65, 66, 67]);

  const length = await file.getLength();
  assert_equals(length, 4,
                'NativeIOFile.getLength() should return the number of' +
                ' bytes in the file');
}, 'NativeIOFile.getLength returns number of bytes written by' +
    'NativeIOFile.write');
