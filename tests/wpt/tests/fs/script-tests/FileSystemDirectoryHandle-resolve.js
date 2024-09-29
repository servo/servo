'use strict';

directory_test(async (t, root_dir) => {
  assert_array_equals(await root_dir.resolve(root_dir), []);
}, 'Resolve returns empty array for same directory');

directory_test(async (t, root_dir) => {
  const subdir = await createDirectory('subdir-name', root_dir);
  const file = await createEmptyFile('file-name', subdir);

  assert_array_equals(await root_dir.resolve(file), ['subdir-name', 'file-name']);
}, 'Resolve returns correct path');

directory_test(async (t, root_dir) => {
  const subdir = await createDirectory('subdir😊', root_dir);
  const file = await createEmptyFile('file-name', subdir);

  assert_array_equals(await root_dir.resolve(file), ['subdir😊', 'file-name']);
  assert_array_equals(await root_dir.resolve(subdir), ['subdir😊']);
}, 'Resolve returns correct path with non-ascii characters');

directory_test(async (t, root_dir) => {
  const subdir = await createDirectory('subdir-name', root_dir);
  const file = await createEmptyFile('file-name', root_dir);

  assert_equals(await subdir.resolve(file), null);
}, 'Resolve returns null when entry is not a child');
