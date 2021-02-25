// META: title=NativeIO API: File deletion is reflected in listing.
// META: global=window,worker

'use strict';

promise_test(async testCase => {
  const file = await storageFoundation.open('test_file');
  testCase.add_cleanup(async () => {
    await storageFoundation.delete('test_file');
  });
  await file.close();

  const fileNamesBeforeDelete = await storageFoundation.getAll();
  assert_in_array('test_file', fileNamesBeforeDelete);

  await storageFoundation.delete('test_file');
  const fileNames = await storageFoundation.getAll();
  assert_equals(fileNames.indexOf('test_file'), -1);
}, 'storageFoundation.getAll does not return file deleted by' +
     ' storageFoundation.delete');

promise_test(async testCase => {
  await storageFoundation.delete('test_file');
  // Delete a second time if the file existed before the first delete.
  await storageFoundation.delete('test_file');
}, 'storageFoundation.delete does not fail when deleting a non-existing file');
