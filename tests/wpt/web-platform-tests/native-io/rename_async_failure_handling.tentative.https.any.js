// META: title=NativeIO API: Failures of rename are properly handled.
// META: global=window,worker
// META: script=resources/support.js
// META: timeout=long

'use strict';

setup(async () => {
  assert_implements(nativeIO.rename, 'nativeIO.rename is not implemented.');
});

promise_test(async testCase => {
  const file1 = await nativeIO.open('test_file_1');
  const file2 = await nativeIO.open('test_file_2');
  testCase.add_cleanup(async () => {
    await file1.close();
    await file2.close();
  });

  const writeSharedArrayBuffer1 = new SharedArrayBuffer(4);
  const writtenBytes1 = new Uint8Array(writeSharedArrayBuffer1);
  writtenBytes1.set([64, 65, 66, 67]);
  const writeSharedArrayBuffer2 = new SharedArrayBuffer(4);
  const writtenBytes2 = new Uint8Array(writeSharedArrayBuffer2);
  writtenBytes2.set([96, 97, 98, 99]);

  await file1.write(writtenBytes1, 0);
  await file2.write(writtenBytes2, 0);
  await file1.close();
  await file2.close();

  await promise_rejects_dom(
      testCase, 'NoModificationAllowedError',
      nativeIO.rename('test_file_1', 'test_file_2'));

  const fileNamesAfterRename = await nativeIO.getAll();
  assert_in_array('test_file_1', fileNamesAfterRename);
  assert_in_array('test_file_2', fileNamesAfterRename);

  // Make sure that a failed rename does not modify file contents.
  const file1_after = await nativeIO.open('test_file_1');
  const file2_after = await nativeIO.open('test_file_2');

  testCase.add_cleanup(async () => {
    await file1_after.close();
    await file2_after.close();
    await nativeIO.delete('test_file_1');
    await nativeIO.delete('test_file_2');
  });

  const readSharedArrayBuffer1 = new SharedArrayBuffer(writtenBytes1.length);
  const readBytes1 = new Uint8Array(readSharedArrayBuffer1);
  await file1_after.read(readBytes1, 0);
  const readSharedArrayBuffer2 = new SharedArrayBuffer(writtenBytes2.length);
  const readBytes2 = new Uint8Array(readSharedArrayBuffer2);
  await file2_after.read(readBytes2, 0);
  assert_array_equals(
      readBytes1, writtenBytes1,
      'the bytes read should match the bytes written');
  assert_array_equals(
      readBytes2, writtenBytes2,
      'the bytes read should match the bytes written');
}, 'nativeIO.rename does not overwrite an existing file.');

promise_test(async testCase => {
  const file = await nativeIO.open('test_file');
  testCase.add_cleanup(async () => {
    await file.close();
    await nativeIO.delete('test_file');
  });
  await promise_rejects_dom(
      testCase, 'NoModificationAllowedError',
      nativeIO.rename('test_file', 'renamed_test_file'));
  await file.close();

  const fileNamesAfterRename = await nativeIO.getAll();
  assert_false(fileNamesAfterRename.includes('renamed_test_file'));
  assert_in_array('test_file', fileNamesAfterRename);
}, 'nativeIO.rename does not allow renaming an open file.');

promise_test(async testCase => {
  testCase.add_cleanup(async () => {
    await file.close();
    await nativeIO.delete('test_file');
    for (let name of await nativeIO.getAll()) {
      await nativeIO.delete(name);
    }
  });

  const file = await nativeIO.open('test_file');
  await file.close();
  for (let name of kBadNativeIoNames) {
    await promise_rejects_js(
        testCase, TypeError, nativeIO.rename('test_file', name));
    await promise_rejects_js(
        testCase, TypeError, nativeIO.rename(name, 'test_file_2'));
  }
}, 'nativeIO.rename does not allow renaming from or to invalid names.');

promise_test(async testCase => {
  const closed_file = await nativeIO.open('closed_file');
  closed_file.close();
  const opened_file = await nativeIO.open('opened_file');
  testCase.add_cleanup(async () => {
    await closed_file.close();
    await opened_file.close();
    await nativeIO.delete('closed_file');
    await nativeIO.delete('opened_file');
  });

  // First rename fails, as source is still open.
  await promise_rejects_dom(
      testCase, 'NoModificationAllowedError',
      nativeIO.rename('opened_file', 'closed_file'));
  // First rename fails again, as source has not been unlocked.
  await promise_rejects_dom(
      testCase, 'NoModificationAllowedError',
      nativeIO.rename('opened_file', 'closed_file'));
}, 'Failed nativeIO.rename does not unlock the source.');

promise_test(async testCase => {
  const closed_file = await nativeIO.open('closed_file');
  await closed_file.close();
  const opened_file = await nativeIO.open('opened_file');
  testCase.add_cleanup(async () => {
    await closed_file.close();
    await opened_file.close();
    await nativeIO.delete('closed_file');
    await nativeIO.delete('opened_file');
  });

  // First rename fails, as destination is still open.
  await promise_rejects_dom(
      testCase, 'NoModificationAllowedError',
      nativeIO.rename('closed_file', 'opened_file'));
  // First rename fails again, as destination has not been unlocked.
  await promise_rejects_dom(
      testCase, 'NoModificationAllowedError',
      nativeIO.rename('closed_file', 'opened_file'));
}, 'Failed nativeIO.rename does not unlock the destination.');

promise_test(async testCase => {
  // Make sure that the file does not exist.
  await nativeIO.delete('does_not_exist');
  testCase.add_cleanup(async () => {
    await nativeIO.delete('new_does_not_exist');
  });
  await promise_rejects_dom(
      testCase, 'NotFoundError',
      nativeIO.rename('does_not_exist', 'new_does_not_exist'));
}, 'Renaming a non-existing file fails with a NotFoundError.');
