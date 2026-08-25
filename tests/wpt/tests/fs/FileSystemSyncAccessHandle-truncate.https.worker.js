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

sync_access_handle_test((t, handle) => {
  const writeBuffer = new Uint8Array(4);
  writeBuffer.set([96, 97, 98, 99]);
  handle.write(writeBuffer, {at: 0});

  // Moves cursor to 2
  handle.truncate(2);
  let readBuffer = new Uint8Array(256);
  assert_equals(handle.read(readBuffer), 0);

  writeBuffer.set([100, 101, 102, 103]);
  handle.write(writeBuffer);

  assert_equals(handle.read(readBuffer, {at: 0}), 6);
  let expected = new Uint8Array(256);
  expected.set([96, 97, 100, 101, 102, 103]);
  assert_array_equals(readBuffer, expected);

  // Resize the file to 10, expect that everything beyond the old size is '0'.
  handle.truncate(10); // file cursor should still be at 6
  // overwrite two bytes
  const writeBuffer2 = new Uint8Array(2);
  writeBuffer2.set([110, 111]);
  handle.write(writeBuffer2);
  expected = new Uint8Array(256);
  expected.set([96, 97, 100, 101, 102, 103, 110, 111, 0, 0]);
  assert_equals(handle.read(readBuffer, {at: 0}), 10);
  assert_array_equals(readBuffer, expected);
}, 'Test truncate effect on cursor');

done();
