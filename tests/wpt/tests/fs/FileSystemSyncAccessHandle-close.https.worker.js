importScripts("/resources/testharness.js");
importScripts('resources/sync-access-handle-test.js');

'use strict';
sync_access_handle_test((t, handle) => {
  assert_equals(handle.close(), undefined);
  assert_equals(handle.close(), undefined);
}, 'SyncAccessHandle.close is idempotent');

sync_access_handle_test((t, handle) => {
  assert_equals(handle.close(), undefined);
  const readBuffer = new Uint8Array(4);
  assert_throws_dom(
      'InvalidStateError', () => handle.read(readBuffer, {at: 0}));
}, 'SyncAccessHandle.read fails after SyncAccessHandle.close');

sync_access_handle_test((t, handle) => {
  assert_equals(handle.close(), undefined);
  const writeBuffer = new Uint8Array(4);
  writeBuffer.set([96, 97, 98, 99]);
  assert_throws_dom(
      'InvalidStateError', () => handle.write(writeBuffer, {at: 0}));
}, 'SyncAccessHandle.write fails after SyncAccessHandle.close');

sync_access_handle_test((t, handle) => {
  assert_equals(handle.close(), undefined);
  assert_throws_dom('InvalidStateError', () => handle.flush());
}, 'SyncAccessHandle.flush fails after SyncAccessHandle.close');

sync_access_handle_test((t, handle) => {
  assert_equals(handle.close(), undefined);
  assert_throws_dom('InvalidStateError', () => handle.getSize());
}, 'SyncAccessHandle.getSize fails after SyncAccessHandle.close');

sync_access_handle_test((t, handle) => {
  assert_equals(handle.close(), undefined);
  assert_throws_dom('InvalidStateError', () => handle.truncate(4));
}, 'SyncAccessHandle.truncate fails after SyncAccessHandle.handle.close');

done();