// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.getownpropertysymbols
description: >
  Property names are returned in ascending chronological order of creation
  that is unaffected by [[DefineOwnProperty]].
info: |
  Object.getOwnPropertySymbols ( O )

  1. Return ? GetOwnPropertyKeys(O, Symbol).

  GetOwnPropertyKeys ( O, type )

  1. Let obj be ? ToObject(O).
  2. Let keys be ? obj.[[OwnPropertyKeys]]().
  [...]

  OrdinaryOwnPropertyKeys ( O )

  [...]
  4. For each own property key P of O that is a Symbol, in ascending
  chronological order of property creation, do
    a. Add P as the last element of keys.
  5. Return keys.
features: [Symbol]
includes: [compareArray.js]
---*/

var symA = Symbol("a");
var symB = Symbol("b");

var obj = {};
obj[symA] = 1;
obj[symB] = 2;
Object.defineProperty(obj, symA, {
  get: function() {},
});
assert.compareArray(Object.getOwnPropertySymbols(obj), [symA, symB]);

var arr = [];
arr[symA] = 1;
arr[symB] = 2;
Object.defineProperty(arr, symA, {writable: false});
assert.compareArray(Object.getOwnPropertySymbols(arr), [symA, symB]);
