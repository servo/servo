// META: global=window,worker
// META: title=IDBFactory.cmp()
// META: script=resources/support-promises.js

// Spec: https://w3c.github.io/IndexedDB/#dom-idbfactory-cmp
// Spec: http://w3c.github.io/IndexedDB/#key-construct

'use strict';

// Test cmp() with valid keys. These tests verify that cmp() returns the correct
// comparison value.
test(function() {
  let greater = indexedDB.cmp(2, 1);
  let equal = indexedDB.cmp(2, 2);
  let less = indexedDB.cmp(1, 2);

  assert_equals(greater, 1, 'greater');
  assert_equals(equal, 0, 'equal');
  assert_equals(less, -1, 'less');
}, 'IDBFactory.cmp() - compared keys return correct value');

// Test cmp() with invalid keys. These tests verify that cmp() throws an
// exception when given invalid input.
test(function() {
  assert_throws_js(TypeError, function() {
    indexedDB.cmp();
  });
}, 'IDBFactory.cmp() - no argument');

test(function() {
  assert_throws_dom('DataError', function() {
    indexedDB.cmp(null, null);
  });
  assert_throws_dom('DataError', function() {
    indexedDB.cmp(1, null);
  });
  assert_throws_dom('DataError', function() {
    indexedDB.cmp(null, 1);
  });
}, 'IDBFactory.cmp() - null');

test(function() {
  assert_throws_dom('DataError', function() {
    indexedDB.cmp(NaN, NaN);
  });
  assert_throws_dom('DataError', function() {
    indexedDB.cmp(1, NaN);
  });
  assert_throws_dom('DataError', function() {
    indexedDB.cmp(NaN, 1);
  });
}, 'IDBFactory.cmp() - NaN');

// Test cmp() with keys of different types. These tests verify that cmp()
// correctly compares keys of different types.
test(function() {
  assert_equals(indexedDB.cmp([0], new Uint8Array([0])), 1, 'Array > Binary');
}, 'Array vs. Binary');

test(function() {
  assert_equals(indexedDB.cmp(new Uint8Array([0]), '0'), 1, 'Binary > String');
}, 'Binary vs. String');

test(function() {
  assert_equals(indexedDB.cmp('', new Date(0)), 1, 'String > Date');
}, 'String vs. Date');

test(function() {
  assert_equals(indexedDB.cmp(new Date(0), 0), 1, 'Date > Number');
}, 'Date vs. Number');

// Test cmp() with binary keys. These tests verify that cmp() correctly compares
// binary keys.
test(function() {
  assert_equals(
      indexedDB.cmp(new Int8Array([-1]), new Uint8Array([0])), 1,
      '255(-1) shall be larger than 0');
}, 'Compare in unsigned octet values (in the range [0, 255])');

test(function() {
  assert_equals(
      indexedDB.cmp(
          new Uint8Array([255, 254, 253]), new Uint8Array([255, 253, 254])),
      1, '[255, 254, 253] shall be larger than [255, 253, 254]');
}, 'Compare values of the same length');

test(function() {
  assert_equals(
      indexedDB.cmp(
          new Uint8Array([255, 254]), new Uint8Array([255, 253, 254])),
      1, '[255, 254] shall be larger than [255, 253, 254]');
}, 'Compare values of different lengths');

test(function() {
  assert_equals(
      indexedDB.cmp(
          new Uint8Array([255, 253, 254]), new Uint8Array([255, 253])),
      1, '[255, 253, 254] shall be larger than [255, 253]');
}, 'Compare when values in the range of their minimal length are the same');
