importScripts('/resources/testharness.js');
importScripts('resources/sandboxed-fs-test-helpers.js');
importScripts('resources/test-helpers.js');

'use strict';

const SAH_MODES = ['readwrite', 'read-only', 'readwrite-unsafe'];

const LOCK_ACCESS = {
  SHARED: 'shared',
  EXCLUSIVE: 'exclusive',
};
const LOCK_WRITE_PERMISSION = {
  NOT_WRITABLE: 'not writable',
  WRITABLE: 'writable',
};

async function testLockAccess(t, fileHandle, sahMode) {
  const syncHandle1 = await fileHandle.createSyncAccessHandle({mode: sahMode});
  t.add_cleanup(() => syncHandle1.close());

  let access;
  try {
    const syncHandle2 =
        await fileHandle.createSyncAccessHandle({mode: sahMode});
    syncHandle2.close();
    access = LOCK_ACCESS.SHARED;
  } catch (e) {
    access = LOCK_ACCESS.EXCLUSIVE;
    assert_throws_dom('NoModificationAllowedError', () => {
      throw e;
    });
  }
  syncHandle1.close();

  // Can open another sync access handle after other handles have closed.
  const syncHandle3 = await fileHandle.createSyncAccessHandle({mode: sahMode});
  syncHandle3.close();

  return access;
}

async function testLockWritePermission(t, fileHandle, sahMode) {
  const syncHandle = await fileHandle.createSyncAccessHandle({mode: sahMode});
  t.add_cleanup(() => syncHandle.close());

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
  const sahOptions = {mode: sahMode};

  directory_test(async (t, rootDir) => {
    const fileHandle =
        await rootDir.getFileHandle('OPFS.test', {create: true});

    const syncHandle = await fileHandle.createSyncAccessHandle(sahOptions);
    const {mode} = syncHandle;
    syncHandle.close();
    assert_equals(mode, sahMode);
  }, `An access handle in ${sahMode} mode has a mode property equal to` +
  ` ${sahMode}`);

  directory_test(async (t, rootDir) => {
    const fileHandle = await rootDir.getFileHandle('OPFS.test', {create: true});
    assert_equals(
        await testLockAccess(t, fileHandle, sahMode), expectedLockAccess);
  }, `${sahMode} mode takes a ${expectedLockAccess}`);

  directory_test(async (t, rootDir) => {
    const fileHandle = await rootDir.getFileHandle('OPFS.test', {create: true});
    assert_equals(
        await testLockWritePermission(t, fileHandle, sahMode),
        expectedLockWritePermission);
  }, `${sahMode} mode is ${expectedLockWritePermission}`);

  directory_test(async (t, rootDir) => {
    const fileHandle =
        await rootDir.getFileHandle('OPFS.test', {create: true});

    const syncHandle = await fileHandle.createSyncAccessHandle(sahOptions);
    t.add_cleanup(() => syncHandle.close());
    for (const mode of SAH_MODES) {
      if (sahMode !== mode) {
        await promise_rejects_dom(
            t, 'NoModificationAllowedError',
            fileHandle.createSyncAccessHandle({mode: mode}));
      }
    }
  }, `When there's an open access handle in ${sahMode} mode on a file, cannot` +
  ` open another access handle of another mode on that same file`);

  directory_test(async (t, rootDir) => {
    const fooFileHandle =
        await rootDir.getFileHandle('foo.test', {create: true});
    const barFileHandle =
        await rootDir.getFileHandle('bar.test', {create: true});

    const fooSyncHandle =
        await fooFileHandle.createSyncAccessHandle(sahOptions);
    t.add_cleanup(() => fooSyncHandle.close());

    for (const mode of SAH_MODES) {
      const barSyncHandle =
          await barFileHandle.createSyncAccessHandle({mode});
      barSyncHandle.close();
    }
  }, `When there's an open access handle in ${sahMode} mode on a file, can` +
  ` open another access handle of any mode on a different file`);

  directory_test(async (t, rootDir) => {
    const fileHandle =
        await rootDir.getFileHandle('OPFS.test', {create: true});

    const syncHandle1 = await fileHandle.createSyncAccessHandle(sahOptions);
    syncHandle1.close();

    for (const mode of SAH_MODES) {
      const syncHandle2 = await fileHandle.createSyncAccessHandle({mode});
      syncHandle2.close();
    }
  }, `After an access handle in ${sahMode} mode on a file has been closed,` +
  ` can open another access handle of any mode on the same file`);
}

directory_test(async (t, rootDir) => {
  const fileHandle = await rootDir.getFileHandle('OPFS.test', {create: true});

  const syncHandle = await fileHandle.createSyncAccessHandle();
  t.add_cleanup(() => syncHandle.close());
  assert_equals(syncHandle.mode, 'readwrite');
}, 'A sync access handle opens in readwrite mode by default');

lockPropertyTests(
    'readwrite', LOCK_ACCESS.EXCLUSIVE, LOCK_WRITE_PERMISSION.WRITABLE);
lockPropertyTests(
    'read-only', LOCK_ACCESS.SHARED, LOCK_WRITE_PERMISSION.NOT_WRITABLE);
lockPropertyTests(
    'readwrite-unsafe', LOCK_ACCESS.SHARED, LOCK_WRITE_PERMISSION.WRITABLE);

done();
