// META: script=resources/test-helpers.js
promise_test(async t => cleanupSandboxedFileSystem(),
  'Cleanup to setup test environment');

promise_test(async t => {
    const dir = await FileSystemDirectoryHandle.getSystemDirectory({ type: 'sandbox' });
    await promise_rejects(t, 'NotFoundError', dir.getFile('non-existing-file'));
}, 'getFile(create=false) rejects for non-existing files');

promise_test(async t => {
    const dir = await FileSystemDirectoryHandle.getSystemDirectory({ type: 'sandbox' });
    const handle = await dir.getFile('non-existing-file', { create: true });
    t.add_cleanup(() => dir.removeEntry('non-existing-file'));

    assert_true(handle.isFile);
    assert_false(handle.isDirectory);
    assert_equals(handle.name, 'non-existing-file');
    assert_equals(await getFileSize(handle), 0);
    assert_equals(await getFileContents(handle), '');
}, 'getFile(create=true) creates an empty file for non-existing files');

promise_test(async t => {
    const existing_handle = await createFileWithContents(t, 'existing-file', '1234567890');

    const dir = await FileSystemDirectoryHandle.getSystemDirectory({ type: 'sandbox' });
    const handle = await dir.getFile('existing-file');

    assert_true(handle.isFile);
    assert_false(handle.isDirectory);
    assert_equals(handle.name, 'existing-file');
    assert_equals(await getFileSize(handle), 10);
    assert_equals(await getFileContents(handle), '1234567890');
}, 'getFile(create=false) returns existing files');

promise_test(async t => {
    const existing_handle = await createFileWithContents(t, 'file-with-contents', '1234567890');

    const dir = await FileSystemDirectoryHandle.getSystemDirectory({ type: 'sandbox' });
    const handle = await dir.getFile('file-with-contents', { create: true });

    assert_true(handle.isFile);
    assert_false(handle.isDirectory);
    assert_equals(handle.name, 'file-with-contents');
    assert_equals(await getFileSize(handle), 10);
    assert_equals(await getFileContents(handle), '1234567890');
}, 'getFile(create=true) returns existing files without erasing');

promise_test(async t => {
    const dir = await FileSystemDirectoryHandle.getSystemDirectory({ type: 'sandbox' });
    const dir_handle = await dir.getDirectory('dir-name', { create: true });
    t.add_cleanup(() => dir.removeEntry('dir-name', { recursive: true }));

    await promise_rejects(t, 'TypeMismatchError', dir.getFile('dir-name'));
}, 'getFile(create=false) when a directory already exists with the same name');

promise_test(async t => {
    const dir = await FileSystemDirectoryHandle.getSystemDirectory({ type: 'sandbox' });
    const dir_handle = await dir.getDirectory('dir-name', { create: true });
    t.add_cleanup(() => dir.removeEntry('dir-name', { recursive: true }));

    await promise_rejects(t, 'TypeMismatchError', dir.getFile('dir-name', { create: true }));
}, 'getFile(create=true) when a directory already exists with the same name');

promise_test(async t => {
    const dir = await FileSystemDirectoryHandle.getSystemDirectory({ type: 'sandbox' });
    await promise_rejects(t, new TypeError(), dir.getFile("", { create: true }));
    await promise_rejects(t, new TypeError(), dir.getFile("", { create: false }));
}, 'getFile() with empty name');

promise_test(async t => {
    const dir = await FileSystemDirectoryHandle.getSystemDirectory({ type: 'sandbox' });
    await promise_rejects(t, new TypeError(), dir.getFile(kCurrentDirectory));
    await promise_rejects(t, new TypeError(), dir.getFile(kCurrentDirectory, { create: true }));
}, `getFile() with "${kCurrentDirectory}" name`);

promise_test(async t => {
    const dir = await FileSystemDirectoryHandle.getSystemDirectory({ type: 'sandbox' });
    const subdir = await createDirectory(t, 'subdir-name', /*parent=*/dir);

    await promise_rejects(t, new TypeError(), subdir.getFile(kParentDirectory));
    await promise_rejects(t, new TypeError(), subdir.getFile(kParentDirectory, { create: true }));
}, `getFile() with "${kParentDirectory}" name`);

promise_test(async t => {
    const dir = await FileSystemDirectoryHandle.getSystemDirectory({ type: 'sandbox' });

    const subdir_name = 'subdir-name';
    const subdir = await createDirectory(t, subdir_name, /*parent=*/dir);

    const file_name = 'file-name';
    await createEmptyFile(t, file_name, /*parent=*/subdir);

    for (let i = 0; i < kPathSeparators.length; ++i) {
        const path_with_separator = `${subdir_name}${kPathSeparators[i]}${file_name}`;
        await promise_rejects(t, new TypeError(), dir.getFile(path_with_separator),
            `getFile() must reject names containing "${kPathSeparators[i]}"`);
    }
}, 'getFile(create=false) with a path separator when the file exists.');

promise_test(async t => {
    const dir = await FileSystemDirectoryHandle.getSystemDirectory({ type: 'sandbox' });

    const subdir_name = 'subdir-name';
    const subdir = await createDirectory(t, subdir_name, /*parent=*/dir);

    for (let i = 0; i < kPathSeparators.length; ++i) {
        const path_with_separator = `${subdir_name}${kPathSeparators[i]}file_name`;
        await promise_rejects(t, new TypeError(), dir.getFile(path_with_separator, { create: true }),
            `getFile(true) must reject names containing "${kPathSeparators[i]}"`);
    }
}, 'getFile(create=true) with a path separator');
