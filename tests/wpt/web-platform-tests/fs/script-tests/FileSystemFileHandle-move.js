// META: script=resources/test-helpers.js

'use strict';

directory_test(async (t, root) => {
  const handle = await createFileWithContents(t, 'file-before', 'foo', root);
  await handle.move('file-after');

  assert_array_equals(await getSortedDirectoryEntries(root), ['file-after']);
  assert_equals(await getFileContents(handle), 'foo');
  assert_equals(await getFileSize(handle), 3);
}, 'move(name) to rename a file');

directory_test(async (t, root) => {
  const handle = await createFileWithContents(t, 'file-before', 'foo', root);
  await handle.move('file-after');
  const newhandle = await root.getFileHandle('file-after');
  assert_equals(await getFileContents(newhandle), 'foo');
  assert_equals(await getFileSize(newhandle), 3);
}, 'get a handle to a moved file');

directory_test(async (t, root) => {
  const handle = await createFileWithContents(t, 'file-before', 'foo', root);
  await handle.move('file-before');

  assert_array_equals(await getSortedDirectoryEntries(root), ['file-before']);
  assert_equals(await getFileContents(handle), 'foo');
  assert_equals(await getFileSize(handle), 3);
}, 'move(name) to rename a file the same name');

directory_test(async (t, root) => {
  const handle = await createFileWithContents(t, 'file-before', 'foo', root);
  await promise_rejects_js(t, TypeError, handle.move(''));

  assert_array_equals(await getSortedDirectoryEntries(root), ['file-before']);
  assert_equals(await getFileContents(handle), 'foo');
  assert_equals(await getFileSize(handle), 3);
}, 'move("") to rename a file fails');

directory_test(async (t, root) => {
  const handle = await createFileWithContents(t, 'file-1', 'foo', root);

  await handle.move('file-2');
  assert_array_equals(await getSortedDirectoryEntries(root), ['file-2']);

  await handle.move('file-3');
  assert_array_equals(await getSortedDirectoryEntries(root), ['file-3']);

  await handle.move('file-1');
  assert_array_equals(await getSortedDirectoryEntries(root), ['file-1']);
}, 'move(name) can be called multiple times');

directory_test(async (t, root) => {
  const dir = await root.getDirectoryHandle('dir', {create: true});
  const handle = await createFileWithContents(t, 'file-before', 'foo', dir);
  await promise_rejects_js(t, TypeError, handle.move('Lorem.'));

  assert_array_equals(await getSortedDirectoryEntries(root), ['dir/']);
  assert_array_equals(await getSortedDirectoryEntries(dir), ['file-before']);
  assert_equals(await getFileContents(handle), 'foo');
  assert_equals(await getFileSize(handle), 3);
}, 'move(name) with a name with a trailing period should fail');

directory_test(async (t, root) => {
  const handle = await createFileWithContents(t, 'file-before', 'foo', root);
  await promise_rejects_js(t, TypeError, handle.move('test/test'));

  assert_array_equals(await getSortedDirectoryEntries(root), ['file-before']);
  assert_equals(await getFileContents(handle), 'foo');
  assert_equals(await getFileSize(handle), 3);
}, 'move(name) with a name with invalid characters should fail');

directory_test(async (t, root) => {
  const handle = await createFileWithContents(t, 'file-before', 'abc', root);

  // Cannot rename handle with an active writable.
  const stream = await cleanup_writable(t, await handle.createWritable());
  await promise_rejects_dom(
      t, 'NoModificationAllowedError', handle.move('file-after'));

  // Can move handle once the writable is closed.
  await stream.close();
  await handle.move('file-after');
  assert_array_equals(await getSortedDirectoryEntries(root), ['file-after']);
}, 'move(name) while the file has an open writable fails');

directory_test(async (t, root) => {
  const handle = await createFileWithContents(t, 'file-before', 'abc', root);
  const handle_dest =
      await createFileWithContents(t, 'file-after', '123', root);

  // Cannot overwrite a handle with an active writable.
  const stream = await cleanup_writable(t, await handle_dest.createWritable());
  await promise_rejects_dom(
      t, 'NoModificationAllowedError', handle.move('file-after'));

  await stream.close();
  assert_array_equals(
      await getSortedDirectoryEntries(root), ['file-after', 'file-before']);
}, 'move(name) while the destination file has an open writable fails');

