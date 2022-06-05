directory_test(async (t, dir) => {
  await promise_rejects_dom(
      t, 'NotFoundError', dir.getFileHandle('non-existing-file'));
}, 'getFileHandle(create=false) rejects for non-existing files');

directory_test(async (t, dir) => {
  const handle = await dir.getFileHandle('non-existing-file', {create: true});
  t.add_cleanup(() => dir.removeEntry('non-existing-file'));

  assert_equals(handle.kind, 'file');
  assert_equals(handle.name, 'non-existing-file');
  assert_equals(await getFileSize(handle), 0);
  assert_equals(await getFileContents(handle), '');
}, 'getFileHandle(create=true) creates an empty file for non-existing files');

directory_test(async (t, dir) => {
  const existing_handle = await createFileWithContents(
      t, 'existing-file', '1234567890', /*parent=*/ dir);
  const handle = await dir.getFileHandle('existing-file');

  assert_equals(handle.kind, 'file');
  assert_equals(handle.name, 'existing-file');
  assert_equals(await getFileSize(handle), 10);
  assert_equals(await getFileContents(handle), '1234567890');
}, 'getFileHandle(create=false) returns existing files');

directory_test(async (t, dir) => {
  const existing_handle = await createFileWithContents(
      t, 'file-with-contents', '1234567890', /*parent=*/ dir);
  const handle = await dir.getFileHandle('file-with-contents', {create: true});

  assert_equals(handle.kind, 'file');
  assert_equals(handle.name, 'file-with-contents');
  assert_equals(await getFileSize(handle), 10);
  assert_equals(await getFileContents(handle), '1234567890');
}, 'getFileHandle(create=true) returns existing files without erasing');

directory_test(async (t, dir) => {
  const dir_handle = await dir.getDirectoryHandle('dir-name', {create: true});
  t.add_cleanup(() => dir.removeEntry('dir-name', {recursive: true}));

  await promise_rejects_dom(
      t, 'TypeMismatchError', dir.getFileHandle('dir-name'));
}, 'getFileHandle(create=false) when a directory already exists with the same name');

directory_test(async (t, dir) => {
  const dir_handle = await dir.getDirectoryHandle('dir-name', {create: true});
  t.add_cleanup(() => dir.removeEntry('dir-name', {recursive: true}));

  await promise_rejects_dom(
      t, 'TypeMismatchError', dir.getFileHandle('dir-name', {create: true}));
}, 'getFileHandle(create=true) when a directory already exists with the same name');

directory_test(async (t, dir) => {
  await promise_rejects_js(t, TypeError, dir.getFileHandle('', {create: true}));
  await promise_rejects_js(
      t, TypeError, dir.getFileHandle('', {create: false}));
}, 'getFileHandle() with empty name');

directory_test(async (t, dir) => {
  await promise_rejects_js(t, TypeError, dir.getFileHandle(kCurrentDirectory));
  await promise_rejects_js(
      t, TypeError, dir.getFileHandle(kCurrentDirectory, {create: true}));
}, `getFileHandle() with "${kCurrentDirectory}" name`);

directory_test(async (t, dir) => {
  const subdir = await createDirectory(t, 'subdir-name', /*parent=*/ dir);

  await promise_rejects_js(
      t, TypeError, subdir.getFileHandle(kParentDirectory));
  await promise_rejects_js(
      t, TypeError, subdir.getFileHandle(kParentDirectory, {create: true}));
}, `getFileHandle() with "${kParentDirectory}" name`);

directory_test(async (t, dir) => {
  const subdir_name = 'subdir-name';
  const subdir = await createDirectory(t, subdir_name, /*parent=*/ dir);

  const file_name = 'file-name';
  await createEmptyFile(t, file_name, /*parent=*/ subdir);

  for (let i = 0; i < kPathSeparators.length; ++i) {
    const path_with_separator =
        `${subdir_name}${kPathSeparators[i]}${file_name}`;
    await promise_rejects_js(
        t, TypeError, dir.getFileHandle(path_with_separator),
        `getFileHandle() must reject names containing "${kPathSeparators[i]}"`);
  }
}, 'getFileHandle(create=false) with a path separator when the file exists.');

directory_test(async (t, dir) => {
  const subdir_name = 'subdir-name';
  const subdir = await createDirectory(t, subdir_name, /*parent=*/ dir);

  for (let i = 0; i < kPathSeparators.length; ++i) {
    const path_with_separator = `${subdir_name}${kPathSeparators[i]}file_name`;
    await promise_rejects_js(
        t, TypeError, dir.getFileHandle(path_with_separator, {create: true}),
        `getFileHandle(create=true) must reject names containing "${
            kPathSeparators[i]}"`);
  }
}, 'getFileHandle(create=true) with a path separator');
