// META: title=Synchronous NativeIO API: File creation is reflected in listing.
// META: global=dedicatedworker

'use strict';

test(testCase => {
  const file = nativeIO.openSync('test_file');
  testCase.add_cleanup(() => {
    nativeIO.deleteSync('test_file');
  });
  file.close();

  const fileNames = nativeIO.getAllSync();
  assert_in_array('test_file', fileNames);
}, 'nativeIO.getAllSync returns file created by nativeIO.openSync');
