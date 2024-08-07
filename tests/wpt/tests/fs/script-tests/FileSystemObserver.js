'use strict';

// This script depends on the following scripts:
//    resources/test-helpers.js
//    resources/collecting-file-system-observer.js
//    resources/change-observer-scope-test.js
//    script-tests/FileSystemObserver-writable-file-stream.js

promise_test(async t => {
  try {
    const observer = new FileSystemObserver(() => {});
  } catch {
    assert_unreached();
  }
}, 'Creating a FileSystemObserver from a supported global succeeds');

directory_test(async (t, root_dir) => {
  const observer = new FileSystemObserver(() => {});
  try {
    observer.unobserve(root_dir);
  } catch {
    assert_unreached();
  }
}, 'Calling unobserve() without a corresponding observe() shouldn\'t throw');

directory_test(async (t, root_dir) => {
  const observer = new FileSystemObserver(() => {});
  try {
    observer.unobserve(root_dir);
    observer.unobserve(root_dir);
  } catch {
    assert_unreached();
  }
}, 'unobserve() is idempotent');

promise_test(async t => {
  const observer = new FileSystemObserver(() => {});
  try {
    observer.disconnect();
  } catch {
    assert_unreached();
  }
}, 'Calling disconnect() without observing shouldn\'t throw');

promise_test(async t => {
  const observer = new FileSystemObserver(() => {});
  try {
    observer.disconnect();
    observer.disconnect();
  } catch {
    assert_unreached();
  }
}, 'disconnect() is idempotent');

directory_test(async (t, root_dir) => {
  const observer = new FileSystemObserver(() => {});

  // Create a `FileSystemFileHandle` and delete its underlying file entry.
  const file = await root_dir.getFileHandle(getUniqueName(), {create: true});
  await file.remove();

  await promise_rejects_dom(t, 'NotFoundError', observer.observe(file));
}, 'observe() fails when file does not exist');

directory_test(async (t, root_dir) => {
  const observer = new FileSystemObserver(() => {});

  // Create a `FileSystemDirectoryHandle` and delete its underlying file entry.
  const dir =
      await root_dir.getDirectoryHandle(getUniqueName(), {create: true});
  await dir.remove();

  await promise_rejects_dom(t, 'NotFoundError', observer.observe(dir));
}, 'observe() fails when directory does not exist');

directory_test(async (t, root_dir) => {
  const dir =
      await root_dir.getDirectoryHandle(getUniqueName(), {create: true});

  const scope_test = new ScopeTest(t, dir);
  const watched_handle = await scope_test.watched_handle();

  for (const recursive of [false, true]) {
    for await (const path of scope_test.in_scope_paths(recursive)) {
      const observer = new CollectingFileSystemObserver(t, root_dir);
      await observer.observe([watched_handle], {recursive});

      // Create `file`.
      const file = await path.createHandle();

      // Expect one "appeared" event to happen on `file`.
      const records = await observer.getRecords();
      await assert_records_equal(
          watched_handle, records,
          [appearedEvent(file, path.relativePathComponents())]);

      observer.disconnect();
    }
  }
}, 'Creating a file through FileSystemDirectoryHandle.getFileHandle is reported as an "appeared" event if in scope');

directory_test(async (t, root_dir) => {
  const dir =
      await root_dir.getDirectoryHandle(getUniqueName(), {create: true});

  const scope_test = new ScopeTest(t, dir);
  const watched_handle = await scope_test.watched_handle();

  for (const recursive of [false, true]) {
    for await (const path of scope_test.in_scope_paths(recursive)) {
      const file = await path.createHandle();

      const observer = new CollectingFileSystemObserver(t, root_dir);
      await observer.observe([watched_handle], {recursive});

      // Remove `file`.
      await file.remove();

      // Expect one "disappeared" event to happen on `file`.
      const records = await observer.getRecords();
      await assert_records_equal(
          watched_handle, records,
          [disappearedEvent(file, path.relativePathComponents())]);

      observer.disconnect();
    }
  }
}, 'Removing a file through FileSystemFileHandle.remove is reported as an "disappeared" event if in scope');

