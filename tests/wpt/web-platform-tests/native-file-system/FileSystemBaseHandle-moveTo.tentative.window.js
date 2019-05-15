// META: script=resources/test-helpers.js
promise_test(async t => cleanupSandboxedFileSystem(),
    'Cleanup to setup test environment');

promise_test(async t => {
    const dir = await FileSystemDirectoryHandle.getSystemDirectory({ type: 'sandbox' });
    const old_handle = await createFileWithContents(t, 'old-file', '12345', dir);
    const new_handle = await old_handle.moveTo(dir, 'new-name');
    t.add_cleanup(() => new_handle.remove());

    // Verify new file.
    assert_true(new_handle.isFile);
    assert_false(new_handle.isDirectory);
    assert_equals(new_handle.name, 'new-name');
    assert_equals(await getFileContents(new_handle), '12345');

    // And verify old file is gone.
    await promise_rejects(t, 'NotFoundError', getFileContents(old_handle));

    // Verify directory entries.
    assert_array_equals(await getSortedDirectoryEntries(dir), ['new-name']);
}, 'moveTo() to rename a file');

promise_test(async t => {
    const dir = await FileSystemDirectoryHandle.getSystemDirectory({ type: 'sandbox' });
    const old_handle = await createFileWithContents(t, 'old-file', '12345');
    const target_dir = await dir.getDirectory('dir-name', { create: true });
    t.add_cleanup(() => target_dir.removeRecursively());

    const new_handle = await old_handle.moveTo(target_dir);

    // Verify new file.
    assert_true(new_handle.isFile);
    assert_false(new_handle.isDirectory);
    assert_equals(new_handle.name, 'old-file');
    assert_equals(await getFileContents(new_handle), '12345');

    // And verify old file is gone.
    await promise_rejects(t, 'NotFoundError', getFileContents(old_handle));

    // Verify directory entries.
    assert_array_equals(await getSortedDirectoryEntries(dir), ['dir-name/']);
    assert_array_equals(await getSortedDirectoryEntries(target_dir), ['old-file']);
}, 'moveTo() to move a file into a sub-directory');


promise_test(async t => {
    const dir = await FileSystemDirectoryHandle.getSystemDirectory({ type: 'sandbox' });
    const handle = await createFileWithContents(t, 'old-file', '12345', dir);

    await promise_rejects(t, 'InvalidModificationError', handle.moveTo(dir));
    await promise_rejects(t, 'InvalidModificationError', handle.moveTo(dir, handle.name));

    // Verify file still exists.
    assert_equals(await getFileContents(handle), '12345');
    assert_array_equals(await getSortedDirectoryEntries(dir), ['old-file']);
}, 'moveTo() with existing name and parent should fail');

promise_test(async t => {
    const dir = await FileSystemDirectoryHandle.getSystemDirectory({ type: 'sandbox' });
    const handle = await createFileWithContents(t, 'old-file', '12345', dir);
    const target_handle = await createFileWithContents(t, 'target', 'abc', dir);

    await handle.moveTo(dir, target_handle.name);

    // Verify state of files.
    await promise_rejects(t, 'NotFoundError', getFileContents(handle));
    assert_equals(await getFileContents(target_handle), '12345');
    assert_array_equals(await getSortedDirectoryEntries(dir), ['target']);
}, 'moveTo() when target file already exists should overwrite target');

// TODO(mek): Tests to move directories.
