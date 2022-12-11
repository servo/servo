'use strict';

directory_test(async (t, root) => {
  const handle =
      await createFileWithContents(t, 'file-to-remove', '12345', root);
  await createFileWithContents(t, 'file-to-keep', 'abc', root);
  await handle.remove();

  assert_array_equals(await getSortedDirectoryEntries(root), ['file-to-keep']);
  await promise_rejects_dom(t, 'NotFoundError', getFileContents(handle));
}, 'remove() to remove a file');

directory_test(async (t, root) => {
  const handle =
      await createFileWithContents(t, 'file-to-remove', '12345', root);
  await handle.remove();

  await promise_rejects_dom(t, 'NotFoundError', handle.remove());
}, 'remove() on an already removed file should fail');

directory_test(async (t, root) => {
  const dir = await root.getDirectoryHandle('dir-to-remove', {create: true});
  await createFileWithContents(t, 'file-to-keep', 'abc', root);
  await dir.remove();

  assert_array_equals(await getSortedDirectoryEntries(root), ['file-to-keep']);
  await promise_rejects_dom(t, 'NotFoundError', getSortedDirectoryEntries(dir));
}, 'remove() to remove an empty directory');

directory_test(async (t, root) => {
  const dir = await root.getDirectoryHandle('dir-to-remove', {create: true});
  await dir.remove();

  await promise_rejects_dom(t, 'NotFoundError', dir.remove());
}, 'remove() on an already removed directory should fail');

directory_test(async (t, root) => {
  const dir = await root.getDirectoryHandle('dir-to-remove', {create: true});
  t.add_cleanup(() => root.removeEntry('dir-to-remove', {recursive: true}));
  await createEmptyFile(t, 'file-in-dir', dir);

  await promise_rejects_dom(t, 'InvalidModificationError', dir.remove());
  assert_array_equals(
      await getSortedDirectoryEntries(root), ['dir-to-remove/']);
  assert_array_equals(await getSortedDirectoryEntries(dir), ['file-in-dir']);
}, 'remove() on a non-empty directory should fail');

directory_test(async (t, root) => {
  // root
  // ├──file-to-keep
  // ├──dir-to-remove
  //    ├── file0
  //    ├── dir1-in-dir
  //    │   └── file1
  //    └── dir2
  const dir = await root.getDirectoryHandle('dir-to-remove', {create: true});
  await createFileWithContents(t, 'file-to-keep', 'abc', root);
  await createEmptyFile(t, 'file0', dir);
  const dir1_in_dir = await createDirectory(t, 'dir1-in-dir', dir);
  await createEmptyFile(t, 'file1', dir1_in_dir);
  await createDirectory(t, 'dir2-in-dir', dir);

  await dir.remove({recursive: true});
  assert_array_equals(await getSortedDirectoryEntries(root), ['file-to-keep']);
}, 'remove() on a directory recursively should delete all sub-items');

directory_test(async (t, root) => {
  const handle =
      await createFileWithContents(t, 'file-to-remove', '12345', root);
  await createFileWithContents(t, 'file-to-keep', 'abc', root);
  await handle.remove({recursive: true});

  assert_array_equals(await getSortedDirectoryEntries(root), ['file-to-keep']);
  await promise_rejects_dom(t, 'NotFoundError', getFileContents(handle));
}, 'remove() on a file should ignore the recursive option');

directory_test(async (t, root) => {
  const handle =
      await createFileWithContents(t, 'file-to-remove', '12345', root);
  await createFileWithContents(t, 'file-to-keep', 'abc', root);

  const writable = await handle.createWritable();
  await promise_rejects_dom(t, 'NoModificationAllowedError', handle.remove());

  await writable.close();
  assert_array_equals(
      await getSortedDirectoryEntries(root),
      ['file-to-keep', 'file-to-remove']);

  await handle.remove();
  assert_array_equals(await getSortedDirectoryEntries(root), ['file-to-keep']);
  await promise_rejects_dom(t, 'NotFoundError', getFileContents(handle));
}, 'remove() while the file has an open writable fails');

promise_test(async (t) => {
  const root = await navigator.storage.getDirectory();
  await root.getFileHandle('file.txt', {create: true});
  assert_array_equals(await getSortedDirectoryEntries(root), ['file.txt']);

  await root.remove();

  // Creates a fresh sandboxed file system.
  const newRoot = await navigator.storage.getDirectory();
  assert_array_equals(await getSortedDirectoryEntries(newRoot), []);
}, 'can remove the root of a sandbox file system');
