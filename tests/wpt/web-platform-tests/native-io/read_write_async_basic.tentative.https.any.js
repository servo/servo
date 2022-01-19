// META: title=NativeIO API: Written bytes are read back.
// META: global=window,worker
// META: script=resources/support.js

'use strict';

promise_test(async testCase => {
  await reserveAndCleanupCapacity(testCase);

  const file = await storageFoundation.open('test_file');
  testCase.add_cleanup(async () => {
    await file.close();
    await storageFoundation.delete('test_file');
  });

  const {buffer: writeBuffer, writtenBytes} =
      await file.write(new Uint8Array([64, 65, 66, 67]), 0);
  assert_equals(
      writtenBytes, 4,
      'NativeIOFile.write() should resolve with the number of bytes written');

  const {buffer: readBuffer, readBytes} = await file.read(new Uint8Array(4), 0);
  assert_equals(
      readBytes, 4,
      'NativeIOFile.read() should return the number of bytes read');

  assert_array_equals(
      readBuffer, writeBuffer, 'the bytes read should match the bytes written');
}, 'NativeIOFile.read returns bytes written by NativeIOFile.write');

promise_test(async testCase => {
  await reserveAndCleanupCapacity(testCase);
  const file = await createFile(testCase, 'test_file');

  const inputBuffer = new Uint8Array(4);
  const readPromise = file.read(inputBuffer, 0);
  assert_equals(
      inputBuffer.byteLength, 0,
      'NativeIOFile.read() should detach the input buffer immediately');

  const readResult = await readPromise;
  assert_equals(
      readResult.buffer.byteLength, 4,
      'NativeIOFile.read() should return a buffer with the same byteLength as' +
          'the input buffer');
}, 'NativeIOFile.read detaches the input buffer');

promise_test(async testCase => {
  await reserveAndCleanupCapacity(testCase);
  const file = await createFile(testCase, 'test_file');

  const inputBuffer = new Uint8Array(4);
  const writePromise = file.write(inputBuffer, 0);
  assert_equals(
      inputBuffer.byteLength, 0,
      'NativeIOFile.write() should detach the input buffer immediately');

  const writeResult = await writePromise;
  assert_equals(
      writeResult.buffer.byteLength, 4,
      'NativeIOFile.write() should return a buffer with the same byteLength' +
          'as the input buffer');
}, 'NativeIOFile.write detaches the input buffer');
