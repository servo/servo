// META: title=Synchronous NativeIO API: File deletion is reflected in listing.
// META: global=dedicatedworker

'use strict';

test(testCase => {
  const file = storageFoundation.openSync('test_file');
  testCase.add_cleanup(() => {
    storageFoundation.deleteSync('test_file');
  });
  file.close();

  const fileNamesBeforeDelete = storageFoundation.getAllSync();
  assert_in_array('test_file', fileNamesBeforeDelete);

  storageFoundation.deleteSync('test_file');
  const fileNames = storageFoundation.getAllSync();
  assert_equals(fileNames.indexOf('test_file'), -1);
}, 'storageFoundation.getAllSync does not return file deleted by' +
     ' storageFoundation.deleteSync');

test(testCase => {
  storageFoundation.deleteSync('test_file');
  // Delete a second time if the file existed before the first delete.
  storageFoundation.deleteSync('test_file');
}, 'storageFoundation.deleteSync does not fail when deleting a' +
     ' non-existing file');
