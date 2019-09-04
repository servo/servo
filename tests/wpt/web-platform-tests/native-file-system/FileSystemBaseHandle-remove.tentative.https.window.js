// META: script=resources/test-helpers.js
promise_test(async t => cleanupSandboxedFileSystem(),
    'Cleanup to setup test environment');

promise_test(async t => {
    const root = await FileSystemDirectoryHandle.getSystemDirectory({ type: 'sandbox' });
    const handle = await createFileWithContents(t, 'file-to-remove', '12345', root);
    await createFileWithContents(t, 'file-to-keep', 'abc', root);
    await root.removeEntry('file-to-remove');

    assert_array_equals(await getSortedDirectoryEntries(root), ['file-to-keep']);
    await promise_rejects(t, 'NotFoundError', getFileContents(handle));
}, 'removeEntry() to remove a file');

promise_test(async t => {
    const root = await FileSystemDirectoryHandle.getSystemDirectory({ type: 'sandbox' });
    const handle = await createFileWithContents(t, 'file-to-remove', '12345', root);
    await root.removeEntry('file-to-remove');

    await promise_rejects(t, 'NotFoundError', root.removeEntry('file-to-remove'));
}, 'removeEntry() on an already removed file should fail');

promise_test(async t => {
    const root = await FileSystemDirectoryHandle.getSystemDirectory({ type: 'sandbox' });
    const dir = await root.getDirectory('dir-to-remove', { create: true });
    await createFileWithContents(t, 'file-to-keep', 'abc', root);
    await root.removeEntry('dir-to-remove');

    assert_array_equals(await getSortedDirectoryEntries(root), ['file-to-keep']);
    await promise_rejects(t, 'NotFoundError', getSortedDirectoryEntries(dir));
}, 'removeEntry() to remove an empty directory');

promise_test(async t => {
    const root = await FileSystemDirectoryHandle.getSystemDirectory({ type: 'sandbox' });
    const dir = await root.getDirectory('dir-to-remove', { create: true });
    t.add_cleanup(() => root.removeEntry('dir-to-remove', { recursive: true }));
    await createEmptyFile(t, 'file-in-dir', dir);

    await promise_rejects(t, 'InvalidModificationError', root.removeEntry('dir-to-remove'));
    assert_array_equals(await getSortedDirectoryEntries(root), ['dir-to-remove/']);
    assert_array_equals(await getSortedDirectoryEntries(dir), ['file-in-dir']);
}, 'removeEntry() on a non-empty directory should fail');

promise_test(async t => {
    const root = await FileSystemDirectoryHandle.getSystemDirectory({ type: 'sandbox' });
    const dir = await createDirectory(t, 'dir', root);
    await promise_rejects(t, new TypeError(), dir.removeEntry(""));
}, 'removeEntry() with empty name should fail');

promise_test(async t => {
    const root = await FileSystemDirectoryHandle.getSystemDirectory({ type: 'sandbox' });
    const dir = await createDirectory(t, 'dir', root);
    await promise_rejects(t, new TypeError(), dir.removeEntry(kCurrentDirectory));
}, `removeEntry() with "${kCurrentDirectory}" name should fail`);

promise_test(async t => {
    const root = await FileSystemDirectoryHandle.getSystemDirectory({ type: 'sandbox' });
    const dir = await createDirectory(t, 'dir', root);
    await promise_rejects(t, new TypeError(), dir.removeEntry(kParentDirectory));
}, `removeEntry() with "${kParentDirectory}" name should fail`);

promise_test(async t => {
    const root = await FileSystemDirectoryHandle.getSystemDirectory({ type: 'sandbox' });

    const dir_name = 'dir-name';
    const dir = await createDirectory(t, dir_name, root);

    const file_name = 'file-name';
    await createEmptyFile(t, file_name, dir);

    for (let i = 0; i < kPathSeparators.length; ++i) {
        const path_with_separator = `${dir_name}${kPathSeparators[i]}${file_name}`;
        await promise_rejects(t, new TypeError(), root.removeEntry(path_with_separator),
            `removeEntry() must reject names containing "${kPathSeparators[i]}"`);
    }
}, 'removeEntry() with a path separator should fail.');
