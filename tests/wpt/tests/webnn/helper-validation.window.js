// META: title=Validate WebNN test helpers
// META: script=resources/utils.js

// This doesn't validate the WebNN API itself, it just verifies behavior of
// non-trivial helper functions.

'use strict';

// Largest integer uniquely representable as a float32.
const MAX_FLOAT32_INTEGER = 2 ** 24;

test(t => {
  const dataType = 'float32';
  [[0.0, 0.0, 0n],
   [0.0, 1e-46, 0n],
   [0.0, 1e-36, 61482021n],
   [1.0, 1.0, 0n],
   [1.0, -1.0, 2130706432n],
   [1.0, 2.0, 8388608n],
   [1.000001, 1.000002, 9n],
   [1.0000001, 1.0000002, 1n],
   [-1.000001, 1.000002, 2130706457n],
   [-1.0000001, 1.0000002, 2130706435n],
   [0.0, 1.401298464324817e-45, 1n],
   [99.28312683105469, 39.03501892089844, 11169050n],
   [MAX_FLOAT32_INTEGER - 1, MAX_FLOAT32_INTEGER, 1n],
  ].forEach(([a, b, expected]) => {
    assert_equals(
        ulpDistance(a, b, dataType), expected,
        `ULP distance between ${a} and ${b}`);
    assert_equals(
        ulpDistance(b, a, dataType), expected,
        `ULP distance between ${b} and ${a} (commutative)`);
    assert_equals(
        ulpDistance(-a, -b, dataType), expected,
        `ULP distance between ${- a} and ${- b} (negated)`);
  });
}, 'ULP Distance - float32');

// TODO: Add test cases for 'float16' data type.

test(t => {
  const dataType = 'int64';
  [[0n, 0n, 0n],
   [1n, 0n, 1n],
   [1n, 2n, 1n],
   [10n, 11n, 1n],
   [10n, 20n, 10n],
   [100000001n, 100000002n, 1n],
   [0x7FFFFFFFFFFFFFFEn, 0x7FFFFFFFFFFFFFFFn, 1n],
   [-0x7FFFFFFFFFFFFFFFn, 0x7FFFFFFFFFFFFFFFn, 0xFFFFFFFFFFFFFFFEn],
  ].forEach(([a, b, expected]) => {
    assert_equals(
        ulpDistance(a, b, dataType), expected,
        `ULP distance between ${a} and ${b}`);
    assert_equals(
        ulpDistance(b, a, dataType), expected,
        `ULP distance between ${b} and ${a} (commutative)`);
    assert_equals(
        ulpDistance(-a, -b, dataType), expected,
        `ULP distance between ${- a} and ${- b} (negated)`);
  });
  assert_equals(
      ulpDistance(-0x8000000000000000n, 0x7FFFFFFFFFFFFFFFn, dataType),
      0xFFFFFFFFFFFFFFFFn, 'ULP distance between min and max int64');
}, 'ULP Distance - int64');

test(t => {
  const dataType = 'uint64';
  [[0n, 0n, 0n],
   [1n, 0n, 1n],
   [1n, 2n, 1n],
   [10n, 11n, 1n],
   [10n, 20n, 10n],
   [100000001n, 100000002, 1n],
   [0xFFFFFFFFFFFFFFFEn, 0xFFFFFFFFFFFFFFFFn, 1n],
   [0n, 0xFFFFFFFFFFFFFFFFn, 0xFFFFFFFFFFFFFFFFn],
  ].forEach(([a, b, expected]) => {
    assert_equals(
        ulpDistance(a, b, dataType), expected,
        `ULP distance between ${a} and ${b}`);
    assert_equals(
        ulpDistance(b, a, dataType), expected,
        `ULP distance between ${b} and ${a} (commutative)`);
  });
}, 'ULP Distance - uint64');

