// META: title=Synchronous NativeIO API: File renaming is reflected in listing.
// META: global=dedicatedworker

'use strict';

test(testCase => {
  const file = storageFoundation.openSync('test_file');
  file.close();

  const fileNamesBeforeRename = storageFoundation.getAllSync();
  assert_in_array('test_file', fileNamesBeforeRename);

  storageFoundation.renameSync('test_file', 'renamed_test_file');
  testCase.add_cleanup(() => {
    file.close();
    storageFoundation.deleteSync('test_file');
    storageFoundation.deleteSync('renamed_test_file');
  });

  const fileNamesAfterRename = storageFoundation.getAllSync();
  assert_equals(fileNamesAfterRename.indexOf('test_file'), -1);
  assert_in_array('renamed_test_file', fileNamesAfterRename);
}, 'storageFoundation.getAllSync returns a file renamed' +
   ' by storageFoundation.renameSync with its new name.');
