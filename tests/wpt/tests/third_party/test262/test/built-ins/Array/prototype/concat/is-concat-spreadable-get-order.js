// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.concat
description: >
  Symbol.isConcatSpreadable should be looked up once for `this value` and
  once for each argument, after ArraySpeciesCreate routine.
info: |
  Array.prototype.concat ( ...arguments )

  1. Let O be ? ToObject(this value).
  2. Let A be ? ArraySpeciesCreate(O, 0).
  [...]
  5. Repeat, while items is not empty
    a. Remove the first element from items and let E be the value of the element.
    b. Let spreadable be ? IsConcatSpreadable(E).
    [...]

  ArraySpeciesCreate ( originalArray, length )

  [...]
  5. Let C be ? Get(originalArray, "constructor").
  [...]

  Runtime Semantics: IsConcatSpreadable ( O )

  1. If Type(O) is not Object, return false.
  2. Let spreadable be ? Get(O, @@isConcatSpreadable).
  [...]
includes: [compareArray.js]
features: [Symbol.isConcatSpreadable]
---*/

var calls = [];
var descConstructor = {
  get: function() {
    calls.push("constructor");
    return Array;
  },
  configurable: true,
};
var descSpreadable = {
  get: function() {
    calls.push("isConcatSpreadable");
  },
  configurable: true,
};

var arr1 = [];
Object.defineProperty(arr1, "constructor", descConstructor);
Object.defineProperty(arr1, Symbol.isConcatSpreadable, descSpreadable);

assert.compareArray(arr1.concat(1), [1], 'arr1.concat(1) must return [1]');
assert.compareArray(
  calls,
  ["constructor", "isConcatSpreadable"],
  'The value of calls is expected to be ["constructor", "isConcatSpreadable"]'
);

calls = [];

var arr2 = [];
var arg = {};
Object.defineProperty(arr2, "constructor", descConstructor);
Object.defineProperty(arg, Symbol.isConcatSpreadable, descSpreadable);

assert.compareArray(arr2.concat(arg), [arg], 'arr2.concat({}) must return [arg]');
assert.compareArray(
  calls,
  ["constructor", "isConcatSpreadable"],
  'The value of calls is expected to be ["constructor", "isConcatSpreadable"]'
);
