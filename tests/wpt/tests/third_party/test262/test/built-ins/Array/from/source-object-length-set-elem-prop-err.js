// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.from
description: >
  TypeError is thrown if CreateDataProperty fails.
  (items is not iterable)
info: |
  Array.from ( items [ , mapfn [ , thisArg ] ] )

  [...]
  4. Let usingIterator be ? GetMethod(items, @@iterator).
  5. If usingIterator is not undefined, then
    [...]
  6. NOTE: items is not an Iterable so assume it is an array-like object.
  [...]
  12. Repeat, while k < len
    [...]
    e. Perform ? CreateDataPropertyOrThrow(A, Pk, mappedValue).
  [...]

  CreateDataPropertyOrThrow ( O, P, V )

  [...]
  3. Let success be ? CreateDataProperty(O, P, V).
  4. If success is false, throw a TypeError exception.
---*/

var items = {
  length: 1,
};

var A1 = function(_length) {
  this.length = 0;
  Object.preventExtensions(this);
};

assert.throws(TypeError, function() {
  Array.from.call(A1, items);
}, 'Array.from.call(A1, items) throws a TypeError exception');

var A2 = function(_length) {
  Object.defineProperty(this, "0", {
    writable: true,
    configurable: false,
  });
};

assert.throws(TypeError, function() {
  Array.from.call(A2, items);
}, 'Array.from.call(A2, items) throws a TypeError exception');
