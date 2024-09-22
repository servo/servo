// This file defines a directory_test() function that can be used to define
// tests that require a FileSystemDirectoryHandle. The implementation of that
// function in this file will return an empty directory in the sandboxed file
// system.
//
// Another implementation of this function exists in
// file-system-access/local-fs-test-helpers.js, where that version uses the
// local file system instead.

function getFileSystemType() {
  return 'sandboxed';
}

async function cleanupDirectory(dir, ignoreRejections) {
  // Get a snapshot of the entries.
  const entries = await Array.fromAsync(dir.values());

  // Call removeEntry on all of them.
  const remove_entry_promises = entries.map(
      entry =>
          dir.removeEntry(entry.name, {recursive: entry.kind === 'directory'}));

  // Wait for them all to resolve or reject.
  if (ignoreRejections) {
    await Promise.allSettled(remove_entry_promises);
  } else {
    await Promise.all(remove_entry_promises);
  }
}

function directory_test(func, description) {
  promise_test(async t => {
    const dir = await navigator.storage.getDirectory();

    // To be extra resilient against bad tests, cleanup before every test.
    await cleanupDirectory(dir, /*ignoreRejections=*/ false);

    // Cleanup after every test.
    t.add_cleanup(async () => {
      // Ignore any rejections since other cleanup code may have deleted them
      // before we could.
      await cleanupDirectory(dir, /*ignoreRejections=*/ true);
    });


    await func(t, dir);
  }, description);
}
