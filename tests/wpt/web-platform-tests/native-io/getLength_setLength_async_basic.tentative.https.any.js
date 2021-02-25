// META: title=NativeIO API: Assigned length is observed back.
// META: global=window,worker
// META: script=resources/support.js

'use strict';

promise_test(async testCase => {
  const file = await storageFoundation.open('test_file');
  testCase.add_cleanup(async () => {
    await file.close();
    await storageFoundation.delete('test_file');
  });

  const writeSharedArrayBuffer = new SharedArrayBuffer(4);
  const writtenBytes = new Uint8Array(writeSharedArrayBuffer);
  writtenBytes.set([97, 98, 99, 100]);
  await file.write(writtenBytes, 0);

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
  const file = await storageFoundation.open('test_file');
  testCase.add_cleanup(async () => {
    await file.close();
    await storageFoundation.delete('test_file');
  });

  const writeSharedArrayBuffer = new SharedArrayBuffer(4);
  const writtenBytes = new Uint8Array(writeSharedArrayBuffer);
  writtenBytes.set([97, 98, 99, 100]);
  await file.write(writtenBytes, 0);

  await file.setLength(5);
  const readBytes = await readIoFile(file);

  const expectedBytes = Uint8Array.from([97, 98, 99, 100, 0]);

  assert_array_equals(
      readBytes, expectedBytes,
      'NativeIOFile.setLength() should append zeros when increasing' +
        ' the file size');
}, 'NativeIOFile.setLength appends zeros to a file, NativeIOFile.getLength ' +
      'reports new length');
