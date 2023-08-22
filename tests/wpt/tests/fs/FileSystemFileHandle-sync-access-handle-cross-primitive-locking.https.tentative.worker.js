importScripts('/resources/testharness.js');
importScripts('resources/sandboxed-fs-test-helpers.js');
importScripts('resources/test-helpers.js');

'use strict';

// Adds tests that assert file operations which acquire a lock cannot
// run on a file that is locked by the primitive created by running
// and awaiting `openScopedPrimitive` on a file handle.
// `openScopedPrimitive` must close the respective primitive during test cleanup
function crossLockOperationTests(primitiveName, openScopedPrimitive) {
  directory_test(async (t, rootDir) => {
    const fileHandle = await rootDir.getFileHandle('OPFS.test', {create: true});

    await openScopedPrimitive(t, fileHandle);

    await promise_rejects_dom(
        t, 'NoModificationAllowedError', fileHandle.remove());
  }, `A file with an open ${primitiveName} cannot be removed`);

  directory_test(async (t, rootDir) => {
    const fileHandle = await rootDir.getFileHandle('OPFS.test', {create: true});

    await openScopedPrimitive(t, fileHandle);

    await promise_rejects_dom(
        t, 'NoModificationAllowedError', fileHandle.move('OPFS2.test'));
  }, `A file with an open ${primitiveName} cannot be moved`);

  directory_test(async (t, rootDir) => {
    const dirHandle =
        await rootDir.getDirectoryHandle('foo', {create: true});
    const fileHandle =
        await dirHandle.getFileHandle('OPFS.test', {create: true});

    await openScopedPrimitive(t, fileHandle);

    await promise_rejects_dom(
        t, 'NoModificationAllowedError', dirHandle.remove());
  }, `A directory containing a file with an open ${primitiveName} cannot be` +
  ` removed`);
}

// Adds tests for expected behaviors of an access handle created in `sahMode`
// mode across primitives.
function crossLockSAHTests(sahMode) {
  const sahOptions = {mode: sahMode};

  crossLockOperationTests(`access handle in ${sahMode} mode`,
    async (t, fileHandle) => {
      const syncHandle = await fileHandle.createSyncAccessHandle(sahOptions);
      t.add_cleanup(() => syncHandle.close());
    });

  directory_test(async (t, rootDir) => {
    const fileHandle =
        await rootDir.getFileHandle('OPFS.test', {create: true});

    const syncHandle = await fileHandle.createSyncAccessHandle(sahOptions);
    t.add_cleanup(() => syncHandle.close());
    await promise_rejects_dom(
        t, 'NoModificationAllowedError', fileHandle.createWritable());

    syncHandle.close();
    const writable = await fileHandle.createWritable();
    await writable.close();
  }, `When there's an open access handle in ${sahMode} mode on a file, cannot` +
  ` open a writable stream on that same file`);

  directory_test(async (t, rootDir) => {
    const fileHandle =
        await rootDir.getFileHandle('OPFS.test', {create: true});

    const writable1 = await fileHandle.createWritable();
    const writable2 = await fileHandle.createWritable();
    await promise_rejects_dom(
        t, 'NoModificationAllowedError',
        fileHandle.createSyncAccessHandle(sahOptions));

    await writable1.close();
    await promise_rejects_dom(
        t, 'NoModificationAllowedError',
        fileHandle.createSyncAccessHandle(sahOptions));

    await writable2.close();
    const syncHandle = await fileHandle.createSyncAccessHandle(sahOptions);
    syncHandle.close();
  }, `When there's an open writable stream on a file, cannot open a access` +
  `handle in ${sahMode} mode on that same file`);

  directory_test(async (t, rootDir) => {
    const fooFileHandle =
        await rootDir.getFileHandle('foo.test', {create: true});
    const barFileHandle =
        await rootDir.getFileHandle('bar.test', {create: true});

    await cleanup_writable(t, await fooFileHandle.createWritable());

    const barSyncHandle =
        await barFileHandle.createSyncAccessHandle(sahOptions);
    t.add_cleanup(() => barSyncHandle.close());
  }, `A writable stream from one file does not interfere with the creation of` +
  ` an access handle in ${sahMode} mode on another file`);

  directory_test(async (t, rootDir) => {
    const fooFileHandle =
        await rootDir.getFileHandle('foo.test', {create: true});
    const barFileHandle =
        await rootDir.getFileHandle('bar.test', {create: true});

    const fooSyncHandle =
        await fooFileHandle.createSyncAccessHandle(sahOptions);
    t.add_cleanup(() => fooSyncHandle.close());

    await cleanup_writable(t, await barFileHandle.createWritable());
  }, `An open access handle in ${sahMode} mode from one file does not interfere` +
  ` with the creation of a writable stream on another file`);
}

crossLockSAHTests('readwrite');
crossLockSAHTests('read-only');
crossLockSAHTests('readwrite-unsafe');

crossLockOperationTests('writable stream', async (t, fileHandle) => {
  await cleanup_writable(t, await fileHandle.createWritable());
});

done();
