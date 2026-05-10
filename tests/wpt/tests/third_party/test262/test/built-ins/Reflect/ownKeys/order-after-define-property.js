// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-reflect.ownkeys
description: >
  Property names are returned in ascending chronological order of creation
  that is unaffected by [[DefineOwnProperty]].
info: |
  Reflect.ownKeys ( target )

  [...]
  2. Let keys be ? target.[[OwnPropertyKeys]]().
  3. Return CreateArrayFromList(keys).

  OrdinaryOwnPropertyKeys ( O )

  [...]
  4. For each own property key P of O that is a Symbol, in ascending
  chronological order of property creation, do
    a. Add P as the last element of keys.
  5. Return keys.

  [[OwnPropertyKeys]] ( )

  [...]
  7. For each own property key P of O such that Type(P) is String and P is not
  an array index, in ascending chronological order of property creation, do
    a. Add P as the last element of keys.
  [...]
  9. Return keys.
features: [Symbol, Reflect]
includes: [compareArray.js]
---*/

var obj = {};
var symA = Symbol("a");
var symB = Symbol("b");
obj[symA] = 1;
obj[symB] = 2;
Object.defineProperty(obj, symA, {configurable: false});
assert.compareArray(Reflect.ownKeys(obj), [symA, symB]);

var str = new String("");
str.a = 1;
str.b = 2;
Object.defineProperty(str, "a", {
  get: function() {},
});
assert.compareArray(Reflect.ownKeys(str), ["length", "a", "b"]);
