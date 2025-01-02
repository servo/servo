'use strict';

directory_test(async (t, root_dir) => {
  assert_true(await root_dir.isSameEntry(root_dir));

  const subdir = await createDirectory('subdir-name', root_dir);
  assert_true(await subdir.isSameEntry(subdir));
}, 'isSameEntry for identical directory handles returns true');

directory_test(async (t, root_dir) => {
  const subdir = await createDirectory('subdir-name', root_dir);

  assert_false(await root_dir.isSameEntry(subdir));
  assert_false(await subdir.isSameEntry(root_dir));
}, 'isSameEntry for different directories returns false');

directory_test(async (t, root_dir) => {
  const subdir = await createDirectory('subdir-name', root_dir);
  const subdir2 = await root_dir.getDirectoryHandle('subdir-name');

  assert_true(await subdir.isSameEntry(subdir2));
  assert_true(await subdir2.isSameEntry(subdir));
}, 'isSameEntry for different handles for the same directory');

directory_test(async (t, root_dir) => {
  const handle = await createEmptyFile('mtime.txt', root_dir);

  assert_true(await handle.isSameEntry(handle));
}, 'isSameEntry for identical file handles returns true');

directory_test(async (t, root_dir) => {
  const handle1 = await createEmptyFile('mtime.txt', root_dir);
  const handle2 = await createEmptyFile('foo.txt', root_dir);

  assert_false(await handle1.isSameEntry(handle2));
  assert_false(await handle2.isSameEntry(handle1));
}, 'isSameEntry for different files returns false');

directory_test(async (t, root_dir) => {
  const handle1 = await createEmptyFile('mtime.txt', root_dir);
  const handle2 = await root_dir.getFileHandle('mtime.txt');

  assert_true(await handle1.isSameEntry(handle2));
  assert_true(await handle2.isSameEntry(handle1));
}, 'isSameEntry for different handles for the same file');

directory_test(async (t, root_dir) => {
  const handle1 = await createEmptyFile('mtime.txt', root_dir);
  const subdir = await createDirectory('subdir-name', root_dir);
  const handle2 = await createEmptyFile('mtime.txt', subdir);

  assert_false(await handle1.isSameEntry(handle2));
  assert_false(await handle2.isSameEntry(handle1));
}, 'isSameEntry comparing a file to a file in a different directory returns false');

directory_test(async (t, root_dir) => {
  const handle1 = await createEmptyFile('mtime.txt', root_dir);
  const handle2 = await createDirectory('subdir-name', root_dir);

  assert_false(await handle1.isSameEntry(handle2));
  assert_false(await handle2.isSameEntry(handle1));
}, 'isSameEntry comparing a file to a directory returns false');

directory_test(async (t, root_dir) => {
  const filename = 'foo';
  const handle1 = await createEmptyFile(filename, root_dir);
  // Remove the file and create a new file of the same path.
  await root_dir.removeEntry(filename);
  const handle2 = await createEmptyFile(filename, root_dir);

  assert_true(
      await handle1.isSameEntry(handle2),
      'two file handles pointing at the same path should be considered the same entry');
  assert_true(
      await handle2.isSameEntry(handle1),
      'two file handles pointing at the same path should be considered the same entry');
}, 'isSameEntry comparing two files pointing to the same path returns true');

directory_test(async (t, root_dir) => {
  const filename = 'foo';
  const handle1 = await createDirectory(filename, root_dir);
  // Remove the directory and create a new directory of the same path.
  await root_dir.removeEntry(filename);
  const handle2 = await createDirectory(filename, root_dir);

  assert_true(
      await handle1.isSameEntry(handle2),
      'two directory handles pointing at the same path should be considered the same entry');
  assert_true(
      await handle2.isSameEntry(handle1),
      'two directory handles pointing at the same path should be considered the same entry');
}, 'isSameEntry comparing two directories pointing to the same path returns true');

directory_test(async (t, root_dir) => {
  const filename = 'foo';
  const dir_handle = await createDirectory(filename, root_dir);
  // Remove the directory and create a file of the same path.
  await root_dir.removeEntry(filename);
  const file_handle = await createEmptyFile(filename, root_dir);

  assert_false(
      await dir_handle.isSameEntry(file_handle),
      'a file and directory handle pointing at the same path should not be considered the same entry');
  assert_false(
      await file_handle.isSameEntry(dir_handle),
      'a file and directory handle pointing at the same path should not be considered the same entry');
}, 'isSameEntry comparing a file to a directory of the same path returns false');
