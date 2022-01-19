importScripts('/resources/testharness.js');
importScripts('resources/sync-access-handle-test.js');

'use strict';

sync_access_handle_test(async (t, handle) => {
  await handle.flush();
}, 'Test flush on an empty file.');

sync_access_handle_test(async (t, handle) => {
  if (!('TextEncoder' in self)) {
    return;
  }
  const encoder = new TextEncoder();
  const decoder = new TextDecoder();

  const text = 'Hello Storage Foundation';
  const writeBuffer = new TextEncoder().encode(text);
  handle.write(writeBuffer, {at: 0});
  await handle.flush();
  let readBuffer = new Uint8Array(text.length);
  handle.read(readBuffer, {at: 0});
  assert_equals(
      text, new TextDecoder().decode(readBuffer),
      'Check that the written bytes and the read bytes match');
},
'SyncAccessHandle.read returns bytes written by SyncAccessHandle.write' +
    ' after SyncAccessHandle.flush');

sync_access_handle_test(async (testCase, handle) => {
  const flushPromise = handle.flush();
  const readBuffer = new Uint8Array(4);
  assert_throws_dom(
      'InvalidStateError', () => handle.read(readBuffer, {at: 0}));
  assert_equals(await flushPromise, undefined);
},
'SyncAccessHandle.read fails when there is a pending SyncAccessHandle.flush');

sync_access_handle_test(async (testCase, handle) => {
  const flushPromise = handle.flush();
  const writeBuffer = new Uint8Array(4);
  writeBuffer.set([96, 97, 98, 99]);
  assert_throws_dom(
      'InvalidStateError', () => handle.write(writeBuffer, {at: 0}));
  assert_equals(await flushPromise, undefined);
},
'SyncAccessHandle.write fails when there is a pending SyncAccessHandle.flush');

done();
