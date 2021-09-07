// META: title=NativeIO API: Failures of rename are properly handled.
// META: global=window,worker
// META: script=resources/support.js
// META: timeout=long

'use strict';

setup(async () => {
  assert_implements(storageFoundation.rename, 'storageFoundation.rename is not' +
                                                ' implemented.');
});

promise_test(async testCase => {
  await reserveAndCleanupCapacity(testCase);

  const file1 = await storageFoundation.open('test_file_1');
  const file2 = await storageFoundation.open('test_file_2');
  testCase.add_cleanup(async () => {
    await file1.close();
    await file2.close();
  });


  const {buffer: writeBuffer1} = await file1.write(new Uint8Array([64, 65, 66, 67]), 0);
  const {buffer: writeBuffer2} = await file2.write(new Uint8Array([96, 97, 98, 99]), 0);

  await file1.close();
  await file2.close();

  await promise_rejects_dom(
      testCase, 'NoModificationAllowedError',
      storageFoundation.rename('test_file_1', 'test_file_2'));

  const fileNamesAfterRename = await storageFoundation.getAll();
  assert_in_array('test_file_1', fileNamesAfterRename);
  assert_in_array('test_file_2', fileNamesAfterRename);

  // Make sure that a failed rename does not modify file contents.
  const file1_after = await storageFoundation.open('test_file_1');
  const file2_after = await storageFoundation.open('test_file_2');

  testCase.add_cleanup(async () => {
    await file1_after.close();
    await file2_after.close();
    await storageFoundation.delete('test_file_1');
    await storageFoundation.delete('test_file_2');
  });

  const {buffer: readBuffer1} = await file1_after.read(new Uint8Array(4), 0);
  const {buffer: readBuffer2} = await file2_after.read(new Uint8Array(4), 0);
  assert_array_equals(
      readBuffer1, writeBuffer1,
      'the bytes read should match the bytes written');
  assert_array_equals(
      readBuffer2, writeBuffer2,
      'the bytes read should match the bytes written');
}, 'storageFoundation.rename does not overwrite an existing file.');

promise_test(async testCase => {
  await reserveAndCleanupCapacity(testCase);

  const file = await storageFoundation.open('test_file');
  testCase.add_cleanup(async () => {
    await file.close();
    await storageFoundation.delete('test_file');
  });
  await promise_rejects_dom(
      testCase, 'NoModificationAllowedError',
      storageFoundation.rename('test_file', 'renamed_test_file'));
  await file.close();

  const fileNamesAfterRename = await storageFoundation.getAll();
  assert_false(fileNamesAfterRename.includes('renamed_test_file'));
  assert_in_array('test_file', fileNamesAfterRename);
}, 'storageFoundation.rename does not allow renaming an open file.');

promise_test(async testCase => {
  await reserveAndCleanupCapacity(testCase);

  testCase.add_cleanup(async () => {
    await file.close();
    await storageFoundation.delete('test_file');
    for (let name of await storageFoundation.getAll()) {
      await storageFoundation.delete(name);
    }
  });

  const file = await storageFoundation.open('test_file');
  await file.close();
  for (let name of kBadNativeIoNames) {
    await promise_rejects_js(
        testCase, TypeError, storageFoundation.rename('test_file', name));
    await promise_rejects_js(
        testCase, TypeError, storageFoundation.rename(name, 'test_file_2'));
  }
}, 'storageFoundation.rename does not allow renaming from or to invalid names.');

promise_test(async testCase => {
  await reserveAndCleanupCapacity(testCase);

  const closed_file = await storageFoundation.open('closed_file');
  closed_file.close();
  const opened_file = await storageFoundation.open('opened_file');
  testCase.add_cleanup(async () => {
    await closed_file.close();
    await opened_file.close();
    await storageFoundation.delete('closed_file');
    await storageFoundation.delete('opened_file');
  });

  // First rename fails, as source is still open.
  await promise_rejects_dom(
      testCase, 'NoModificationAllowedError',
      storageFoundation.rename('opened_file', 'closed_file'));
  // First rename fails again, as source has not been unlocked.
  await promise_rejects_dom(
      testCase, 'NoModificationAllowedError',
      storageFoundation.rename('opened_file', 'closed_file'));
}, 'Failed storageFoundation.rename does not unlock the source.');

promise_test(async testCase => {
  await reserveAndCleanupCapacity(testCase);

  const closed_file = await storageFoundation.open('closed_file');
  await closed_file.close();
  const opened_file = await storageFoundation.open('opened_file');
  testCase.add_cleanup(async () => {
    await closed_file.close();
    await opened_file.close();
    await storageFoundation.delete('closed_file');
    await storageFoundation.delete('opened_file');
  });

  // First rename fails, as destination is still open.
  await promise_rejects_dom(
      testCase, 'NoModificationAllowedError',
      storageFoundation.rename('closed_file', 'opened_file'));
  // First rename fails again, as destination has not been unlocked.
  await promise_rejects_dom(
      testCase, 'NoModificationAllowedError',
      storageFoundation.rename('closed_file', 'opened_file'));
}, 'Failed storageFoundation.rename does not unlock the destination.');

promise_test(async testCase => {
  await reserveAndCleanupCapacity(testCase);

  // Make sure that the file does not exist.
  await storageFoundation.delete('does_not_exist');
  testCase.add_cleanup(async () => {
    await storageFoundation.delete('new_does_not_exist');
  });
  await promise_rejects_dom(
      testCase, 'NotFoundError',
      storageFoundation.rename('does_not_exist', 'new_does_not_exist'));
}, 'Renaming a non-existing file fails with a NotFoundError.');
