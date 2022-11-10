'use strict';

directory_test(async (t, root_dir) => {
  assert_equals(await root_dir.getUniqueId(), await root_dir.getUniqueId());

  const subdir = await createDirectory(t, 'subdir-name', root_dir);
  assert_equals(await subdir.getUniqueId(), await subdir.getUniqueId());
}, 'identical directory handles return the same ID');

directory_test(async (t, root_dir) => {
  const subdir = await createDirectory(t, 'subdir-name', root_dir);

  assert_not_equals(await root_dir.getUniqueId(), await subdir.getUniqueId());
}, 'different directories return different IDs');

directory_test(async (t, root_dir) => {
  const subdir = await createDirectory(t, 'subdir-name', root_dir);
  const subdir2 = await root_dir.getDirectoryHandle('subdir-name');

  assert_equals(await subdir.getUniqueId(), await subdir2.getUniqueId());
}, 'different handles for the same directory return the same ID');

directory_test(async (t, root_dir) => {
  const handle = await createEmptyFile(t, 'foo.txt', root_dir);

  assert_equals(await handle.getUniqueId(), await handle.getUniqueId());
}, 'identical file handles return the same unique ID');

directory_test(async (t, root_dir) => {
  const handle1 = await createEmptyFile(t, 'foo.txt', root_dir);
  const handle2 = await createEmptyFile(t, 'bar.txt', root_dir);

  assert_not_equals(await handle1.getUniqueId(), await handle2.getUniqueId());
}, 'different files return different IDs');

directory_test(async (t, root_dir) => {
  const handle1 = await createEmptyFile(t, 'foo.txt', root_dir);
  const handle2 = await root_dir.getFileHandle('foo.txt');

  assert_equals(await handle1.getUniqueId(), await handle2.getUniqueId());
}, 'different handles for the same file return the same ID');

directory_test(async (t, root_dir) => {
  const handle1 = await createEmptyFile(t, 'foo.txt', root_dir);
  const subdir = await createDirectory(t, 'subdir-name', root_dir);
  const handle2 = await createEmptyFile(t, 'foo.txt', subdir);

  assert_not_equals(await handle1.getUniqueId(), await handle2.getUniqueId());
}, 'two files of the same name in different directories return different IDs');

directory_test(async (t, root_dir) => {
  const handle1 = await createEmptyFile(t, 'foo.txt', root_dir);
  const handle2 = await createDirectory(t, 'subdir-name', root_dir);

  assert_not_equals(await handle1.getUniqueId(), await handle2.getUniqueId());
}, 'a file and a directory return different IDs');

directory_test(async (t, root_dir) => {
  const file_handle = await createEmptyFile(t, 'foo', root_dir);
  const file_id = await file_handle.getUniqueId();

  // Remove the file.
  await root_dir.removeEntry('foo');

  // Create a directory of the same name and path.
  const dir_handle = await createDirectory(t, 'foo', root_dir);
  assert_not_equals(await dir_handle.getUniqueId(), file_id);
}, 'a file and a directory of the same path return different IDs');

directory_test(async (t, root_dir) => {
  const handle = await createEmptyFile(t, 'foo.txt', root_dir);
  const id_before = await handle.getUniqueId();

  // Write to the file. The unique ID should not change.
  const writable = await handle.createWritable();
  await writable.write("blah");
  await writable.close();

  assert_equals(await handle.getUniqueId(), id_before);
}, 'unique ID of a file handle does not change after writes');

directory_test(async (t, root_dir) => {
  const subdir = await createDirectory(t, 'subdir-name', root_dir);

  const UUIDRegex =
      /^[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}$/
  assert_true(UUIDRegex.test(await root_dir.getUniqueId()));
  assert_true(UUIDRegex.test(await subdir.getUniqueId()));
}, 'unique ID is in GUID version 4 format');