directory_test(async (t, root) => {
  const handle = await createFileWithContents(t, 'file-before', 'abc', root);
  const handle_dest =
      await createFileWithContents(t, 'file-after', '123', root);

  await handle.move('file-after');
  assert_array_equals(await getSortedDirectoryEntries(root), ['file-after']);
  assert_equals(await getFileContents(handle), 'abc');
  assert_equals(await getFileContents(handle_dest), 'abc');
}, 'move(name) can overwrite an existing file');

directory_test(async (t, root) => {
  const handle = await createFileWithContents(t, 'file-before', 'foo', root);
  await handle.move(root, 'file-after');

  assert_array_equals(await getSortedDirectoryEntries(root), ['file-after']);
  assert_equals(await getFileContents(handle), 'foo');
  assert_equals(await getFileSize(handle), 3);
}, 'move(dir, name) to rename a file');

directory_test(async (t, root) => {
  const handle = await createFileWithContents(t, 'file-before', 'foo', root);
  await handle.move(root, 'file-before');

  assert_array_equals(await getSortedDirectoryEntries(root), ['file-before']);
  assert_equals(await getFileContents(handle), 'foo');
  assert_equals(await getFileSize(handle), 3);
}, 'move(dir, name) to rename a file the same name');

directory_test(async (t, root) => {
  const dir_src = await root.getDirectoryHandle('dir-src', {create: true});
  const dir_dest = await root.getDirectoryHandle('dir-dest', {create: true});
  const file = await createFileWithContents(t, 'file', 'abc', dir_src);
  await file.move(dir_dest);

  assert_array_equals(
      await getSortedDirectoryEntries(root), ['dir-dest/', 'dir-src/']);
  assert_array_equals(await getSortedDirectoryEntries(dir_src), []);
  assert_array_equals(await getSortedDirectoryEntries(dir_dest), ['file']);
  assert_equals(await getFileContents(file), 'abc');
  assert_equals(await getFileSize(file), 3);
}, 'move(dir) to move a file to a new directory');

directory_test(async (t, root) => {
  const dir_src = await root.getDirectoryHandle('dir-src', {create: true});
  const dir_dest = await root.getDirectoryHandle('dir-dest', {create: true});
  const file = await createFileWithContents(t, 'file', 'abc', dir_src);
  await promise_rejects_js(t, TypeError, file.move(dir_dest, ''));

  assert_array_equals(
      await getSortedDirectoryEntries(root), ['dir-dest/', 'dir-src/']);
  assert_array_equals(await getSortedDirectoryEntries(dir_src), ['file']);
  assert_array_equals(await getSortedDirectoryEntries(dir_dest), []);
  assert_equals(await getFileContents(file), 'abc');
  assert_equals(await getFileSize(file), 3);
}, 'move(dir, "") to move a file to a new directory fails');

directory_test(async (t, root) => {
  const dir_src = await root.getDirectoryHandle('dir-src', {create: true});
  const dir_dest = await root.getDirectoryHandle('dir-dest', {create: true});
  const file =
      await createFileWithContents(t, 'file-in-dir-src', 'abc', dir_src);
  await file.move(dir_dest, 'file-in-dir-dest');

  assert_array_equals(
      await getSortedDirectoryEntries(root), ['dir-dest/', 'dir-src/']);
  assert_array_equals(await getSortedDirectoryEntries(dir_src), []);
  assert_array_equals(
      await getSortedDirectoryEntries(dir_dest), ['file-in-dir-dest']);
  assert_equals(await getFileContents(file), 'abc');
  assert_equals(await getFileSize(file), 3);
}, 'move(dir, name) to move a file to a new directory');

