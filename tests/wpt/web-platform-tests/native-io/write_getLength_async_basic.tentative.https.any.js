// META: title=Synchronous NativeIO API: getLength reports written bytes.
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

  const length = await file.getLength();
  assert_equals(length, 4,
                'NativeIOFile.getLength() should return the number of' +
                ' bytes in the file');
}, 'NativeIOFile.getLength returns number of bytes written by' +
    'NativeIOFile.write');
