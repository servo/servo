// META: script=resources/test-helpers.js
promise_test(async t => cleanupSandboxedFileSystem(),
    'Cleanup to setup test environment');

promise_test(async t => {
    const dir = await FileSystemDirectoryHandle.getSystemDirectory({ type: 'sandbox' });
    const handle = await createFileWithContents(t, 'file-to-remove', '12345', dir);
    await createFileWithContents(t, 'file-to-keep', 'abc');
    await handle.remove();

    assert_array_equals(await getSortedDirectoryEntries(dir), ['file-to-keep']);
    await promise_rejects(t, 'NotFoundError', getFileContents(handle));
}, 'remove() to remove a file');

promise_test(async t => {
    const handle = await createFileWithContents(t, 'file-to-remove', '12345');
    await handle.remove();

    await promise_rejects(t, 'NotFoundError', handle.remove());
}, 'remove() on an already removed file should fail');

promise_test(async t => {
    const root = await FileSystemDirectoryHandle.getSystemDirectory({ type: 'sandbox' });
    const dir = await root.getDirectory('dir-to-remove', { create: true });
    await createFileWithContents(t, 'file-to-keep', 'abc');
    await dir.remove();

    assert_array_equals(await getSortedDirectoryEntries(root), ['file-to-keep']);
    await promise_rejects(t, 'NotFoundError', getSortedDirectoryEntries(dir));
}, 'remove() to remove an empty directory');

promise_test(async t => {
    const root = await FileSystemDirectoryHandle.getSystemDirectory({ type: 'sandbox' });
    const dir = await root.getDirectory('dir-to-remove', { create: true });
    t.add_cleanup(() => dir.removeRecursively());
    await createEmptyFile(t, 'file-in-dir', dir);

    await promise_rejects(t, 'InvalidModificationError', dir.remove());
    assert_array_equals(await getSortedDirectoryEntries(root), ['dir-to-remove/']);
    assert_array_equals(await getSortedDirectoryEntries(dir), ['file-in-dir']);
}, 'remove() on a non-empty directory should fail');
