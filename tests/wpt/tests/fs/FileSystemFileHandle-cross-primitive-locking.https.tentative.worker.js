importScripts('/resources/testharness.js');
importScripts('resources/sandboxed-fs-test-helpers.js');
importScripts('resources/test-helpers.js');

'use strict';

// Adds tests to test the interaction between a lock created by the move
// operation and a lock created by `createLock`.
function generateCrossLockMoveTests(lockName, createLock) {
  generateCrossLockTests(createMoveWithCleanup, createLock, {
    diffFile: `A file with an ongoing move operation does not interfere with` +
        ` ${lockName} on another file`,
    acquireAfterRelease: `After a file has finished moving, that file can` +
        ` have ${lockName}`,
    // TODO(https://github.com/whatwg/fs/pull/10): Add tests for directory moves
    // once supported.
  });

  directory_test(async (t, rootDir) => {
    const [fooFileHandle, barFileHandle] =
        await createFileHandles(rootDir, 'foo.test', 'bar.test');

    createLock(t, fooFileHandle);
    await promise_rejects_dom(
        t, 'NoModificationAllowedError',
        createMoveWithCleanup(t, barFileHandle, 'foo.test'));
  }, `A file cannot be moved to a location with ${lockName}`);
}

// Adds tests to test the interaction between a lock created by the remove
// operation and a lock created by `createLock`.
function generateCrossLockRemoveTests(lockName, createLock) {
  generateCrossLockTests(createRemoveWithCleanup, createLock, {
    diffFile: `A file with an ongoing remove operation does not interfere` +
        ` with the creation of ${lockName} on another file`,
    acquireAfterRelease: `After a file has finished being removed, that file` +
        ` can have ${lockName}`,
  });
  generateCrossLockTests(createLock, createRemoveWithCleanup, {
    takeFileThenDir: `A directory cannot be removed if it contains a file` +
        ` that has ${lockName}.`,
  });
}

// Gets the name of a writable file stream opened in `wfsMode` to be used in
// tests.
function getWFSLockName(wfsMode) {
  return `an open writable stream in ${wfsMode} mode`
}

// Adds tests to test the interaction between a lock created by an open writable
// and a lock created by `createLock`.
function generateCrossLockWFSTests(lockName, createLock, wfsMode) {
  const WFSLockName = getWFSLockName(wfsMode);
  const tests = {
    sameFile: `When there's ${WFSLockName} on a file, cannot have` +
        ` ${lockName} on that same file`,
    diffFile: `A file with ${WFSLockName} does not interfere with` +
        ` ${lockName} on another file`,
  };
  if (wfsMode === 'siloed') {
    tests.multiAcquireAfterRelease = `After all writable streams in siloed` +
        ` mode have been closed for a file, that file can have ${lockName}`;
  } else {
    tests.acquireAfterRelease = `After a writable stream in exclusive mode` +
        ` has been closed for a file, that file can have ${lockName}`;
  }
  generateCrossLockTests(
      createWFSWithCleanupFactory({mode: wfsMode}), createLock, tests);
}

// Adds tests to test the interaction between a lock created by an open access
// handle in `sahMode and locks created by other file primitives and operations.
function generateCrossLockSAHTests(sahMode) {
  const createSAHLock = createSAHWithCleanupFactory({mode: sahMode});
  const SAHLockName = `an open access handle in ${sahMode} mode`;

  // Test interaction between move locks and SAH locks.
  generateCrossLockMoveTests(SAHLockName, createSAHLock);
  generateCrossLockTests(createSAHLock, createMoveWithCleanup, {
    sameFile: `A file with ${SAHLockName} cannot be moved`,
    diffFile: `A file with ${SAHLockName} does not interfere with moving` +
        ` another file`,
    acquireAfterRelease: `After ${SAHLockName} on a file has been closed,` +
        ` that file can be moved`,
  });

  // Test interaction between remove locks and SAH locks.
  generateCrossLockRemoveTests(SAHLockName, createSAHLock);
  generateCrossLockTests(createSAHLock, createRemoveWithCleanup, {
    sameFile: `A file with ${SAHLockName} cannot be removed`,
    diffFile: `A file with ${SAHLockName} does not interfere with removing` +
        ` another file`,
    acquireAfterRelease: `After ${SAHLockName} on a file has been closed,` +
        ` that file can be removed`,
  });

  // Test interaction between WFS locks and SAH locks.
  for (const wfsMode of WFS_MODES) {
    const WFSLockName = getWFSLockName(wfsMode);
    const wfsOptions = {mode: wfsMode};
    generateCrossLockWFSTests(SAHLockName, createSAHLock, wfsMode);
    generateCrossLockTests(
        createSAHLock, createWFSWithCleanupFactory(wfsOptions), {
          sameFile: `When there's ${SAHLockName} on a file, cannot open` +
              ` ${WFSLockName} on that same file`,
          diffFile: `A file with ${SAHLockName} does not interfere with the` +
              ` creation of ${WFSLockName} on another file`,
        });
  }
}

// Test interaction for each SAH lock mode.
for (const sahMode of SAH_MODES) {
  generateCrossLockSAHTests(sahMode);
}

// Test interaction for each WFS lock mode.
for (const wfsMode of WFS_MODES) {
  const WFSLockName = getWFSLockName(wfsMode);
  const wfsOptions = {mode: wfsMode};
  // Test interaction between move locks and WFS locks.
  generateCrossLockMoveTests(
      WFSLockName, createWFSWithCleanupFactory(wfsOptions));
  generateCrossLockWFSTests(
      'an ongoing move operation', createMoveWithCleanup, wfsMode);

  // Test interaction between remove locks and WFS locks.
  generateCrossLockRemoveTests(
      WFSLockName, createWFSWithCleanupFactory(wfsOptions));
  generateCrossLockWFSTests(
      'an ongoing remove operation', createRemoveWithCleanup, wfsMode);
}

done();