directory_test(async (t, root) => {
  const dir1 = await root.getDirectoryHandle('dir1', {create: true});
  const dir2 = await root.getDirectoryHandle('dir2', {create: true});
  const handle = await createFileWithContents(t, 'file', 'foo', root);

  await handle.move(dir1);
  assert_array_equals(
      await getSortedDirectoryEntries(root), ['dir1/', 'dir2/']);
  assert_array_equals(await getSortedDirectoryEntries(dir1), ['file']);
  assert_array_equals(await getSortedDirectoryEntries(dir2), []);
  assert_equals(await getFileContents(handle), 'foo');

  await handle.move(dir2);
  assert_array_equals(
      await getSortedDirectoryEntries(root), ['dir1/', 'dir2/']);
  assert_array_equals(await getSortedDirectoryEntries(dir1), []);
  assert_array_equals(await getSortedDirectoryEntries(dir2), ['file']);
  assert_equals(await getFileContents(handle), 'foo');

  await handle.move(root);
  assert_array_equals(
      await getSortedDirectoryEntries(root), ['dir1/', 'dir2/', 'file']);
  assert_array_equals(await getSortedDirectoryEntries(dir1), []);
  assert_array_equals(await getSortedDirectoryEntries(dir2), []);
  assert_equals(await getFileContents(handle), 'foo');
}, 'move(dir) can be called multiple times');

directory_test(async (t, root) => {
  const dir1 = await root.getDirectoryHandle('dir1', {create: true});
  const dir2 = await root.getDirectoryHandle('dir2', {create: true});
  const handle = await createFileWithContents(t, 'file', 'foo', root);

  await handle.move(dir1, 'file-1');
  assert_array_equals(
      await getSortedDirectoryEntries(root), ['dir1/', 'dir2/']);
  assert_array_equals(await getSortedDirectoryEntries(dir1), ['file-1']);
  assert_array_equals(await getSortedDirectoryEntries(dir2), []);
  assert_equals(await getFileContents(handle), 'foo');

  await handle.move(dir2, 'file-2');
  assert_array_equals(
      await getSortedDirectoryEntries(root), ['dir1/', 'dir2/']);
  assert_array_equals(await getSortedDirectoryEntries(dir1), []);
  assert_array_equals(await getSortedDirectoryEntries(dir2), ['file-2']);
  assert_equals(await getFileContents(handle), 'foo');

  await handle.move(root, 'file-3');
  assert_array_equals(
      await getSortedDirectoryEntries(root), ['dir1/', 'dir2/', 'file-3']);
  assert_array_equals(await getSortedDirectoryEntries(dir1), []);
  assert_array_equals(await getSortedDirectoryEntries(dir2), []);
  assert_equals(await getFileContents(handle), 'foo');
}, 'move(dir, name) can be called multiple times');

directory_test(async (t, root) => {
  const handle = await createFileWithContents(t, 'file-before', 'foo', root);
  await promise_rejects_js(t, TypeError, handle.move(root, '..'));

  assert_array_equals(await getSortedDirectoryEntries(root), ['file-before']);
  assert_equals(await getFileContents(handle), 'foo');
  assert_equals(await getFileSize(handle), 3);
}, 'move(dir, name) with a name with invalid characters should fail');

directory_test(async (t, root) => {
  const dir_src = await root.getDirectoryHandle('dir-src', {create: true});
  const dir_dest = await root.getDirectoryHandle('dir-dest', {create: true});
  const file = await createFileWithContents(t, 'file', 'abc', dir_src);

  // Cannot move handle with an active writable.
  const stream = await cleanup_writable(t, await file.createWritable());
  await promise_rejects_dom(t, 'NoModificationAllowedError', file.move(dir_dest));

  assert_array_equals(
      await getSortedDirectoryEntries(root), ['dir-dest/', 'dir-src/']);
  // Assert the file hasn't been moved to the destination directory.
  assert_array_equals(await getSortedDirectoryEntries(dir_dest), []);

  // Can move handle once the writable is closed.
  await stream.close();
  await file.move(dir_dest);
  assert_array_equals(
      await getSortedDirectoryEntries(root), ['dir-dest/', 'dir-src/']);
  assert_array_equals(await getSortedDirectoryEntries(dir_src), []);
  assert_array_equals(await getSortedDirectoryEntries(dir_dest), ['file']);
}, 'move(dir) while the file has an open writable fails');

