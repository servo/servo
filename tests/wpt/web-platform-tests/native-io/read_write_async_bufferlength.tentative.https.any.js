// META: title=NativeIO API: Read/Write correctly handle small buffer lengths.
// META: global=window,worker
// META: script=resources/support.js

'use strict';


promise_test(async testCase => {
  await reserveAndCleanupCapacity(testCase);
  const file = await createFile(testCase, 'test_file');

  for (let bufferLength = 0; bufferLength < 4; bufferLength++) {
    let writeBuffer = new Uint8Array(bufferLength);
    let writeResult = await file.write(writeBuffer, 0);
    assert_equals(
        writeResult.writtenBytes, bufferLength,
        'NativeIOFile.write() should return success if the buffer size is ' +
            `${bufferLength}.`);
  }
}, 'NativeIOFile.write succeeds when writing a small number of bytes');

promise_test(async testCase => {
  await reserveAndCleanupCapacity(testCase);
  const file = await createFile(testCase, 'test_file');

  for (let bufferLength = 0; bufferLength < 4; bufferLength++) {
    const readBuffer = new Uint8Array(bufferLength);
    const readResult = await file.read(readBuffer, 0);
    assert_equals(
        readResult.readBytes, bufferLength,
        'NativeIOFile.read() should return success if the buffer size is ' +
            `${bufferLength}.`);
  }
}, 'NativeIOFile.read succeeds when reading a small number of bytes');