test(t => {
  const dataType = 'int32';
  [[0, 0, 0],
   [1, 0, 1],
   [1, 2, 1],
   [10, 11, 1],
   [10, 20, 10],
   [100000001, 100000002, 1],
   [0x7FFFFFFE, 0x7FFFFFFF, 1],
   [-0x7FFFFFFF, 0x7FFFFFFF, 0xFFFFFFFE],
  ].forEach(([a, b, expected]) => {
    assert_equals(
        ulpDistance(a, b, dataType), expected,
        `ULP distance between ${a} and ${b}`);
    assert_equals(
        ulpDistance(b, a, dataType), expected,
        `ULP distance between ${b} and ${a} (commutative)`);
    assert_equals(
        ulpDistance(-a, -b, dataType), expected,
        `ULP distance between ${- a} and ${- b} (negated)`);
  });
  assert_equals(
      ulpDistance(-0x80000000, 0x7FFFFFFF, dataType), 0xFFFFFFFF,
      'ULP distance between min and max int32');
}, 'ULP Distance - int32');

test(t => {
  const dataType = 'uint32';
  [[0, 0, 0],
   [1, 0, 1],
   [1, 2, 1],
   [10, 11, 1],
   [10, 20, 10],
   [100000001, 100000002, 1],
   [0xFFFFFFFE, 0xFFFFFFFF, 1],
   [0, 0xFFFFFFFF, 0xFFFFFFFF],
  ].forEach(([a, b, expected]) => {
    assert_equals(
        ulpDistance(a, b, dataType), expected,
        `ULP distance between ${a} and ${b}`);
    assert_equals(
        ulpDistance(b, a, dataType), expected,
        `ULP distance between ${b} and ${a} (commutative)`);
  });
}, 'ULP Distance - uint32');

test(t => {
  const dataType = 'int8';
  [[0, 0, 0],
   [1, 0, 1],
   [1, 2, 1],
   [10, 11, 1],
   [10, 20, 10],
   [101, 102, 1],
   [0x7E, 0x7F, 1],
   [-0x7F, 0x7F, 0xFE],
  ].forEach(([a, b, expected]) => {
    assert_equals(
        ulpDistance(a, b, dataType), expected,
        `ULP distance between ${a} and ${b}`);
    assert_equals(
        ulpDistance(b, a, dataType), expected,
        `ULP distance between ${b} and ${a} (commutative)`);
    assert_equals(
        ulpDistance(-a, -b, dataType), expected,
        `ULP distance between ${- a} and ${- b} (negated)`);
  });
  assert_equals(
      ulpDistance(-0x80, 0x7F, dataType), 0xFF,
      'ULP distance between min and max int8');
}, 'ULP Distance - int8');

test(t => {
  const dataType = 'uint8';
  [[0, 0, 0],
   [1, 0, 1],
   [1, 2, 1],
   [10, 11, 1],
   [10, 20, 10],
   [101, 102, 1],
   [0xFE, 0xFF, 1],
   [0, 0xFF, 0xFF],
  ].forEach(([a, b, expected]) => {
    assert_equals(
        ulpDistance(a, b, dataType), expected,
        `ULP distance between ${a} and ${b}`);
    assert_equals(
        ulpDistance(b, a, dataType), expected,
        `ULP distance between ${b} and ${a} (commutative)`);
  });
}, 'ULP Distance - uint8');

test(t => {
  const dataType = 'int4';
  [[0, 0, 0],
   [1, 0, 1],
   [1, 2, 1],
   [10, 11, 1],
   [1, 10, 9],
   [6, 7, 1],
   [-7, 7, 14],
  ].forEach(([a, b, expected]) => {
    assert_equals(
        ulpDistance(a, b, dataType), expected,
        `ULP distance between ${a} and ${b}`);
    assert_equals(
        ulpDistance(b, a, dataType), expected,
        `ULP distance between ${b} and ${a} (commutative)`);
    assert_equals(
        ulpDistance(-a, -b, dataType), expected,
        `ULP distance between ${- a} and ${- b} (negated)`);
  });
  assert_equals(
      ulpDistance(-0x8, 0x7, dataType), 0xF,
      'ULP distance between min and max int4');
}, 'ULP Distance - int4');

test(t => {
  const dataType = 'uint4';
  [[0, 0, 0],
   [1, 0, 1],
   [1, 2, 1],
   [10, 11, 1],
   [1, 10, 9],
   [0xE, 0xF, 1],
   [0, 0xF, 0xF],
  ].forEach(([a, b, expected]) => {
    assert_equals(
        ulpDistance(a, b, dataType), expected,
        `ULP distance between ${a} and ${b}`);
    assert_equals(
        ulpDistance(b, a, dataType), expected,
        `ULP distance between ${b} and ${a} (commutative)`);
  });
}, 'ULP Distance - uint4');
