importScripts('/resources/testharness.js');
importScripts('resources/sandboxed-fs-test-helpers.js');
importScripts('resources/test-helpers.js');

'use strict';

// Adds tests for expected behaviors of a writable stream created in `wfsMode`
// mode.
function lockPropertyTests(wfsMode, expectedLockAccess) {
  const createWFSLock = createWFSWithCleanupFactory({mode: wfsMode});

  directory_test(async (t, rootDir) => {
    const [fileHandle] = await createFileHandles(rootDir, 'BFS.test');

    const {mode} = await createWFSLock(t, fileHandle);
    assert_equals(mode, wfsMode);
  }, `A writable stream in ${wfsMode} mode has a mode property equal to` +
    ` ${wfsMode}`);

  directory_test(async (t, rootDir) => {
    const [fileHandle] = await createFileHandles(rootDir, 'BFS.test');
    assert_equals(
        await testLockAccess(t, fileHandle, createWFSLock), expectedLockAccess);
  }, `A writable stream in ${wfsMode} mode takes a lock that is` +
    ` ${expectedLockAccess}`);

  // Test interaction with other writable stream modes.
  for (const mode of WFS_MODES) {
    // Add tests depending on which writable stream modes are being tested
    // against each other.
    const testingAgainstSelf = mode === wfsMode;
    const testingExclusiveLock = expectedLockAccess === 'exclusive';
    const tests = {
      diffFile: `When there's an open writable stream in ${wfsMode} mode on a` +
          ` file, can open another writable stream in ${mode} on a different` +
          ` file`,
    };
    if (!testingAgainstSelf || testingExclusiveLock) {
      tests.sameFile = `When there's an open writable stream in ${wfsMode}` +
          ` mode on a file, cannot open another writable stream in ${mode} on` +
          ` that same file`;
    }
    if (testingExclusiveLock) {
      tests.acquireAfterRelease = `After a writable stream in ${wfsMode} mode` +
          ` on a file has been closed, can open another writable stream in` +
          ` ${mode} on the same file`;
    }
    if (!testingExclusiveLock && !testingAgainstSelf) {
      tests.multiAcquireAfterRelease = `After all writable streams in` +
          ` ${wfsMode} mode on a file has been closed, can open another` +
          ` writable stream in ${mode} on the same file`;
    }

    generateCrossLockTests(
        createWFSLock, createWFSWithCleanupFactory({mode: mode}), tests);
  }
}

directory_test(async (t, rootDir) => {
  const [fileHandle] = await createFileHandles(rootDir, 'BFS.test');

  const syncHandle = await createWFSWithCleanup(t, fileHandle);
  assert_equals(syncHandle.mode, 'siloed');
}, 'A writable stream opens in siloed mode by default');

lockPropertyTests('siloed', LOCK_ACCESS.SHARED);
lockPropertyTests('exclusive', LOCK_ACCESS.EXCLUSIVE);

done();
