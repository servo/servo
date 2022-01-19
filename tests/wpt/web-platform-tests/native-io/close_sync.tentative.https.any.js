// META: title=Synchronous NativeIO API: close().
// META: global=dedicatedworker
// META: script=resources/support.js

'use strict';

test(testCase => {
  reserveAndCleanupCapacitySync(testCase);

  const file = createFileSync(testCase, 'file_name');
  assert_equals(undefined, file.close());

  assert_equals(undefined, file.close());
}, 'NativeIOFileSync.close is idempotent');

test(testCase => {
  reserveAndCleanupCapacitySync(testCase);

  const file = createFileSync(testCase, 'file_name');
  assert_equals(undefined, file.close());

  const readBuffer = new Uint8Array(4);
  assert_throws_dom('InvalidStateError', () => file.read(readBuffer, 4));
}, 'NativeIOFileSync.read fails after NativeIOFileSync.close');

test(testCase => {
  reserveAndCleanupCapacitySync(testCase);

  const file = createFileSync(testCase, 'file_name');
  assert_equals(undefined, file.close());

  const writeBuffer = Uint8Array.from([96, 97, 98, 99]);
  assert_throws_dom('InvalidStateError', () => file.write(writeBuffer, 4));
}, 'NativeIOFile.write fails after NativeIOFile.close');

test(testCase => {
  reserveAndCleanupCapacitySync(testCase);

  const file = createFileSync(testCase, 'file_name');
  assert_equals(undefined, file.close());

  assert_throws_dom('InvalidStateError', () => file.getLength());
}, 'NativeIOFileSync.getLength fails after NativeIOFileSync.close');

test(testCase => {
  reserveAndCleanupCapacitySync(testCase);

  const file = createFileSync(testCase, 'file_name');
  assert_equals(undefined, file.close());

  assert_throws_dom('InvalidStateError', () => file.flush());
}, 'NativeIOFileSync.flush fails after NativeIOFileSync.close');

test(testCase => {
  reserveAndCleanupCapacitySync(testCase);

  const file = createFileSync(testCase, 'file_name');
  assert_equals(undefined, file.close());

  assert_throws_dom('InvalidStateError', () => file.setLength(4));
}, 'NativeIOFileSync.setLength fails after NativeIOFileSync.close');
