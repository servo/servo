// META: title=Synchronous NativeIO API: Failures of rename are properly handled.
// META: global=dedicatedworker
// META: script=resources/support.js

'use strict';

setup(() => {
  // Without this assertion, one test passes even if renameSync is not defined
  assert_implements(
    nativeIO.renameSync, 'nativeIO.renameSync is not implemented.');
});

test(testCase => {
  const file1 = nativeIO.openSync('test_file_1');
  const file2 = nativeIO.openSync('test_file_2');
  testCase.add_cleanup(() => {
    file1.close();
    file2.close();
  });

  const writtenBytes1 = Uint8Array.from([64, 65, 66, 67]);
  file1.write(writtenBytes1, 0);
  const writtenBytes2 = Uint8Array.from([96, 97, 98, 99]);
  file2.write(writtenBytes2, 0);

  file1.close();
  file2.close();

  assert_throws_dom(
    'NoModificationAllowedError',
    () => nativeIO.renameSync('test_file_1', 'test_file_2'));

  const fileNamesAfterRename = nativeIO.getAllSync();
  assert_in_array('test_file_1', fileNamesAfterRename);
  assert_in_array('test_file_2', fileNamesAfterRename);

  // Make sure that a failed rename does not modify file contents.
  const file1_after = nativeIO.openSync('test_file_1');
  const file2_after = nativeIO.openSync('test_file_2');

  testCase.add_cleanup(() => {
    file1_after.close();
    file2_after.close();
    nativeIO.deleteSync('test_file_1');
    nativeIO.deleteSync('test_file_2');
  });
  const readBytes1 = new Uint8Array(writtenBytes1.length);
  file1_after.read(readBytes1, 0);
  assert_array_equals(
    readBytes1, writtenBytes1,
    'the bytes read should match the bytes written');
  const readBytes2 = new Uint8Array(writtenBytes2.length);
  file2_after.read(readBytes2, 0);
  assert_array_equals(
    readBytes2, writtenBytes2,
    'the bytes read should match the bytes written');
}, 'nativeIO.renameSync does not overwrite an existing file.');

test(testCase => {
  const file = nativeIO.openSync('test_file');
  testCase.add_cleanup(() => {
    file.close();
    nativeIO.deleteSync('test_file');
  });
  assert_throws_dom(
    'NoModificationAllowedError',
    () => nativeIO.renameSync('test_file', 'renamed_test_file'));
  file.close();

  const fileNamesAfterRename = nativeIO.getAllSync();
  assert_equals(fileNamesAfterRename.indexOf('renamed_test_file'), -1);
  assert_in_array('test_file', fileNamesAfterRename);
}, 'nativeIO.renameSync allows renaming an open file.');

test(testCase => {
  testCase.add_cleanup(() => {
    file.close();
    nativeIO.deleteSync('test_file');
    for (let name of nativeIO.getAllSync()) {
      nativeIO.deleteSync(name);
    }
  });

  const file = nativeIO.openSync('test_file');
  file.close();
  for (let name of kBadNativeIoNames) {
    assert_throws_js(TypeError, () => nativeIO.renameSync('test_file', name));
    assert_throws_js(TypeError, () => nativeIO.renameSync(name, 'test_file_2'));
  }
}, 'nativeIO.renameSync does not allow renaming from or to invalid names.');

test(testCase => {
  const closed_file = nativeIO.openSync('closed_file');
  closed_file.close();
  const opened_file = nativeIO.openSync('opened_file');
  testCase.add_cleanup(() => {
    closed_file.close();
    opened_file.close();
    nativeIO.deleteSync('closed_file');
    nativeIO.deleteSync('opened_file');
  });

  // First rename fails, as source is still open.
  assert_throws_dom(
    'NoModificationAllowedError',
    () => nativeIO.renameSync('opened_file', 'closed_file'));
  // First rename fails again, as source has not been unlocked.
  assert_throws_dom(
    'NoModificationAllowedError',
    () => nativeIO.renameSync('opened_file', 'closed_file'));
}, 'Failed nativeIO.renameSync does not unlock the source.');

test(testCase => {
  const closed_file = nativeIO.openSync('closed_file');
  closed_file.close();
  const opened_file = nativeIO.openSync('opened_file');
  testCase.add_cleanup(() => {
    closed_file.close();
    opened_file.close();
    nativeIO.deleteSync('closed_file');
    nativeIO.deleteSync('opened_file');
  });

  // First rename fails, as destination is still open.
  assert_throws_dom(
    'NoModificationAllowedError',
    () => nativeIO.renameSync('closed_file', 'opened_file'));
  // First rename fails again, as destination has not been unlocked.
  assert_throws_dom(
    'NoModificationAllowedError',
    () => nativeIO.renameSync('closed_file', 'opened_file'));
}, 'Failed nativeIO.renameSync does not unlock the destination.');

test(testCase => {
  // Make sure that the file does not exist.
  nativeIO.deleteSync('does_not_exist');
  testCase.add_cleanup(() => {
    nativeIO.deleteSync('new_name');
  });
  assert_throws_dom(
    'NotFoundError',
    () => nativeIO.renameSync('does_not_exist', 'new_name'));
}, 'Renaming a non-existing file fails with a NotFoundError.');
