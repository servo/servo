// META: title=NativeIO API: File renaming is reflected in listing.
// META: global=window,worker

'use strict';

promise_test(async testCase => {
  const file = await storageFoundation.open('test_file');
  await file.close();

  const fileNamesBeforeRename = await storageFoundation.getAll();
  assert_in_array('test_file', fileNamesBeforeRename);

  await storageFoundation.rename('test_file', 'renamed_test_file');
  testCase.add_cleanup(async () => {
    await storageFoundation.delete('test_file');
    await storageFoundation.delete('renamed_test_file');
  });

  const fileNamesAfterRename = await storageFoundation.getAll();
  assert_false(fileNamesAfterRename.includes('test_file'));
  assert_in_array('renamed_test_file', fileNamesAfterRename);
}, 'storageFoundation.getAll returns a file renamed by' +
     ' storageFoundation.rename with its new name.');
