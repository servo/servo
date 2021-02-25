// META: title=NativeIO API: File creation is reflected in listing.
// META: global=window,worker

'use strict';

promise_test(async testCase => {
  const file = await storageFoundation.open('test_file');
  testCase.add_cleanup(async () => {
    await storageFoundation.delete('test_file');
  });
  await file.close();

  const fileNames = await storageFoundation.getAll();
  assert_in_array('test_file', fileNames);
}, 'storageFoundation.getAll returns file created by storageFoundation.open');
