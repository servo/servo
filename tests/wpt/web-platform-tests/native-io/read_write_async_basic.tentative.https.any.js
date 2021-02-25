// META: title=NativeIO API: Written bytes are read back.
// META: global=window,worker

'use strict';

promise_test(async testCase => {
  const file = await storageFoundation.open('test_file');
  testCase.add_cleanup(async () => {
    await file.close();
    await storageFoundation.delete('test_file');
  });

  const writeSharedArrayBuffer = new SharedArrayBuffer(4);
  const writtenBytes = new Uint8Array(writeSharedArrayBuffer);
  writtenBytes.set([64, 65, 66, 67]);
  const writeCount = await file.write(writtenBytes, 0);
  assert_equals(
      writeCount, 4,
      'NativeIOFile.write() should resolve with the number of bytes written');

  const readSharedArrayBuffer = new SharedArrayBuffer(writtenBytes.length);
  const readBytes = new Uint8Array(readSharedArrayBuffer);
  const readCount = await file.read(readBytes, 0);
  assert_equals(readCount, 4,
                'NativeIOFile.read() should return the number of bytes read');

  assert_array_equals(readBytes, writtenBytes,
                      'the bytes read should match the bytes written');
}, 'NativeIOFile.read returns bytes written by NativeIOFile.write');
