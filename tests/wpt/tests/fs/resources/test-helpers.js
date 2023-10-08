// A special path component meaning "this directory."
const kCurrentDirectory = '.';

// A special path component meaning "the parent directory."
const kParentDirectory = '..';

// The lock modes of a writable file stream.
const WFS_MODES = ['siloed', 'exclusive'];

// The lock modes of an access handle.
const SAH_MODES = ['readwrite', 'read-only', 'readwrite-unsafe'];

// Possible return values of testLockAccess.
const LOCK_ACCESS = {
  SHARED: 'shared',
  EXCLUSIVE: 'exclusive',
};

// Array of separators used to separate components in hierarchical paths.
// Consider both '/' and '\' as path separators to ensure file names are
// platform-agnostic.
let kPathSeparators = ['/', '\\'];

async function getFileSize(handle) {
  const file = await handle.getFile();
  return file.size;
}

async function getFileContents(handle) {
  const file = await handle.getFile();
  return new Response(file).text();
}

async function getDirectoryEntryCount(handle) {
  let result = 0;
  for await (let entry of handle) {
    result++;
  }
  return result;
}

async function getSortedDirectoryEntries(handle) {
  let result = [];
  for await (let entry of handle.values()) {
    if (entry.kind === 'directory') {
      result.push(entry.name + '/');
    } else {
      result.push(entry.name);
    }
  }
  result.sort();
  return result;
}

async function createDirectory(test, name, parent) {
  const new_dir_handle = await parent.getDirectoryHandle(name, {create: true});
  test.add_cleanup(async () => {
    try {
      await parent.removeEntry(name, {recursive: true});
    } catch (e) {
      // Ignore any errors when removing directories, as tests might
      // have already removed the directory.
    }
  });
  return new_dir_handle;
}

async function createEmptyFile(test, name, parent) {
  const handle = await parent.getFileHandle(name, {create: true});
  test.add_cleanup(async () => {
    try {
      await parent.removeEntry(name);
    } catch (e) {
      // Ignore any errors when removing files, as tests might already remove
      // the file.
    }
  });
  // Make sure the file is empty.
  assert_equals(await getFileSize(handle), 0);
  return handle;
}

async function createFileWithContents(test, name, contents, parent) {
  const handle = await createEmptyFile(test, name, parent);
  const writer = await handle.createWritable();
  await writer.write(new Blob([contents]));
  await writer.close();
  return handle;
}

async function cleanup(test, value, cleanup_func) {
  test.add_cleanup(async () => {
    try {
      await cleanup_func();
    } catch (e) {
      // Ignore any errors when removing files, as tests might already remove
      // the file.
    }
  });
  return value;
}

async function cleanup_writable(test, value) {
  return cleanup(test, value, async () => {
    try {
      await value.close();
    } catch (e) {
      // Ignore any errors when closing writables, since attempting to close
      // aborted or closed writables will error.
    }
  });
}

function createFileHandles(dir, ...fileNames) {
  return Promise.all(
      fileNames.map(fileName => dir.getFileHandle(fileName, {create: true})));
}

// Releases a lock created by one of the create*WithCleanup functions below.
async function releaseLock(lockPromise) {
  const result = await lockPromise;
  if (result?.close) {
    await result.close();
  }
}

function cleanupLockPromise(t, lockPromise) {
  return cleanup(t, lockPromise, () => releaseLock(lockPromise));
}

function createWFSWithCleanup(t, fileHandle, wfsOptions) {
  return cleanupLockPromise(t, fileHandle.createWritable(wfsOptions));
}

// Returns createWFSWithCleanup bound with wfsOptions.
function createWFSWithCleanupFactory(wfsOptions) {
  return (t, fileHandle) => createWFSWithCleanup(t, fileHandle, wfsOptions);
}

function createSAHWithCleanup(t, fileHandle, sahOptions) {
  return cleanupLockPromise(t, fileHandle.createSyncAccessHandle(sahOptions));
}

// Returns createSAHWithCleanup bound with sahOptions.
function createSAHWithCleanupFactory(sahOptions) {
  return (t, fileHandle) => createSAHWithCleanup(t, fileHandle, sahOptions);
}

function createMoveWithCleanup(
    t, fileHandle, fileName = 'unique-file-name.test') {
  return cleanupLockPromise(t, fileHandle.move(fileName));
}

function createRemoveWithCleanup(t, fileHandle) {
  return cleanupLockPromise(t, fileHandle.remove({recursive: true}));
}

// For each key in `testFuncs` if there is a matching key in `testDescs`,
// creates a directory_test passing the respective key's value for the func and
// description arguments. If there is not a matching key in `testDescs`, the
// test is not created. This will throw if `testDescs` contains a key that is
// not in `testFuncs`.
function selectDirectoryTests(testDescs, testFuncs) {
  for (const testDesc in testDescs) {
    if (!testFuncs.hasOwnProperty(testDesc)) {
      throw new Error(
          'Passed a test description in testDescs that wasn\'t in testFuncs.');
    }
    directory_test(testFuncs[testDesc], testDescs[testDesc]);
  }
}

