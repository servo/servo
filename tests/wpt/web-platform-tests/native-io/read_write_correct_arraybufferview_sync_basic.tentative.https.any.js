// META: title=NativeIO API: Transferred buffer is of the same type as input.
// META: global=dedicatedworker
// META: script=resources/support.js

'use strict';

const kViewTypes = [
  Int8Array,
  Uint8Array,
  Uint8ClampedArray,
  Int16Array,
  Uint16Array,
  Int32Array,
  Uint32Array,
  Float32Array,
  Float64Array,
  BigInt64Array,
  BigUint64Array,
];

kViewTypes.forEach(view_type => {
  test(testCase => {
    reserveAndCleanupCapacitySync(testCase);
    const file = createFileSync(testCase, 'test_file');

    const {buffer} = file.write(new view_type(4), 0);

    assert_true(
        buffer instanceof view_type,
        `NativeIOFileSync.write() should return a ${view_type.name}`);
    assert_equals(
        buffer.length, 4,
        `NativeIOFileSync.write() should return a ${view_type.name} of the ` +
          `same length as the input`);

  }, `NativeIOFileSync.write returns a ${view_type.name} when given a ` +
       `${view_type.name}`);
});

test(testCase => {
  reserveAndCleanupCapacitySync(testCase);
  const file = createFileSync(testCase, 'test_file');

  const {buffer} = file.write(new DataView(new ArrayBuffer(4)), 0);

  assert_true(
      buffer instanceof DataView,
      'NativeIOFileSync.write() should return a DataView');
  assert_equals(
      buffer.byteLength, 4,
      'NativeIOFileSync.write() should return a DataView of the same ' +
        'byte length as the input');

}, 'NativeIOFileSync.write returns a DataView when given a DataView');

kViewTypes.forEach(view_type => {
  test(testCase => {
    reserveAndCleanupCapacitySync(testCase);
    const file = createFileSync(testCase, 'test_file');

    const {buffer} = file.read(new view_type(4), 0);

    assert_true(
        buffer instanceof view_type,
        `NativeIOFileSync.read() should return a ${view_type.name}`);
    assert_equals(
        buffer.length, 4,
        `NativeIOFileSync.read() should return a ${view_type.name} of the ` +
          `same length as the input`);

  }, `NativeIOFileSync.read returns a ${view_type.name} when given a ` +
       `${view_type.name}`);
});

test(testCase => {
  reserveAndCleanupCapacitySync(testCase);
  const file = createFileSync(testCase, 'test_file');

  const {buffer} = file.read(new DataView(new ArrayBuffer(4)), 0);

  assert_true(
      buffer instanceof DataView,
      'NativeIOFileSync.read() should return a DataView');
  assert_equals(
      buffer.byteLength, 4,
      'NativeIOFileSync.read() should return a DataView of the same ' +
        'byte length as the input');

}, 'NativeIOFileSync.read returns a DataView when given a DataView ' +
       'buffer');
