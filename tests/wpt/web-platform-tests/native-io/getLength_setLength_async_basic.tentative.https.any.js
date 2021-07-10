// META: title=NativeIO API: Assigned length is observed back.
// META: global=window,worker
// META: script=resources/support.js

'use strict';

promise_test(async testCase => {
  await reserveAndCleanupCapacity(testCase);

  const file = await createFile(testCase, 'test_file', [97, 98, 99, 100]);

  await file.setLength(3);
  const readBytes = await readIoFile(file);

  const remainingBytes = Uint8Array.from([97, 98, 99]);

  assert_array_equals(
      readBytes, remainingBytes,
      'NativeIOFile.setLength() should remove bytes from the end of ' +
        'a file when decreasing its length');
}, 'NativeIOFile.setLength shrinks a file, NativeIOFile.getLength reports ' +
      'new length');

promise_test(async testCase => {
  await reserveAndCleanupCapacity(testCase);

  const file = await createFile(testCase, 'test_file', [97, 98, 99, 100]);

  await file.setLength(5);
  const readBytes = await readIoFile(file);

  const expectedBytes = Uint8Array.from([97, 98, 99, 100, 0]);

  assert_array_equals(
      readBytes, expectedBytes,
      'NativeIOFile.setLength() should append zeros when increasing' +
        ' the file size');
}, 'NativeIOFile.setLength appends zeros to a file, NativeIOFile.getLength ' +
      'reports new length');