directory_test(async (t, root) => {
  const dir_src = await root.getDirectoryHandle('dir-src', {create: true});
  const dir_dest = await root.getDirectoryHandle('dir-dest', {create: true});
  const file = await createFileWithContents(t, 'file-before', 'abc', dir_src);

  // Cannot move handle with an active writable.
  const stream = await cleanup_writable(t, await file.createWritable());
  await promise_rejects_dom(t, 'NoModificationAllowedError', file.move(dir_dest));

  assert_array_equals(
      await getSortedDirectoryEntries(root), ['dir-dest/', 'dir-src/']);
  // Assert the file hasn't been moved to the destination directory.
  assert_array_equals(await getSortedDirectoryEntries(dir_dest), []);

  // Can move handle once the writable is closed.
  await stream.close();
  await file.move(dir_dest, 'file-after');
  assert_array_equals(
      await getSortedDirectoryEntries(root), ['dir-dest/', 'dir-src/']);
  assert_array_equals(await getSortedDirectoryEntries(dir_src), []);
  assert_array_equals(
      await getSortedDirectoryEntries(dir_dest), ['file-after']);
}, 'move(dir, name) while the file has an open writable fails');

directory_test(async (t, root) => {
  const dir_src = await root.getDirectoryHandle('dir-src', {create: true});
  const dir_dest = await root.getDirectoryHandle('dir-dest', {create: true});
  const file = await createFileWithContents(t, 'file', 'abc', dir_src);
  const file_dest = await createFileWithContents(t, 'file', '123', dir_dest);

  // Cannot overwrite handle with an active writable.
  const stream = await cleanup_writable(t, await file_dest.createWritable());
  await promise_rejects_dom(t, 'NoModificationAllowedError', file.move(dir_dest));

  assert_array_equals(
      await getSortedDirectoryEntries(root), ['dir-dest/', 'dir-src/']);
  // Assert the file is still in the source directory.
  assert_array_equals(await getSortedDirectoryEntries(dir_src), ['file']);

  await stream.close();
  assert_array_equals(await getSortedDirectoryEntries(dir_dest), ['file']);
}, 'move(dir) while the destination file has an open writable fails');

directory_test(async (t, root) => {
  const dir_src = await root.getDirectoryHandle('dir-src', {create: true});
  const dir_dest = await root.getDirectoryHandle('dir-dest', {create: true});
  const file = await createFileWithContents(t, 'file', 'abc', dir_src);
  const file_dest = await createFileWithContents(t, 'file', '123', dir_dest);

  await file.move(dir_dest);
  assert_array_equals(await getSortedDirectoryEntries(dir_src), []);
  assert_array_equals(await getSortedDirectoryEntries(dir_dest), ['file']);
  assert_equals(await getFileContents(file), 'abc');
  assert_equals(await getFileContents(file_dest), 'abc');
}, 'move(dir) can overwrite an existing file');

directory_test(async (t, root) => {
  const dir_src = await root.getDirectoryHandle('dir-src', {create: true});
  const dir_dest = await root.getDirectoryHandle('dir-dest', {create: true});
  const file = await createFileWithContents(t, 'file-src', 'abc', dir_src);
  const file_dest =
      await createFileWithContents(t, 'file-dest', '123', dir_dest);

  // Cannot overwrite handle with an active writable.
  const stream = await cleanup_writable(t, await file_dest.createWritable());
  await promise_rejects_dom(
      t, 'NoModificationAllowedError', file.move(dir_dest, 'file-dest'));

  assert_array_equals(
      await getSortedDirectoryEntries(root), ['dir-dest/', 'dir-src/']);
  // Assert the file is still in the source directory.
  assert_array_equals(await getSortedDirectoryEntries(dir_src), ['file-src']);

  await stream.close();
  assert_array_equals(await getSortedDirectoryEntries(dir_dest), ['file-dest']);
}, 'move(dir, name) while the destination file has an open writable fails');

directory_test(async (t, root) => {
  const dir_src = await root.getDirectoryHandle('dir-src', {create: true});
  const dir_dest = await root.getDirectoryHandle('dir-dest', {create: true});
  const file = await createFileWithContents(t, 'file-src', 'abc', dir_src);
  const file_dest =
      await createFileWithContents(t, 'file-dest', '123', dir_dest);

  await file.move(dir_dest, 'file-dest');

  // Assert the file has been moved to the destination directory and renamed.
  assert_array_equals(await getSortedDirectoryEntries(dir_src), []);
  assert_array_equals(await getSortedDirectoryEntries(dir_dest), ['file-dest']);
  assert_equals(await getFileContents(file), 'abc');
  assert_equals(await getFileContents(file_dest), 'abc');
}, 'move(dir, name) can overwrite an existing file');
