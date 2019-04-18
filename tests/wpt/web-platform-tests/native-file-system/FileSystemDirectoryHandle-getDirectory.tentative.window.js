// META: script=resources/test-helpers.js
promise_test(async t => cleanupSandboxedFileSystem(),
        'Cleanup to setup test environment');

promise_test(async t => {
    const root = await FileSystemDirectoryHandle.getSystemDirectory({ type: 'sandbox' });
    await promise_rejects(t, 'NotFoundError', root.getDirectory('non-existing-dir'));
}, 'getDirectory(create=false) rejects for non-existing directories');

promise_test(async t => {
    const root = await FileSystemDirectoryHandle.getSystemDirectory({ type: 'sandbox' });
    const handle = await root.getDirectory('non-existing-dir', { create: true });
    t.add_cleanup(() => handle.removeRecursively());

    assert_false(handle.isFile);
    assert_true(handle.isDirectory);
    assert_equals(handle.name, 'non-existing-dir');
    assert_equals(await getDirectoryEntryCount(handle), 0);
    assert_array_equals(await getSortedDirectoryEntries(root), ['non-existing-dir/']);
}, 'getDirectory(create=true) creates an empty directory');

promise_test(async t => {
    const root = await FileSystemDirectoryHandle.getSystemDirectory({ type: 'sandbox' });
    const existing_handle = await root.getDirectory('dir-with-contents', { create: true });
    t.add_cleanup(() => existing_handle.removeRecursively());
    const file_handle = await createEmptyFile(t, 'test-file', existing_handle);

    const handle = await root.getDirectory('dir-with-contents', { create: false });

    assert_false(handle.isFile);
    assert_true(handle.isDirectory);
    assert_equals(handle.name, 'dir-with-contents');
    assert_array_equals(await getSortedDirectoryEntries(handle), ['test-file']);
}, 'getDirectory(create=false) returns existing directories');

promise_test(async t => {
    const root = await FileSystemDirectoryHandle.getSystemDirectory({ type: 'sandbox' });
    const existing_handle = await root.getDirectory('dir-with-contents', { create: true });
    t.add_cleanup(() => existing_handle.removeRecursively());
    const file_handle = await existing_handle.getFile('test-file', { create: true });

    const handle = await root.getDirectory('dir-with-contents', { create: true });

    assert_false(handle.isFile);
    assert_true(handle.isDirectory);
    assert_equals(handle.name, 'dir-with-contents');
    assert_array_equals(await getSortedDirectoryEntries(handle), ['test-file']);
}, 'getDirectory(create=true) returns existing directories without erasing');

promise_test(async t => {
    const root = await FileSystemDirectoryHandle.getSystemDirectory({ type: 'sandbox' });
    await createEmptyFile(t, 'file-name');

    await promise_rejects(t, 'TypeMismatchError', root.getDirectory('file-name'));
    await promise_rejects(t, 'TypeMismatchError', root.getDirectory('file-name', { create: false }));
    await promise_rejects(t, 'TypeMismatchError', root.getDirectory('file-name', { create: true }));
}, 'getDirectory() when a file already exists with the same name');
