importScripts('/resources/testharness.js');
importScripts('resources/sandboxed-fs-test-helpers.js');
importScripts('resources/test-helpers.js');
importScripts('resources/collecting-file-system-observer.js');

'use strict';

directory_test(async (t, root_dir) => {
  const file = await root_dir.getFileHandle(getUniqueName(), {create: true});

  const observer = new CollectingFileSystemObserver(t, root_dir);
  await observer.observe([file]);

  // Write to `file` through a `FileSystemSyncAccessHandle`.
  const syncHandle = await createSAHWithCleanup(t, file);
  const writeBuffer = new TextEncoder().encode('contents');
  syncHandle.write(writeBuffer);
  syncHandle.close();

  // Expect one "modified" event to happen on `file`.
  const records = await observer.getRecords();
  await assert_records_equal(file, records, [modifiedEvent(file, [])]);
}, 'FileSystemSyncAccessHandle.write produces a "modified" event');

directory_test(async (t, root_dir) => {
  const file = await root_dir.getFileHandle(getUniqueName(), {create: true});

  const observer = new CollectingFileSystemObserver(t, root_dir);
  await observer.observe([file]);

  // Write to `file` through a `FileSystemSyncAccessHandle` multiple times.
  const syncHandle = await createSAHWithCleanup(t, file);
  const writeBuffer = new TextEncoder().encode('contents');

  const numberOfWrites = 3;
  const expectedEvents = [];
  for (let i = 0; i < numberOfWrites; i++) {
    expectedEvents.push(modifiedEvent(file, []));
    syncHandle.write(writeBuffer);
  }
  syncHandle.close();

  // Expect the same number of "modified" event to happen on `file` as there
  // were writes to it.
  const records = await observer.getRecords();
  await assert_records_equal(file, records, expectedEvents);
}, 'Multiple FileSystemSyncAccessHandle.writes produces the same amount of "modified" event');

directory_test(async (t, root_dir) => {
  const file = await root_dir.getFileHandle(getUniqueName(), {create: true});

  const observer = new CollectingFileSystemObserver(t, root_dir);
  await observer.observe([file]);

  // Truncate to `file` through a `FileSystemSyncAccessHandle`.
  const syncHandle = await createSAHWithCleanup(t, file);
  syncHandle.truncate(1);
  syncHandle.close();

  // Expect one "modified" event to happen on `file`.
  const records = await observer.getRecords();
  await assert_records_equal(file, records, [modifiedEvent(file, [])]);
}, 'FileSystemSyncAccessHandle.truncate produces a "modified" event');

directory_test(async (t, root_dir) => {
  const file = await root_dir.getFileHandle(getUniqueName(), {create: true});

  const observer = new CollectingFileSystemObserver(t, root_dir);
  await observer.observe([file]);

  // Write to `file`.
  const syncHandle = await createSAHWithCleanup(t, file);
  const readBuffer = new Uint8Array(24);
  syncHandle.read(readBuffer);
  syncHandle.flush();
  syncHandle.getSize();
  syncHandle.close();

  // Expect no events to happen.
  const records = await observer.getRecords();
  await assert_records_equal(file, records, []);
}, 'FileSystemSyncAccessHandle methods that don\'t modify the file don\'t produce events');

done();
