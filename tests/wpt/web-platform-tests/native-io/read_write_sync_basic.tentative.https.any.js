// META: title=Synchronous NativeIO API: Written bytes are read back.
// META: global=dedicatedworker
// META: script=resources/support.js

'use strict';


test(testCase => {
  reserveAndCleanupCapacitySync(testCase);

  const file = storageFoundation.openSync('test_file');
  testCase.add_cleanup(() => {
    file.close();
    storageFoundation.deleteSync('test_file');
  });

  const {buffer: writeBuffer, writtenBytes} = file.write(new Uint8Array([64, 65, 66, 67]) , 0);
  assert_equals(
      writtenBytes, 4,
      'NativeIOFileSync.write() should resolve with the number of bytes written');

  const {buffer: readBuffer, readBytes} = file.read(new Uint8Array(4), 0);
  assert_equals(
      readBytes, 4,
      'NativeIOFileSync.read() should return the number of bytes read');

  assert_array_equals(
      readBuffer, writeBuffer, 'the bytes read should match the bytes written');
}, 'NativeIOFileSync.read returns bytes written by NativeIOFileSync.write');

test(testCase => {
  reserveAndCleanupCapacitySync(testCase);
  const file = createFileSync(testCase, 'test_file');

  const inputBuffer = new Uint8Array(4);
  const originalByteLength = inputBuffer.byteLength;
  const readResult = file.read(inputBuffer, 0);

  assert_equals(
      readResult.buffer.byteLength, originalByteLength,
      'NativeIOFileSync.read() should return a buffer with the same ' +
          'byteLength as the input buffer');

  assert_equals(
      inputBuffer.byteLength, 0,
      'NativeIOFileSync.read() should detach the input buffer');
}, 'NativeIOFileSync.read detaches the input buffer');

test(testCase => {
  reserveAndCleanupCapacitySync(testCase);
  const file = createFileSync(testCase, 'test_file');

  const inputBuffer = new Uint8Array(4);
  const originalByteLength = inputBuffer.byteLength;
  const writeResult = file.write(inputBuffer, 0);

  assert_equals(
      writeResult.buffer.byteLength, originalByteLength,
      'NativeIOFileSync.write() should return a buffer with the same ' +
          'byteLength as the input buffer');

  assert_equals(
      inputBuffer.byteLength, 0,
      'NativeIOFileSync.write() should detach the input buffer');
}, 'NativeIOFileSync.write detaches the input buffer');
