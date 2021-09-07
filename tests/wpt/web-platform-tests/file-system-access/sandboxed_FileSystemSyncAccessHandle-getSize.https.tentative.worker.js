importScripts("/resources/testharness.js");
importScripts('resources/sync-access-handle-test.js');

'use strict';

sync_access_handle_test(async (testCase, handle) => {
  assert_equals(await handle.getSize(), 0);
  const bufferSize = 4;
  const writeBuffer = new Uint8Array(bufferSize);
  writeBuffer.set([96, 97, 98, 99]);
  handle.write(writeBuffer, {at: 0});
  assert_equals(await handle.getSize(), bufferSize);
  let offset = 3;
  handle.write(writeBuffer, {at: offset});
  assert_equals(await handle.getSize(), bufferSize + offset);
  offset = 10;
  handle.write(writeBuffer, {at: offset});
  assert_equals(await handle.getSize(), bufferSize + offset);
}, 'test SyncAccessHandle.getSize after SyncAccessHandle.write');

sync_access_handle_test(async (testCase, handle) => {
  const getSizePromise = handle.getSize();
  await promise_rejects_dom(testCase, 'InvalidStateError', handle.getSize());
  assert_equals(await getSizePromise, 0);
}, 'test createSyncAccessHandle.getSize with pending operation');
done();
