directory_test(async (t, root) => {
  await promise_rejects_dom(
      t, 'NotFoundError', root.getDirectory('non-existing-dir'));
}, 'getDirectory(create=false) rejects for non-existing directories');

directory_test(async (t, root) => {
  const handle = await root.getDirectory('non-existing-dir', {create: true});
  t.add_cleanup(() => root.removeEntry('non-existing-dir', {recursive: true}));

  assert_false(handle.isFile);
  assert_true(handle.isDirectory);
  assert_equals(handle.name, 'non-existing-dir');
  assert_equals(await getDirectoryEntryCount(handle), 0);
  assert_array_equals(
      await getSortedDirectoryEntries(root), ['non-existing-dir/']);
}, 'getDirectory(create=true) creates an empty directory');

directory_test(async (t, root) => {
  const existing_handle =
      await root.getDirectory('dir-with-contents', {create: true});
  t.add_cleanup(() => root.removeEntry('dir-with-contents', {recursive: true}));
  const file_handle = await createEmptyFile(t, 'test-file', existing_handle);

  const handle = await root.getDirectory('dir-with-contents', {create: false});

  assert_false(handle.isFile);
  assert_true(handle.isDirectory);
  assert_equals(handle.name, 'dir-with-contents');
  assert_array_equals(await getSortedDirectoryEntries(handle), ['test-file']);
}, 'getDirectory(create=false) returns existing directories');

directory_test(async (t, root) => {
  const existing_handle =
      await root.getDirectory('dir-with-contents', {create: true});
  t.add_cleanup(() => root.removeEntry('dir-with-contents', {recursive: true}));
  const file_handle =
      await existing_handle.getFile('test-file', {create: true});

  const handle = await root.getDirectory('dir-with-contents', {create: true});

  assert_false(handle.isFile);
  assert_true(handle.isDirectory);
  assert_equals(handle.name, 'dir-with-contents');
  assert_array_equals(await getSortedDirectoryEntries(handle), ['test-file']);
}, 'getDirectory(create=true) returns existing directories without erasing');

directory_test(async (t, root) => {
  await createEmptyFile(t, 'file-name', root);

  await promise_rejects_dom(t, 'TypeMismatchError', root.getDirectory('file-name'));
  await promise_rejects_dom(
      t, 'TypeMismatchError', root.getDirectory('file-name', {create: false}));
  await promise_rejects_dom(
      t, 'TypeMismatchError', root.getDirectory('file-name', {create: true}));
}, 'getDirectory() when a file already exists with the same name');

directory_test(async (t, dir) => {
  await promise_rejects_js(
      t, TypeError, dir.getDirectory('', {create: true}));
  await promise_rejects_js(
      t, TypeError, dir.getDirectory('', {create: false}));
}, 'getDirectory() with empty name');

directory_test(async (t, dir) => {
  await promise_rejects_js(
      t, TypeError, dir.getDirectory(kCurrentDirectory));
  await promise_rejects_js(
      t, TypeError, dir.getDirectory(kCurrentDirectory, {create: true}));
}, `getDirectory() with "${kCurrentDirectory}" name`);

directory_test(async (t, dir) => {
  const subdir = await createDirectory(t, 'subdir-name', /*parent=*/ dir);

  await promise_rejects_js(
      t, TypeError, subdir.getDirectory(kParentDirectory));
  await promise_rejects_js(
      t, TypeError,
      subdir.getDirectory(kParentDirectory, {create: true}));
}, `getDirectory() with "${kParentDirectory}" name`);

directory_test(async (t, dir) => {
  const first_subdir_name = 'first-subdir-name';
  const first_subdir =
      await createDirectory(t, first_subdir_name, /*parent=*/ dir);

  const second_subdir_name = 'second-subdir-name';
  const second_subdir =
      await createDirectory(t, second_subdir_name, /*parent=*/ first_subdir);

  for (let i = 0; i < kPathSeparators.length; ++i) {
    const path_with_separator =
        `${first_subdir_name}${kPathSeparators[i]}${second_subdir_name}`;
    await promise_rejects_js(
        t, TypeError, dir.getDirectory(path_with_separator),
        `getDirectory() must reject names containing "${kPathSeparators[i]}"`);
  }
}, 'getDirectory(create=false) with a path separator when the directory exists');

directory_test(async (t, dir) => {
  const subdir_name = 'subdir-name';
  const subdir = await createDirectory(t, subdir_name, /*parent=*/ dir);

  for (let i = 0; i < kPathSeparators.length; ++i) {
    const path_with_separator = `${subdir_name}${kPathSeparators[i]}file_name`;
    await promise_rejects_js(
        t, TypeError,
        dir.getDirectory(path_with_separator, {create: true}),
        `getDirectory(true) must reject names containing "${
            kPathSeparators[i]}"`);
  }
}, 'getDirectory(create=true) with a path separator');
