// META: title=Synchronous NativeIO API: Read/Write correctly handle small buffer lengths.
// META: global=dedicatedworker
// META: script=resources/support.js

'use strict';


test(testCase => {
  reserveAndCleanupCapacitySync(testCase);
  const file = createFileSync(testCase, 'test_file');

  for (let bufferLength = 0; bufferLength < 4; bufferLength++) {
    let writeBuffer = new Uint8Array(bufferLength);
    let writeResult = file.write(writeBuffer, 0);
    assert_equals(
        writeResult.writtenBytes, bufferLength,
        'NativeIOFileSync.write() should return success if the buffer size' +
            ` is ${bufferLength}.`);
  }
}, 'NativeIOFileSync.write succeeds when writing small number of bytes');

test(testCase => {
  reserveAndCleanupCapacitySync(testCase);
  const file = createFileSync(testCase, 'test_file');

  for (let bufferLength = 0; bufferLength < 4; bufferLength++) {
    let readBuffer = new Uint8Array(bufferLength);
    let readResult = file.read(readBuffer, 0);
    assert_equals(
        readResult.readBytes, bufferLength,
        'NativeIOFileSync.read() should return success if the buffer size' +
            ` is ${bufferLength}.`);
  }
}, 'NativeIOFileSync.read succeeds when reading small number of bytes');
