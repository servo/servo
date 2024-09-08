// META: global=window,worker
// META: script=resources/test-helpers.js

promise_test(async t => {
  const fileName = 'testFile';
  t.add_cleanup(async () => {
    try {
      await parent.removeEntry(fileName);
    } catch {
      // Ignore any errors in case the test failed.
    }
  });

  const directory = await navigator.storage.getDirectory();
  return directory.getFileHandle(fileName, {create: true});
}, 'Call getFileHandle successfully');

promise_test(async t => {
  const directoryName = 'testDirectory';
  t.add_cleanup(async () => {
    try {
      await parent.removeEntry(fileName, {recursive: true});
    } catch {
      // Ignore any errors in case the test failed.
    }
  });

  const directory = await navigator.storage.getDirectory();
  return directory.getDirectoryHandle(directoryName, {create: true});
}, 'Call getDirectoryHandle successfully');