directory_test(async (t, root_dir) => {
  const dir =
      await root_dir.getDirectoryHandle(getUniqueName(), {create: true});

  const scope_test = new ScopeTest(t, dir);
  const watched_handle = await scope_test.watched_handle();

  for (const recursive of [false, true]) {
    for await (const path of scope_test.out_of_scope_paths(recursive)) {
      const observer = new CollectingFileSystemObserver(t, root_dir);
      await observer.observe([watched_handle], {recursive});

      // Create and remove `file`.
      const file = await path.createHandle();
      await file.remove();

      // Expect the observer to receive no events.
      const records = await observer.getRecords();
      await assert_records_equal(watched_handle, records, []);

      observer.disconnect();
    }
  }
}, 'Events outside the watch scope are not sent to the observer\'s callback');

directory_test(async (t, root_dir) => {
  const dir =
      await root_dir.getDirectoryHandle(getUniqueName(), {create: true});

  const scope_test = new ScopeTest(t, dir);
  const watched_handle = await scope_test.watched_handle();

  for (const recursive of [false, true]) {
    for await (const src of scope_test.in_scope_paths(recursive)) {
      for await (const dest of scope_test.in_scope_paths(recursive)) {
        const file = await src.createHandle();

        const observer = new CollectingFileSystemObserver(t, root_dir);
        await observer.observe([watched_handle], {recursive});

        // Move `file`.
        await file.move(dest.parentHandle(), dest.fileName());

        // Expect one "moved" event to happen on `file`.
        const records = await observer.getRecords();
        await assert_records_equal(
            watched_handle, records, [movedEvent(
                                         file, dest.relativePathComponents(),
                                         src.relativePathComponents())]);

        observer.disconnect();
      }
    }
  }
}, 'Moving a file through FileSystemFileHandle.move is reported as a "moved" event if destination and source are in scope');

directory_test(async (t, root_dir) => {
  const dir =
      await root_dir.getDirectoryHandle(getUniqueName(), {create: true});

  const scope_test = new ScopeTest(t, dir);
  const watched_handle = await scope_test.watched_handle();

  for (const recursive of [false, true]) {
    for await (const src of scope_test.out_of_scope_paths(recursive)) {
      for await (const dest of scope_test.out_of_scope_paths(recursive)) {
        const file = await src.createHandle();

        const observer = new CollectingFileSystemObserver(t, root_dir);
        await observer.observe([watched_handle], {recursive});

        // Move `file`.
        await file.move(dest.parentHandle(), dest.fileName());

        // Expect the observer to not receive any events.
        const records = await observer.getRecords();
        await assert_records_equal(watched_handle, records, []);
      }
    }
  }
}, 'Moving a file through FileSystemFileHandle.move is not reported if destination and source are not in scope');

directory_test(async (t, root_dir) => {
  const dir =
      await root_dir.getDirectoryHandle(getUniqueName(), {create: true});

  const scope_test = new ScopeTest(t, dir);
  const watched_handle = await scope_test.watched_handle();

  for (const recursive of [false, true]) {
    for await (const src of scope_test.out_of_scope_paths(recursive)) {
      for await (const dest of scope_test.in_scope_paths(recursive)) {
        const file = await src.createHandle();

        const observer = new CollectingFileSystemObserver(t, root_dir);
        await observer.observe([watched_handle], {recursive});

        // Move `file`.
        await file.move(dest.parentHandle(), dest.fileName());

        // Expect one "appeared" event to happen on `file`.
        const records = await observer.getRecords();
        await assert_records_equal(
            watched_handle, records,
            [appearedEvent(file, dest.relativePathComponents())]);
      }
    }
  }
}, 'Moving a file through FileSystemFileHandle.move is reported as a "appeared" event if only destination is in scope');

