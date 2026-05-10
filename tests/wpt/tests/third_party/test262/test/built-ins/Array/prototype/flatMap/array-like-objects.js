// Copyright (C) 2018 Shilpi Jain and Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.flatmap
description: >
  array-like objects can be flattened
info: |
  Array.prototype.flatMap ( mapperFunction [ , thisArg ] )

  1. Let O be ? ToObject(this value).
  2. Let sourceLen be ? ToLength(? Get(O, "length")).
  ...
  5. Let A be ? ArraySpeciesCreate(O, 0).
  ...

  ArraySpeciesCreate ( originalArray, length )

  3. Let isArray be ? IsArray(originalArray).
  4. If isArray is false, return ? ArrayCreate(length).

  FlattenIntoArray(target, source, sourceLen, start, depth [ , mapperFunction, thisArg ])

  1. Let targetIndex be start.
  2. Let sourceIndex be 0.
  3. Repeat, while sourceIndex < sourceLen
    a. Let P be ! ToString(sourceIndex).
    b. Let exists be ? HasProperty(source, P).
    c. If exists is true, then
      ...
    ** Skip if property does not exist **
includes: [compareArray.js]
features: [Array.prototype.flatMap]
---*/

function fn(e) {
  return [39, e * 2]; // returns an array to observe it being flattened after
}

var a;
var actual;

a = {
  length: 3,
  0: 1,
  // property 1 will be fully skipped
  2: 21,
  get 3() { throw 'it should not get this property'; }
};
actual = [].flatMap.call(a, fn);
assert.compareArray(actual, [39, 2, 39, 42], 'The value of actual is expected to be [39, 2, 39, 42]');
assert.sameValue(Object.getPrototypeOf(actual), Array.prototype, 'Object.getPrototypeOf([].flatMap.call(a, fn)") returns Array.prototype');

a = {
  length: undefined,
  get 0() { throw 'it should not get this property'; },
};
actual = [].flatMap.call(a, fn);
assert.compareArray(actual, [], 'The value of actual is expected to be []');
assert.sameValue(Object.getPrototypeOf(actual), Array.prototype, 'Object.getPrototypeOf([].flatMap.call(a, fn)") returns Array.prototype');

var called = false;
a = {
  get length() {
    if (!called) {
      called = true;
      return 2;
    } else {
      throw 'is should get the length only once';
    }
  },
  0: 21,
  1: 19.5,
  get 2() { throw 'it should not get this property'; },
};
actual = [].flatMap.call(a, fn);
assert.compareArray(actual, [39, 42, 39, 39], 'The value of actual is expected to be [39, 42, 39, 39]');
assert.sameValue(Object.getPrototypeOf(actual), Array.prototype, 'Object.getPrototypeOf([].flatMap.call(a, fn)") returns Array.prototype');

a = {
  length: 10001,
  [10000]: 7,
};
actual = [].flatMap.call(a, fn);
assert.compareArray(actual, [39, 14], 'The value of actual is expected to be [39, 14]');
assert.sameValue(Object.getPrototypeOf(actual), Array.prototype, 'Object.getPrototypeOf([].flatMap.call(a, fn)") returns Array.prototype');
