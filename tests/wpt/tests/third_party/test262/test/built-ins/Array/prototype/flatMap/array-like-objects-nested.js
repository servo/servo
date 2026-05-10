// Copyright (C) 2018 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.flatmap
description: >
  Does not flatten array-like objects nested into the main array
info: |
  FlattenIntoArray(target, source, sourceLen, start, depth [ , mapperFunction, thisArg ])

  1. Let targetIndex be start.
  2. Let sourceIndex be 0.
  3. Repeat, while sourceIndex < sourceLen
    a. Let P be ! ToString(sourceIndex).
    b. Let exists be ? HasProperty(source, P).
    c. If exists is true, then
      i. Let element be ? Get(source, P).
      ii. If mapperFunction is present, then
        1. Assert: thisArg is present.
        2. Set element to ? Call(mapperFunction, thisArg , « element, sourceIndex, source »).
      iii. Let shouldFlatten be false.
      iv. If depth > 0, then
        1. Set shouldFlatten to ? IsArray(element).
      v. If shouldFlatten is true, then
        1. Let elementLen be ? ToLength(? Get(element, "length")).
        2. Set targetIndex to ? FlattenIntoArray(target, element, elementLen, targetIndex, depth - 1).
      vi. Else,
        1. If targetIndex ≥ 253-1, throw a TypeError exception.
        2. Perform ? CreateDataPropertyOrThrow(target, ! ToString(targetIndex), element).
        3. Increase targetIndex by 1.
includes: [compareArray.js]
features: [Array.prototype.flatMap, Int32Array]
---*/

function fn(e) {
  return e;
}

var obj1 = {
  length: 1,
  0: 'a',
  toString() { return 'obj1'; }
};

var obj2 = new Int32Array(2);

var obj3 = {
  get length() { throw "should not even consider the length property" },
  toString() { return 'obj3'; }
};

var arr = [obj1, obj2, obj3];
var actual = arr.flatMap(fn);
assert.compareArray(actual, arr, 'The value of actual is expected to equal the value of arr');
assert.notSameValue(actual, arr, 'The value of actual is expected to not equal the value of `arr`');

var arrLike = {
  length: 4,
  0: obj1,
  1: obj2,
  2: obj3,
  get 3() { return arrLike },
  toString() { return 'obj4'; }
};

actual = [].flatMap.call(arrLike, fn);
assert.compareArray(actual, [obj1, obj2, obj3, arrLike], 'The value of actual is expected to be [obj1, obj2, obj3, arrLike]');
assert.notSameValue(actual, arrLike, 'The value of actual is expected to not equal the value of `arrLike`');
assert.sameValue(
  Object.getPrototypeOf(actual),
  Array.prototype,
  'Object.getPrototypeOf([].flatMap.call(arrLike, fn)") returns Array.prototype'
);
