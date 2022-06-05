// This file defines a directory_test() function that can be used to define
// tests that require a FileSystemDirectoryHandle. The implementation of that
// function in this file will ask the user to select an empty directory and uses
// that directory.
//
// Another implementation of this function exists in
// fs/resources/sandboxed-fs-test-helpers.js, where that version uses the
// sandboxed file system instead.

const directory_promise = (async () => {
  await new Promise(resolve => {
    window.addEventListener('DOMContentLoaded', resolve);
  });

  // Small delay to give chrome's test automation a chance to actually install
  // itself.
  await new Promise(resolve => step_timeout(resolve, 100));

  await window.test_driver.bless(
      'show a file picker.<br />Please select an empty directory');
  const entries = await self.showDirectoryPicker();
  assert_true(entries instanceof FileSystemHandle);
  assert_true(entries instanceof FileSystemDirectoryHandle);
  for await (const entry of entries) {
    assert_unreached('Selected directory is not empty');
  }
  return entries;
})();

function directory_test(func, description) {
  promise_test(async t => {
    const directory = await directory_promise;
    // To be resilient against tests not cleaning up properly, cleanup before
    // every test.
    for await (let entry of directory.values()) {
      await directory.removeEntry(
          entry.name, {recursive: entry.kind === 'directory'});
    }
    await func(t, directory);
  }, description);
}

directory_test(async (t, dir) => {
  assert_equals(await dir.queryPermission({mode: 'read'}), 'granted');
}, 'User succesfully selected an empty directory.');

directory_test(async (t, dir) => {
  const status = await dir.queryPermission({mode: 'readwrite'});
  if (status == 'granted')
    return;

  await window.test_driver.bless('ask for write permission');
  assert_equals(await dir.requestPermission({mode: 'readwrite'}), 'granted');
}, 'User granted write access.');
