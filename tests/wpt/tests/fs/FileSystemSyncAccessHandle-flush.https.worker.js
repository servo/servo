importScripts('/resources/testharness.js');
importScripts('resources/sync-access-handle-test.js');

'use strict';

sync_access_handle_test((t, handle) => {
  handle.flush();
}, 'Test flush on an empty file.');

sync_access_handle_test((t, handle) => {
  if (!('TextEncoder' in self)) {
    return;
  }
  const encoder = new TextEncoder();
  const decoder = new TextDecoder();

  const text = 'Hello Storage Foundation';
  const writeBuffer = new TextEncoder().encode(text);
  handle.write(writeBuffer, {at: 0});
  handle.flush();
  let readBuffer = new Uint8Array(text.length);
  handle.read(readBuffer, {at: 0});
  assert_equals(
      text, new TextDecoder().decode(readBuffer),
      'Check that the written bytes and the read bytes match');
}, 'SyncAccessHandle.read returns bytes written by SyncAccessHandle.write' +
    ' after SyncAccessHandle.flush');

done();
