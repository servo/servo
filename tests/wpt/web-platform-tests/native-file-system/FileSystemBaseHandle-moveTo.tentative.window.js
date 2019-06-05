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

promise_test(async t => {
    const dir = await FileSystemDirectoryHandle.getSystemDirectory({ type: 'sandbox' });

    const subdir_name = 'subdir-name';
    const subdir = await createDirectory(t, subdir_name, /*parent=*/dir);

    const file_name = 'file-name';
    const file = await createEmptyFile(t, file_name, /*parent=*/subdir);

    // An empty name indicates that the filename should remain unchanged.
    await file.moveTo(dir, /*name=*/"");
    await dir.getFile(file_name);
}, `moveTo() when target is empty`);

promise_test(async t => {
    const dir = await FileSystemDirectoryHandle.getSystemDirectory({ type: 'sandbox' });

    const subdir_name = 'subdir-name';
    const subdir = await createDirectory(t, subdir_name, /*parent=*/dir);

    const file_name = 'file-name';
    const file = await createEmptyFile(t, file_name, /*parent=*/subdir);

    await promise_rejects(t, 'SecurityError', file.moveTo(dir, /*name=*/kCurrentDirectory));
}, `moveTo() when target is ${kCurrentDirectory}`);

promise_test(async t => {
    const dir = await FileSystemDirectoryHandle.getSystemDirectory({ type: 'sandbox' });

    const first_subdir_name = 'first-subdir-name';
    const first_subdir = await createDirectory(t, first_subdir_name, /*parent=*/dir);

    const second_subdir_name = 'second-subdir-name';
    const second_subdir = await createDirectory(t, second_subdir_name, /*parent=*/first_subdir);

    const file_name = 'file-name';
    const file = await createEmptyFile(t, file_name, /*parent=*/second_subdir);

    await promise_rejects(t, 'SecurityError', file.moveTo(first_subdir, /*name=*/kParentDirectory));
}, `moveTo() when target is ${kParentDirectory}`);

promise_test(async t => {
    const dir = await FileSystemDirectoryHandle.getSystemDirectory({ type: 'sandbox' });

    const file_name = 'file-name';
    const file = await createEmptyFile(t, file_name, /*parent=*/dir);

    const first_subdir_name = 'first-subdir-name';
    const first_subdir = await createDirectory(t, first_subdir_name, /*parent=*/dir);

    const second_subdir_name = 'second-subdir-name';
    const second_subdir = await createDirectory(t, second_subdir_name, /*parent=*/first_subdir);

    for (let i = 0; i < kPathSeparators.length; ++i) {
        const path_with_separator = `${second_subdir_name}${kPathSeparators[i]}${file_name}`;
        await promise_rejects(t, 'SecurityError', file.moveTo(first_subdir, path_with_separator),
            `moveTo() must reject names containing "${kPathSeparators[i]}"`);
    }
}, 'moveTo() when target contains path separator');

// TODO(mek): Tests to move directories.
