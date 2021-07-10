// META: title=Synchronous NativeIO API: Flushed data is read back.
// META: global=window,worker
// META: script=resources/support.js
// META: timeout=long

'use strict';

promise_test(async testCase => {
  await reserveAndCleanupCapacity(testCase);

  const size = 1024;
  const longArray = createLargeArray(size, /*seed = */ 103);

  const file = await createFile(testCase, 'test_file', longArray);

  await file.flush();
  const readBytes = await readIoFile(file);

  assert_array_equals(readBytes, longArray,
                      'the bytes read should match the bytes written');
}, 'NativeIOFile.read returns bytes written by NativeIOFile.write' +
     ' after NativeIOFile.flush');
