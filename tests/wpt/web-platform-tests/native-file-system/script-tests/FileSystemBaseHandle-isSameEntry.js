'use strict';

directory_test(async (t, root_dir) => {
  assert_true(await root_dir.isSameEntry(root_dir));

  const subdir = await createDirectory(t, 'subdir-name', root_dir);
  assert_true(await subdir.isSameEntry(subdir));
}, 'isSameEntry for identical directory handles returns true');

directory_test(async (t, root_dir) => {
  const subdir = await createDirectory(t, 'subdir-name', root_dir);

  assert_false(await root_dir.isSameEntry(subdir));
  assert_false(await subdir.isSameEntry(root_dir));
}, 'isSameEntry for different directories returns false');

directory_test(async (t, root_dir) => {
  const subdir = await createDirectory(t, 'subdir-name', root_dir);
  const subdir2 = await root_dir.getDirectory('subdir-name');

  assert_true(await subdir.isSameEntry(subdir2));
  assert_true(await subdir2.isSameEntry(subdir));
}, 'isSameEntry for different handles for the same directory');

directory_test(async (t, root_dir) => {
  const handle = await createEmptyFile(t, 'mtime.txt', root_dir);

  assert_true(await handle.isSameEntry(handle));
}, 'isSameEntry for identical file handles returns true');

directory_test(async (t, root_dir) => {
  const handle1 = await createEmptyFile(t, 'mtime.txt', root_dir);
  const handle2 = await createEmptyFile(t, 'foo.txt', root_dir);

  assert_false(await handle1.isSameEntry(handle2));
  assert_false(await handle2.isSameEntry(handle1));
}, 'isSameEntry for different files returns false');

directory_test(async (t, root_dir) => {
  const handle1 = await createEmptyFile(t, 'mtime.txt', root_dir);
  const handle2 = await root_dir.getFile('mtime.txt');

  assert_true(await handle1.isSameEntry(handle2));
  assert_true(await handle2.isSameEntry(handle1));
}, 'isSameEntry for different handles for the same file');

directory_test(async (t, root_dir) => {
  const handle1 = await createEmptyFile(t, 'mtime.txt', root_dir);
  const subdir = await createDirectory(t, 'subdir-name', root_dir);
  const handle2 = await createEmptyFile(t, 'mtime.txt', subdir);

  assert_false(await handle1.isSameEntry(handle2));
  assert_false(await handle2.isSameEntry(handle1));
}, 'isSameEntry comparing a file to a file in a different directory returns false');

directory_test(async (t, root_dir) => {
  const handle1 = await createEmptyFile(t, 'mtime.txt', root_dir);
  const handle2 = await createDirectory(t, 'subdir-name', root_dir);

  assert_false(await handle1.isSameEntry(handle2));
  assert_false(await handle2.isSameEntry(handle1));
}, 'isSameEntry comparing a file to a directory returns false');
