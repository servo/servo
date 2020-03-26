// META: title=NativeIO API: File deletion is reflected in listing.
// META: global=window,worker

'use strict';

promise_test(async testCase => {
  const file = await nativeIO.open('test_file');
  testCase.add_cleanup(async () => {
    await nativeIO.delete('test_file');
  });
  await file.close();

  const fileNamesBeforeDelete = await nativeIO.getAll();
  assert_in_array('test_file', fileNamesBeforeDelete);

  await nativeIO.delete('test_file');
  const fileNames = await nativeIO.getAll();
  assert_equals(fileNames.indexOf('test_file'), -1);
}, 'nativeIO.getAll does not return file deleted by nativeIO.delete');

