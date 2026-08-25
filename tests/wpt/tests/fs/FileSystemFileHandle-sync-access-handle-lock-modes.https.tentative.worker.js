importScripts('/resources/testharness.js');
importScripts('resources/sandboxed-fs-test-helpers.js');
importScripts('resources/test-helpers.js');

'use strict';

const LOCK_WRITE_PERMISSION = {
  NOT_WRITABLE: 'not writable',
  WRITABLE: 'writable',
};

async function testLockWritePermission(t, fileHandle, createSAHLock) {
  const syncHandle = await createSAHLock(t, fileHandle);

  let permission;
  const writeBuffer = new TextEncoder().encode('Hello Storage Foundation');
  try {
    syncHandle.write(writeBuffer, {at: 0});
    permission = LOCK_WRITE_PERMISSION.WRITABLE;
  } catch (e) {
    permission = LOCK_WRITE_PERMISSION.NOT_WRITABLE;
    assert_throws_dom('NoModificationAllowedError', () => {
      throw e;
    });
  }
  // truncate and flush should throw a NoModificationAllowedError if an only if
  // write threw a NoModificationAllowedError.
  if (permission == LOCK_WRITE_PERMISSION.WRITABLE) {
    syncHandle.truncate(0);
    syncHandle.flush();
  } else {
    assert_throws_dom(
        'NoModificationAllowedError', () => syncHandle.truncate(0));
    assert_throws_dom('NoModificationAllowedError', () => syncHandle.flush());
  }

  return permission;
}

// Adds tests for expected behaviors of an access handle created in `sahMode`
// mode.
function lockPropertyTests(
    sahMode, expectedLockAccess, expectedLockWritePermission) {
  const createSAHLock = createSAHWithCleanupFactory({mode: sahMode});

  directory_test(async (t, rootDir) => {
    const [fileHandle] = await createFileHandles(rootDir, 'BFS.test');

    const {mode} = await createSAHLock(t, fileHandle);
    assert_equals(mode, sahMode);
  }, `An access handle in ${sahMode} mode has a mode property equal to` +
    ` ${sahMode}`);

  directory_test(async (t, rootDir) => {
    const [fileHandle] = await createFileHandles(rootDir, 'BFS.test');
    assert_equals(
        await testLockAccess(t, fileHandle, createSAHLock), expectedLockAccess);
  }, `An access handle in ${sahMode} mode takes a lock that is` +
    ` ${expectedLockAccess}`);

  directory_test(async (t, rootDir) => {
    const [fileHandle] = await createFileHandles(rootDir, 'BFS.test');
    assert_equals(
        await testLockWritePermission(t, fileHandle, createSAHLock),
        expectedLockWritePermission);
  }, `An access handle in ${sahMode} mode is ${expectedLockWritePermission}`);

  // Test interaction with other access handle modes.
  for (const mode of SAH_MODES) {
    // Add tests depending on which access handle modes are being tested against
    // each other.
    const testingAgainstSelf = mode === sahMode;
    const testingExclusiveLock = expectedLockAccess === 'exclusive';
    const tests = {
      diffFile: `When there's an open access handle in ${sahMode} mode on a` +
          ` file, can open another access handle in ${mode} on a different` +
          ` file`,
    };
    if (!testingAgainstSelf || testingExclusiveLock) {
      tests.sameFile = `When there's an open access handle in ${sahMode} mode` +
          ` on a file, cannot open another access handle in ${mode} on that` +
          ` same file`;
    }
    if (testingExclusiveLock) {
      tests.acquireAfterRelease = `After an access handle in ${sahMode} mode` +
          ` on a file has been closed, can open another access handle in` +
          ` ${mode} on the same file`;
    }
    if (!testingExclusiveLock && !testingAgainstSelf) {
      tests.multiAcquireAfterRelease = `After all access handles in` +
          ` ${sahMode} mode on a file has been closed, can open another` +
          ` access handle in ${mode} on the same file`;
    }

    generateCrossLockTests(
        createSAHLock, createSAHWithCleanupFactory({mode: mode}), tests);
  }
}

directory_test(async (t, rootDir) => {
  const [fileHandle] = await createFileHandles(rootDir, 'BFS.test');

  const syncHandle = await createSAHWithCleanup(t, fileHandle);
  assert_equals(syncHandle.mode, 'readwrite');
}, 'A sync access handle opens in readwrite mode by default');

lockPropertyTests(
    'readwrite', LOCK_ACCESS.EXCLUSIVE, LOCK_WRITE_PERMISSION.WRITABLE);
lockPropertyTests(
    'read-only', LOCK_ACCESS.SHARED, LOCK_WRITE_PERMISSION.NOT_WRITABLE);
lockPropertyTests(
    'readwrite-unsafe', LOCK_ACCESS.SHARED, LOCK_WRITE_PERMISSION.WRITABLE);

done();
