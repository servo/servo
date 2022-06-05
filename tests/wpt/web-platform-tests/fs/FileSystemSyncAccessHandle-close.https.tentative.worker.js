importScripts("/resources/testharness.js");
importScripts('resources/sync-access-handle-test.js');

'use strict';

sync_access_handle_test(async (testCase, handle) => {
  assert_equals(await handle.close(), undefined);

  assert_equals(await handle.close(), undefined);
}, 'SyncAccessHandle.close is idempotent');

sync_access_handle_test(async (testCase, handle) => {
  const closePromise = handle.close();

  assert_equals(await handle.close(), undefined);
  assert_equals(await closePromise, undefined);
}, 'SyncAccessHandle.close is idempotent when called immediately');

sync_access_handle_test(async (testCase, handle) => {
  assert_equals(await handle.close(), undefined);

  const readBuffer = new Uint8Array(4);
  assert_throws_dom('InvalidStateError', () => handle.read(readBuffer, {at: 0}));
}, 'SyncAccessHandle.read fails after SyncAccessHandle.close settles');

sync_access_handle_test(async (testCase, handle) => {
  const closePromise = handle.close();

  const readBuffer = new Uint8Array(4);
  assert_throws_dom('InvalidStateError', () => handle.read(readBuffer, {at: 0}));
  assert_equals(await closePromise, undefined);
}, 'SyncAccessHandle.read fails immediately after calling SyncAccessHandle.close');

sync_access_handle_test(async (testCase, handle) => {
  assert_equals(await handle.close(), undefined);

  const writeBuffer = new Uint8Array(4);
  writeBuffer.set([96, 97, 98, 99]);
  assert_throws_dom('InvalidStateError', () => handle.write(writeBuffer, {at: 0}));
}, 'SyncAccessHandle.write fails after SyncAccessHandle.close settles');

sync_access_handle_test(async (testCase, handle) => {
  const closePromise = handle.close();

  const writeBuffer = new Uint8Array(4);
  writeBuffer.set([96, 97, 98, 99]);
  assert_throws_dom('InvalidStateError', () => handle.write(writeBuffer, {at: 0}));
  assert_equals(await closePromise, undefined);
}, 'SyncAccessHandle.write fails immediately after calling SyncAccessHandle.close');

sync_access_handle_test(async (testCase, handle) => {
  assert_equals(await handle.close(), undefined);

  await promise_rejects_dom(testCase, 'InvalidStateError', handle.flush());
}, 'SyncAccessHandle.flush fails after SyncAccessHandle.close settles');

sync_access_handle_test(async (testCase, handle) => {
  const closePromise = handle.close();

  await promise_rejects_dom(testCase, 'InvalidStateError', handle.flush());
  assert_equals(await closePromise, undefined);
}, 'SyncAccessHandle.flush fails immediately after calling SyncAccessHandle.close');

sync_access_handle_test(async (testCase, handle) => {
  assert_equals(await handle.close(), undefined);

  await promise_rejects_dom(testCase, 'InvalidStateError', handle.getSize());
}, 'SyncAccessHandle.getSize fails after SyncAccessHandle.close settles');

sync_access_handle_test(async (testCase, handle) => {
  const closePromise = handle.close();

  await promise_rejects_dom(testCase, 'InvalidStateError', handle.getSize());
  assert_equals(await closePromise, undefined);
}, 'SyncAccessHandle.getSize fails immediately after calling SyncAccessHandle.close');

sync_access_handle_test(async (testCase, handle) => {
  assert_equals(await handle.close(), undefined);

  await promise_rejects_dom(testCase, 'InvalidStateError', handle.truncate(4));
}, 'SyncAccessHandle.truncate fails after SyncAccessHandle.close settles');

sync_access_handle_test(async (testCase, handle) => {
  const closePromise = handle.close();

  await promise_rejects_dom(testCase, 'InvalidStateError', handle.truncate(4));
  assert_equals(await closePromise, undefined);
}, 'SyncAccessHandle.truncate fails immediately after calling SyncAccessHandle.close');

done();
