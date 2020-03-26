// META: title=NativeIO API: File creation is reflected in listing.
// META: global=window,worker

'use strict';

promise_test(async testCase => {
  const file = await nativeIO.open('test_file');
  testCase.add_cleanup(async () => {
    await nativeIO.delete('test_file');
  });
  await file.close();

  const fileNames = await nativeIO.getAll();
  assert_in_array('test_file', fileNames);
}, 'nativeIO.getAll returns file created by nativeIO.open');
