// META: title=Synchronous NativeIO API: File creation is reflected in listing.
// META: global=dedicatedworker

'use strict';

test(testCase => {
  const file = storageFoundation.openSync('test_file');
  testCase.add_cleanup(() => {
    storageFoundation.deleteSync('test_file');
  });
  file.close();

  const fileNames = storageFoundation.getAllSync();
  assert_in_array('test_file', fileNames);
}, 'storageFoundation.getAllSync returns file created by' +
     ' storageFoundation.openSync');
