importScripts("/resources/testharness.js");
importScripts('resources/sync-access-handle-test.js');

'use strict';

sync_access_handle_test((t, handle) => {
  assert_equals(handle.getSize(), 0);
  const bufferSize = 4;
  const writeBuffer = new Uint8Array(bufferSize);
  writeBuffer.set([96, 97, 98, 99]);
  handle.write(writeBuffer, {at: 0});
  assert_equals(handle.getSize(), bufferSize);
  let offset = 3;
  handle.write(writeBuffer, {at: offset});
  assert_equals(handle.getSize(), bufferSize + offset);
  offset = 10;
  handle.write(writeBuffer, {at: offset});
  assert_equals(handle.getSize(), bufferSize + offset);
}, 'test SyncAccessHandle.getSize after SyncAccessHandle.write');

done();
