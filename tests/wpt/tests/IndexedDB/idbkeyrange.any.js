// META: title=IDBKeyRange Tests
// META: global=window,worker
// META: script=resources/support.js

'use strict';

test(() => {
  const keyRange = IDBKeyRange.only(1);
  assert_true(
      keyRange instanceof IDBKeyRange, 'keyRange instanceof IDBKeyRange');
  assert_equals(keyRange.lower, 1, 'keyRange');
  assert_equals(keyRange.upper, 1, 'keyRange');
  assert_false(keyRange.lowerOpen, 'keyRange.lowerOpen');
  assert_false(keyRange.upperOpen, 'keyRange.upperOpen');
}, 'IDBKeyRange.only() - returns an IDBKeyRange and the properties are set correctly');

test(() => {
  assert_throws_dom('DataError', () => {
    IDBKeyRange.only(undefined);
  }, 'undefined is not a valid key');
  assert_throws_dom('DataError', () => {
    IDBKeyRange.only(null);
  }, 'null is not a valid key');
  assert_throws_dom('DataError', () => {
    IDBKeyRange.only({});
  }, 'Object is not a valid key');
  assert_throws_dom('DataError', () => {
    IDBKeyRange.only(Symbol());
  }, 'Symbol is not a valid key');
  assert_throws_dom('DataError', () => {
    IDBKeyRange.only(true);
  }, 'boolean is not a valid key');
  assert_throws_dom('DataError', () => {
    IDBKeyRange.only(() => {});
  }, 'function is not a valid key');
}, 'IDBKeyRange.only() - throws on invalid keys');

test(() => {
  const keyRange = IDBKeyRange.lowerBound(1, true);
  assert_true(
      keyRange instanceof IDBKeyRange, 'keyRange instanceof IDBKeyRange');
  assert_equals(keyRange.lower, 1, 'keyRange.lower');
  assert_equals(keyRange.upper, undefined, 'keyRange.upper');
  assert_true(keyRange.lowerOpen, 'keyRange.lowerOpen');
  assert_true(keyRange.upperOpen, 'keyRange.upperOpen');
}, 'IDBKeyRange.lowerBound() - returns an IDBKeyRange and the properties are set correctly');

test(() => {
  const keyRange = IDBKeyRange.lowerBound(1);
  assert_false(keyRange.lowerOpen, 'keyRange.lowerOpen');
}, 'IDBKeyRange.lowerBound() - \'open\' parameter has correct default set');

test(() => {
  assert_throws_dom('DataError', () => {
    IDBKeyRange.lowerBound(undefined);
  }, 'undefined is not a valid key');
  assert_throws_dom('DataError', () => {
    IDBKeyRange.lowerBound(null);
  }, 'null is not a valid key');
  assert_throws_dom('DataError', () => {
    IDBKeyRange.lowerBound({});
  }, 'Object is not a valid key');
  assert_throws_dom('DataError', () => {
    IDBKeyRange.lowerBound(Symbol());
  }, 'Symbol is not a valid key');
  assert_throws_dom('DataError', () => {
    IDBKeyRange.lowerBound(true);
  }, 'boolean is not a valid key');
  assert_throws_dom('DataError', () => {
    IDBKeyRange.lowerBound(() => {});
  }, 'function is not a valid key');
}, 'IDBKeyRange.lowerBound() - throws on invalid keys');

test(() => {
  const keyRange = IDBKeyRange.upperBound(1, true);
  assert_true(
      keyRange instanceof IDBKeyRange, 'keyRange instanceof IDBKeyRange');
  assert_equals(keyRange.lower, undefined, 'keyRange.lower');
  assert_equals(keyRange.upper, 1, 'keyRange.upper');
  assert_true(keyRange.lowerOpen, 'keyRange.lowerOpen');
  assert_true(keyRange.upperOpen, 'keyRange.upperOpen');
}, 'IDBKeyRange.upperBound() - returns an IDBKeyRange and the properties are set correctly');

test(() => {
  const keyRange = IDBKeyRange.upperBound(1);
  assert_false(keyRange.upperOpen, 'keyRange.upperOpen');
}, 'IDBKeyRange.upperBound() - \'open\' parameter has correct default set');

test(() => {
  assert_throws_dom('DataError', () => {
    IDBKeyRange.upperBound(undefined);
  }, 'undefined is not a valid key');
  assert_throws_dom('DataError', () => {
    IDBKeyRange.upperBound(null);
  }, 'null is not a valid key');
  assert_throws_dom('DataError', () => {
    IDBKeyRange.upperBound({});
  }, 'Object is not a valid key');
  assert_throws_dom('DataError', () => {
    IDBKeyRange.upperBound(Symbol());
  }, 'Symbol is not a valid key');
  assert_throws_dom('DataError', () => {
    IDBKeyRange.upperBound(true);
  }, 'boolean is not a valid key');
  assert_throws_dom('DataError', () => {
    IDBKeyRange.upperBound(() => {});
  }, 'function is not a valid key');
}, 'IDBKeyRange.upperBound() - throws on invalid keys');

test(() => {
  const keyRange = IDBKeyRange.bound(1, 2, true, true);
  assert_true(
      keyRange instanceof IDBKeyRange, 'keyRange instanceof IDBKeyRange');
  assert_equals(keyRange.lower, 1, 'keyRange');
  assert_equals(keyRange.upper, 2, 'keyRange');
  assert_true(keyRange.lowerOpen, 'keyRange.lowerOpen');
  assert_true(keyRange.upperOpen, 'keyRange.upperOpen');
}, 'IDBKeyRange.bound() - returns an IDBKeyRange and the properties are set correctly');

test(() => {
  const keyRange = IDBKeyRange.bound(1, 2);
  assert_false(keyRange.lowerOpen, 'keyRange.lowerOpen');
  assert_false(keyRange.upperOpen, 'keyRange.upperOpen');
}, 'IDBKeyRange.bound() - \'lowerOpen\' and \'upperOpen\' parameters have correct defaults set');
