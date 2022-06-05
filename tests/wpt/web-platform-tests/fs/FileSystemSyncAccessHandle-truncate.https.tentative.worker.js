importScripts("/resources/testharness.js");
importScripts('resources/sync-access-handle-test.js');

'use strict';

sync_access_handle_test(async (testCase, handle) => {
  const getSizePromise = handle.getSize();
  await promise_rejects_dom(testCase, 'InvalidStateError', handle.truncate(4));
  assert_equals(await getSizePromise, 0);
}, 'test createSyncAccessHandle.truncate with pending operation');

sync_access_handle_test(async (testCase, handle) => {
  await handle.truncate(4);
  assert_equals(await handle.getSize(), 4);

  await handle.truncate(2);
  assert_equals(await handle.getSize(), 2);

  await handle.truncate(7);
  assert_equals(await handle.getSize(), 7);

  await promise_rejects_js(testCase, TypeError, handle.truncate(-4));
}, 'test SyncAccessHandle.truncate with different sizes');

sync_access_handle_test(async (testCase, handle) => {
  const writeBuffer = new Uint8Array(4);
  writeBuffer.set([96, 97, 98, 99]);
  handle.write(writeBuffer, {at: 0});

  await handle.truncate(2);
  let readBuffer = new Uint8Array(6);
  assert_equals(2, handle.read(readBuffer, {at: 0}));
  let expected = new Uint8Array(6);
  expected.set([96, 97, 0, 0, 0, 0]);
  assert_array_equals(expected, readBuffer);

  // Resize the file to 6, expect that everything beyond the old size is '0'.
  await handle.truncate(6);
  assert_equals(6, handle.read(readBuffer, {at: 0}));
  assert_array_equals(expected, readBuffer);
}, 'test SyncAccessHandle.truncate after SyncAccessHandle.write');
done();
