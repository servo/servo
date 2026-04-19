// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.keys
description: >
  Property names are returned in ascending chronological order of creation
  that is unaffected by [[DefineOwnProperty]].
info: |
  Object.keys ( O )

  [...]
  2. Let nameList be ? EnumerableOwnPropertyNames(obj, key).
  3. Return CreateArrayFromList(nameList).

  EnumerableOwnPropertyNames ( O, kind )

  [...]
  2. Let ownKeys be ? O.[[OwnPropertyKeys]]().
  [...]

  OrdinaryOwnPropertyKeys ( O )

  [...]
  3. For each own property key P of O that is a String but is not an array index,
  in ascending chronological order of property creation, do
    a. Add P as the last element of keys.
  [...]
  5. Return keys.
includes: [compareArray.js]
---*/

var obj = {};
Object.defineProperty(obj, "a", {
  get: function() {},
  set: function(_value) {},
  enumerable: true,
  configurable: true,
});
obj.b = 2;
Object.defineProperty(obj, "a", {value: 1});
assert.compareArray(Object.keys(obj), ["a", "b"]);
