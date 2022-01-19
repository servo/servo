// META: title=NativeIO API: close().
// META: global=window,worker
// META: script=resources/support.js
// META: timeout=long

'use strict';

promise_test(async testCase => {
  await reserveAndCleanupCapacity(testCase);

  const file = await createFile(testCase, 'file_name');
  assert_equals(await file.close(), undefined);

  assert_equals(await file.close(), undefined);
}, 'NativeIOFile.close is idempotent');

promise_test(async testCase => {
  await reserveAndCleanupCapacity(testCase);

  const file = await createFile(testCase, 'file_name');
  const closePromise = file.close();

  assert_equals(await file.close(), undefined);
  assert_equals(await closePromise, undefined);
}, 'NativeIOFile.close is idempotent when called immediately');

promise_test(async testCase => {
  await reserveAndCleanupCapacity(testCase);

  const file = await createFile(testCase, 'file_name');
  assert_equals(await file.close(), undefined);

  const readBuffer = new Uint8Array(4);
  await promise_rejects_dom(testCase, 'InvalidStateError',
                            file.read(readBuffer, 4));
}, 'NativeIOFile.read fails after NativeIOFile.close settles');

promise_test(async testCase => {
  await reserveAndCleanupCapacity(testCase);

  const file = await createFile(testCase, 'file_name');
  const closePromise = file.close();

  const readBuffer = new Uint8Array(4);
  await promise_rejects_dom(testCase, 'InvalidStateError',
                            file.read(readBuffer, 4));
  assert_equals(await closePromise, undefined);
}, 'NativeIOFile.read fails immediately after calling NativeIOFile.close');

promise_test(async testCase => {
  await reserveAndCleanupCapacity(testCase);

  const file = await createFile(testCase, 'file_name');
  assert_equals(await file.close(), undefined);

  const writeBuffer = new Uint8Array(4);
  writeBuffer.set([96, 97, 98, 99]);
  await promise_rejects_dom(testCase, 'InvalidStateError',
                            file.write(writeBuffer, 4));
}, 'NativeIOFile.write fails after NativeIOFile.close settles');

promise_test(async testCase => {
  await reserveAndCleanupCapacity(testCase);

  const file = await createFile(testCase, 'file_name');
  const closePromise = file.close();

  const writeBuffer = new Uint8Array(4);
  writeBuffer.set([96, 97, 98, 99]);
  await promise_rejects_dom(testCase, 'InvalidStateError',
                            file.write(writeBuffer, 4));
  assert_equals(await closePromise, undefined);
}, 'NativeIOFile.write fails immediately after calling NativeIOFile.close');

promise_test(async testCase => {
  await reserveAndCleanupCapacity(testCase);

  const file = await createFile(testCase, 'file_name');
  assert_equals(await file.close(), undefined);

  await promise_rejects_dom(testCase, 'InvalidStateError', file.getLength());
}, 'NativeIOFile.getLength fails after NativeIOFile.close settles');

promise_test(async testCase => {
  await reserveAndCleanupCapacity(testCase);

  const file = await createFile(testCase, 'file_name');
  const closePromise = file.close();

  await promise_rejects_dom(testCase, 'InvalidStateError', file.getLength());
  assert_equals(await closePromise, undefined);
}, 'NativeIOFile.getLength fails immediately after calling NativeIOFile.close');

promise_test(async testCase => {
  await reserveAndCleanupCapacity(testCase);

  const file = await createFile(testCase, 'file_name');
  assert_equals(await file.close(), undefined);

  await promise_rejects_dom(testCase, 'InvalidStateError', file.flush());
}, 'NativeIOFile.flush fails after NativeIOFile.close settles');

promise_test(async testCase => {
  await reserveAndCleanupCapacity(testCase);

  const file = await createFile(testCase, 'file_name');
  const closePromise = file.close();

  await promise_rejects_dom(testCase, 'InvalidStateError', file.flush());
  assert_equals(await closePromise, undefined);
}, 'NativeIOFile.flush fails immediately after calling NativeIOFile.close');

promise_test(async testCase => {
  await reserveAndCleanupCapacity(testCase);

  const file = await createFile(testCase, 'file_name');
  assert_equals(await file.close(), undefined);

  await promise_rejects_dom(testCase, 'InvalidStateError', file.setLength(5));
}, 'NativeIOFile.setLength fails after NativeIOFile.close settles');

promise_test(async testCase => {
  await reserveAndCleanupCapacity(testCase);

  const file = await createFile(testCase, 'file_name');
  const closePromise = file.close();

  await promise_rejects_dom(testCase, 'InvalidStateError', file.setLength(5));
  assert_equals(await closePromise, undefined);
}, 'NativeIOFile.setLength fails immediately after calling NativeIOFile.close');
