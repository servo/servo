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
features: [arrow-function]
includes: [compareArray.js]
---*/

var fn = () => {};
fn.a = 1;
Object.defineProperty(fn, "name", {enumerable: true});
var fnKeys = Object.entries(fn).map(e => e[0]);
assert.compareArray(fnKeys, ["name", "a"]);
