// META: title=Synchronous NativeIO API: File deletion is reflected in listing.
// META: global=!default,dedicatedworker

'use strict';

test(testCase => {
  const file = nativeIO.openSync('test_file');
  testCase.add_cleanup(() => {
    nativeIO.deleteSync('test_file');
  });
  file.close();

  const fileNamesBeforeDelete = nativeIO.getAllSync();
  assert_in_array('test_file', fileNamesBeforeDelete);

  nativeIO.deleteSync('test_file');
  const fileNames = nativeIO.getAllSync();
  assert_equals(fileNames.indexOf('test_file'), -1);
}, 'nativeIO.getAllSync does not return file deleted by nativeIO.deleteSync');
