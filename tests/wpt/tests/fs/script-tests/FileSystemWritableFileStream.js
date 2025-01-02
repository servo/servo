'use strict';

directory_test(async (t, root) => {
  const handle = await createEmptyFile('trunc_shrink', root);
  const stream = await handle.createWritable();

  await stream.write('1234567890');
  await stream.truncate(5);
  await stream.close();

  assert_equals(await getFileContents(handle), '12345');
  assert_equals(await getFileSize(handle), 5);
}, 'truncate() to shrink a file');

directory_test(async (t, root) => {
  const handle = await createEmptyFile('trunc_grow', root);
  const stream = await handle.createWritable();

  await stream.write('abc');
  await stream.truncate(5);
  await stream.close();

  assert_equals(await getFileContents(handle), 'abc\0\0');
  assert_equals(await getFileSize(handle), 5);
}, 'truncate() to grow a file');

directory_test(async (t, root) => {
  const dir = await createDirectory('parent_dir', root);
  const file_name = 'create_writable_fails_when_dir_removed.txt';
  const handle = await createEmptyFile(file_name, dir);

  await root.removeEntry('parent_dir', {recursive: true});
  await promise_rejects_dom(t, 'NotFoundError', handle.createWritable());
}, 'createWritable() fails when parent directory is removed');

directory_test(async (t, root) => {
  const handle =
      await createFileWithContents('atomic_file_is_copied.txt', 'fooks', root);
  const stream = await handle.createWritable({keepExistingData: true});

  await stream.write('bar');
  await stream.close();
  assert_equals(await getFileContents(handle), 'barks');
  assert_equals(await getFileSize(handle), 5);
}, 'createWritable({keepExistingData: true}): atomic writable file stream initialized with source contents');

directory_test(async (t, root) => {
  const handle = await createFileWithContents(
      'atomic_file_is_not_copied.txt', 'very long string', root);
  const stream = await handle.createWritable({keepExistingData: false});

  await stream.write('bar');
  assert_equals(await getFileContents(handle), 'very long string');
  await stream.close();
  assert_equals(await getFileContents(handle), 'bar');
  assert_equals(await getFileSize(handle), 3);
}, 'createWritable({keepExistingData: false}): atomic writable file stream initialized with empty file');

directory_test(async (t, root) => {
  const handle = await createFileWithContents(
      'trunc_smaller_offset.txt', '1234567890', root);
  const stream = await handle.createWritable({keepExistingData: true});

  await stream.truncate(5);
  await stream.write('abc');
  await stream.close();

  assert_equals(await getFileContents(handle), 'abc45');
  assert_equals(await getFileSize(handle), 5);
}, 'cursor position: truncate size > offset');

directory_test(async (t, root) => {
  const handle = await createFileWithContents(
      'trunc_bigger_offset.txt', '1234567890', root);
  const stream = await handle.createWritable({keepExistingData: true});

  await stream.seek(6);
  await stream.truncate(5);
  await stream.write('abc');
  await stream.close();

  assert_equals(await getFileContents(handle), '12345abc');
  assert_equals(await getFileSize(handle), 8);
}, 'cursor position: truncate size < offset');

directory_test(async (t, root) => {
  const handle = await createEmptyFile('contents', root);
  const stream = await handle.createWritable();
  assert_false(stream.locked);

  stream.write('abc');
  assert_false(stream.locked);
  stream.write('def');
  assert_false(stream.locked);
  stream.truncate(9);
  assert_false(stream.locked);
  stream.seek(0);
  assert_false(stream.locked);
  stream.write('xyz');
  assert_false(stream.locked);
  await stream.close();

  assert_equals(await getFileContents(handle), 'xyzdef\0\0\0');
  assert_equals(await getFileSize(handle), 9);
}, 'commands are queued, stream is unlocked after each operation');
