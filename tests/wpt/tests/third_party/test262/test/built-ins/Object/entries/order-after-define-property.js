// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.entries
description: >
  Property names are returned in ascending chronological order of creation
  that is unaffected by [[DefineOwnProperty]].
info: |
  Object.entries ( O )

  [...]
  2. Let nameList be ? EnumerableOwnPropertyNames(obj, key+value).
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
obj.a = 1;
obj.b = 2;
Object.defineProperty(obj, "a", {writable: false});
var objKeys = Object.entries(obj).map(e => e[0]);
assert.compareArray(objKeys, ["a", "b"]);
