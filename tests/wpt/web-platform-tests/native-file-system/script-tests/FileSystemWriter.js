directory_test(async (t, root) => {
  const handle = await createEmptyFile(t, 'empty_blob', root);
  const writer = await handle.createWriter();

  await writer.write(0, new Blob([]));
  await writer.close();

  assert_equals(await getFileContents(handle), '');
  assert_equals(await getFileSize(handle), 0);
}, 'write() with an empty blob to an empty file');

directory_test(async (t, root) => {
  const handle = await createEmptyFile(t, 'valid_blob', root);
  const writer = await handle.createWriter();

  await writer.write(0, new Blob(['1234567890']));
  await writer.close();

  assert_equals(await getFileContents(handle), '1234567890');
  assert_equals(await getFileSize(handle), 10);
}, 'write() a blob to an empty file');

directory_test(async (t, root) => {
  const handle = await createEmptyFile(t, 'blob_with_offset', root);
  const writer = await handle.createWriter();

  await writer.write(0, new Blob(['1234567890']));
  await writer.write(4, new Blob(['abc']));
  await writer.close();

  assert_equals(await getFileContents(handle), '1234abc890');
  assert_equals(await getFileSize(handle), 10);
}, 'write() called with a blob and a valid offset');

directory_test(async (t, root) => {
  const handle = await createEmptyFile(t, 'bad_offset', root);
  const writer = await handle.createWriter();

  await promise_rejects(
      t, 'InvalidStateError', writer.write(4, new Blob(['abc'])));
  await writer.close();

  assert_equals(await getFileContents(handle), '');
  assert_equals(await getFileSize(handle), 0);
}, 'write() called with an invalid offset');

directory_test(async (t, root) => {
  const handle = await createEmptyFile(t, 'empty_string', root);
  const writer = await handle.createWriter();

  await writer.write(0, '');
  await writer.close();
  assert_equals(await getFileContents(handle), '');
  assert_equals(await getFileSize(handle), 0);
}, 'write() with an empty string to an empty file');

directory_test(async (t, root) => {
  const handle = await createEmptyFile(t, 'valid_utf8_string', root);
  const writer = await handle.createWriter();

  await writer.write(0, 'foo🤘');
  await writer.close();
  assert_equals(await getFileContents(handle), 'foo🤘');
  assert_equals(await getFileSize(handle), 7);
}, 'write() with a valid utf-8 string');

directory_test(async (t, root) => {
  const handle = await createEmptyFile(t, 'string_with_unix_line_ending', root);
  const writer = await handle.createWriter();

  await writer.write(0, 'foo\n');
  await writer.close();
  assert_equals(await getFileContents(handle), 'foo\n');
  assert_equals(await getFileSize(handle), 4);
}, 'write() with a string with unix line ending preserved');

directory_test(async (t, root) => {
  const handle =
      await createEmptyFile(t, 'string_with_windows_line_ending', root);
  const writer = await handle.createWriter();

  await writer.write(0, 'foo\r\n');
  await writer.close();
  assert_equals(await getFileContents(handle), 'foo\r\n');
  assert_equals(await getFileSize(handle), 5);
}, 'write() with a string with windows line ending preserved');

directory_test(async (t, root) => {
  const handle = await createEmptyFile(t, 'empty_array_buffer', root);
  const writer = await handle.createWriter();

  let buf = new ArrayBuffer(0);
  await writer.write(0, buf);
  await writer.close();
  assert_equals(await getFileContents(handle), '');
  assert_equals(await getFileSize(handle), 0);
}, 'write() with an empty array buffer to an empty file');

directory_test(async (t, root) => {
  const handle =
      await createEmptyFile(t, 'valid_string_typed_byte_array', root);
  const writer = await handle.createWriter();

  let buf = new ArrayBuffer(3);
  let intView = new Uint8Array(buf);
  intView[0] = 0x66;
  intView[1] = 0x6f;
  intView[2] = 0x6f;
  await writer.write(0, buf);
  await writer.close();
  assert_equals(await getFileContents(handle), 'foo');
  assert_equals(await getFileSize(handle), 3);
}, 'write() with a valid typed array buffer');

directory_test(async (t, root) => {
  const handle = await createEmptyFile(t, 'trunc_shrink', root);
  const writer = await handle.createWriter();

  await writer.write(0, new Blob(['1234567890']));
  await writer.truncate(5);
  await writer.close();

  assert_equals(await getFileContents(handle), '12345');
  assert_equals(await getFileSize(handle), 5);
}, 'truncate() to shrink a file');

directory_test(async (t, root) => {
  const handle = await createEmptyFile(t, 'trunc_grow', root);
  const writer = await handle.createWriter();

  await writer.write(0, new Blob(['abc']));
  await writer.truncate(5);
  await writer.close();

  assert_equals(await getFileContents(handle), 'abc\0\0');
  assert_equals(await getFileSize(handle), 5);
}, 'truncate() to grow a file');

directory_test(async (t, root) => {
  const dir = await createDirectory(t, 'parent_dir', root);
  const file_name = 'create_writer_fails_when_dir_removed.txt';
  const handle = await createEmptyFile(t, file_name, dir);

  await root.removeEntry('parent_dir', {recursive: true});
  await promise_rejects(t, 'NotFoundError', handle.createWriter());
}, 'createWriter() fails when parent directory is removed');

