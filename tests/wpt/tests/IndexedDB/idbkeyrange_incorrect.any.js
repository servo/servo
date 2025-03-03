// META: title=IDBKeyRange Tests - Incorrect
// META: global=window,worker
// META: script=resources/support.js

// Spec: https://w3c.github.io/IndexedDB/#keyrange

'use strict';

// TypeError: bound requires more than 0 arguments
test(() => {
  assert_throws_js(TypeError, () => {
    IDBKeyRange.bound();
  });
}, 'IDBKeyRange.bound() - bound requires more than 0 arguments.');

// Null parameters
test(() => {
  assert_throws_dom('DataError', () => {
    IDBKeyRange.bound(null, null);
  });
}, 'IDBKeyRange.bound(null, null) - null parameters are incorrect.');

// Null parameter
test(() => {
  assert_throws_dom('DataError', () => {
    IDBKeyRange.bound(1, null);
  });
  assert_throws_dom('DataError', () => {
    IDBKeyRange.bound(null, 1);
  });
}, 'IDBKeyRange.bound(1, null / null, 1) - null parameter is incorrect.');

// bound incorrect
test(() => {
  const lowerBad = Math.floor(Math.random() * 31) + 5;
  const upper = lowerBad - 1;
  assert_throws_dom('DataError', () => {
    IDBKeyRange.bound(lowerBad, upper);
  });
  assert_throws_dom('DataError', () => {
    IDBKeyRange.bound('b', 'a');
  });
}, "IDBKeyRange.bound(lower, upper / lower > upper) - lower' is greater than 'upper'.");

test(() => {
  assert_throws_dom('DataError', () => {
    IDBKeyRange.bound('a', 1);
  });
  assert_throws_dom('DataError', () => {
    IDBKeyRange.bound(new Date(), 1);
  });
  assert_throws_dom('DataError', () => {
    IDBKeyRange.bound([1, 2], 1);
  });
}, "IDBKeyRange.bound(DOMString/Date/Array, 1) - A DOMString, Date and Array are greater than a float.");

// ReferenceError: the variable is not defined
test(() => {
  const goodVariable = 1;
  assert_throws_js(ReferenceError, () => {
    IDBKeyRange.bound(noExistingVariable, 1);
  });
  assert_throws_js(
      ReferenceError,
      () => {
        IDBKeyRange.bound(goodVariable, noExistingVariable);
      });
}, 'IDBKeyRange.bound(noExistingVariable, 1 / goodVariable, noExistingVariable) -\
    noExistingVariable is not defined.');

// Valid type key error
test(() => {
  assert_throws_dom('DataError', () => {
    IDBKeyRange.bound(true, 1);
  });
}, 'IDBKeyRange.bound(true, 1) - boolean is not a valid key type.');
