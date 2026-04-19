// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.getownpropertynames
description: >
  Property names are returned in ascending chronological order of creation
  that is unaffected by [[DefineOwnProperty]].
info: |
  Object.getOwnPropertyNames ( O )

  1. Return ? GetOwnPropertyKeys(O, String).

  GetOwnPropertyKeys ( O, type )

  1. Let obj be ? ToObject(O).
  2. Let keys be ? obj.[[OwnPropertyKeys]]().
  [...]

  OrdinaryOwnPropertyKeys ( O )

  [...]
  3. For each own property key P of O that is a String but is not an array index,
  in ascending chronological order of property creation, do
    a. Add P as the last element of keys.
  [...]
  5. Return keys.
features: [arrow-function]
includes: [compareArray.js]
---*/

var obj = {};
Object.defineProperty(obj, "a", {
  get: function() {},
  set: function(_value) {},
  enumerable: true,
  configurable: true,
})
obj.b = 2;
Object.defineProperty(obj, "a", {
  set: function(_value) {},
});
assert.compareArray(Object.getOwnPropertyNames(obj), ["a", "b"]);

var arr = [];
arr.a = 1;
Object.defineProperty(arr, "length", {value: 2});
assert.compareArray(Object.getOwnPropertyNames(arr), ["length", "a"]);
