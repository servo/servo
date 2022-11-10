'use strict';

directory_test(async (t, root) => {
  const handle =
      await createFileWithContents(t, 'file-to-remove', '12345', root);
  await createFileWithContents(t, 'file-to-keep', 'abc', root);

  const writable = await cleanup_writable(t, await handle.createWritable());
  await promise_rejects_dom(
    t, 'InvalidModificationError', root.removeEntry('file-to-remove'));

  await writable.close();
  await root.removeEntry('file-to-remove');

  assert_array_equals(
      await getSortedDirectoryEntries(root),
      ['file-to-keep']);
}, 'removeEntry() while the file has an open writable fails');
