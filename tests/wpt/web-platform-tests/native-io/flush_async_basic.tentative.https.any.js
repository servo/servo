// META: title=Synchronous NativeIO API: Flushed data is read back.
// META: global=window,worker
// META: script=resources/support.js
// META: timeout=long

'use strict';

promise_test(async testCase => {
  const file = await storageFoundation.open('test_file');
  testCase.add_cleanup(async () => {
    await file.close();
    await storageFoundation.delete('test_file');
  });

  const size = 1024;
  const longarray = createLargeArray(size, /*seed = */ 103);
  const writeSharedArrayBuffer = new SharedArrayBuffer(size);
  const writtenBytes = new Uint8Array(writeSharedArrayBuffer);
  writtenBytes.set(longarray);
  const writeCount = await file.write(writtenBytes, 0);
  assert_equals(
      writeCount, size,
      'NativeIOFile.write() should resolve with the number of bytes written');

  await file.flush();
  const readBytes = await readIoFile(file);

  assert_array_equals(readBytes, writtenBytes,
                      'the bytes read should match the bytes written');
}, 'NativeIOFile.read returns bytes written by NativeIOFile.write' +
     ' after NativeIOFile.flush');
