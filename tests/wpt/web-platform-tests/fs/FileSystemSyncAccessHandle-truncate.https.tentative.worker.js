importScripts("/resources/testharness.js");
importScripts('resources/sync-access-handle-test.js');

'use strict';

sync_access_handle_test((t, handle) => {
  // Without this assertion, the test passes even if truncate is not defined.
  assert_implements(handle.truncate,
    "SyncAccessHandle.truncate is not implemented.");

  handle.truncate(4);
  assert_equals(handle.getSize(), 4);
  handle.truncate(2);
  assert_equals(handle.getSize(), 2);
  handle.truncate(7);
  assert_equals(handle.getSize(), 7);
  handle.truncate(0);
  assert_equals(handle.getSize(), 0);
  assert_throws_js(TypeError, () => handle.truncate(-4));
}, 'test SyncAccessHandle.truncate with different sizes');

sync_access_handle_test((t, handle) => {
  const writeBuffer = new Uint8Array(4);
  writeBuffer.set([96, 97, 98, 99]);
  handle.write(writeBuffer, {at: 0});

  handle.truncate(2);
  let readBuffer = new Uint8Array(6);
  assert_equals(2, handle.read(readBuffer, {at: 0}));
  let expected = new Uint8Array(6);
  expected.set([96, 97, 0, 0, 0, 0]);
  assert_array_equals(expected, readBuffer);

  // Resize the file to 6, expect that everything beyond the old size is '0'.
  handle.truncate(6);
  assert_equals(6, handle.read(readBuffer, {at: 0}));
  assert_array_equals(expected, readBuffer);
}, 'test SyncAccessHandle.truncate after SyncAccessHandle.write');

done();
