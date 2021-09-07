// META: title=Synchronous NativeIO API: Failures of rename are properly handled.
// META: global=dedicatedworker
// META: script=resources/support.js

'use strict';

setup(() => {
  // Without this assertion, one test passes even if renameSync is not defined
  assert_implements(
    storageFoundation.renameSync, 'storageFoundation.renameSync is' +
                                    ' not implemented.');
});

test(testCase => {
  reserveAndCleanupCapacitySync(testCase);

  const file1 = storageFoundation.openSync('test_file_1');
  const file2 = storageFoundation.openSync('test_file_2');
  testCase.add_cleanup(() => {
    file1.close();
    file2.close();
  });

  const {buffer: writeBuffer1} = file1.write(new Uint8Array([64, 65, 66, 67]), 0);
  const {buffer: writeBuffer2} = file2.write(new Uint8Array([96, 97, 98, 99]), 0);

  file1.close();
  file2.close();

  assert_throws_dom(
    'NoModificationAllowedError',
    () => storageFoundation.renameSync('test_file_1', 'test_file_2'));

  const fileNamesAfterRename = storageFoundation.getAllSync();
  assert_in_array('test_file_1', fileNamesAfterRename);
  assert_in_array('test_file_2', fileNamesAfterRename);

  // Make sure that a failed rename does not modify file contents.
  const file1_after = storageFoundation.openSync('test_file_1');
  const file2_after = storageFoundation.openSync('test_file_2');

  testCase.add_cleanup(() => {
    file1_after.close();
    file2_after.close();
    storageFoundation.deleteSync('test_file_1');
    storageFoundation.deleteSync('test_file_2');
  });
  const {buffer: readBuffer1} = file1_after.read(new Uint8Array(4), 0);
  assert_array_equals(
    readBuffer1, writeBuffer1,
    'the bytes read should match the bytes written');
  const {buffer: readBuffer2} = file2_after.read(new Uint8Array(4), 0);
  assert_array_equals(
    readBuffer2, writeBuffer2,
    'the bytes read should match the bytes written');
}, 'storageFoundation.renameSync does not overwrite an existing file.');

test(testCase => {
  reserveAndCleanupCapacitySync(testCase);

  const file = storageFoundation.openSync('test_file');
  testCase.add_cleanup(() => {
    file.close();
    storageFoundation.deleteSync('test_file');
  });
  assert_throws_dom(
    'NoModificationAllowedError',
    () => storageFoundation.renameSync('test_file', 'renamed_test_file'));
  file.close();

  const fileNamesAfterRename = storageFoundation.getAllSync();
  assert_equals(fileNamesAfterRename.indexOf('renamed_test_file'), -1);
  assert_in_array('test_file', fileNamesAfterRename);
}, 'storageFoundation.renameSync allows renaming an open file.');

test(testCase => {
  reserveAndCleanupCapacitySync(testCase);

  testCase.add_cleanup(() => {
    file.close();
    storageFoundation.deleteSync('test_file');
    for (let name of storageFoundation.getAllSync()) {
      storageFoundation.deleteSync(name);
    }
  });

  const file = storageFoundation.openSync('test_file');
  file.close();
  for (let name of kBadNativeIoNames) {
    assert_throws_js(TypeError, () => storageFoundation.renameSync('test_file',
                                        name));
    assert_throws_js(TypeError, () => storageFoundation.renameSync(name,
                                        'test_file_2'));
  }
}, 'storageFoundation.renameSync does not allow renaming from or to invalid' +
     ' names.');

test(testCase => {
  reserveAndCleanupCapacitySync(testCase);

  const closed_file = storageFoundation.openSync('closed_file');
  closed_file.close();
  const opened_file = storageFoundation.openSync('opened_file');
  testCase.add_cleanup(() => {
    closed_file.close();
    opened_file.close();
    storageFoundation.deleteSync('closed_file');
    storageFoundation.deleteSync('opened_file');
  });

  // First rename fails, as source is still open.
  assert_throws_dom(
    'NoModificationAllowedError',
    () => storageFoundation.renameSync('opened_file', 'closed_file'));
  // First rename fails again, as source has not been unlocked.
  assert_throws_dom(
    'NoModificationAllowedError',
    () => storageFoundation.renameSync('opened_file', 'closed_file'));
}, 'Failed storageFoundation.renameSync does not unlock the source.');

test(testCase => {
  reserveAndCleanupCapacitySync(testCase);

  const closed_file = storageFoundation.openSync('closed_file');
  closed_file.close();
  const opened_file = storageFoundation.openSync('opened_file');
  testCase.add_cleanup(() => {
    closed_file.close();
    opened_file.close();
    storageFoundation.deleteSync('closed_file');
    storageFoundation.deleteSync('opened_file');
  });

  // First rename fails, as destination is still open.
  assert_throws_dom(
    'NoModificationAllowedError',
    () => storageFoundation.renameSync('closed_file', 'opened_file'));
  // First rename fails again, as destination has not been unlocked.
  assert_throws_dom(
    'NoModificationAllowedError',
    () => storageFoundation.renameSync('closed_file', 'opened_file'));
}, 'Failed storageFoundation.renameSync does not unlock the destination.');

test(testCase => {
  reserveAndCleanupCapacitySync(testCase);

  // Make sure that the file does not exist.
  storageFoundation.deleteSync('does_not_exist');
  testCase.add_cleanup(() => {
    storageFoundation.deleteSync('new_name');
  });
  assert_throws_dom(
    'NotFoundError',
    () => storageFoundation.renameSync('does_not_exist', 'new_name'));
}, 'Renaming a non-existing file fails with a NotFoundError.');