directory_test(async (t, root_dir) => {
  const dir =
      await root_dir.getDirectoryHandle(getUniqueName(), {create: true});

  const scope_test = new ScopeTest(t, dir);
  const watched_handle = await scope_test.watched_handle();

  for (const recursive of [false, true]) {
    for await (const src of scope_test.in_scope_paths(recursive)) {
      for await (const dest of scope_test.out_of_scope_paths(recursive)) {
        // These both point to the same underlying file entry initially until
        // move is called on `fileToMove`. `file` is kept so that we have a
        // handle that still points at the source file entry.
        const file = await src.createHandle();
        const fileToMove = await src.createHandle();

        const observer = new CollectingFileSystemObserver(t, root_dir);
        await observer.observe([watched_handle], {recursive});

        // Move `fileToMove`.
        await fileToMove.move(dest.parentHandle(), dest.fileName());

        // Expect one "disappeared" event to happen on `file`.
        const records = await observer.getRecords();
        await assert_records_equal(
            watched_handle, records,
            [disappearedEvent(file, src.relativePathComponents())]);
      }
    }
  }
}, 'Moving a file through FileSystemFileHandle.move is reported as a "disappeared" event if only source is in scope');

// Wraps a `CollectingFileSystemObserver` and disconnects the observer after it's
// received `num_of_records_to_observe`.
class DisconnectingFileSystemObserver {
  #collectingObserver;

  #num_of_records_to_observe;

  #called_disconnect = false;
  #records_observed_count = 0;

  constructor(test, root_dir, num_of_records_to_observe) {
    this.#collectingObserver = new CollectingFileSystemObserver(
        test, root_dir, this.#callback.bind(this));
    this.#num_of_records_to_observe = num_of_records_to_observe;
  }

  #callback(records, observer) {
    this.#records_observed_count += records.length;

    const called_disconnect = this.#called_disconnect;

    // Call `disconnect` once after we've received `num_of_records_to_observe`.
    if (!called_disconnect &&
        this.#records_observed_count >= this.#num_of_records_to_observe) {
      observer.disconnect();
      this.#called_disconnect = true;
    }

    return {called_disconnect};
  }

  getRecordsWithCallbackInfo() {
    return this.#collectingObserver.getRecordsWithCallbackInfo();
  }

  observe(handles) {
    return this.#collectingObserver.observe(handles);
  }
}


directory_test(async (t, root_dir) => {
  const total_files_to_create = 100;

  const child_dir =
      await root_dir.getDirectoryHandle(getUniqueName(), {create: true});

  // Create a `FileSystemObserver` that will disconnect after its
  // received half of the total files we're going to create.
  const observer = new DisconnectingFileSystemObserver(
      t, root_dir, total_files_to_create / 2);

  // Observe the child directory and create files in it.
  await observer.observe([child_dir]);
  for (let i = 0; i < total_files_to_create; i++) {
    child_dir.getFileHandle(`file${i}`, {create: true});
  }

  // Wait for `disconnect` to be called.
  const records_with_disconnect_state =
      await observer.getRecordsWithCallbackInfo();

  // No observations should have been received after disconnected has been
  // called.
  assert_false(
      records_with_disconnect_state.some(
          ({called_disconnect}) => called_disconnect),
      'Received records after disconnect.');
}, 'Observations stop after disconnect()');

directory_test(async (t, root_dir) => {
  const num_of_child_dirs = 5;
  const num_files_to_create_per_directory = 100;
  const total_files_to_create =
      num_files_to_create_per_directory * num_of_child_dirs;

  const child_dirs = await createDirectoryHandles(
      root_dir, getUniqueName(), getUniqueName(), getUniqueName());

  // Create a `FileSystemObserver` that will disconnect after its received half
  // of the total files we're going to create.
  const observer = new DisconnectingFileSystemObserver(
      t, root_dir, total_files_to_create / 2);

  // Observe the child directories and create files in them.
  await observer.observe(child_dirs);
  for (let i = 0; i < num_files_to_create_per_directory; i++) {
    child_dirs.forEach(
        child_dir => child_dir.getFileHandle(`file${i}`, {create: true}));
  }

  // Wait for `disconnect` to be called.
  const records_with_disconnect_state =
      await observer.getRecordsWithCallbackInfo();

  // No observations should have been received after disconnected has been
  // called.
  assert_false(
      records_with_disconnect_state.some(
          ({called_disconnect}) => called_disconnect),
      'Received records after disconnect.');
}, 'Observations stop for all observed handles after disconnect()');
