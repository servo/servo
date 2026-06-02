'use strict';

// This script depends on the following scripts:
//    resources/test-helpers.js
//    resources/collecting-file-system-observer.js
//    script-tests/FileSystemObserver-writable-file-stream.js

directory_test(async (t, root_dir) => {
  const file = await root_dir.getFileHandle(getUniqueName(), {create: true});

  const observer = new CollectingFileSystemObserver(t, root_dir);
  await observer.observe([file]);

  // Write to `file` through a `FileSystemWritableFileStream`.
  const writable = await createWFSWithCleanup(t, file);
  await writable.write('contents');
  await writable.close();

  // Expect one "modified" event to happen on `file`.
  const records = await observer.getRecords();
  await assert_records_equal(file, records, [modifiedEvent(file, [])]);
}, 'Closing a FileSystemWritableFileStream that\'s modified the file produces a "modified" event');

directory_test(async (t, root_dir) => {
  const file = await root_dir.getFileHandle(getUniqueName(), {create: true});

  const observer = new CollectingFileSystemObserver(t, root_dir);
  await observer.observe([file]);

  // Write to `file`.
  const writable = await createWFSWithCleanup(t, file);
  await writable.write('contents');
  await writable.truncate(1);
  await writable.seek(1);

  {
    // Expect no events to happen.
    const records = await observer.getRecords();
    await assert_records_equal(file, records, []);
  }

  await writable.abort();

  {
    // Expect no events to happen.
    const records = await observer.getRecords();
    await assert_records_equal(file, records, []);
  }
}, 'All FileSystemWritableFileStream methods that aren\'t closed don\'t produce events');
