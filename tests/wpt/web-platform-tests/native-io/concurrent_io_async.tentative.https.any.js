// META: title=NativeIO API: close().
// META: global=window,worker

'use strict';

// Returns a handle to a newly created file that holds some data.
//
// The file will be closed and deleted when the test ends.
async function createFile(testCase, fileName) {
  const file = await nativeIO.open(fileName);
  testCase.add_cleanup(async () => {
    await file.close();
    await nativeIO.delete('test_file');
  });

  const writeSharedArrayBuffer = new SharedArrayBuffer(4);
  const writtenBytes = new Uint8Array(writeSharedArrayBuffer);
  writtenBytes.set([64, 65, 66, 67]);
  const writeCount = await file.write(writtenBytes, 0);
  assert_equals(writeCount, 4);

  return file;
}

promise_test(async testCase => {
  const file = await createFile(testCase, 'file_name');

  const readSharedArrayBuffer = new SharedArrayBuffer(4);
  const readBytes = new Uint8Array(readSharedArrayBuffer);
  const readSharedArrayBuffer2 = new SharedArrayBuffer(4);
  const readBytes2 = new Uint8Array(readSharedArrayBuffer2);

  const readPromise = file.read(readBytes, 0);
  await promise_rejects_dom(testCase, 'InvalidStateError',
                            file.read(readBytes2, 4));

  assert_equals(await readPromise, 4);
  assert_array_equals(readBytes, [64, 65, 66, 67]);
  assert_array_equals(readBytes2, [0, 0, 0, 0]);
}, 'read() rejects wrile read() is resolving');

promise_test(async testCase => {
  const file = await createFile(testCase, 'file_name');

  const writeSharedArrayBuffer = new SharedArrayBuffer(4);
  const writtenBytes = new Uint8Array(writeSharedArrayBuffer);
  writtenBytes.set([96, 97, 98, 99]);
  const readSharedArrayBuffer = new SharedArrayBuffer(4);
  const readBytes = new Uint8Array(readSharedArrayBuffer);

  const writePromise = file.write(writtenBytes, 0);
  await promise_rejects_dom(testCase, 'InvalidStateError',
                            file.read(readBytes, 4));

  assert_equals(await writePromise, 4);
  assert_array_equals(readBytes, [0, 0, 0, 0]);
}, 'read() rejects wrile write() is resolving');

promise_test(async testCase => {
  const file = await createFile(testCase, 'file_name');

  const readSharedArrayBuffer = new SharedArrayBuffer(4);
  const readBytes = new Uint8Array(readSharedArrayBuffer);
  const writeSharedArrayBuffer = new SharedArrayBuffer(4);
  const writtenBytes = new Uint8Array(writeSharedArrayBuffer);
  writtenBytes.set([96, 97, 98, 99]);

  const readPromise = file.read(readBytes, 0);
  await promise_rejects_dom(testCase, 'InvalidStateError',
                            file.write(writtenBytes, 4));

  assert_equals(await readPromise, 4);
  assert_array_equals(readBytes, [64, 65, 66, 67]);

  readBytes.fill(0);
  assert_equals(await file.read(readBytes, 0), 4,
                'NativeIOFile.read() should not fail after a rejected write');
  assert_array_equals(readBytes, [64, 65, 66, 67],
                      'The rejected write should not change the file');
}, 'write() rejects wrile read() is resolving');

promise_test(async testCase => {
  const file = await createFile(testCase, 'file_name');

  const writeSharedArrayBuffer = new SharedArrayBuffer(4);
  const writtenBytes = new Uint8Array(writeSharedArrayBuffer);
  writtenBytes.set([96, 97, 98, 99]);

  const writeSharedArrayBuffer2 = new SharedArrayBuffer(4);
  const writtenBytes2 = new Uint8Array(writeSharedArrayBuffer2);
  writtenBytes2.set([48, 49, 50, 51]);

  const writePromise = file.write(writtenBytes, 0);
  await promise_rejects_dom(testCase, 'InvalidStateError',
                            file.write(writtenBytes2, 4));

  assert_equals(await writePromise, 4);

  const readSharedArrayBuffer = new SharedArrayBuffer(4);
  const readBytes = new Uint8Array(readSharedArrayBuffer);
  assert_equals(await file.read(readBytes, 0), 4,
                'NativeIOFile.read() should not fail after a rejected write');
  assert_array_equals(readBytes, writtenBytes,
                      'The rejected write should not change the file');
}, 'write() rejects wrile write() is resolving');
