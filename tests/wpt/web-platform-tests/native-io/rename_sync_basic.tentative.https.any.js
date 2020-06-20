// META: title=Synchronous NativeIO API: File renaming is reflected in listing.
// META: global=dedicatedworker

'use strict';

test(testCase => {
  const file = nativeIO.openSync('test_file');
  file.close();

  const fileNamesBeforeRename = nativeIO.getAllSync();
  assert_in_array('test_file', fileNamesBeforeRename);

  nativeIO.renameSync('test_file', 'renamed_test_file');
  testCase.add_cleanup(() => {
    file.close();
    nativeIO.deleteSync('test_file');
    nativeIO.deleteSync('renamed_test_file');
  });

  const fileNamesAfterRename = nativeIO.getAllSync();
  assert_equals(fileNamesAfterRename.indexOf('test_file'), -1);
  assert_in_array('renamed_test_file', fileNamesAfterRename);
}, 'nativeIO.getAllSync returns a file renamed' +
   ' by nativeIOFile.renameSync with its new name.');
