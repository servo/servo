// META: title=NativeIO API: File renaming is reflected in listing.
// META: global=window,worker

'use strict';

promise_test(async testCase => {
  const file = await nativeIO.open('test_file');
  await file.close();

  const fileNamesBeforeRename = await nativeIO.getAll();
  assert_in_array('test_file', fileNamesBeforeRename);

  await nativeIO.rename('test_file', 'renamed_test_file');
  testCase.add_cleanup(async () => {
    await nativeIO.delete('test_file');
    await nativeIO.delete('renamed_test_file');
  });

  const fileNamesAfterRename = await nativeIO.getAll();
  assert_false(fileNamesAfterRename.includes('test_file'));
  assert_in_array('renamed_test_file', fileNamesAfterRename);
}, 'nativeIO.getAll returns a file renamed by nativeIO.rename' +
     ' with its new name.');