directory_test(async (t, root) => {
  const dir = await createDirectory(t, 'parent_dir', root);
  const file_name = 'write_fails_when_dir_removed.txt';
  const handle = await createEmptyFile(t, file_name, dir);
  const writer = await handle.createWriter();

  await root.removeEntry('parent_dir', {recursive: true});
  await promise_rejects(t, 'NotFoundError', writer.write(0, new Blob(['foo'])));
}, 'write() fails when parent directory is removed');

directory_test(async (t, root) => {
  const dir = await createDirectory(t, 'parent_dir', root);
  const file_name = 'truncate_fails_when_dir_removed.txt';
  const handle = await createEmptyFile(t, file_name, dir);
  const writer = await handle.createWriter();

  await root.removeEntry('parent_dir', {recursive: true});
  await promise_rejects(t, 'NotFoundError', writer.truncate(0));
}, 'truncate() fails when parent directory is removed');

directory_test(async (t, root) => {
  const dir = await createDirectory(t, 'parent_dir', root);
  const file_name = 'close_fails_when_dir_removed.txt';
  const handle = await createEmptyFile(t, file_name, dir);
  const writer = await handle.createWriter();
  await writer.write(0, new Blob(['foo']));

  await root.removeEntry('parent_dir', {recursive: true});
  await promise_rejects(t, 'NotFoundError', writer.close());
}, 'atomic writes: close() fails when parent directory is removed');

directory_test(async (t, root) => {
  const handle = await createEmptyFile(t, 'atomic_writes.txt', root);
  const writer = await handle.createWriter();
  await writer.write(0, new Blob(['foox']));

  const writer2 = await handle.createWriter();
  await writer2.write(0, new Blob(['bar']));

  assert_equals(await getFileSize(handle), 0);

  await writer2.close();
  assert_equals(await getFileContents(handle), 'bar');
  assert_equals(await getFileSize(handle), 3);

  await writer.close();
  assert_equals(await getFileContents(handle), 'foox');
  assert_equals(await getFileSize(handle), 4);
}, 'atomic writes: writers make atomic changes on close');

directory_test(async (t, root) => {
  const handle = await createEmptyFile(t, 'atomic_write_after_close.txt', root);
  const writer = await handle.createWriter();
  await writer.write(0, new Blob(['foo']));

  await writer.close();
  assert_equals(await getFileContents(handle), 'foo');
  assert_equals(await getFileSize(handle), 3);

  await promise_rejects(
      t, 'InvalidStateError', writer.write(0, new Blob(['abc'])));
}, 'atomic writes: write() after close() fails');

directory_test(async (t, root) => {
  const handle =
      await createEmptyFile(t, 'atomic_truncate_after_close.txt', root);
  const writer = await handle.createWriter();
  await writer.write(0, new Blob(['foo']));

  await writer.close();
  assert_equals(await getFileContents(handle), 'foo');
  assert_equals(await getFileSize(handle), 3);

  await promise_rejects(t, 'InvalidStateError', writer.truncate(0));
}, 'atomic writes: truncate() after close() fails');

directory_test(async (t, root) => {
  const handle = await createEmptyFile(t, 'atomic_close_after_close.txt', root);
  const writer = await handle.createWriter();
  await writer.write(0, new Blob(['foo']));

  await writer.close();
  assert_equals(await getFileContents(handle), 'foo');
  assert_equals(await getFileSize(handle), 3);

  await promise_rejects(t, 'InvalidStateError', writer.close());
}, 'atomic writes: close() after close() fails');

directory_test(async (t, root) => {
  const handle = await createEmptyFile(t, 'there_can_be_only_one.txt', root);
  const writer = await handle.createWriter();
  await writer.write(0, new Blob(['foo']));

  // This test might be flaky if there is a race condition allowing
  // close() to be called multiple times.
  let success_promises =
      [...Array(100)].map(() => writer.close().then(() => 1).catch(() => 0));
  let close_attempts = await Promise.all(success_promises);
  let success_count = close_attempts.reduce((x, y) => x + y);
  assert_equals(success_count, 1);
}, 'atomic writes: only one close() operation may succeed');

directory_test(async (t, root) => {
  const handle = await createFileWithContents(
      t, 'atomic_file_is_copied.txt', 'fooks', root);
  const writer = await handle.createWriter({keepExistingData: true});

  await writer.write(0, new Blob(['bar']));
  await writer.close();
  assert_equals(await getFileContents(handle), 'barks');
  assert_equals(await getFileSize(handle), 5);
}, 'createWriter({keepExistingData: true}): atomic writer initialized with source contents');

directory_test(async (t, root) => {
  const handle = await createFileWithContents(
      t, 'atomic_file_is_not_copied.txt', 'very long string', root);
  const writer = await handle.createWriter({keepExistingData: false});

  await writer.write(0, new Blob(['bar']));
  assert_equals(await getFileContents(handle), 'very long string');
  await writer.close();
  assert_equals(await getFileContents(handle), 'bar');
  assert_equals(await getFileSize(handle), 3);
}, 'createWriter({keepExistingData: false}): atomic writer initialized with empty file');

directory_test(async (t, root) => {
  const dir = await createDirectory(t, 'parent_dir', root);
  const file_name = 'atomic_writer_persists_removed.txt';
  const handle = await createFileWithContents(t, file_name, 'foo', dir);

  const writer = await handle.createWriter();
  await writer.write(0, new Blob(['bar']));

  await dir.removeEntry(file_name);
  await promise_rejects(t, 'NotFoundError', getFileContents(handle));

  await writer.close();
  assert_equals(await getFileContents(handle), 'bar');
  assert_equals(await getFileSize(handle), 3);
}, 'atomic writes: writer persists file on close, even if file is removed');