// Adds tests to test the interaction between a lock created by `createLock1`
// and a lock created by `createLock2`.
//
// The description of each test is passed in through `testDescs`. If a test
// description is omitted, it is not run.
//
// For all tests, `createLock1` is called first.
function generateCrossLockTests(createLock1, createLock2, testDescs) {
  if (testDescs === undefined) {
    throw new Error('Must pass testDescs.');
  }
  selectDirectoryTests(testDescs, {

    // This tests that a lock can't be acquired on a file that already has a
    // lock of another type.
    sameFile: async (t, rootDir) => {
      const [fileHandle] = await createFileHandles(rootDir, 'BFS.test');

      createLock1(t, fileHandle);
      await promise_rejects_dom(
          t, 'NoModificationAllowedError', createLock2(t, fileHandle));
    },

    // This tests that a lock on one file does not interfere with the creation
    // of a lock on another file.
    diffFile: async (t, rootDir) => {
      const [fooFileHandle, barFileHandle] =
          await createFileHandles(rootDir, 'foo.test', 'bar.test');

      createLock1(t, fooFileHandle);
      await createLock2(t, barFileHandle);
    },

    // This tests that after a lock has been acquired on a file and then
    // released, another lock of another type can be acquired. This will fail if
    // `createLock1` and `createLock2` create the same shared lock.
    acquireAfterRelease: async (t, rootDir) => {
      let [fileHandle] = await createFileHandles(rootDir, 'BFS.test');

      const lockPromise = createLock1(t, fileHandle);
      await promise_rejects_dom(
          t, 'NoModificationAllowedError', createLock2(t, fileHandle));

      await releaseLock(lockPromise);
      // Recreate the file in case releasing the lock moves/removes it.
      [fileHandle] = await createFileHandles(rootDir, 'BFS.test');
      await createLock2(t, fileHandle);
    },

    // This tests that after multiple locks of some shared lock type have been
    // acquired on a file and then all released, another lock of another lock
    // type can be acquired.
    multiAcquireAfterRelease: async (t, rootDir) => {
      const [fileHandle] = await createFileHandles(rootDir, 'BFS.test');

      const lock1 = await createLock1(t, fileHandle);
      const lock2 = await createLock1(t, fileHandle);

      await promise_rejects_dom(
          t, 'NoModificationAllowedError', createLock2(t, fileHandle));
      await lock1.close();
      await promise_rejects_dom(
          t, 'NoModificationAllowedError', createLock2(t, fileHandle));
      await lock2.close();

      await createLock2(t, fileHandle);
    },

    // This tests that a lock taken on a directory prevents a lock being
    // acquired on a file contained within that directory.
    takeDirThenFile: async (t, rootDir) => {
      const dirHandle = await rootDir.getDirectoryHandle('foo', {create: true});
      const [fileHandle] = await createFileHandles(dirHandle, 'BFS.test');

      createLock1(t, dirHandle);
      await promise_rejects_dom(
          t, 'NoModificationAllowedError', createLock2(t, fileHandle));
    },

    // This tests that a lock acquired on a file prevents a lock being acquired
    // on an ancestor of that file.
    takeFileThenDir: async (t, rootDir) => {
      const grandparentHandle =
          await rootDir.getDirectoryHandle('foo', {create: true});
      const parentHandle =
          await grandparentHandle.getDirectoryHandle('bar', {create: true});
      let [fileHandle] = await createFileHandles(parentHandle, 'BFS.test');

      // Test parent handle.
      const lock1 = createLock1(t, fileHandle);
      await promise_rejects_dom(
          t, 'NoModificationAllowedError', createLock2(t, parentHandle));

      // Release the lock so we can recreate it.
      await releaseLock(lock1);
      // Recreate the file in case releasing the lock moves/removes it.
      [fileHandle] = await createFileHandles(parentHandle, 'BFS.test');

      // Test grandparent handle.
      createLock1(t, fileHandle);
      await promise_rejects_dom(
          t, 'NoModificationAllowedError', createLock2(t, grandparentHandle));
    },
  });
}

// Tests whether the multiple locks can be created by createLock on a file
// handle or if only one can. Returns LOCK_ACCESS.SHARED for the former and
// LOCK_ACCESS.EXCLUSIVE for the latter.
async function testLockAccess(t, fileHandle, createLock) {
  createLock(t, fileHandle);

  let access;
  try {
    await createLock(t, fileHandle);
    access = LOCK_ACCESS.SHARED;
  } catch (e) {
    access = LOCK_ACCESS.EXCLUSIVE;
    assert_throws_dom('NoModificationAllowedError', () => {
      throw e;
    });
  }

  return access;
}
