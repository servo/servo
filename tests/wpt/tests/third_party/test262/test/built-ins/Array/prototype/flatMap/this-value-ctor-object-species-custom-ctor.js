// Copyright (C) 2018 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.flatmap
description: >
  Assert behavior if this value has a custom species constructor
info: |
  Array.prototype.flatMap ( mapperFunction [ , thisArg ] )

  1. Let O be ? ToObject(this value).
  2. Let sourceLen be ? ToLength(? Get(O, "length")).
  ...
  5. Let A be ? ArraySpeciesCreate(O, 0).
  6. Perform ? FlattenIntoArray(A, O, sourceLen, 0, 1, mapperFunction, T).
  7. Return A.

  ArraySpeciesCreate ( originalArray, length )

  3. Let isArray be ? IsArray(originalArray).
  4. If isArray is false, return ? ArrayCreate(length).
  5. Let C be ? Get(originalArray, "constructor").
  6. If IsConstructor(C) is true, then
    ...
  7. If Type(C) is Object, then
    a. Set C to ? Get(C, @@species).
    b. If C is null, set C to undefined.
  8. If C is undefined, return ? ArrayCreate(length).
  9. If IsConstructor(C) is false, throw a TypeError exception.
  10. Return ? Construct(C, « length »).

  FlattenIntoArray(target, source, sourceLen, start, depth [ , mapperFunction, thisArg ])

  3. Repeat, while sourceIndex < sourceLen
    a. Let P be ! ToString(sourceIndex).
    b. Let exists be ? HasProperty(source, P).
    c. If exists is true, then
      ...
      vi. Else,
        ...
        2. Perform ? CreateDataPropertyOrThrow(target, ! ToString(targetIndex), element).
features: [Array.prototype.flatMap, Symbol, Symbol.species]
includes: [propertyHelper.js]
---*/

assert.sameValue(typeof Array.prototype.flatMap, 'function');

var arr = [[42, 1], [42, 2]];
var mapperFn = function(e) { return e; };

var called = 0;
var ctorCalled = 0;
function ctor(len) {
  assert.sameValue(new.target, ctor, 'new target is defined');
  assert.sameValue(len, 0, 'first argument is always 0');
  ctorCalled++;
}

arr.constructor = {
  get [Symbol.species]() {
    called++;
    return ctor;
  }
};
var actual = arr.flatMap(mapperFn);
assert(actual instanceof ctor, 'returned value is an instance of custom ctor');
assert.sameValue(called, 1, 'got species once');
assert.sameValue(ctorCalled, 1, 'called custom ctor once');

assert.sameValue(Object.prototype.hasOwnProperty.call(actual, 'length'), false, 'it does not define an own length property');
verifyProperty(actual, '0', {
  configurable: true,
  writable: true,
  enumerable: true,
  value: 42
});
verifyProperty(actual, '1', {
  configurable: true,
  writable: true,
  enumerable: true,
  value: 1
});
verifyProperty(actual, '2', {
  configurable: true,
  writable: true,
  enumerable: true,
  value: 42
});
verifyProperty(actual, '3', {
  configurable: true,
  writable: true,
  enumerable: true,
  value: 2
});
