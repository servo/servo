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
  assert_precondition(writeCount == 4);

  return file;
}

promise_test(async testCase => {
  const file = await createFile(testCase, 'file_name');
  assert_equals(await file.close(), undefined);

  assert_equals(await file.close(), undefined);
}, 'NativeIOFile.close is idempotent');

promise_test(async testCase => {
  const file = await createFile(testCase, 'file_name');
  const closePromise = file.close();

  assert_equals(await file.close(), undefined);
  assert_equals(await closePromise, undefined);
}, 'NativeIOFile.close is idempotent when called immediately');

promise_test(async testCase => {
  const file = await createFile(testCase, 'file_name');
  assert_equals(await file.close(), undefined);

  const readSharedArrayBuffer = new SharedArrayBuffer(4);
  const readBytes = new Uint8Array(readSharedArrayBuffer);
  await promise_rejects_dom(testCase, 'InvalidStateError',
                            file.read(readBytes, 4));
}, 'NativeIOFile.read fails after NativeIOFile.close settles');

promise_test(async testCase => {
  const file = await createFile(testCase, 'file_name');
  const closePromise = file.close();

  const readSharedArrayBuffer = new SharedArrayBuffer(4);
  const readBytes = new Uint8Array(readSharedArrayBuffer);
  await promise_rejects_dom(testCase, 'InvalidStateError',
                            file.read(readBytes, 4));
  assert_equals(await closePromise, undefined);
}, 'NativeIOFile.read fails immediately after calling NativeIOFile.close');

promise_test(async testCase => {
  const file = await createFile(testCase, 'file_name');
  assert_equals(await file.close(), undefined);

  const writeSharedArrayBuffer = new SharedArrayBuffer(4);
  const writtenBytes = new Uint8Array(writeSharedArrayBuffer);
  writtenBytes.set([96, 97, 98, 99]);
  await promise_rejects_dom(testCase, 'InvalidStateError',
                            file.write(writtenBytes, 4));
}, 'NativeIOFile.write fails after NativeIOFile.close settles');

promise_test(async testCase => {
  const file = await createFile(testCase, 'file_name');
  const closePromise = file.close();

  const writeSharedArrayBuffer = new SharedArrayBuffer(4);
  const writtenBytes = new Uint8Array(writeSharedArrayBuffer);
  writtenBytes.set([96, 97, 98, 99]);
  await promise_rejects_dom(testCase, 'InvalidStateError',
                            file.write(writtenBytes, 4));
  assert_equals(await closePromise, undefined);
}, 'NativeIOFile.write fails immediately after calling NativeIOFile.close');
