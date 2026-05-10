// Copyright (C) 2018 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.flatmap
description: >
  Assert behavior if this value has a custom object constructor property species
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
  5. Let C be ? Get(originalArray, "constructor").
  6. If IsConstructor(C) is true, then
    a. Let thisRealm be the current Realm Record.
    b. Let realmC be ? GetFunctionRealm(C).
    c. If thisRealm and realmC are not the same Realm Record, then
      i. If SameValue(C, realmC.[[Intrinsics]].[[%Array%]]) is true, set C to undefined.
  7. If Type(C) is Object, then
    a. Set C to ? Get(C, @@species).
    b. If C is null, set C to undefined.
  8. If C is undefined, return ? ArrayCreate(length).
  9. If IsConstructor(C) is false, throw a TypeError exception.
  10. Return ? Construct(C, « length »).
features: [Array.prototype.flatMap, Symbol, Symbol.species]
includes: [compareArray.js]
---*/

assert.sameValue(
  typeof Array.prototype.flatMap,
  'function',
  'The value of `typeof Array.prototype.flatMap` is expected to be "function"'
);

var arr = [[42, 1], [42, 2]];
var mapperFn = function(e) { return e; };

arr.constructor = {};
var actual = arr.flatMap(mapperFn);
assert.compareArray(actual, [42, 1, 42, 2], 'The value of actual is expected to be [42, 1, 42, 2]');
assert.sameValue(
  Object.getPrototypeOf(actual),
  Array.prototype,
  'Object.getPrototypeOf(arr.flatMap(mapperFn)) returns Array.prototype'
);

var called = 0;
arr.constructor = {
  get [Symbol.species]() {
    called++;
    return 0;
  }
};
assert.throws(TypeError, function() {
  arr.flatMap(mapperFn);
}, 'arr.flatMap(mapperFn) throws a TypeError exception');
assert.sameValue(called, 1, 'The value of called is expected to be 1');

called = 0;
arr.constructor = {
  get [Symbol.species]() {
    called++;
    return '';
  }
};
assert.throws(TypeError, function() {
  arr.flatMap(mapperFn);
}, 'arr.flatMap(mapperFn) throws a TypeError exception');
assert.sameValue(called, 1, 'The value of called is expected to be 1');

called = 0;
arr.constructor = {
  get [Symbol.species]() {
    called++;
    return false;
  }
};
assert.throws(TypeError, function() {
  arr.flatMap(mapperFn);
}, 'arr.flatMap(mapperFn) throws a TypeError exception');
assert.sameValue(called, 1, 'The value of called is expected to be 1');

called = 0;
arr.constructor = {
  get [Symbol.species]() {
    called++;
    return {};
  }
};
assert.throws(TypeError, function() {
  arr.flatMap(mapperFn);
}, 'arr.flatMap(mapperFn) throws a TypeError exception');
assert.sameValue(called, 1, 'The value of called is expected to be 1');

called = 0;
arr.constructor = {
  get [Symbol.species]() {
    called++;
    return [];
  }
};
assert.throws(TypeError, function() {
  arr.flatMap(mapperFn);
}, 'arr.flatMap(mapperFn) throws a TypeError exception');
assert.sameValue(called, 1, 'The value of called is expected to be 1');

called = 0;
arr.constructor = {
  get [Symbol.species]() {
    called++;
    return Symbol();
  }
};
assert.throws(TypeError, function() {
  arr.flatMap(mapperFn);
}, 'arr.flatMap(mapperFn) throws a TypeError exception');
assert.sameValue(called, 1, 'The value of called is expected to be 1');

called = 0;
arr.constructor = {
  get [Symbol.species]() {
    called++;
    throw new Test262Error
  }
};
assert.throws(Test262Error, function() {
  arr.flatMap(mapperFn);
}, 'arr.flatMap(mapperFn) throws a Test262Error exception');
assert.sameValue(called, 1, 'The value of called is expected to be 1');
